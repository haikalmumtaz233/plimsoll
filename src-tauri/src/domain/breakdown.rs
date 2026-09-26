use std::collections::HashMap;

use super::clock::{Span, Timestamp};
use super::history;
use super::period::Window;
use super::record::UsageEvent;

pub const TOP_ENTRIES: usize = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Share {
    pub name: String,
    pub tokens: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Ranking {
    pub top: Vec<Share>,
    pub other: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Breakdown {
    pub models: Ranking,
    pub projects: Ranking,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Breakdowns {
    pub day: Breakdown,
    pub week: Breakdown,
}

#[must_use]
pub fn recent(events: &[UsageEvent], now: Timestamp) -> Breakdowns {
    let week = history::window(now);
    let day = Window::ending_at(week.end(), Span::days(1));
    Breakdowns {
        day: breakdown(events, day),
        week: breakdown(events, week),
    }
}

#[must_use]
pub fn breakdown(events: &[UsageEvent], window: Window) -> Breakdown {
    let inside: Vec<&UsageEvent> = events
        .iter()
        .filter(|event| window.contains(event.at))
        .collect();
    Breakdown {
        models: rank(inside.iter().map(|event| (&event.model, *event))),
        projects: rank(inside.iter().map(|event| (&event.project, *event))),
    }
}

fn rank<'a, I>(entries: I) -> Ranking
where
    I: Iterator<Item = (&'a String, &'a UsageEvent)>,
{
    let mut totals: HashMap<&str, u64> = HashMap::new();
    for (name, event) in entries {
        let tokens = event.tokens.excluding_cache_reads();
        if tokens > 0 {
            let total = totals.entry(name.as_str()).or_default();
            *total = total.saturating_add(tokens);
        }
    }
    let mut shares: Vec<Share> = totals
        .into_iter()
        .map(|(name, tokens)| Share {
            name: name.to_owned(),
            tokens,
        })
        .collect();
    shares.sort_by(|first, second| {
        second
            .tokens
            .cmp(&first.tokens)
            .then_with(|| first.name.cmp(&second.name))
    });
    let rest = shares.split_off(shares.len().min(TOP_ENTRIES));
    Ranking {
        top: shares,
        other: rest
            .iter()
            .fold(0_u64, |sum, share| sum.saturating_add(share.tokens)),
    }
}

#[cfg(test)]
mod tests {
    use super::{Share, TOP_ENTRIES, breakdown, recent};
    use crate::domain::clock::{Span, Timestamp};
    use crate::domain::period::Window;
    use crate::domain::record::UsageEvent;
    use crate::domain::tokens::TokenCounts;

    const HOUR: i64 = 3_600_000;
    const NOW: Timestamp = Timestamp::from_unix_millis(1_790_300_000_000);

    fn event(hours_ago: i64, model: &str, project: &str, output: u64) -> UsageEvent {
        UsageEvent {
            at: NOW - Span::from_millis(hours_ago * HOUR),
            model: model.to_owned(),
            project: project.to_owned(),
            tokens: TokenCounts {
                output,
                cache_read: 1_000_000,
                ..TokenCounts::default()
            },
        }
    }

    fn share(name: &str, tokens: u64) -> Share {
        Share {
            name: name.to_owned(),
            tokens,
        }
    }

    fn everything() -> Window {
        Window::ending_at(NOW + Span::hours(1), Span::days(30))
    }

    #[test]
    fn ranks_models_and_projects_by_tokens_without_cache_reads() {
        let events = [
            event(1, "claude-opus-5", "plimsoll", 30),
            event(2, "claude-sonnet-5", "plimsoll", 50),
            event(3, "claude-opus-5", "website", 40),
        ];
        let result = breakdown(&events, everything());
        assert_eq!(
            result.models.top,
            vec![share("claude-opus-5", 70), share("claude-sonnet-5", 50)]
        );
        assert_eq!(
            result.projects.top,
            vec![share("plimsoll", 80), share("website", 40)]
        );
        assert_eq!(result.models.other, 0);
    }

    #[test]
    fn folds_everything_past_the_top_entries_into_other() {
        let events: Vec<UsageEvent> = (0..6)
            .map(|index| event(1, &format!("model-{index}"), "p", 10 + index))
            .collect();
        let models = breakdown(&events, everything()).models;
        assert_eq!(models.top.len(), TOP_ENTRIES);
        assert_eq!(models.top[0], share("model-5", 15));
        assert_eq!(models.other, 10 + 11);
    }

    #[test]
    fn ties_are_ordered_by_name_and_empty_usage_is_skipped() {
        let events = [
            event(1, "b", "p", 5),
            event(1, "a", "p", 5),
            event(1, "c", "p", 0),
        ];
        assert_eq!(
            breakdown(&events, everything()).models.top,
            vec![share("a", 5), share("b", 5)]
        );
    }

    #[test]
    fn recent_splits_the_last_day_from_the_last_week() {
        let events = [
            event(2, "claude-opus-5", "plimsoll", 10),
            event(30, "claude-opus-5", "plimsoll", 20),
            event(24 * 8, "claude-opus-5", "plimsoll", 40),
        ];
        let result = recent(&events, NOW);
        assert_eq!(result.day.models.top, vec![share("claude-opus-5", 10)]);
        assert_eq!(result.week.models.top, vec![share("claude-opus-5", 30)]);
    }
}
