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
pub const RENEWAL_FIRST_SPACING: Span = Span::minutes(5);
pub const RENEWAL_MAX_SPACING: Span = Span::hours(1);

const RENEWAL_MAX_DOUBLINGS: u32 = 4;

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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Renewal {
    pub runs: u32,
    pub last_run: Option<Timestamp>,
}

#[must_use]
pub fn renewal_due(renewal: Renewal, now: Timestamp) -> bool {
    renewal
        .last_run
        .is_none_or(|last| now - last >= renewal_spacing(renewal.runs))
}

fn renewal_spacing(runs: u32) -> Span {
    let doublings = runs.saturating_sub(1).min(RENEWAL_MAX_DOUBLINGS);
    let first = RENEWAL_FIRST_SPACING.millis();
    Span::from_millis(first.saturating_mul(1 << doublings)).min(RENEWAL_MAX_SPACING)
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
pub const fn refresh_allowed(accurate_mode: bool, rate_limited: bool) -> bool {
    accurate_mode && !rate_limited
}

#[must_use]
pub fn manual_refresh(allowed: bool, attempts: Attempts, now: Timestamp) -> ManualRefresh {
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
    if !allowed {
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
        RENEWAL_FIRST_SPACING, RENEWAL_MAX_SPACING, RUNNING_TIMEOUT, Renewal, STALE_GRACE,
        WARM_DELAY, adaptive_delay, cli_fallback_due, is_outdated, manual_refresh, refresh_allowed,
        renewal_due,
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
    fn a_failing_source_can_refresh_unless_it_is_rate_limited() {
        let failed = attempts(Some(Span::minutes(10)), None);
        assert!(refresh_allowed(true, false));
        assert_eq!(
            manual_refresh(refresh_allowed(true, false), failed, NOW),
            ManualRefresh::Ready
        );
        assert!(!refresh_allowed(true, true));
        assert!(!refresh_allowed(false, false));
        assert_eq!(
            manual_refresh(refresh_allowed(true, true), failed, NOW),
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

    fn renewal(runs: u32, last_run: Timestamp) -> Renewal {
        Renewal {
            runs,
            last_run: Some(last_run),
        }
    }

    #[test]
    fn the_first_renewal_of_an_expired_sign_in_starts_at_once() {
        assert!(renewal_due(Renewal::default(), NOW));
    }

    #[test]
    fn renewals_that_leave_the_sign_in_expired_back_off_up_to_an_hour() {
        assert_eq!(RENEWAL_FIRST_SPACING, Span::minutes(5));
        assert_eq!(RENEWAL_MAX_SPACING, Span::hours(1));
        let spacings = [
            (1, 5),
            (2, 10),
            (3, 20),
            (4, 40),
            (5, 60),
            (9, 60),
            (u32::MAX, 60),
        ];
        for (runs, minutes) in spacings {
            let last_run = NOW - Span::minutes(minutes);
            let almost = NOW - Span::seconds(1);
            assert!(!renewal_due(renewal(runs, last_run), almost), "{runs}");
            assert!(renewal_due(renewal(runs, last_run), NOW), "{runs}");
        }
    }
}
