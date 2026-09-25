use std::ops::{Add, Sub};

const MILLIS_PER_HOUR: i64 = 3_600_000;
const MILLIS_PER_DAY: i64 = 86_400_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(i64);

impl Timestamp {
    #[must_use]
    pub const fn from_unix_millis(millis: i64) -> Self {
        Self(millis)
    }

    #[must_use]
    pub const fn unix_millis(self) -> i64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span(i64);

impl Span {
    pub const ZERO: Self = Self(0);
    pub const FIVE_HOURS: Self = Self::hours(5);
    pub const WEEK: Self = Self::days(7);

    #[must_use]
    pub const fn from_millis(millis: i64) -> Self {
        Self(millis)
    }

    #[must_use]
    pub const fn hours(hours: i64) -> Self {
        Self(hours.saturating_mul(MILLIS_PER_HOUR))
    }

    #[must_use]
    pub const fn days(days: i64) -> Self {
        Self(days.saturating_mul(MILLIS_PER_DAY))
    }

    #[must_use]
    pub const fn millis(self) -> i64 {
        self.0
    }
}

impl Add<Span> for Timestamp {
    type Output = Self;

    fn add(self, span: Span) -> Self {
        Self(self.0.saturating_add(span.0))
    }
}

impl Sub<Span> for Timestamp {
    type Output = Self;

    fn sub(self, span: Span) -> Self {
        Self(self.0.saturating_sub(span.0))
    }
}

impl Sub for Timestamp {
    type Output = Span;

    fn sub(self, earlier: Self) -> Span {
        Span(self.0.saturating_sub(earlier.0))
    }
}

#[cfg(test)]
mod tests {
    use super::{Span, Timestamp};

    #[test]
    fn named_spans_have_expected_lengths() {
        assert_eq!(Span::FIVE_HOURS.millis(), 18_000_000);
        assert_eq!(Span::WEEK.millis(), 604_800_000);
        assert_eq!(Span::ZERO.millis(), 0);
    }

    #[test]
    fn adding_and_subtracting_spans_moves_the_timestamp() {
        let start = Timestamp::from_unix_millis(1_000);
        let span = Span::from_millis(500);
        assert_eq!((start + span).unix_millis(), 1_500);
        assert_eq!((start - span).unix_millis(), 500);
    }

    #[test]
    fn difference_between_timestamps_is_a_span() {
        let earlier = Timestamp::from_unix_millis(1_000);
        let later = Timestamp::from_unix_millis(4_600_000);
        assert_eq!(later - earlier, Span::from_millis(4_599_000));
        assert_eq!(earlier - later, Span::from_millis(-4_599_000));
    }

    #[test]
    fn arithmetic_saturates_instead_of_overflowing() {
        let latest = Timestamp::from_unix_millis(i64::MAX);
        let earliest = Timestamp::from_unix_millis(i64::MIN);
        assert_eq!(latest + Span::hours(1), latest);
        assert_eq!(earliest - Span::hours(1), earliest);
        assert_eq!(Span::days(i64::MAX).millis(), i64::MAX);
    }
}
