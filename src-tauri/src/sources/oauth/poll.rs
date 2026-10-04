use std::ops::ControlFlow;
use std::time::Duration;

use tokio::sync::Notify;
use tokio::time::Instant;

use super::schedule::{Jitter, MIN_INTERVAL, PollSchedule};
use super::transport::Transport;
use super::{OAuthError, OAuthUsageSource};
use crate::domain::clock::Timestamp;
use crate::domain::extras::OfficialUsage;

pub type PollResult = Result<OfficialUsage, OAuthError>;

pub async fn run<T, N, C, J, F>(
    source: &OAuthUsageSource<T>,
    mut schedule: PollSchedule,
    now: N,
    mut cadence: C,
    mut jitter: J,
    wake: &Notify,
    mut on_result: F,
) where
    T: Transport,
    N: Fn() -> Timestamp,
    C: FnMut() -> Duration,
    J: FnMut() -> Jitter,
    F: FnMut(PollResult) -> ControlFlow<()>,
{
    loop {
        let result = source.fetch(now()).await;
        schedule.set_base(cadence());
        let rate_limited = matches!(result, Err(OAuthError::RateLimited { .. }));
        let delay = match &result {
            Ok(_) => schedule.after_success(jitter()),
            Err(error) => schedule.after_failure(error.retry_after(), jitter()),
        };
        if on_result(result).is_break() {
            return;
        }
        pause(delay, rate_limited, wake).await;
    }
}

async fn pause(delay: Duration, rate_limited: bool, wake: &Notify) {
    if rate_limited {
        tokio::time::sleep(delay).await;
        return;
    }
    let started = Instant::now();
    if tokio::time::timeout(delay, wake.notified()).await.is_ok()
        && let Some(rest) = MIN_INTERVAL.checked_sub(started.elapsed())
    {
        tokio::time::sleep(rest).await;
    }
}

#[cfg(test)]
mod tests {
    use super::run;
    use crate::domain::clock::Timestamp;
    use crate::sources::oauth::OAuthUsageSource;
    use crate::sources::oauth::fake::{FakeTransport, credentials_file, reply};
    use crate::sources::oauth::schedule::{Jitter, PollSchedule};
    use crate::sources::oauth::transport::Reply;
    use std::fs;
    use std::ops::ControlFlow;
    use std::time::Duration;
    use tokio::sync::Notify;
    use tokio::time::Instant;

    const USAGE: &str = r#"{"limits":[{"kind":"session","percent":13}]}"#;

    #[tokio::test(start_paused = true)]
    async fn polls_on_the_backoff_schedule_until_told_to_stop() {
        let path = credentials_file("poll");
        let source = OAuthUsageSource::new(
            FakeTransport::replying(vec![
                reply(200, USAGE),
                reply(500, ""),
                reply(500, ""),
                Reply {
                    status: 429,
                    retry_after: Some(Duration::from_secs(600)),
                    body: Vec::new(),
                },
                reply(200, USAGE),
                reply(200, USAGE),
            ]),
            path.clone(),
        );
        let wake = Notify::new();
        let started = Instant::now();
        let mut polls = Vec::new();
        run(
            &source,
            PollSchedule::default(),
            || Timestamp::from_unix_millis(0),
            || Duration::from_secs(60),
            || Jitter::NONE,
            &wake,
            |result| {
                polls.push((started.elapsed().as_secs(), result.is_ok()));
                if polls.len() == 6 {
                    ControlFlow::Break(())
                } else {
                    ControlFlow::Continue(())
                }
            },
        )
        .await;
        assert_eq!(
            polls,
            vec![
                (0, true),
                (60, false),
                (180, false),
                (420, false),
                (1_020, true),
                (1_080, true),
            ]
        );
        fs::remove_file(path).expect("cleanup");
    }

    #[tokio::test(start_paused = true)]
    async fn a_wake_up_polls_early_but_not_before_the_minimum_interval() {
        let path = credentials_file("wake");
        let replies = vec![reply(200, USAGE), reply(200, USAGE), reply(200, USAGE)];
        let source = OAuthUsageSource::new(FakeTransport::replying(replies), path.clone());
        let wake = Notify::new();
        wake.notify_one();
        let started = Instant::now();
        let mut polls = Vec::new();
        run(
            &source,
            PollSchedule::default(),
            || Timestamp::from_unix_millis(0),
            || Duration::from_secs(300),
            || Jitter::NONE,
            &wake,
            |result| {
                polls.push((started.elapsed().as_secs(), result.is_ok()));
                if polls.len() == 3 {
                    ControlFlow::Break(())
                } else {
                    ControlFlow::Continue(())
                }
            },
        )
        .await;
        assert_eq!(polls, vec![(0, true), (60, true), (360, true)]);
        fs::remove_file(path).expect("cleanup");
    }

    #[tokio::test(start_paused = true)]
    async fn a_wake_up_retries_a_failure_after_the_minimum_interval() {
        let path = credentials_file("wake-failure");
        let source = OAuthUsageSource::new(
            FakeTransport::replying(vec![reply(403, ""), reply(500, ""), reply(200, USAGE)]),
            path.clone(),
        );
        let wake = Notify::new();
        wake.notify_one();
        let started = Instant::now();
        let mut polls = Vec::new();
        run(
            &source,
            PollSchedule::new(Duration::from_secs(30 * 60)),
            || Timestamp::from_unix_millis(0),
            || Duration::from_secs(30 * 60),
            || Jitter::NONE,
            &wake,
            |result| {
                polls.push((started.elapsed().as_secs(), result.is_ok()));
                if polls.len() == 3 {
                    ControlFlow::Break(())
                } else {
                    ControlFlow::Continue(())
                }
            },
        )
        .await;
        assert_eq!(polls, vec![(0, false), (60, false), (300, true)]);
        fs::remove_file(path).expect("cleanup");
    }

    #[tokio::test(start_paused = true)]
    async fn a_wake_up_never_cuts_a_rate_limit_pause_short() {
        let path = credentials_file("wake-backoff");
        let limited = Reply {
            status: 429,
            retry_after: Some(Duration::from_secs(600)),
            body: Vec::new(),
        };
        let source = OAuthUsageSource::new(
            FakeTransport::replying(vec![limited, reply(200, USAGE)]),
            path.clone(),
        );
        let wake = Notify::new();
        wake.notify_one();
        let started = Instant::now();
        let mut polls = Vec::new();
        run(
            &source,
            PollSchedule::default(),
            || Timestamp::from_unix_millis(0),
            || Duration::from_secs(60),
            || Jitter::NONE,
            &wake,
            |result| {
                polls.push((started.elapsed().as_secs(), result.is_ok()));
                if polls.len() == 2 {
                    ControlFlow::Break(())
                } else {
                    ControlFlow::Continue(())
                }
            },
        )
        .await;
        assert_eq!(polls, vec![(0, false), (600, true)]);
        fs::remove_file(path).expect("cleanup");
    }
}
