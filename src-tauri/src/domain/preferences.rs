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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    English,
    Indonesian,
}

impl Language {
    pub const ALL: [Self; 2] = [Self::English, Self::Indonesian];

    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::Indonesian => "id",
        }
    }

    #[must_use]
    pub fn from_tag(tag: &str) -> Option<Self> {
        let primary = tag.split(['-', '_']).next()?.to_ascii_lowercase();
        Self::ALL
            .into_iter()
            .find(|language| language.code() == primary)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum LanguageChoice {
    #[default]
    System,
    Fixed(Language),
}

impl LanguageChoice {
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Fixed(language) => language.code(),
        }
    }

    #[must_use]
    pub fn from_code(code: &str) -> Option<Self> {
        if code == Self::System.code() {
            return Some(Self::System);
        }
        Language::ALL
            .into_iter()
            .find(|language| language.code() == code)
            .map(Self::Fixed)
    }

    #[must_use]
    pub fn resolve(self, system_tag: Option<&str>) -> Language {
        match self {
            Self::Fixed(language) => language,
            Self::System => system_tag
                .and_then(Language::from_tag)
                .unwrap_or(Language::English),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Preferences {
    pub thresholds: Thresholds,
    pub poll_interval: PollInterval,
    pub language: LanguageChoice,
}

#[cfg(test)]
mod tests {
    use super::{Language, LanguageChoice, PollInterval, Preferences};
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
        assert_eq!(preferences.language, LanguageChoice::System);
    }

    #[test]
    fn language_choices_round_trip_through_codes() {
        for choice in [
            LanguageChoice::System,
            LanguageChoice::Fixed(Language::English),
            LanguageChoice::Fixed(Language::Indonesian),
        ] {
            assert_eq!(LanguageChoice::from_code(choice.code()), Some(choice));
        }
        assert_eq!(LanguageChoice::from_code("fr"), None);
    }

    #[test]
    fn system_language_follows_the_primary_tag_with_an_english_fallback() {
        let system = LanguageChoice::System;
        assert_eq!(system.resolve(Some("id-ID")), Language::Indonesian);
        assert_eq!(system.resolve(Some("in_ID")), Language::English);
        assert_eq!(system.resolve(Some("EN-us")), Language::English);
        assert_eq!(system.resolve(Some("fr-FR")), Language::English);
        assert_eq!(system.resolve(None), Language::English);
        assert_eq!(
            LanguageChoice::Fixed(Language::Indonesian).resolve(Some("en-US")),
            Language::Indonesian
        );
    }
}
