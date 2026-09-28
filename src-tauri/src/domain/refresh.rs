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

pub const MIN_SPACING: Span = Span::minutes(1);
pub const RUNNING_TIMEOUT: Span = Span::seconds(30);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManualRefresh {
    Ready,
    Running,
    CoolingDown { until: Timestamp },
    Blocked,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Attempts {
    pub last: Option<Timestamp>,
    pub requested: Option<Timestamp>,
}

#[must_use]
pub fn manual_refresh(healthy: bool, attempts: Attempts, now: Timestamp) -> ManualRefresh {
    let answered = attempts
        .last
        .zip(attempts.requested)
        .is_some_and(|(last, requested)| last >= requested);
    let pending = attempts
        .requested
        .is_some_and(|requested| now - requested < RUNNING_TIMEOUT);
    if pending && !answered {
        return ManualRefresh::Running;
    }
    if !healthy {
        return ManualRefresh::Blocked;
    }
    match attempts.last {
        Some(last) if now - last < MIN_SPACING => ManualRefresh::CoolingDown {
            until: last + MIN_SPACING,
        },
        _ => ManualRefresh::Ready,
    }
}

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
        Activity, Attempts, CODING_DELAY, IDLE_DELAY, LONG_IDLE_DELAY, MIN_SPACING, ManualRefresh,
        RECENT_DELAY, RUNNING_TIMEOUT, WARM_DELAY, adaptive_delay, manual_refresh,
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

    fn attempts(last: Option<Span>, requested: Option<Span>) -> Attempts {
        Attempts {
            last: last.map(|ago| NOW - ago),
            requested: requested.map(|ago| NOW - ago),
        }
    }

    #[test]
    fn a_healthy_source_can_refresh_once_the_spacing_has_passed() {
        assert_eq!(
            manual_refresh(true, Attempts::default(), NOW),
            ManualRefresh::Ready
        );
        assert_eq!(
            manual_refresh(true, attempts(Some(MIN_SPACING), None), NOW),
            ManualRefresh::Ready
        );
        assert_eq!(
            manual_refresh(true, attempts(Some(Span::seconds(20)), None), NOW),
            ManualRefresh::CoolingDown {
                until: NOW - Span::seconds(20) + MIN_SPACING
            }
        );
    }

    #[test]
    fn failing_or_pending_sources_keep_their_backoff() {
        assert_eq!(
            manual_refresh(false, attempts(Some(Span::minutes(10)), None), NOW),
            ManualRefresh::Blocked
        );
        assert_eq!(
            manual_refresh(false, Attempts::default(), NOW),
            ManualRefresh::Blocked
        );
    }

    #[test]
    fn a_request_runs_until_the_next_attempt_or_the_timeout() {
        let requested = attempts(Some(Span::minutes(3)), Some(Span::seconds(2)));
        assert_eq!(manual_refresh(true, requested, NOW), ManualRefresh::Running);
        assert_eq!(
            manual_refresh(false, requested, NOW),
            ManualRefresh::Running
        );
        let answered = attempts(Some(Span::seconds(1)), Some(Span::seconds(2)));
        assert!(matches!(
            manual_refresh(true, answered, NOW),
            ManualRefresh::CoolingDown { .. }
        ));
        let abandoned = attempts(Some(Span::minutes(3)), Some(RUNNING_TIMEOUT));
        assert_eq!(manual_refresh(true, abandoned, NOW), ManualRefresh::Ready);
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
