use std::time::Duration;

use super::severity::Thresholds;

const SECONDS_PER_MINUTE: u64 = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PollInterval(u8);

impl PollInterval {
    pub const CHOICES: [u8; 4] = [1, 2, 5, 10];
    pub const DEFAULT: Self = Self(1);

    #[must_use]
    pub fn from_minutes(minutes: u8) -> Option<Self> {
        Self::CHOICES.contains(&minutes).then_some(Self(minutes))
    }

    #[must_use]
    pub const fn minutes(self) -> u8 {
        self.0
    }

    #[must_use]
    pub fn duration(self) -> Duration {
        Duration::from_secs(u64::from(self.0) * SECONDS_PER_MINUTE)
    }
}

impl Default for PollInterval {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Preferences {
    pub thresholds: Thresholds,
    pub poll_interval: PollInterval,
}

#[cfg(test)]
mod tests {
    use super::{PollInterval, Preferences};
    use crate::domain::severity::Thresholds;
    use std::time::Duration;

    #[test]
    fn only_listed_minutes_are_valid_intervals() {
        for minutes in PollInterval::CHOICES {
            assert_eq!(
                PollInterval::from_minutes(minutes).map(PollInterval::minutes),
                Some(minutes)
            );
        }
        for minutes in [0, 3, 30, u8::MAX] {
            assert_eq!(PollInterval::from_minutes(minutes), None);
        }
    }

    #[test]
    fn intervals_convert_to_durations() {
        assert_eq!(PollInterval::DEFAULT.duration(), Duration::from_secs(60));
        assert_eq!(
            PollInterval::from_minutes(10).map(PollInterval::duration),
            Some(Duration::from_secs(600))
        );
    }

    #[test]
    fn default_preferences_match_the_documented_defaults() {
        let preferences = Preferences::default();
        assert_eq!(preferences.thresholds, Thresholds::DEFAULT);
        assert_eq!(preferences.poll_interval, PollInterval::DEFAULT);
    }
}
