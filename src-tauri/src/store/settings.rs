use rusqlite::{OptionalExtension, params};

use super::{Database, DatabaseError};
use crate::domain::alerts::Notified;
use crate::domain::clock::Timestamp;
use crate::domain::limit::{LimitKind, Utilization};
use crate::domain::manual::ManualReading;
use crate::domain::preferences::{LanguageChoice, PollInterval, Preferences};
use crate::domain::severity::{Severity, Thresholds};

const OAUTH_OPT_IN: &str = "oauth.opt_in";
const CLI_FALLBACK: &str = "cli.fallback";
const ALERT_THRESHOLDS: &str = "alerts.thresholds";
const POLL_MINUTES: &str = "oauth.poll_minutes";
const LANGUAGE: &str = "ui.language";
const LIST_SEPARATOR: char = ',';
const NOTIFIED_PREFIX: &str = "alerts.notified.";
const NOTIFIED_SEPARATOR: char = '@';
const NO_RESET: &str = "none";
const MANUAL_PREFIX: &str = "manual.";
const ENABLED: &str = "true";
const ADAPTIVE_MIGRATION: &str = "migrations.adaptive_poll";
const LEGACY_POLL_MINUTES: &str = "1";
const DONE: &str = "done";
const DISABLED: &str = "false";

impl Database {
    pub fn preferences(&self) -> Result<Preferences, DatabaseError> {
        Ok(Preferences {
            thresholds: self
                .setting(ALERT_THRESHOLDS)?
                .as_deref()
                .and_then(parse_thresholds)
                .unwrap_or_default(),
            poll_interval: self
                .setting(POLL_MINUTES)?
                .and_then(|value| value.parse().ok())
                .and_then(PollInterval::from_minutes)
                .unwrap_or_default(),
            language: self
                .setting(LANGUAGE)?
                .as_deref()
                .and_then(LanguageChoice::from_code)
                .unwrap_or_default(),
        })
    }

    pub fn set_preferences(&self, preferences: Preferences) -> Result<(), DatabaseError> {
        let thresholds = preferences.thresholds;
        self.set_setting(
            ALERT_THRESHOLDS,
            &format!(
                "{}{LIST_SEPARATOR}{}{LIST_SEPARATOR}{}",
                thresholds.elevated(),
                thresholds.high(),
                thresholds.critical()
            ),
        )?;
        self.set_setting(
            POLL_MINUTES,
            &preferences.poll_interval.minutes().to_string(),
        )?;
        self.set_setting(LANGUAGE, preferences.language.code())
    }

    pub fn migrate_legacy_poll_interval(&self) -> Result<bool, DatabaseError> {
        if self.setting(ADAPTIVE_MIGRATION)?.is_some() {
            return Ok(false);
        }
        let legacy = self.setting(POLL_MINUTES)?.as_deref() == Some(LEGACY_POLL_MINUTES);
        if legacy {
            self.set_setting(POLL_MINUTES, &PollInterval::ADAPTIVE.minutes().to_string())?;
        }
        self.set_setting(ADAPTIVE_MIGRATION, DONE)?;
        Ok(legacy)
    }

    pub fn notified_alerts(&self) -> Result<Vec<Notified>, DatabaseError> {
        let mut notified = Vec::new();
        for kind in LimitKind::ALL {
            if let Some(entry) = self
                .setting(&notified_key(kind))?
                .as_deref()
                .and_then(|value| parse_notified(kind, value))
            {
                notified.push(entry);
            }
        }
        Ok(notified)
    }

    pub fn record_notified(&self, notified: Notified) -> Result<(), DatabaseError> {
        let reset = notified
            .resets_at
            .map_or_else(|| NO_RESET.to_owned(), |at| at.unix_millis().to_string());
        self.set_setting(
            &notified_key(notified.kind),
            &format!("{}{NOTIFIED_SEPARATOR}{reset}", notified.severity.name()),
        )
    }

    pub fn manual_readings(&self) -> Result<Vec<ManualReading>, DatabaseError> {
        let mut readings = Vec::new();
        for kind in LimitKind::ALL {
            if let Some(reading) = self
                .setting(&manual_key(kind))?
                .as_deref()
                .and_then(|value| parse_manual(kind, value))
            {
                readings.push(reading);
            }
        }
        Ok(readings)
    }

    pub fn set_manual_reading(
        &self,
        kind: LimitKind,
        reading: Option<(Utilization, Timestamp)>,
    ) -> Result<(), DatabaseError> {
        match reading {
            Some((utilization, entered_at)) => self.set_setting(
                &manual_key(kind),
                &format!(
                    "{}{NOTIFIED_SEPARATOR}{}",
                    utilization.percent(),
                    entered_at.unix_millis()
                ),
            ),
            None => self.remove_setting(&manual_key(kind)),
        }
    }

    pub fn remove_setting(&self, name: &str) -> Result<(), DatabaseError> {
        self.connection
            .execute("DELETE FROM settings WHERE name = ?1", params![name])?;
        Ok(())
    }

    pub fn oauth_opted_in(&self) -> Result<bool, DatabaseError> {
        Ok(self.setting(OAUTH_OPT_IN)?.as_deref() == Some(ENABLED))
    }

    pub fn set_oauth_opted_in(&self, opted_in: bool) -> Result<(), DatabaseError> {
        self.set_setting(OAUTH_OPT_IN, if opted_in { ENABLED } else { DISABLED })
    }

    pub fn cli_fallback_enabled(&self) -> Result<bool, DatabaseError> {
        Ok(self.setting(CLI_FALLBACK)?.as_deref() == Some(ENABLED))
    }

    pub fn set_cli_fallback(&self, enabled: bool) -> Result<(), DatabaseError> {
        self.set_setting(CLI_FALLBACK, if enabled { ENABLED } else { DISABLED })
    }

    pub fn setting(&self, name: &str) -> Result<Option<String>, DatabaseError> {
        self.connection
            .query_row(
                "SELECT value FROM settings WHERE name = ?1",
                params![name],
                |row| row.get(0),
            )
            .optional()
            .map_err(DatabaseError::from)
    }

    pub fn set_setting(&self, name: &str, value: &str) -> Result<(), DatabaseError> {
        self.connection.execute(
            "INSERT INTO settings (name, value) VALUES (?1, ?2) \
             ON CONFLICT(name) DO UPDATE SET value = excluded.value",
            params![name, value],
        )?;
        Ok(())
    }
}

fn manual_key(kind: LimitKind) -> String {
    format!("{MANUAL_PREFIX}{}", kind.name())
}

fn parse_manual(kind: LimitKind, value: &str) -> Option<ManualReading> {
    let (percent, entered_at) = value.split_once(NOTIFIED_SEPARATOR)?;
    Some(ManualReading {
        kind,
        utilization: Utilization::from_percent(percent.parse().ok()?)?,
        entered_at: Timestamp::from_unix_millis(entered_at.parse().ok()?),
    })
}

fn notified_key(kind: LimitKind) -> String {
    format!("{NOTIFIED_PREFIX}{}", kind.name())
}

fn parse_notified(kind: LimitKind, value: &str) -> Option<Notified> {
    let (severity, reset) = value.split_once(NOTIFIED_SEPARATOR)?;
    let resets_at = if reset == NO_RESET {
        None
    } else {
        Some(Timestamp::from_unix_millis(reset.parse().ok()?))
    };
    Some(Notified {
        kind,
        severity: Severity::from_name(severity)?,
        resets_at,
    })
}

fn parse_thresholds(value: &str) -> Option<Thresholds> {
    let mut parts = value.split(LIST_SEPARATOR).map(str::parse::<u8>);
    let elevated = parts.next()?.ok()?;
    let high = parts.next()?.ok()?;
    let critical = parts.next()?.ok()?;
    if parts.next().is_some() {
        return None;
    }
    Thresholds::new(elevated, high, critical)
}

#[cfg(test)]
mod tests {
    use crate::domain::alerts::Notified;
    use crate::domain::clock::Timestamp;
    use crate::domain::limit::{LimitKind, Utilization};
    use crate::domain::manual::ManualReading;
    use crate::domain::preferences::{Language, LanguageChoice, PollInterval, Preferences};
    use crate::domain::severity::{Severity, Thresholds};
    use crate::store::Database;

    #[test]
    fn manual_readings_can_be_set_and_cleared() {
        let database = Database::open_in_memory().expect("open");
        assert!(database.manual_readings().expect("read").is_empty());
        let entered_at = Timestamp::from_unix_millis(1_790_300_000_000);
        let percent = Utilization::from_percent(42.5).expect("valid percent");
        database
            .set_manual_reading(LimitKind::SevenDay, Some((percent, entered_at)))
            .expect("write");
        assert_eq!(
            database.manual_readings().expect("read"),
            vec![ManualReading {
                kind: LimitKind::SevenDay,
                utilization: percent,
                entered_at,
            }]
        );
        database
            .set_manual_reading(LimitKind::SevenDay, None)
            .expect("clear");
        assert!(database.manual_readings().expect("read").is_empty());
        database
            .set_setting("manual.five_hour", "lots@soon")
            .expect("write");
        assert!(database.manual_readings().expect("read").is_empty());
    }

    #[test]
    fn notified_alerts_round_trip_per_limit() {
        let database = Database::open_in_memory().expect("open");
        assert!(database.notified_alerts().expect("read").is_empty());
        let five_hour = Notified {
            kind: LimitKind::FiveHour,
            severity: Severity::High,
            resets_at: Some(Timestamp::from_unix_millis(1_790_317_800_000)),
        };
        let weekly = Notified {
            kind: LimitKind::SevenDay,
            severity: Severity::Elevated,
            resets_at: None,
        };
        database.record_notified(five_hour).expect("write");
        database.record_notified(weekly).expect("write");
        let critical = Notified {
            severity: Severity::Critical,
            ..five_hour
        };
        database.record_notified(critical).expect("overwrite");
        assert_eq!(
            database.notified_alerts().expect("read"),
            vec![critical, weekly]
        );
    }

    #[test]
    fn corrupt_notified_entries_are_ignored() {
        let database = Database::open_in_memory().expect("open");
        for value in ["", "high", "loud@none", "high@soon"] {
            database
                .set_setting("alerts.notified.five_hour", value)
                .expect("write");
            assert!(
                database.notified_alerts().expect("read").is_empty(),
                "{value}"
            );
        }
    }

    #[test]
    fn preferences_default_until_saved() {
        let database = Database::open_in_memory().expect("open");
        assert_eq!(
            database.preferences().expect("read"),
            Preferences::default()
        );
    }

    #[test]
    fn preferences_round_trip() {
        let database = Database::open_in_memory().expect("open");
        let preferences = Preferences {
            thresholds: Thresholds::new(40, 70, 90).expect("valid thresholds"),
            poll_interval: PollInterval::from_minutes(5).expect("valid interval"),
            language: LanguageChoice::Fixed(Language::Indonesian),
        };
        database.set_preferences(preferences).expect("write");
        assert_eq!(database.preferences().expect("read"), preferences);
    }

    #[test]
    fn corrupt_preferences_fall_back_to_defaults() {
        let database = Database::open_in_memory().expect("open");
        for value in ["", "50,80", "50,80,95,99", "90,80,95", "a,b,c", "50,80,300"] {
            database
                .set_setting("alerts.thresholds", value)
                .expect("write");
            assert_eq!(
                database.preferences().expect("read").thresholds,
                Thresholds::DEFAULT,
                "{value}"
            );
        }
        database
            .set_setting("oauth.poll_minutes", "7")
            .expect("write");
        database
            .set_setting("ui.language", "klingon")
            .expect("write");
        let preferences = database.preferences().expect("read");
        assert_eq!(preferences.poll_interval, PollInterval::DEFAULT);
        assert_eq!(preferences.language, LanguageChoice::System);
    }

    #[test]
    fn a_legacy_one_minute_interval_moves_to_adaptive_once() {
        let database = Database::open_in_memory().expect("open");
        database
            .set_setting("oauth.poll_minutes", "1")
            .expect("write");
        assert!(database.migrate_legacy_poll_interval().expect("migrate"));
        assert_eq!(
            database.preferences().expect("read").poll_interval,
            PollInterval::ADAPTIVE
        );
        database
            .set_setting("oauth.poll_minutes", "1")
            .expect("chosen again");
        assert!(!database.migrate_legacy_poll_interval().expect("again"));
        assert_eq!(
            database.preferences().expect("read").poll_interval,
            PollInterval::from_minutes(1).expect("valid interval")
        );
    }

    #[test]
    fn other_intervals_are_kept_and_the_migration_is_marked_done() {
        for value in [None, Some("0"), Some("2"), Some("10")] {
            let database = Database::open_in_memory().expect("open");
            if let Some(value) = value {
                database
                    .set_setting("oauth.poll_minutes", value)
                    .expect("write");
            }
            assert!(!database.migrate_legacy_poll_interval().expect("migrate"));
            assert_eq!(
                database
                    .setting("oauth.poll_minutes")
                    .expect("read")
                    .as_deref(),
                value
            );
            assert!(
                database
                    .setting("migrations.adaptive_poll")
                    .expect("marker")
                    .is_some()
            );
        }
    }

    #[test]
    fn missing_setting_is_none() {
        let database = Database::open_in_memory().expect("open");
        assert_eq!(database.setting("theme").expect("read"), None);
    }

    #[test]
    fn oauth_is_opt_in_and_can_be_turned_off_again() {
        let database = Database::open_in_memory().expect("open");
        assert!(!database.oauth_opted_in().expect("default"));
        database.set_oauth_opted_in(true).expect("opt in");
        assert!(database.oauth_opted_in().expect("opted in"));
        database.set_oauth_opted_in(false).expect("opt out");
        assert!(!database.oauth_opted_in().expect("opted out"));
    }

    #[test]
    fn the_cli_fallback_is_off_until_chosen() {
        let database = Database::open_in_memory().expect("open");
        assert!(!database.cli_fallback_enabled().expect("default"));
        database.set_cli_fallback(true).expect("enable");
        assert!(database.cli_fallback_enabled().expect("enabled"));
        database
            .set_setting("cli.fallback", "maybe")
            .expect("write");
        assert!(!database.cli_fallback_enabled().expect("corrupt"));
    }

    #[test]
    fn unexpected_opt_in_values_count_as_opted_out() {
        let database = Database::open_in_memory().expect("open");
        database.set_setting("oauth.opt_in", "yes").expect("write");
        assert!(!database.oauth_opted_in().expect("read"));
    }

    #[test]
    fn settings_are_upserted() {
        let database = Database::open_in_memory().expect("open");
        database.set_setting("theme", "light").expect("write");
        database.set_setting("theme", "dark").expect("overwrite");
        assert_eq!(
            database.setting("theme").expect("read").as_deref(),
            Some("dark")
        );
    }
}
