use super::clock::{Span, Timestamp};

pub const DEFAULT_WEEKLY_ANCHOR: Timestamp = Timestamp::from_unix_millis(345_600_000);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Window {
    start: Timestamp,
    end: Timestamp,
}

impl Window {
    #[must_use]
    pub fn starting_at(start: Timestamp, length: Span) -> Self {
        Self {
            start,
            end: start + length,
        }
    }

    #[must_use]
    pub fn ending_at(end: Timestamp, length: Span) -> Self {
        Self {
            start: end - length,
            end,
        }
    }

    #[must_use]
    pub const fn start(self) -> Timestamp {
        self.start
    }

    #[must_use]
    pub const fn end(self) -> Timestamp {
        self.end
    }

    #[must_use]
    pub fn contains(self, instant: Timestamp) -> bool {
        self.start <= instant && instant < self.end
    }

    #[must_use]
    pub fn remaining(self, now: Timestamp) -> Span {
        if now >= self.end {
            Span::ZERO
        } else {
            self.end - now.max(self.start)
        }
    }
}

#[must_use]
pub fn five_hour_from_reset(resets_at: Timestamp) -> Window {
    Window::ending_at(resets_at, Span::FIVE_HOURS)
}

#[must_use]
pub fn weekly_from_reset(resets_at: Timestamp) -> Window {
    Window::ending_at(resets_at, Span::WEEK)
}

#[must_use]
pub fn weekly_containing(anchor: Timestamp, now: Timestamp) -> Window {
    let week = Span::WEEK.millis();
    let elapsed_weeks = (now - anchor).millis().div_euclid(week);
    let start = anchor + Span::from_millis(elapsed_weeks.saturating_mul(week));
    Window::starting_at(start, Span::WEEK)
}

#[cfg(test)]
mod tests {
    use super::{
        DEFAULT_WEEKLY_ANCHOR, Window, five_hour_from_reset, weekly_containing, weekly_from_reset,
    };
    use crate::domain::clock::{Span, Timestamp};

    const SEP_21_00_00: i64 = 1_789_948_800_000;
    const SEP_25_01_30: i64 = 1_790_299_800_000;
    const SEP_25_03_37: i64 = 1_790_307_420_000;
    const SEP_25_06_30: i64 = 1_790_317_800_000;
    const SEP_28_00_00: i64 = 1_790_553_600_000;

    fn at(millis: i64) -> Timestamp {
        Timestamp::from_unix_millis(millis)
    }

    #[test]
    fn five_hour_window_ends_at_the_reported_reset() {
        let window = five_hour_from_reset(at(SEP_25_06_30));
        assert_eq!(window.start(), at(SEP_25_01_30));
        assert_eq!(window.end(), at(SEP_25_06_30));
    }

    #[test]
    fn weekly_window_ends_at_the_reported_reset() {
        let window = weekly_from_reset(at(SEP_28_00_00));
        assert_eq!(window.start(), at(SEP_21_00_00));
    }

    #[test]
    fn window_is_half_open() {
        let window = Window::starting_at(at(1_000), Span::from_millis(500));
        assert!(!window.contains(at(999)));
        assert!(window.contains(at(1_000)));
        assert!(window.contains(at(1_499)));
        assert!(!window.contains(at(1_500)));
    }

    #[test]
    fn remaining_counts_down_to_zero() {
        let window = Window::starting_at(at(1_000), Span::from_millis(500));
        assert_eq!(window.remaining(at(500)), Span::from_millis(500));
        assert_eq!(window.remaining(at(1_200)), Span::from_millis(300));
        assert_eq!(window.remaining(at(1_500)), Span::ZERO);
        assert_eq!(window.remaining(at(9_000)), Span::ZERO);
    }

    #[test]
    fn default_anchor_matches_observed_monday_reset() {
        let window = weekly_containing(DEFAULT_WEEKLY_ANCHOR, at(SEP_25_03_37));
        assert_eq!(window.start(), at(SEP_21_00_00));
        assert_eq!(window.end(), at(SEP_28_00_00));
    }

    #[test]
    fn weekly_window_rolls_over_exactly_at_the_boundary() {
        let window = weekly_containing(DEFAULT_WEEKLY_ANCHOR, at(SEP_28_00_00));
        assert_eq!(window.start(), at(SEP_28_00_00));
    }

    #[test]
    fn weekly_window_handles_instants_before_the_anchor() {
        let window = weekly_containing(DEFAULT_WEEKLY_ANCHOR, at(0));
        assert_eq!(window.start(), at(-259_200_000));
        assert!(window.contains(at(0)));
    }
}
