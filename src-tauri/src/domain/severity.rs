use super::limit::Utilization;

pub const MAX_PERCENT: u8 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Thresholds {
    elevated: u8,
    high: u8,
    critical: u8,
}

impl Thresholds {
    pub const DEFAULT: Self = Self {
        elevated: 50,
        high: 80,
        critical: 95,
    };

    #[must_use]
    pub const fn new(elevated: u8, high: u8, critical: u8) -> Option<Self> {
        if 0 < elevated && elevated < high && high < critical && critical <= MAX_PERCENT {
            Some(Self {
                elevated,
                high,
                critical,
            })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn elevated(self) -> u8 {
        self.elevated
    }

    #[must_use]
    pub const fn high(self) -> u8 {
        self.high
    }

    #[must_use]
    pub const fn critical(self) -> u8 {
        self.critical
    }
}

impl Default for Thresholds {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    Normal,
    Elevated,
    High,
    Critical,
}

impl Severity {
    pub const ALL: [Self; 4] = [Self::Normal, Self::Elevated, Self::High, Self::Critical];

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Elevated => "elevated",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }

    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|severity| severity.name() == name)
    }

    #[must_use]
    pub fn of(utilization: Utilization, thresholds: Thresholds) -> Self {
        let percent = utilization.percent();
        if percent >= f64::from(thresholds.critical) {
            Self::Critical
        } else if percent >= f64::from(thresholds.high) {
            Self::High
        } else if percent >= f64::from(thresholds.elevated) {
            Self::Elevated
        } else {
            Self::Normal
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Severity, Thresholds};
    use crate::domain::limit::Utilization;

    fn severity(percent: f64) -> Severity {
        Severity::of(
            Utilization::from_percent(percent).expect("valid percent"),
            Thresholds::DEFAULT,
        )
    }

    #[test]
    fn thresholds_are_inclusive_lower_bounds() {
        assert_eq!(severity(0.0), Severity::Normal);
        assert_eq!(severity(49.9), Severity::Normal);
        assert_eq!(severity(50.0), Severity::Elevated);
        assert_eq!(severity(79.9), Severity::Elevated);
        assert_eq!(severity(80.0), Severity::High);
        assert_eq!(severity(94.9), Severity::High);
        assert_eq!(severity(95.0), Severity::Critical);
        assert_eq!(severity(130.0), Severity::Critical);
    }

    #[test]
    fn custom_thresholds_move_the_boundaries() {
        let thresholds = Thresholds::new(20, 40, 60).expect("valid thresholds");
        let at = |percent: f64| {
            Severity::of(
                Utilization::from_percent(percent).expect("valid percent"),
                thresholds,
            )
        };
        assert_eq!(at(19.0), Severity::Normal);
        assert_eq!(at(20.0), Severity::Elevated);
        assert_eq!(at(40.0), Severity::High);
        assert_eq!(at(60.0), Severity::Critical);
    }

    #[test]
    fn thresholds_must_rise_within_one_to_one_hundred() {
        assert_eq!(Thresholds::new(50, 80, 95), Some(Thresholds::DEFAULT));
        assert_eq!(Thresholds::default(), Thresholds::DEFAULT);
        assert!(Thresholds::new(1, 2, 100).is_some());
        for (elevated, high, critical) in [(0, 80, 95), (80, 80, 95), (50, 96, 95), (50, 80, 101)] {
            assert_eq!(Thresholds::new(elevated, high, critical), None);
        }
        let custom = Thresholds::new(10, 20, 30).expect("valid thresholds");
        assert_eq!(
            (custom.elevated(), custom.high(), custom.critical()),
            (10, 20, 30)
        );
    }

    #[test]
    fn severity_names_round_trip() {
        for severity in Severity::ALL {
            assert_eq!(Severity::from_name(severity.name()), Some(severity));
        }
        assert_eq!(Severity::from_name("urgent"), None);
    }

    #[test]
    fn severities_are_ordered_by_urgency() {
        assert!(Severity::Normal < Severity::Elevated);
        assert!(Severity::Elevated < Severity::High);
        assert!(Severity::High < Severity::Critical);
    }
}
