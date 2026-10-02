use super::aggregate::totals_in;
use super::breakdown::{self, Breakdowns};
use super::clock::{Span, Timestamp};
use super::history::{self, HourlyHistory};
use super::limit::{LimitKind, LimitSnapshot};
use super::period::{
    DEFAULT_WEEKLY_ANCHOR, Window, five_hour_from_reset, weekly_containing, weekly_from_reset,
};
use super::record::UsageEvent;
use super::session::infer_five_hour;
use super::tokens::TokenCounts;

pub const INFERENCE_LOOKBACK: Span = Span::days(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenWindow {
    pub window: Option<Window>,
    pub tokens: TokenCounts,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageSummary {
    pub limits: Vec<LimitSnapshot>,
    pub five_hour: TokenWindow,
    pub weekly: TokenWindow,
    pub history: HourlyHistory,
    pub breakdowns: Breakdowns,
}

#[must_use]
pub fn lookback(limits: &[LimitSnapshot], now: Timestamp) -> Window {
    let start = weekly_window(limits, now)
        .start()
        .min(now - INFERENCE_LOOKBACK)
        .min(history::window(now).start());
    Window::starting_at(start, (now + Span::from_millis(1)) - start)
}

#[must_use]
pub fn summarize(
    limits: Vec<LimitSnapshot>,
    events: &[UsageEvent],
    now: Timestamp,
) -> UsageSummary {
    let five_hour = five_hour_window(&limits, events, now);
    let weekly = weekly_window(&limits, now);
    UsageSummary {
        five_hour: tokens_in(five_hour, events),
        weekly: tokens_in(Some(weekly), events),
        history: history::hourly(events, now),
        breakdowns: breakdown::recent(events, now),
        limits,
    }
}

fn five_hour_window(
    limits: &[LimitSnapshot],
    events: &[UsageEvent],
    now: Timestamp,
) -> Option<Window> {
    reset_of(limits, LimitKind::FiveHour)
        .map(five_hour_from_reset)
        .or_else(|| {
            let activity: Vec<Timestamp> = events.iter().map(|event| event.at).collect();
            infer_five_hour(&activity, now)
        })
}

fn weekly_window(limits: &[LimitSnapshot], now: Timestamp) -> Window {
    reset_of(limits, LimitKind::SevenDay).map_or_else(
        || weekly_containing(DEFAULT_WEEKLY_ANCHOR, now),
        weekly_from_reset,
    )
}

fn reset_of(limits: &[LimitSnapshot], kind: LimitKind) -> Option<Timestamp> {
    limits
        .iter()
        .find(|limit| limit.kind == kind)
        .and_then(|limit| limit.resets_at)
}

fn tokens_in(window: Option<Window>, events: &[UsageEvent]) -> TokenWindow {
    TokenWindow {
        window,
        tokens: window.map_or_else(TokenCounts::default, |window| {
            totals_in(window, events).tokens
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::{INFERENCE_LOOKBACK, lookback, summarize};
    use crate::domain::clock::{Span, Timestamp};
    use crate::domain::history;
    use crate::domain::limit::{LimitKind, LimitSnapshot, Utilization};
    use crate::domain::period::{DEFAULT_WEEKLY_ANCHOR, Window, weekly_containing};
    use crate::domain::record::UsageEvent;
    use crate::domain::tokens::TokenCounts;

    const HOUR: i64 = 3_600_000;
    const NOW: Timestamp = Timestamp::from_unix_millis(1_790_300_000_000);

    fn event(hours_ago: i64, output: u64) -> UsageEvent {
        UsageEvent {
            at: NOW - Span::from_millis(hours_ago * HOUR),
            model: "claude-opus-5".to_owned(),
            project: "plimsoll".to_owned(),
            tokens: TokenCounts {
                output,
                ..TokenCounts::default()
            },
        }
    }

    fn limit(kind: LimitKind, resets_in_hours: i64) -> LimitSnapshot {
        LimitSnapshot {
            kind,
            utilization: Utilization::from_percent(30.0).expect("valid percent"),
            resets_at: Some(NOW + Span::from_millis(resets_in_hours * HOUR)),
        }
    }

    #[test]
    fn without_limits_the_session_is_inferred_from_activity() {
        let events = [event(8, 1), event(3, 10), event(1, 100)];
        let summary = summarize(Vec::new(), &events, NOW);
        assert_eq!(
            summary.five_hour.window.map(Window::start),
            Some(NOW - Span::hours(3))
        );
        assert_eq!(summary.five_hour.tokens.output, 110);
        assert_eq!(summary.history.tokens.iter().sum::<u64>(), 111);
        assert_eq!(summary.breakdowns.week.models.top.len(), 1);
        assert_eq!(
            summary.weekly.window,
            Some(weekly_containing(DEFAULT_WEEKLY_ANCHOR, NOW))
        );
        assert!(summary.limits.is_empty(), "{:?}", summary.limits);
    }

    #[test]
    fn idle_sessions_have_no_five_hour_window() {
        let summary = summarize(Vec::new(), &[event(6, 5)], NOW);
        assert_eq!(summary.five_hour.window, None);
        assert_eq!(summary.five_hour.tokens, TokenCounts::default());
    }

    #[test]
    fn official_resets_define_both_windows() {
        let limits = vec![
            limit(LimitKind::FiveHour, 1),
            limit(LimitKind::SevenDay, 24),
        ];
        let events = [event(6, 1), event(3, 10), event(30, 1_000)];
        let summary = summarize(limits.clone(), &events, NOW);
        assert_eq!(
            summary.five_hour.window.map(Window::start),
            Some(NOW - Span::hours(4))
        );
        assert_eq!(summary.five_hour.tokens.output, 10);
        assert_eq!(
            summary.weekly.window.map(Window::start),
            Some(NOW + Span::hours(24) - Span::WEEK)
        );
        assert_eq!(summary.weekly.tokens.output, 1_011);
        assert_eq!(summary.limits, limits);
    }

    #[test]
    fn lookback_covers_the_week_and_a_day_of_activity() {
        let window = lookback(&[], NOW);
        assert!(window.contains(NOW));
        assert!(window.contains(NOW - INFERENCE_LOOKBACK));
        assert!(window.contains(weekly_containing(DEFAULT_WEEKLY_ANCHOR, NOW).start()));
        assert!(window.contains(history::window(NOW).start()));
        let official = lookback(&[limit(LimitKind::SevenDay, 1)], NOW);
        assert_eq!(official.start(), history::window(NOW).start());
    }
}
