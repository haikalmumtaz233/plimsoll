use serde_json::{Map, Value};

use crate::domain::clock::Timestamp;
use crate::domain::extras::{Credits, CreditsState, Currency, ModelLimit, ModelName, Money};
use crate::domain::limit::Utilization;
use crate::sources::rfc3339;

const WEEKLY_PREFIX: &str = "weekly_";
const WEEKLY_ALL: &str = "weekly_all";
const LEGACY_MODELS: [(&str, &str); 2] =
    [("seven_day_opus", "opus"), ("seven_day_sonnet", "sonnet")];
const MAX_MODELS: usize = 8;

#[must_use]
pub fn parse(body: &[u8]) -> (Vec<ModelLimit>, Option<Credits>) {
    let Ok(Value::Object(body)) = serde_json::from_slice::<Value>(body) else {
        return (Vec::new(), None);
    };
    (models(&body), credits(&body))
}

fn models(body: &Map<String, Value>) -> Vec<ModelLimit> {
    let mut found: Vec<ModelLimit> = Vec::new();
    let listed = body
        .get("limits")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(listed_model);
    let legacy = LEGACY_MODELS
        .into_iter()
        .filter_map(|(key, name)| legacy_model(body.get(key)?, name));
    for limit in listed.chain(legacy) {
        if found.len() < MAX_MODELS && found.iter().all(|known| known.model != limit.model) {
            found.push(limit);
        }
    }
    found
}

fn listed_model(entry: &Value) -> Option<ModelLimit> {
    let kind = entry.get("kind")?.as_str()?;
    let suffix = kind
        .strip_prefix(WEEKLY_PREFIX)
        .filter(|_| kind != WEEKLY_ALL)?;
    let name = entry
        .get("scope")
        .and_then(Value::as_str)
        .and_then(ModelName::parse)
        .or_else(|| ModelName::parse(suffix))?;
    Some(ModelLimit {
        model: name,
        utilization: Utilization::from_percent(entry.get("percent")?.as_f64()?)?,
        resets_at: reset_time(entry.get("resets_at")),
    })
}

fn legacy_model(window: &Value, name: &str) -> Option<ModelLimit> {
    Some(ModelLimit {
        model: ModelName::parse(name)?,
        utilization: Utilization::from_percent(window.get("utilization")?.as_f64()?)?,
        resets_at: reset_time(window.get("resets_at")),
    })
}

fn reset_time(value: Option<&Value>) -> Option<Timestamp> {
    value.and_then(Value::as_str).and_then(rfc3339::parse)
}

fn credits(body: &Map<String, Value>) -> Option<Credits> {
    let spend = body.get("spend").and_then(Value::as_object);
    let extra = body.get("extra_usage").and_then(Value::as_object);
    if spend.is_none() && extra.is_none() {
        return None;
    }
    let enabled = spend
        .and_then(|spend| flag(spend, "enabled"))
        .or_else(|| extra.and_then(|extra| flag(extra, "is_enabled")))
        .unwrap_or(false);
    let reason = spend
        .and_then(|spend| text(spend, "disabled_reason"))
        .or_else(|| extra.and_then(|extra| text(extra, "disabled_reason")));
    let state = credits_state(enabled, reason, extra);
    let used = spend
        .and_then(|spend| money(spend.get("used")?))
        .or_else(|| extra.and_then(|extra| legacy_money(extra, "used_credits")));
    let limit = spend
        .and_then(|spend| money(spend.get("limit")?))
        .or_else(|| extra.and_then(|extra| legacy_money(extra, "monthly_limit")));
    let utilization = spend
        .and_then(|spend| spend.get("percent")?.as_f64())
        .or_else(|| extra.and_then(|extra| extra.get("utilization")?.as_f64()))
        .and_then(Utilization::from_percent);
    let ever_enabled = extra
        .and_then(|extra| flag(extra, "credits_ever_enabled"))
        .unwrap_or(false);
    Some(Credits {
        state,
        used,
        limit,
        utilization,
        ever_enabled,
    })
}

fn credits_state(
    enabled: bool,
    reason: Option<&str>,
    extra: Option<&Map<String, Value>>,
) -> CreditsState {
    if enabled {
        return CreditsState::On;
    }
    let extra_flag = |key| extra.and_then(|extra| flag(extra, key)).unwrap_or(false);
    if extra_flag("user_disabled") {
        return CreditsState::TurnedOff;
    }
    if extra_flag("spend_limit_reached") {
        return CreditsState::LimitReached;
    }
    match reason {
        Some("out_of_credits") => CreditsState::OutOfCredits,
        Some("spend_limit_reached" | "limit_reached") => CreditsState::LimitReached,
        Some("user_disabled") => CreditsState::TurnedOff,
        _ => CreditsState::Off,
    }
}

fn money(value: &Value) -> Option<Money> {
    let currency = Currency::parse(value.get("currency")?.as_str()?)?;
    let exponent = u8::try_from(whole(value.get("exponent")?)?).ok()?;
    Money::new(whole(value.get("amount_minor")?)?, exponent, currency)
}

fn legacy_money(extra: &Map<String, Value>, key: &str) -> Option<Money> {
    let currency = Currency::parse(text(extra, "currency")?)?;
    let exponent = u8::try_from(whole(extra.get("decimal_places")?)?).ok()?;
    Money::new(whole(extra.get(key)?)?, exponent, currency)
}

fn whole(value: &Value) -> Option<i64> {
    value.as_i64().or_else(|| {
        let number = value.as_f64()?;
        let integral = number.fract() == 0.0 && number.abs() < 9.0e15;
        #[allow(clippy::cast_possible_truncation)]
        integral.then_some(number as i64)
    })
}

fn flag(object: &Map<String, Value>, key: &str) -> Option<bool> {
    object.get(key)?.as_bool()
}

fn text<'a>(object: &'a Map<String, Value>, key: &str) -> Option<&'a str> {
    object.get(key)?.as_str()
}

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::domain::extras::{CreditsState, Money};
    use crate::domain::limit::Utilization;
    use serde_json::{Value, json};

    fn spike_body() -> Value {
        json!({
            "seven_day": { "utilization": 97, "resets_at": "2026-10-04T23:59:59.721175+00:00" },
            "seven_day_opus": null,
            "seven_day_sonnet": null,
            "extra_usage": {
                "is_enabled": false, "monthly_limit": 2000, "used_credits": 1750,
                "utilization": 0, "currency": "USD", "decimal_places": 2,
                "disabled_reason": "out_of_credits", "user_disabled": false,
                "spend_limit_reached": false, "credits_ever_enabled": true,
                "daily": null, "weekly": null
            },
            "limits": [
                { "kind": "session", "group": "session", "percent": 6, "severity": "normal",
                  "resets_at": "2026-10-04T10:49:59.721155+00:00", "scope": null, "is_active": false },
                { "kind": "weekly_all", "group": "weekly", "percent": 97, "severity": "critical",
                  "resets_at": "2026-10-04T23:59:59.721175+00:00", "scope": null, "is_active": true }
            ],
            "spend": {
                "used": { "amount_minor": 1750, "currency": "USD", "exponent": 2 },
                "limit": { "amount_minor": 2000, "currency": "USD", "exponent": 2 },
                "percent": 0, "severity": "normal", "enabled": false,
                "disabled_reason": "out_of_credits",
                "cap": { "money": null, "credits": { "amount_minor": 0, "exponent": 2 } },
                "balance": null, "auto_reload": null, "disclaimer": "text",
                "can_purchase_credits": false, "can_toggle": false
            }
        })
    }

    fn bytes(body: &Value) -> Vec<u8> {
        body.to_string().into_bytes()
    }

    fn amount(money: Option<&Money>) -> Option<(i64, u8, String)> {
        money.map(|money| {
            (
                money.minor,
                money.exponent,
                money.currency.as_str().to_owned(),
            )
        })
    }

    #[test]
    fn a_pro_account_has_no_model_limits_and_spent_credits() {
        let (models, credits) = parse(&bytes(&spike_body()));
        assert!(models.is_empty(), "{models:?}");
        let credits = credits.expect("credits");
        assert_eq!(credits.state, CreditsState::OutOfCredits);
        assert_eq!(
            amount(credits.used.as_ref()),
            Some((1_750, 2, "USD".to_owned()))
        );
        assert_eq!(
            amount(credits.limit.as_ref()),
            Some((2_000, 2, "USD".to_owned()))
        );
        assert!(credits.ever_enabled);
        assert!(credits.is_relevant());
    }

    #[test]
    fn reads_per_model_weekly_limits_from_the_limits_list() {
        let mut body = spike_body();
        body["limits"] = json!([
            { "kind": "weekly_all", "percent": 50 },
            { "kind": "weekly_opus", "percent": 81, "resets_at": "2026-10-05T00:00:00Z" },
            { "kind": "weekly_scoped", "percent": 12, "scope": "Sonnet" },
            { "kind": "weekly_", "percent": 3 },
            { "kind": "session", "percent": 9 },
            { "kind": "weekly_broken", "percent": "a lot" }
        ]);
        let (models, _) = parse(&bytes(&body));
        let names: Vec<(&str, f64, bool)> = models
            .iter()
            .map(|limit| {
                (
                    limit.model.as_str(),
                    limit.utilization.percent(),
                    limit.resets_at.is_some(),
                )
            })
            .collect();
        assert_eq!(names, vec![("opus", 81.0, true), ("sonnet", 12.0, false)]);
    }

    #[test]
    fn falls_back_to_legacy_model_windows() {
        let mut body = spike_body();
        body["seven_day_opus"] =
            json!({ "utilization": 44.5, "resets_at": "2026-10-05T00:00:00Z" });
        body["seven_day_sonnet"] = json!({ "utilization": 10 });
        body["limits"] = json!([{ "kind": "weekly_sonnet", "percent": 11 }]);
        let (models, _) = parse(&bytes(&body));
        let names: Vec<(&str, f64)> = models
            .iter()
            .map(|limit| (limit.model.as_str(), limit.utilization.percent()))
            .collect();
        assert_eq!(names, vec![("sonnet", 11.0), ("opus", 44.5)]);
    }

    #[test]
    fn enabled_credits_report_their_spend() {
        let mut body = spike_body();
        body["spend"]["enabled"] = json!(true);
        body["spend"]["percent"] = json!(35.5);
        body["spend"]["disabled_reason"] = Value::Null;
        let credits = parse(&bytes(&body)).1.expect("credits");
        assert_eq!(credits.state, CreditsState::On);
        assert_eq!(credits.utilization.map(Utilization::percent), Some(35.5));
    }

    #[test]
    fn uses_extra_usage_when_spend_is_missing() {
        let mut body = spike_body();
        body["spend"] = Value::Null;
        body["extra_usage"]["is_enabled"] = json!(true);
        let credits = parse(&bytes(&body)).1.expect("credits");
        assert_eq!(credits.state, CreditsState::On);
        assert_eq!(
            amount(credits.used.as_ref()),
            Some((1_750, 2, "USD".to_owned()))
        );
    }

    #[test]
    fn explains_why_credits_are_off() {
        let mut body = spike_body();
        body["extra_usage"]["user_disabled"] = json!(true);
        assert_eq!(
            parse(&bytes(&body)).1.map(|credits| credits.state),
            Some(CreditsState::TurnedOff)
        );
        let mut body = spike_body();
        body["extra_usage"]["spend_limit_reached"] = json!(true);
        assert_eq!(
            parse(&bytes(&body)).1.map(|credits| credits.state),
            Some(CreditsState::LimitReached)
        );
        let mut body = spike_body();
        body["spend"]["disabled_reason"] = json!("something_new");
        body["extra_usage"]["disabled_reason"] = json!("something_new");
        assert_eq!(
            parse(&bytes(&body)).1.map(|credits| credits.state),
            Some(CreditsState::Off)
        );
    }

    #[test]
    fn rejects_untrusted_money_values() {
        let mut body = spike_body();
        body["spend"]["used"] = json!({ "amount_minor": -5, "currency": "USD", "exponent": 2 });
        body["spend"]["limit"] = json!({ "amount_minor": 10, "currency": "<b>", "exponent": 2 });
        body["extra_usage"] = Value::Null;
        let credits = parse(&bytes(&body)).1.expect("credits");
        assert_eq!(credits.used, None);
        assert_eq!(credits.limit, None);
    }

    #[test]
    fn bodies_without_extras_yield_nothing() {
        for body in [
            json!({}),
            json!([1]),
            json!({ "spend": "x", "extra_usage": 3 }),
        ] {
            let (models, credits) = parse(&bytes(&body));
            assert!(models.is_empty(), "{body}");
            assert_eq!(credits, None, "{body}");
        }
        assert_eq!(parse(b"not json").1, None);
    }
}
