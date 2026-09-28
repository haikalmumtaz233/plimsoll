const TIER_SEPARATOR: char = '_';
const MULTIPLIER_SUFFIX: char = 'x';

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanKind {
    Free,
    Pro,
    Max,
    Team,
    Enterprise,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Plan {
    pub kind: PlanKind,
    pub multiplier: Option<u8>,
}

impl PlanKind {
    pub const ALL: [Self; 5] = [
        Self::Free,
        Self::Pro,
        Self::Max,
        Self::Team,
        Self::Enterprise,
    ];

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Free => "Free",
            Self::Pro => "Pro",
            Self::Max => "Max",
            Self::Team => "Team",
            Self::Enterprise => "Enterprise",
        }
    }

    #[must_use]
    pub fn from_subscription_type(value: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|kind| kind.name().eq_ignore_ascii_case(value))
    }
}

impl Plan {
    #[must_use]
    pub fn from_fields(subscription_type: &str, rate_limit_tier: Option<&str>) -> Option<Self> {
        let kind = PlanKind::from_subscription_type(subscription_type)?;
        let multiplier = match kind {
            PlanKind::Max => rate_limit_tier.and_then(tier_multiplier),
            _ => None,
        };
        Some(Self { kind, multiplier })
    }

    #[must_use]
    pub fn label(self) -> String {
        match self.multiplier {
            Some(multiplier) => format!("{} {multiplier}{MULTIPLIER_SUFFIX}", self.kind.name()),
            None => self.kind.name().to_owned(),
        }
    }
}

fn tier_multiplier(tier: &str) -> Option<u8> {
    tier.rsplit(TIER_SEPARATOR)
        .next()?
        .strip_suffix(MULTIPLIER_SUFFIX)?
        .parse()
        .ok()
        .filter(|multiplier| *multiplier > 0)
}

#[cfg(test)]
mod tests {
    use super::{Plan, PlanKind};

    #[test]
    fn known_subscription_types_map_to_plans() {
        let cases = [
            ("pro", PlanKind::Pro),
            ("Max", PlanKind::Max),
            ("free", PlanKind::Free),
            ("team", PlanKind::Team),
            ("enterprise", PlanKind::Enterprise),
        ];
        for (value, kind) in cases {
            assert_eq!(
                Plan::from_fields(value, None).map(|plan| plan.kind),
                Some(kind),
                "{value}"
            );
        }
        for value in ["", "platinum", "pro\n", "max max"] {
            assert_eq!(Plan::from_fields(value, None), None, "{value:?}");
        }
    }

    #[test]
    fn max_plans_carry_the_rate_limit_multiplier() {
        let plan = Plan::from_fields("max", Some("default_claude_max_20x")).expect("plan");
        assert_eq!(plan.multiplier, Some(20));
        assert_eq!(plan.label(), "Max 20x");
        assert_eq!(
            Plan::from_fields("max", Some("default_claude_max_5x")).map(Plan::label),
            Some("Max 5x".to_owned())
        );
        for tier in [
            None,
            Some(""),
            Some("default_claude_ai"),
            Some("max_x"),
            Some("max_999x"),
        ] {
            assert_eq!(
                Plan::from_fields("max", tier).map(Plan::label),
                Some("Max".to_owned()),
                "{tier:?}"
            );
        }
    }

    #[test]
    fn multipliers_are_ignored_outside_max() {
        let plan = Plan::from_fields("pro", Some("default_claude_max_20x")).expect("plan");
        assert_eq!(plan.multiplier, None);
        assert_eq!(plan.label(), "Pro");
        assert_eq!(
            Plan::from_fields("enterprise", None).map(Plan::label),
            Some("Enterprise".to_owned())
        );
    }
}
