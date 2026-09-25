use super::limit::Utilization;

pub const ELEVATED_PERCENT: f64 = 50.0;
pub const HIGH_PERCENT: f64 = 80.0;
pub const CRITICAL_PERCENT: f64 = 95.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    Normal,
    Elevated,
    High,
    Critical,
}

impl Severity {
    #[must_use]
    pub fn of(utilization: Utilization) -> Self {
        let percent = utilization.percent();
        if percent >= CRITICAL_PERCENT {
            Self::Critical
        } else if percent >= HIGH_PERCENT {
            Self::High
        } else if percent >= ELEVATED_PERCENT {
            Self::Elevated
        } else {
            Self::Normal
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Severity;
    use crate::domain::limit::Utilization;

    fn severity(percent: f64) -> Severity {
        Severity::of(Utilization::from_percent(percent).expect("valid percent"))
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
    fn severities_are_ordered_by_urgency() {
        assert!(Severity::Normal < Severity::Elevated);
        assert!(Severity::Elevated < Severity::High);
        assert!(Severity::High < Severity::Critical);
    }
}
