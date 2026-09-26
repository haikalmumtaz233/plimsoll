use std::ops::ControlFlow;

use super::schedule::{Jitter, PollSchedule};
use super::transport::Transport;
use super::{OAuthError, OAuthUsageSource};
use crate::domain::clock::Timestamp;
use crate::domain::limit::LimitSnapshot;

pub type PollResult = Result<Vec<LimitSnapshot>, OAuthError>;

pub async fn run<T, N, J, F>(
    source: &OAuthUsageSource<T>,
    mut schedule: PollSchedule,
    now: N,
    mut jitter: J,
    mut on_result: F,
) where
    T: Transport,
    N: Fn() -> Timestamp,
    J: FnMut() -> Jitter,
    F: FnMut(PollResult) -> ControlFlow<()>,
{
    loop {
        let result = source.fetch(now()).await;
        let delay = match &result {
            Ok(_) => schedule.after_success(jitter()),
            Err(error) => schedule.after_failure(error.retry_after(), jitter()),
        };
        if on_result(result).is_break() {
            return;
        }
        tokio::time::sleep(delay).await;
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
        let started = Instant::now();
        let mut polls = Vec::new();
        run(
            &source,
            PollSchedule::default(),
            || Timestamp::from_unix_millis(0),
            || Jitter::NONE,
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
}
