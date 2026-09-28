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
pub const STALE_GRACE: Span = Span::minutes(5);
pub const CLI_FALLBACK_SPACING: Span = Span::minutes(10);
pub const CLI_FALLBACK_FAILURES: u32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FallbackState {
    pub enabled: bool,
    pub failures: u32,
    pub token_expired: bool,
    pub last_run: Option<Timestamp>,
}
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
pub fn cli_fallback_due(state: FallbackState, now: Timestamp) -> bool {
    let failing = state.token_expired || state.failures >= CLI_FALLBACK_FAILURES;
    let spaced = state
        .last_run
        .is_none_or(|last| now - last >= CLI_FALLBACK_SPACING);
    state.enabled && failing && spaced
}

#[must_use]
pub fn fallback_is_recent(last_success: Option<Timestamp>, now: Timestamp) -> bool {
    last_success.is_some_and(|at| now - at <= CLI_FALLBACK_SPACING + STALE_GRACE)
}

#[must_use]
pub fn is_outdated(healthy: bool, official_age: Span, expected_delay: Span) -> bool {
    !healthy || official_age > expected_delay + STALE_GRACE
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
        Activity, Attempts, CLI_FALLBACK_FAILURES, CLI_FALLBACK_SPACING, CODING_DELAY,
        FallbackState, IDLE_DELAY, LONG_IDLE_DELAY, MIN_SPACING, ManualRefresh, RECENT_DELAY,
        RUNNING_TIMEOUT, STALE_GRACE, WARM_DELAY, adaptive_delay, cli_fallback_due, is_outdated,
        manual_refresh,
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
    fn official_readings_are_outdated_after_a_failure_or_a_missed_refresh() {
        assert!(!is_outdated(true, Span::minutes(3), RECENT_DELAY));
        assert!(!is_outdated(true, RECENT_DELAY + STALE_GRACE, RECENT_DELAY));
        assert!(is_outdated(
            true,
            RECENT_DELAY + STALE_GRACE + Span::seconds(1),
            RECENT_DELAY
        ));
        assert!(!is_outdated(true, Span::minutes(30), LONG_IDLE_DELAY));
        assert!(is_outdated(false, Span::ZERO, LONG_IDLE_DELAY));
    }

    fn fallback(failures: u32, token_expired: bool, last_run: Option<Span>) -> FallbackState {
        FallbackState {
            enabled: true,
            failures,
            token_expired,
            last_run: last_run.map(|ago| NOW - ago),
        }
    }

    #[test]
    fn the_cli_fallback_waits_for_repeated_failures_or_an_expired_token() {
        assert!(!cli_fallback_due(fallback(0, false, None), NOW));
        assert!(!cli_fallback_due(fallback(1, false, None), NOW));
        assert!(cli_fallback_due(
            fallback(CLI_FALLBACK_FAILURES, false, None),
            NOW
        ));
        assert!(cli_fallback_due(fallback(1, true, None), NOW));
    }

    #[test]
    fn a_fallback_reading_counts_as_fresh_until_the_next_run_is_overdue() {
        assert!(!super::fallback_is_recent(None, NOW));
        assert!(super::fallback_is_recent(
            Some(NOW - Span::minutes(14)),
            NOW
        ));
        assert!(super::fallback_is_recent(
            Some(NOW - Span::minutes(15)),
            NOW
        ));
        assert!(!super::fallback_is_recent(
            Some(NOW - Span::minutes(16)),
            NOW
        ));
    }

    #[test]
    fn the_cli_fallback_is_opt_in_and_spaced_out() {
        let disabled = FallbackState {
            enabled: false,
            ..fallback(5, true, None)
        };
        assert!(!cli_fallback_due(disabled, NOW));
        assert!(!cli_fallback_due(
            fallback(5, false, Some(Span::seconds(599))),
            NOW
        ));
        assert!(cli_fallback_due(
            fallback(5, false, Some(CLI_FALLBACK_SPACING)),
            NOW
        ));
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
