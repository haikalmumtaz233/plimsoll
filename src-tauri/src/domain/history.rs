use super::clock::{Span, Timestamp};
use super::period::Window;
use super::record::UsageEvent;

pub const BUCKET: Span = Span::hours(1);
pub const HOURS: i64 = 168;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HourlyHistory {
    pub start: Timestamp,
    pub tokens: Vec<u64>,
}

#[must_use]
pub fn window(now: Timestamp) -> Window {
    let bucket = BUCKET.millis();
    let current = Timestamp::from_unix_millis(now.unix_millis().div_euclid(bucket) * bucket);
    Window::ending_at(current + BUCKET, Span::hours(HOURS))
}

#[must_use]
pub fn hourly(events: &[UsageEvent], now: Timestamp) -> HourlyHistory {
    let window = window(now);
    let mut tokens = vec![0_u64; usize::try_from(HOURS).unwrap_or(0)];
    for event in events.iter().filter(|event| window.contains(event.at)) {
        let offset = (event.at - window.start()).millis() / BUCKET.millis();
        if let Some(bucket) = usize::try_from(offset)
            .ok()
            .and_then(|index| tokens.get_mut(index))
        {
            *bucket = bucket.saturating_add(event.tokens.excluding_cache_reads());
        }
    }
    HourlyHistory {
        start: window.start(),
        tokens,
    }
}

#[cfg(test)]
mod tests {
    use super::{BUCKET, HOURS, hourly, window};
    use crate::domain::clock::{Span, Timestamp};
    use crate::domain::record::UsageEvent;
    use crate::domain::tokens::TokenCounts;

    const HOUR: i64 = 3_600_000;
    const NOW: Timestamp = Timestamp::from_unix_millis(1_790_300_000_000 + 25 * 60_000);

    fn event(at: Timestamp, output: u64, cache_read: u64) -> UsageEvent {
        UsageEvent {
            at,
            model: "claude-opus-5".to_owned(),
            project: "plimsoll".to_owned(),
            tokens: TokenCounts {
                output,
                cache_read,
                ..TokenCounts::default()
            },
        }
    }

    #[test]
    fn the_window_ends_after_the_current_hour() {
        let window = window(NOW);
        assert_eq!(window.end().unix_millis() % HOUR, 0);
        assert!(window.contains(NOW));
        assert!(window.end() - NOW <= BUCKET);
        assert_eq!(window.end() - window.start(), Span::hours(HOURS));
    }

    #[test]
    fn events_land_in_their_hour_without_cache_reads() {
        let window = window(NOW);
        let events = [
            event(window.start(), 1, 500),
            event(window.start() + Span::from_millis(HOUR - 1), 2, 0),
            event(window.start() + Span::from_millis(HOUR), 4, 0),
            event(NOW, 8, 0),
            event(NOW, 16, 0),
        ];
        let history = hourly(&events, NOW);
        assert_eq!(history.start, window.start());
        assert_eq!(history.tokens.len(), 168);
        assert_eq!(history.tokens[0], 3);
        assert_eq!(history.tokens[1], 4);
        assert_eq!(history.tokens[167], 24);
        assert_eq!(history.tokens.iter().sum::<u64>(), 31);
    }

    #[test]
    fn events_outside_the_window_are_ignored() {
        let window = window(NOW);
        let events = [
            event(window.start() - Span::from_millis(1), 100, 0),
            event(window.end(), 100, 0),
        ];
        assert!(
            hourly(&events, NOW)
                .tokens
                .iter()
                .all(|tokens| *tokens == 0)
        );
    }
}
