use super::clock::{Span, Timestamp};

pub const RECENT_POPUP: Span = Span::minutes(5);
pub const WARM_POPUP: Span = Span::hours(1);
pub const IDLE_POPUP: Span = Span::hours(4);
pub const RECENT_CODING: Span = Span::minutes(5);

pub const RECENT_DELAY: Span = Span::minutes(2);
pub const WARM_DELAY: Span = Span::minutes(5);
pub const IDLE_DELAY: Span = Span::minutes(15);
pub const LONG_IDLE_DELAY: Span = Span::minutes(30);
pub const CODING_DELAY: Span = Span::minutes(5);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Activity {
    pub popup_opened_at: Option<Timestamp>,
    pub coding_at: Option<Timestamp>,
}

#[must_use]
pub fn adaptive_delay(activity: Activity, now: Timestamp) -> Span {
    let base = activity
        .popup_opened_at
        .map_or(LONG_IDLE_DELAY, |opened| popup_delay(now - opened));
    let recent_coding = activity
        .coding_at
        .is_some_and(|at| now - at <= RECENT_CODING);
    if recent_coding {
        return base.min(CODING_DELAY);
    }
    base
}

fn popup_delay(since: Span) -> Span {
    if since <= RECENT_POPUP {
        RECENT_DELAY
    } else if since <= WARM_POPUP {
        WARM_DELAY
    } else if since <= IDLE_POPUP {
        IDLE_DELAY
    } else {
        LONG_IDLE_DELAY
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Activity, CODING_DELAY, IDLE_DELAY, LONG_IDLE_DELAY, RECENT_DELAY, WARM_DELAY,
        adaptive_delay,
    };
    use crate::domain::clock::{Span, Timestamp};

    const NOW: Timestamp = Timestamp::from_unix_millis(1_790_300_000_000);

    fn opened(ago: Span) -> Activity {
        Activity {
            popup_opened_at: Some(NOW - ago),
            coding_at: None,
        }
    }

    #[test]
    fn no_activity_waits_the_longest() {
        assert_eq!(adaptive_delay(Activity::default(), NOW), LONG_IDLE_DELAY);
    }

    #[test]
    fn recent_popup_opens_refresh_faster() {
        assert_eq!(adaptive_delay(opened(Span::ZERO), NOW), RECENT_DELAY);
        assert_eq!(adaptive_delay(opened(Span::minutes(5)), NOW), RECENT_DELAY);
        assert_eq!(adaptive_delay(opened(Span::minutes(6)), NOW), WARM_DELAY);
        assert_eq!(adaptive_delay(opened(Span::hours(1)), NOW), WARM_DELAY);
        assert_eq!(adaptive_delay(opened(Span::hours(2)), NOW), IDLE_DELAY);
        assert_eq!(adaptive_delay(opened(Span::hours(5)), NOW), LONG_IDLE_DELAY);
    }

    #[test]
    fn recent_coding_caps_the_delay() {
        let coding = Activity {
            popup_opened_at: None,
            coding_at: Some(NOW - Span::minutes(3)),
        };
        assert_eq!(adaptive_delay(coding, NOW), CODING_DELAY);
        let stale = Activity {
            popup_opened_at: None,
            coding_at: Some(NOW - Span::minutes(6)),
        };
        assert_eq!(adaptive_delay(stale, NOW), LONG_IDLE_DELAY);
    }

    #[test]
    fn coding_never_slows_a_faster_popup_cadence() {
        let both = Activity {
            popup_opened_at: Some(NOW - Span::minutes(1)),
            coding_at: Some(NOW),
        };
        assert_eq!(adaptive_delay(both, NOW), RECENT_DELAY);
    }
}
