use super::clock::Timestamp;

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

#[cfg(test)]
mod tests {
    use super::{LimitKind, Utilization};

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
