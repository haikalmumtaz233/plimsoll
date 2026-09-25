use super::clock::{Span, Timestamp};
use super::period::Window;

#[must_use]
pub fn infer_five_hour(activity: &[Timestamp], now: Timestamp) -> Option<Window> {
    let mut past: Vec<Timestamp> = activity.iter().copied().filter(|at| *at <= now).collect();
    past.sort_unstable();

    let mut current: Option<Window> = None;
    for at in past {
        if current.is_none_or(|window| !window.contains(at)) {
            current = Some(Window::starting_at(at, Span::FIVE_HOURS));
        }
    }
    current.filter(|window| window.contains(now))
}

#[cfg(test)]
mod tests {
    use super::infer_five_hour;
    use crate::domain::clock::{Span, Timestamp};

    const HOUR: i64 = 3_600_000;
    const MINUTE: i64 = 60_000;

    fn at(millis: i64) -> Timestamp {
        Timestamp::from_unix_millis(millis)
    }

    #[test]
    fn no_activity_means_no_window() {
        assert_eq!(infer_five_hour(&[], at(HOUR)), None);
    }

    #[test]
    fn window_starts_at_first_activity() {
        let window = infer_five_hour(&[at(HOUR), at(2 * HOUR)], at(3 * HOUR));
        let window = window.expect("active window");
        assert_eq!(window.start(), at(HOUR));
        assert_eq!(window.end(), at(HOUR) + Span::FIVE_HOURS);
    }

    #[test]
    fn expired_window_is_not_active() {
        assert_eq!(infer_five_hour(&[at(HOUR)], at(6 * HOUR)), None);
    }

    #[test]
    fn activity_after_expiry_opens_a_new_window() {
        let activity = [at(0), at(4 * HOUR), at(5 * HOUR + 30 * MINUTE)];
        let window = infer_five_hour(&activity, at(6 * HOUR)).expect("active window");
        assert_eq!(window.start(), at(5 * HOUR + 30 * MINUTE));
    }

    #[test]
    fn unsorted_activity_is_handled() {
        let activity = [at(2 * HOUR), at(HOUR), at(90 * MINUTE)];
        let window = infer_five_hour(&activity, at(3 * HOUR)).expect("active window");
        assert_eq!(window.start(), at(HOUR));
    }

    #[test]
    fn future_activity_is_ignored() {
        let activity = [at(HOUR), at(10 * HOUR)];
        assert_eq!(infer_five_hour(&activity, at(7 * HOUR)), None);
    }
}
