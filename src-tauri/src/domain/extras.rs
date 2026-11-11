use super::clock::Timestamp;
use super::limit::{LimitSnapshot, STALE_AFTER, Utilization};

const MAX_MODEL_NAME: usize = 32;
const MODEL_NAME_PUNCTUATION: &[u8] = b" _-.()";
const MAX_EXPONENT: u8 = 6;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ModelName(String);

impl ModelName {
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let name = text.trim();
        if name.is_empty() || name.len() > MAX_MODEL_NAME || !name.bytes().all(is_label_byte) {
            return None;
        }
        Some(Self(name.to_owned()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[must_use]
    pub fn matches(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
    }
}

fn is_label_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || MODEL_NAME_PUNCTUATION.contains(&byte)
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModelLimit {
    pub model: ModelName,
    pub utilization: Utilization,
    pub resets_at: Option<Timestamp>,
}

impl ModelLimit {
    #[must_use]
    pub fn is_current(&self, observed_at: Timestamp, now: Timestamp) -> bool {
        now - observed_at <= STALE_AFTER && self.resets_at.is_none_or(|resets_at| resets_at > now)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Currency(String);

impl Currency {
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        (text.len() == 3 && text.bytes().all(|byte| byte.is_ascii_uppercase()))
            .then(|| Self(text.to_owned()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Money {
    pub minor: i64,
    pub exponent: u8,
    pub currency: Currency,
}

impl Money {
    #[must_use]
    pub fn new(minor: i64, exponent: u8, currency: Currency) -> Option<Self> {
        (minor >= 0 && exponent <= MAX_EXPONENT).then_some(Self {
            minor,
            exponent,
            currency,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreditsState {
    On,
    OutOfCredits,
    LimitReached,
    TurnedOff,
    Off,
}

impl CreditsState {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::On => "on",
            Self::OutOfCredits => "out_of_credits",
            Self::LimitReached => "limit_reached",
            Self::TurnedOff => "turned_off",
            Self::Off => "off",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Credits {
    pub state: CreditsState,
    pub used: Option<Money>,
    pub limit: Option<Money>,
    pub utilization: Option<Utilization>,
    pub ever_enabled: bool,
}

impl Credits {
    #[must_use]
    pub fn is_relevant(&self) -> bool {
        self.state == CreditsState::On
            || self.ever_enabled
            || self.used.as_ref().is_some_and(|used| used.minor > 0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OfficialUsage {
    pub limits: Vec<LimitSnapshot>,
    pub models: Vec<ModelLimit>,
    pub credits: Option<Credits>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Extras {
    pub models: Vec<ModelLimit>,
    pub credits: Option<Credits>,
    pub observed_at: Timestamp,
}

impl Extras {
    #[must_use]
    pub fn current_models(&self, now: Timestamp) -> Vec<ModelLimit> {
        self.models
            .iter()
            .filter(|limit| limit.is_current(self.observed_at, now))
            .cloned()
            .collect()
    }

    #[must_use]
    pub fn current_credits(&self, now: Timestamp) -> Option<Credits> {
        self.credits
            .clone()
            .filter(|credits| now - self.observed_at <= STALE_AFTER && credits.is_relevant())
    }
}

#[cfg(test)]
mod tests {
    use super::{Credits, CreditsState, Currency, Extras, ModelLimit, ModelName, Money};
    use crate::domain::clock::{Span, Timestamp};
    use crate::domain::limit::{STALE_AFTER, Utilization};

    const NOW: Timestamp = Timestamp::from_unix_millis(1_790_300_000_000);

    fn usd(minor: i64) -> Money {
        Money::new(minor, 2, Currency::parse("USD").expect("currency")).expect("money")
    }

    fn opus(resets_at: Option<Timestamp>) -> ModelLimit {
        ModelLimit {
            model: ModelName::parse("opus").expect("name"),
            utilization: Utilization::from_percent(40.0).expect("percent"),
            resets_at,
        }
    }

    fn credits(state: CreditsState, used: i64, ever_enabled: bool) -> Credits {
        Credits {
            state,
            used: Some(usd(used)),
            limit: Some(usd(2_000)),
            utilization: None,
            ever_enabled,
        }
    }

    #[test]
    fn model_names_keep_the_server_label() {
        let label = |text: &str| ModelName::parse(text).map(|name| name.as_str().to_owned());
        assert_eq!(label(" Fable "), Some("Fable".to_owned()));
        assert_eq!(label("Opus 4.5 (1M)"), Some("Opus 4.5 (1M)".to_owned()));
        assert_eq!(label("sonnet-4_5"), Some("sonnet-4_5".to_owned()));
        assert_eq!(label(&"a".repeat(32)), Some("a".repeat(32)));
    }

    #[test]
    fn model_names_reject_empty_long_and_markup_text() {
        assert_eq!(ModelName::parse(""), None);
        assert_eq!(ModelName::parse("   "), None);
        assert_eq!(ModelName::parse("<script>"), None);
        assert_eq!(ModelName::parse("line\nbreak"), None);
        assert_eq!(ModelName::parse("Fäble"), None);
        assert_eq!(ModelName::parse(&"a".repeat(33)), None);
    }

    #[test]
    fn model_names_match_regardless_of_case() {
        let fable = ModelName::parse("Fable").expect("name");
        assert!(fable.matches(&ModelName::parse("fable").expect("name")));
        assert!(!fable.matches(&ModelName::parse("Opus").expect("name")));
    }

    #[test]
    fn currencies_are_three_uppercase_letters() {
        assert!(Currency::parse("USD").is_some());
        assert!(Currency::parse("usd").is_none());
        assert!(Currency::parse("US").is_none());
        assert!(Currency::parse("USDT").is_none());
    }

    #[test]
    fn money_rejects_negative_amounts_and_odd_exponents() {
        let currency = Currency::parse("EUR").expect("currency");
        assert!(Money::new(0, 2, currency.clone()).is_some());
        assert!(Money::new(-1, 2, currency.clone()).is_none());
        assert!(Money::new(5, 7, currency).is_none());
    }

    #[test]
    fn credits_matter_once_they_have_ever_been_used() {
        assert!(credits(CreditsState::On, 0, false).is_relevant());
        assert!(credits(CreditsState::OutOfCredits, 0, true).is_relevant());
        assert!(credits(CreditsState::Off, 150, false).is_relevant());
        assert!(!credits(CreditsState::Off, 0, false).is_relevant());
        assert_eq!(CreditsState::OutOfCredits.name(), "out_of_credits");
    }

    #[test]
    fn extras_expire_with_the_official_reading() {
        let extras = Extras {
            models: vec![opus(Some(NOW + Span::days(2))), opus(Some(NOW))],
            credits: Some(credits(CreditsState::On, 100, true)),
            observed_at: NOW - Span::minutes(5),
        };
        assert_eq!(extras.current_models(NOW).len(), 1);
        assert!(extras.current_credits(NOW).is_some());
        let stale = Extras {
            observed_at: NOW - STALE_AFTER - Span::minutes(1),
            ..extras
        };
        assert!(stale.current_models(NOW).is_empty(), "{stale:?}");
        assert_eq!(stale.current_credits(NOW), None);
    }

    #[test]
    fn irrelevant_credits_stay_hidden() {
        let extras = Extras {
            models: Vec::new(),
            credits: Some(credits(CreditsState::Off, 0, false)),
            observed_at: NOW,
        };
        assert_eq!(extras.current_credits(NOW), None);
    }
}
