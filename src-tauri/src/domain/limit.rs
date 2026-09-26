use super::clock::{Span, Timestamp};

pub const STALE_AFTER: Span = Span::from_millis(15 * 60_000);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LimitKind {
    FiveHour,
    SevenDay,
}

impl LimitKind {
    pub const ALL: [Self; 2] = [Self::FiveHour, Self::SevenDay];

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::FiveHour => "five_hour",
            Self::SevenDay => "seven_day",
        }
    }

    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.name() == name)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Utilization(f64);

impl Utilization {
    #[must_use]
    pub fn from_percent(percent: f64) -> Option<Self> {
        (percent.is_finite() && percent >= 0.0).then_some(Self(percent))
    }

    #[must_use]
    pub const fn percent(self) -> f64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LimitSnapshot {
    pub kind: LimitKind,
    pub utilization: Utilization,
    pub resets_at: Option<Timestamp>,
}

impl LimitSnapshot {
    #[must_use]
    pub fn is_current(&self, observed_at: Timestamp, now: Timestamp) -> bool {
        now - observed_at <= STALE_AFTER && self.resets_at.is_none_or(|resets_at| resets_at > now)
    }
}

#[cfg(test)]
mod tests {
    use super::{LimitKind, LimitSnapshot, STALE_AFTER, Utilization};
    use crate::domain::clock::{Span, Timestamp};

    const NOW: Timestamp = Timestamp::from_unix_millis(1_790_300_000_000);

    fn snapshot(resets_at: Option<Timestamp>) -> LimitSnapshot {
        LimitSnapshot {
            kind: LimitKind::FiveHour,
            utilization: Utilization::from_percent(40.0).expect("valid percent"),
            resets_at,
        }
    }

    #[test]
    fn recent_snapshots_before_their_reset_are_current() {
        let fresh = snapshot(Some(NOW + Span::hours(1)));
        assert!(fresh.is_current(NOW, NOW));
        assert!(fresh.is_current(NOW - STALE_AFTER, NOW));
        assert!(snapshot(None).is_current(NOW - Span::from_millis(1), NOW));
    }

    #[test]
    fn old_snapshots_are_stale() {
        let stale = snapshot(Some(NOW + Span::hours(1)));
        assert!(!stale.is_current(NOW - STALE_AFTER - Span::from_millis(1), NOW));
    }

    #[test]
    fn snapshots_past_their_reset_no_longer_apply() {
        assert!(!snapshot(Some(NOW)).is_current(NOW, NOW));
        assert!(!snapshot(Some(NOW - Span::hours(1))).is_current(NOW, NOW));
    }

    #[test]
    fn kind_names_round_trip() {
        for kind in LimitKind::ALL {
            assert_eq!(LimitKind::from_name(kind.name()), Some(kind));
        }
        assert_eq!(LimitKind::FiveHour.name(), "five_hour");
        assert_eq!(LimitKind::SevenDay.name(), "seven_day");
        assert_eq!(LimitKind::from_name("seven_day_opus"), None);
    }

    #[test]
    fn utilization_accepts_finite_non_negative_percentages() {
        assert_eq!(
            Utilization::from_percent(0.0).map(Utilization::percent),
            Some(0.0)
        );
        assert_eq!(
            Utilization::from_percent(13.5).map(Utilization::percent),
            Some(13.5)
        );
        assert_eq!(
            Utilization::from_percent(104.0).map(Utilization::percent),
            Some(104.0)
        );
    }

    #[test]
    fn utilization_rejects_negative_or_non_finite_values() {
        for percent in [-0.1, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(Utilization::from_percent(percent), None, "{percent}");
        }
    }
}
