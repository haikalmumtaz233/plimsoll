use super::calibration::Calibration;
use super::clock::{Span, Timestamp};
use super::limit::{LimitKind, Utilization};
use super::period::Window;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ManualReading {
    pub kind: LimitKind,
    pub utilization: Utilization,
    pub entered_at: Timestamp,
}

impl ManualReading {
    #[must_use]
    pub fn applies(&self, window: Option<Window>, now: Timestamp) -> bool {
        let span = match self.kind {
            LimitKind::FiveHour => Span::FIVE_HOURS,
            LimitKind::SevenDay => Span::WEEK,
        };
        let not_expired = self.entered_at <= now && now < self.entered_at + span;
        let same_window = window.is_none_or(|window| window.start() <= self.entered_at);
        not_expired && same_window
    }

    #[must_use]
    pub fn project(&self, tokens_since: u64, calibration: Option<Calibration>) -> Utilization {
        calibration
            .and_then(|calibration| calibration.estimate(tokens_since))
            .and_then(|extra| {
                Utilization::from_percent(self.utilization.percent() + extra.percent())
            })
            .unwrap_or(self.utilization)
    }
}

#[cfg(test)]
mod tests {
    use super::ManualReading;
    use crate::domain::calibration::{Sample, calibrate};
    use crate::domain::clock::{Span, Timestamp};
    use crate::domain::limit::{LimitKind, Utilization};
    use crate::domain::period::Window;

    const NOW: Timestamp = Timestamp::from_unix_millis(1_790_300_000_000);

    fn reading(kind: LimitKind, percent: f64, entered_minutes_ago: i64) -> ManualReading {
        ManualReading {
            kind,
            utilization: Utilization::from_percent(percent).expect("valid percent"),
            entered_at: NOW - Span::from_millis(entered_minutes_ago * 60_000),
        }
    }

    #[test]
    fn five_hour_readings_expire_after_five_hours() {
        assert!(reading(LimitKind::FiveHour, 40.0, 60).applies(None, NOW));
        assert!(!reading(LimitKind::FiveHour, 40.0, 300).applies(None, NOW));
        assert!(reading(LimitKind::SevenDay, 40.0, 300).applies(None, NOW));
    }

    #[test]
    fn a_window_that_started_after_the_entry_invalidates_it() {
        let reading = reading(LimitKind::FiveHour, 40.0, 60);
        let later = Window::starting_at(NOW - Span::from_millis(30 * 60_000), Span::FIVE_HOURS);
        let earlier = Window::starting_at(NOW - Span::hours(2), Span::FIVE_HOURS);
        assert!(!reading.applies(Some(later), NOW));
        assert!(reading.applies(Some(earlier), NOW));
    }

    #[test]
    fn future_entries_do_not_apply() {
        let future = ManualReading {
            entered_at: NOW + Span::hours(1),
            ..reading(LimitKind::FiveHour, 40.0, 0)
        };
        assert!(!future.applies(None, NOW));
    }

    #[test]
    fn projection_adds_calibrated_usage_since_the_entry() {
        let calibration = calibrate(
            &[Sample {
                utilization: Utilization::from_percent(10.0).expect("valid percent"),
                tokens: 1_000,
            }; 3],
        );
        let reading = reading(LimitKind::FiveHour, 40.0, 30);
        assert!((reading.project(500, calibration).percent() - 45.0).abs() < 1e-9);
        assert!((reading.project(500, None).percent() - 40.0).abs() < 1e-9);
    }
}
