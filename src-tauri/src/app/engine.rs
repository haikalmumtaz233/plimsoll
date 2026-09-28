use std::path::PathBuf;

use crate::domain::alerts::{self, Alert};
use crate::domain::calibration::{self, Basis, Calibration, Estimate, Sample};
use crate::domain::clock::{Span, Timestamp};
use crate::domain::limit::{LimitKind, LimitSnapshot, Utilization};
use crate::domain::manual::ManualReading;
use crate::domain::period::Window;
use crate::domain::preferences::Preferences;
use crate::domain::record::KeyedEvent;
use crate::domain::summary::{self, UsageSummary};
use crate::sources::oauth::poll::PollResult;
use crate::sources::oauth::status::OAuthStatus;
use crate::store::{Database, DatabaseError};

pub const CALIBRATION_LOOKBACK: Span = Span::days(30);

#[derive(Debug, Clone, PartialEq)]
pub struct Report {
    pub accurate_mode: bool,
    pub status: OAuthStatus,
    pub preferences: Preferences,
    pub summary: UsageSummary,
    pub estimates: Vec<Estimate>,
    pub manual: Vec<ManualReading>,
}

#[derive(Debug)]
pub struct Engine {
    database: Database,
    status: OAuthStatus,
}

impl Engine {
    pub fn new(database: Database) -> Result<Self, DatabaseError> {
        let status = if database.oauth_opted_in()? {
            OAuthStatus::Pending
        } else {
            OAuthStatus::Disabled
        };
        Ok(Self { database, status })
    }

    pub fn accurate_mode(&self) -> Result<bool, DatabaseError> {
        self.database.oauth_opted_in()
    }

    #[must_use]
    pub const fn status(&self) -> OAuthStatus {
        self.status
    }

    pub fn last_official_at(&self) -> Result<Option<Timestamp>, DatabaseError> {
        Ok(self
            .database
            .latest_snapshots()?
            .into_iter()
            .map(|stored| stored.at)
            .max())
    }

    pub fn set_accurate_mode(&mut self, enabled: bool) -> Result<(), DatabaseError> {
        self.database.set_oauth_opted_in(enabled)?;
        self.status = if enabled {
            OAuthStatus::Pending
        } else {
            OAuthStatus::Disabled
        };
        Ok(())
    }

    pub fn preferences(&self) -> Result<Preferences, DatabaseError> {
        self.database.preferences()
    }

    pub fn set_preferences(&mut self, preferences: Preferences) -> Result<(), DatabaseError> {
        self.database.set_preferences(preferences)
    }

    pub fn offsets(&self) -> Result<Vec<(PathBuf, u64)>, DatabaseError> {
        self.database.load_offsets()
    }

    pub fn store_jsonl(
        &mut self,
        events: &[KeyedEvent],
        offsets: &[(PathBuf, u64)],
    ) -> Result<usize, DatabaseError> {
        let inserted = self.database.insert_events(events)?;
        self.database.save_offsets(offsets)?;
        Ok(inserted)
    }

    pub fn record_oauth(
        &mut self,
        result: &PollResult,
        now: Timestamp,
    ) -> Result<(), DatabaseError> {
        if !self.accurate_mode()? {
            return Ok(());
        }
        if let Ok(snapshots) = result {
            self.database.insert_snapshots(now, snapshots)?;
        }
        self.status = OAuthStatus::from_result(result);
        Ok(())
    }

    pub fn take_alerts(&mut self, now: Timestamp) -> Result<Vec<Alert>, DatabaseError> {
        if !self.accurate_mode()? {
            return Ok(Vec::new());
        }
        let limits = self.current_limits(now)?;
        let thresholds = self.database.preferences()?.thresholds;
        let alerts = alerts::crossings(&limits, thresholds, &self.database.notified_alerts()?);
        for alert in &alerts {
            self.database.record_notified(alert.notified())?;
        }
        Ok(alerts)
    }

    pub fn prune(&self, now: Timestamp) -> Result<usize, DatabaseError> {
        self.database.prune_expired(now)
    }

    pub fn report(&self, now: Timestamp) -> Result<Report, DatabaseError> {
        let accurate_mode = self.accurate_mode()?;
        let limits = if accurate_mode {
            self.current_limits(now)?
        } else {
            Vec::new()
        };
        let events = self.database.events_in(summary::lookback(&limits, now))?;
        let summary = summary::summarize(limits, &events, now);
        let manual = self.manual_readings(&summary, now)?;
        Ok(Report {
            accurate_mode,
            status: self.status,
            preferences: self.database.preferences()?,
            estimates: self.estimates(&summary, &manual, now)?,
            manual,
            summary,
        })
    }

    fn estimates(
        &self,
        summary: &UsageSummary,
        manual: &[ManualReading],
        now: Timestamp,
    ) -> Result<Vec<Estimate>, DatabaseError> {
        if !summary.limits.is_empty() {
            return Ok(Vec::new());
        }
        let mut estimates = Vec::new();
        for (kind, window) in [
            (LimitKind::FiveHour, summary.five_hour),
            (LimitKind::SevenDay, summary.weekly),
        ] {
            let calibration = self.calibration(kind, now)?;
            if let Some(reading) = manual.iter().find(|reading| reading.kind == kind) {
                let since = Window::starting_at(
                    reading.entered_at,
                    (now + Span::from_millis(1)) - reading.entered_at,
                );
                let tokens_since = self.database.tokens_in(since)?.excluding_cache_reads();
                estimates.push(Estimate {
                    kind,
                    utilization: reading.project(tokens_since, calibration),
                    basis: Basis::Manual {
                        entered_at: reading.entered_at,
                        entered: reading.utilization,
                    },
                });
                continue;
            }
            if window.window.is_none() {
                continue;
            }
            if let Some(calibration) = calibration
                && let Some(utilization) =
                    calibration.estimate(window.tokens.excluding_cache_reads())
            {
                estimates.push(Estimate {
                    kind,
                    utilization,
                    basis: Basis::Calibrated {
                        samples: calibration.samples(),
                    },
                });
            }
        }
        Ok(estimates)
    }

    pub fn set_manual_reading(
        &mut self,
        kind: LimitKind,
        utilization: Option<Utilization>,
        now: Timestamp,
    ) -> Result<(), DatabaseError> {
        self.database
            .set_manual_reading(kind, utilization.map(|utilization| (utilization, now)))
    }

    fn manual_readings(
        &self,
        summary: &UsageSummary,
        now: Timestamp,
    ) -> Result<Vec<ManualReading>, DatabaseError> {
        Ok(self
            .database
            .manual_readings()?
            .into_iter()
            .filter(|reading| {
                let window = match reading.kind {
                    LimitKind::FiveHour => None,
                    LimitKind::SevenDay => summary.weekly.window,
                };
                reading.applies(window, now)
            })
            .collect())
    }

    fn calibration(
        &self,
        kind: LimitKind,
        now: Timestamp,
    ) -> Result<Option<Calibration>, DatabaseError> {
        let observations = self
            .database
            .window_peaks(kind, now - CALIBRATION_LOOKBACK)?;
        let mut samples = Vec::new();
        for peak in calibration::peaks(&observations, kind) {
            let start = peak.window.start();
            let seen =
                Window::starting_at(start, (peak.observed_at + Span::from_millis(1)) - start);
            samples.push(Sample {
                utilization: peak.utilization,
                tokens: self.database.tokens_in(seen)?.excluding_cache_reads(),
            });
        }
        Ok(calibration::calibrate(&samples))
    }

    fn current_limits(&self, now: Timestamp) -> Result<Vec<LimitSnapshot>, DatabaseError> {
        Ok(self
            .database
            .latest_snapshots()?
            .into_iter()
            .filter(|stored| stored.snapshot.is_current(stored.at, now))
            .map(|stored| stored.snapshot)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::Engine;
    use crate::domain::calibration::Basis;
    use crate::domain::clock::{Span, Timestamp};
    use crate::domain::limit::{LimitKind, LimitSnapshot, STALE_AFTER, Utilization};
    use crate::domain::preferences::{PollInterval, Preferences};
    use crate::domain::record::{EventKey, KeyedEvent, UsageEvent};
    use crate::domain::severity::{Severity, Thresholds};
    use crate::domain::tokens::TokenCounts;
    use crate::sources::oauth::OAuthError;
    use crate::sources::oauth::status::OAuthStatus;
    use crate::store::Database;
    use std::path::PathBuf;

    const NOW: Timestamp = Timestamp::from_unix_millis(1_790_300_000_000);

    fn engine() -> Engine {
        Engine::new(Database::open_in_memory().expect("open")).expect("engine")
    }

    fn keyed(id: &str, minutes_ago: i64, output: u64) -> KeyedEvent {
        KeyedEvent {
            key: EventKey {
                message_id: id.to_owned(),
                request_id: format!("req_{id}"),
            },
            event: UsageEvent {
                at: NOW - Span::from_millis(minutes_ago * 60_000),
                model: "claude-opus-5".to_owned(),
                project: "plimsoll".to_owned(),
                tokens: TokenCounts {
                    output,
                    ..TokenCounts::default()
                },
            },
        }
    }

    fn five_hour(percent: f64) -> LimitSnapshot {
        LimitSnapshot {
            kind: LimitKind::FiveHour,
            utilization: Utilization::from_percent(percent).expect("valid percent"),
            resets_at: Some(NOW + Span::hours(2)),
        }
    }

    #[test]
    fn starts_in_estimate_mode_with_jsonl_totals() {
        let mut engine = engine();
        let offsets = vec![(PathBuf::from("C:\\p\\s.jsonl"), 120)];
        let inserted = engine
            .store_jsonl(&[keyed("a", 30, 5), keyed("b", 10, 7)], &offsets)
            .expect("store");
        assert_eq!(inserted, 2);
        assert_eq!(engine.offsets().expect("offsets"), offsets);

        let report = engine.report(NOW).expect("report");
        assert!(!report.accurate_mode);
        assert_eq!(report.status, OAuthStatus::Disabled);
        assert!(report.summary.limits.is_empty());
        assert_eq!(report.summary.five_hour.tokens.output, 12);
        assert_eq!(report.summary.weekly.tokens.output, 12);
    }

    #[test]
    fn oauth_results_are_ignored_until_opted_in() {
        let mut engine = engine();
        engine
            .record_oauth(&Ok(vec![five_hour(40.0)]), NOW)
            .expect("record");
        let report = engine.report(NOW).expect("report");
        assert!(report.summary.limits.is_empty());
        assert_eq!(report.status, OAuthStatus::Disabled);
    }

    #[test]
    fn opted_in_reports_show_current_limits_and_status() {
        let mut engine = engine();
        engine.set_accurate_mode(true).expect("opt in");
        assert_eq!(
            engine.report(NOW).expect("report").status,
            OAuthStatus::Pending
        );

        engine
            .record_oauth(&Ok(vec![five_hour(40.0)]), NOW)
            .expect("record");
        let report = engine.report(NOW).expect("report");
        assert!(report.accurate_mode);
        assert_eq!(report.status, OAuthStatus::Active);
        assert_eq!(report.summary.limits, vec![five_hour(40.0)]);

        engine
            .record_oauth(&Err(OAuthError::Unauthorized), NOW)
            .expect("record failure");
        let report = engine.report(NOW).expect("report");
        assert_eq!(report.status, OAuthStatus::Unauthorized);
        assert_eq!(report.summary.limits, vec![five_hour(40.0)]);

        let later = NOW + STALE_AFTER + Span::from_millis(1);
        assert!(
            engine
                .report(later)
                .expect("report")
                .summary
                .limits
                .is_empty()
        );
    }

    #[test]
    fn opting_out_hides_limits_and_survives_restarts() {
        let mut engine = engine();
        engine.set_accurate_mode(true).expect("opt in");
        engine
            .record_oauth(&Ok(vec![five_hour(40.0)]), NOW)
            .expect("record");
        engine.set_accurate_mode(false).expect("opt out");
        let report = engine.report(NOW).expect("report");
        assert!(!report.accurate_mode);
        assert_eq!(report.status, OAuthStatus::Disabled);
        assert!(report.summary.limits.is_empty());
    }

    #[test]
    fn alerts_fire_once_per_level_while_accurate() {
        let mut engine = engine();
        engine
            .record_oauth(&Ok(vec![five_hour(85.0)]), NOW)
            .expect("record");
        assert!(engine.take_alerts(NOW).expect("alerts").is_empty());

        engine.set_accurate_mode(true).expect("opt in");
        engine
            .record_oauth(&Ok(vec![five_hour(85.0)]), NOW)
            .expect("record");
        let alerts = engine.take_alerts(NOW).expect("alerts");
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].severity, Severity::High);
        assert!(engine.take_alerts(NOW).expect("alerts").is_empty());

        engine
            .record_oauth(&Ok(vec![five_hour(97.0)]), NOW)
            .expect("record");
        assert_eq!(
            engine.take_alerts(NOW).expect("alerts")[0].severity,
            Severity::Critical
        );
    }

    #[test]
    fn fallback_reports_estimate_percent_once_calibrated() {
        let mut engine = engine();
        engine.set_accurate_mode(true).expect("opt in");
        let mut past_events = Vec::new();
        for index in 1..=3_i64 {
            let resets_at = NOW - Span::hours(6 * index);
            past_events.push(keyed(&format!("past{index}"), (6 * index + 2) * 60, 1_000));
            engine
                .record_oauth(
                    &Ok(vec![LimitSnapshot {
                        kind: LimitKind::FiveHour,
                        utilization: Utilization::from_percent(10.0).expect("valid percent"),
                        resets_at: Some(resets_at),
                    }]),
                    resets_at - Span::hours(1),
                )
                .expect("record");
        }
        past_events.push(keyed("now", 10, 4_200));
        engine.store_jsonl(&past_events, &[]).expect("store");

        engine.set_accurate_mode(false).expect("opt out");
        let report = engine.report(NOW).expect("report");
        assert!(report.summary.limits.is_empty());
        assert_eq!(report.estimates.len(), 1);
        assert_eq!(report.estimates[0].kind, LimitKind::FiveHour);
        assert_eq!(report.estimates[0].basis, Basis::Calibrated { samples: 3 });
        assert!((report.estimates[0].utilization.percent() - 42.0).abs() < 1e-9);

        engine.set_accurate_mode(true).expect("opt in again");
        engine
            .record_oauth(&Ok(vec![five_hour(40.0)]), NOW)
            .expect("record current");
        assert!(engine.report(NOW).expect("report").estimates.is_empty());
    }

    #[test]
    fn manual_readings_win_over_calibration_until_they_expire() {
        let mut engine = engine();
        engine
            .store_jsonl(&[keyed("recent", 20, 500)], &[])
            .expect("store");
        engine
            .set_manual_reading(
                LimitKind::FiveHour,
                Some(Utilization::from_percent(30.0).expect("valid percent")),
                NOW - Span::from_millis(60 * 60_000),
            )
            .expect("manual");
        let report = engine.report(NOW).expect("report");
        assert_eq!(report.estimates.len(), 1);
        assert!((report.estimates[0].utilization.percent() - 30.0).abs() < 1e-9);
        assert!(matches!(report.estimates[0].basis, Basis::Manual { .. }));
        assert_eq!(report.manual.len(), 1);

        let later = NOW + Span::hours(5);
        assert!(engine.report(later).expect("report").estimates.is_empty());

        engine
            .set_manual_reading(LimitKind::FiveHour, None, NOW)
            .expect("clear");
        assert!(engine.report(NOW).expect("report").estimates.is_empty());
    }

    #[test]
    fn preferences_are_saved_and_reported() {
        let mut engine = engine();
        assert_eq!(engine.preferences().expect("read"), Preferences::default());
        let preferences = Preferences {
            thresholds: Thresholds::new(30, 60, 90).expect("valid thresholds"),
            poll_interval: PollInterval::from_minutes(2).expect("valid interval"),
            ..Preferences::default()
        };
        engine.set_preferences(preferences).expect("write");
        assert_eq!(engine.report(NOW).expect("report").preferences, preferences);
    }

    #[test]
    fn a_stored_opt_in_starts_pending() {
        let database = Database::open_in_memory().expect("open");
        database.set_oauth_opted_in(true).expect("opt in");
        let engine = Engine::new(database).expect("engine");
        assert!(engine.accurate_mode().expect("mode"));
        assert_eq!(
            engine.report(NOW).expect("report").status,
            OAuthStatus::Pending
        );
        assert_eq!(engine.prune(NOW).expect("prune"), 0);
    }
}
