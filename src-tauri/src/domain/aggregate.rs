use super::period::Window;
use super::record::UsageEvent;
use super::tokens::TokenCounts;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowTotals {
    pub window: Window,
    pub tokens: TokenCounts,
    pub events: usize,
}

#[must_use]
pub fn totals_in(window: Window, events: &[UsageEvent]) -> WindowTotals {
    let (tokens, count) = events
        .iter()
        .filter(|event| window.contains(event.at))
        .fold(
            (TokenCounts::default(), 0_usize),
            |(tokens, count), event| (tokens + event.tokens, count + 1),
        );
    WindowTotals {
        window,
        tokens,
        events: count,
    }
}

#[cfg(test)]
mod tests {
    use super::totals_in;
    use crate::domain::clock::{Span, Timestamp};
    use crate::domain::period::Window;
    use crate::domain::record::UsageEvent;
    use crate::domain::tokens::TokenCounts;

    fn event(millis: i64, output: u64) -> UsageEvent {
        UsageEvent {
            at: Timestamp::from_unix_millis(millis),
            model: "claude-opus-5".to_owned(),
            project: "plimsoll".to_owned(),
            tokens: TokenCounts {
                input: 1,
                output,
                cache_creation: 10,
                cache_read: 100,
            },
        }
    }

    fn window() -> Window {
        Window::starting_at(Timestamp::from_unix_millis(1_000), Span::from_millis(1_000))
    }

    #[test]
    fn only_events_inside_the_window_are_counted() {
        let events = [
            event(999, 1),
            event(1_000, 2),
            event(1_999, 4),
            event(2_000, 8),
        ];
        let totals = totals_in(window(), &events);
        assert_eq!(totals.events, 2);
        assert_eq!(totals.tokens.output, 6);
        assert_eq!(totals.tokens.input, 2);
        assert_eq!(totals.tokens.cache_read, 200);
        assert_eq!(totals.window, window());
    }

    #[test]
    fn empty_input_yields_zero_totals() {
        let totals = totals_in(window(), &[]);
        assert_eq!(totals.events, 0);
        assert_eq!(totals.tokens, TokenCounts::default());
    }
}
