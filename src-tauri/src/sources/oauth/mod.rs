pub mod credentials;
pub mod poll;
pub mod response;
pub mod schedule;
pub mod status;
pub mod transport;

use std::path::PathBuf;
use std::time::Duration;

use thiserror::Error;

use self::credentials::CredentialsError;
use self::transport::{Reply, Transport, TransportError};
use crate::domain::clock::Timestamp;
use crate::domain::limit::LimitSnapshot;

#[derive(Debug, Error)]
pub enum OAuthError {
    #[error(transparent)]
    Credentials(#[from] CredentialsError),
    #[error(transparent)]
    Transport(#[from] TransportError),
    #[error("oauth usage request was not authorized (http 401/403)")]
    Unauthorized,
    #[error("oauth usage endpoint was not found (http 404)")]
    EndpointGone,
    #[error("oauth usage endpoint is rate limiting requests (http 429{})", retry_hint(*.retry_after))]
    RateLimited { retry_after: Option<Duration> },
    #[error("oauth usage endpoint returned http {0}")]
    Status(u16),
    #[error("oauth usage response schema is not recognized")]
    SchemaChanged,
}

impl OAuthError {
    #[must_use]
    pub const fn retry_after(&self) -> Option<Duration> {
        match self {
            Self::RateLimited { retry_after } => *retry_after,
            _ => None,
        }
    }
}

fn retry_hint(retry_after: Option<Duration>) -> String {
    retry_after.map_or_else(String::new, |wait| {
        format!(", retry after {} s", wait.as_secs())
    })
}

pub fn interpret(reply: &Reply) -> Result<Vec<LimitSnapshot>, OAuthError> {
    match reply.status {
        200 => response::parse(&reply.body).ok_or(OAuthError::SchemaChanged),
        401 | 403 => Err(OAuthError::Unauthorized),
        404 => Err(OAuthError::EndpointGone),
        429 => Err(OAuthError::RateLimited {
            retry_after: reply.retry_after,
        }),
        status => Err(OAuthError::Status(status)),
    }
}

#[derive(Debug)]
pub struct OAuthUsageSource<T> {
    transport: T,
    credentials: PathBuf,
}

impl<T: Transport> OAuthUsageSource<T> {
    #[must_use]
    pub const fn new(transport: T, credentials: PathBuf) -> Self {
        Self {
            transport,
            credentials,
        }
    }

    pub async fn fetch(&self, now: Timestamp) -> Result<Vec<LimitSnapshot>, OAuthError> {
        let reply = {
            let token = credentials::read_access_token(&self.credentials, now)?;
            self.transport.get_usage(&token).await?
        };
        interpret(&reply)
    }
}

#[cfg(test)]
pub(crate) mod fake {
    use super::credentials::AccessToken;
    use super::transport::{Reply, Transport, TransportError};
    use std::collections::VecDeque;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::Mutex;

    pub const FAKE_TOKEN: &str = "fixture-access-token";

    #[derive(Debug, Default)]
    pub struct FakeTransport {
        replies: Mutex<VecDeque<Reply>>,
        seen_tokens: Mutex<Vec<String>>,
    }

    impl FakeTransport {
        pub fn replying(replies: Vec<Reply>) -> Self {
            Self {
                replies: Mutex::new(replies.into()),
                seen_tokens: Mutex::default(),
            }
        }

        pub fn seen_tokens(&self) -> Vec<String> {
            self.seen_tokens.lock().expect("lock").clone()
        }
    }

    impl Transport for FakeTransport {
        fn get_usage(
            &self,
            token: &AccessToken,
        ) -> impl Future<Output = Result<Reply, TransportError>> + Send {
            self.seen_tokens
                .lock()
                .expect("lock")
                .push(token.secret().to_owned());
            let next = self
                .replies
                .lock()
                .expect("lock")
                .pop_front()
                .expect("a scripted reply");
            std::future::ready(Ok(next))
        }
    }

    pub fn reply(status: u16, body: &str) -> Reply {
        Reply {
            status,
            retry_after: None,
            body: body.as_bytes().to_vec(),
        }
    }

    pub fn credentials_file(name: &str) -> PathBuf {
        let path =
            std::env::temp_dir().join(format!("plimsoll-oauth-{name}-{}.json", std::process::id()));
        let body = format!(r#"{{"claudeAiOauth":{{"accessToken":"{FAKE_TOKEN}"}}}}"#);
        fs::write(&path, body).expect("write scratch credentials");
        path
    }
}

#[cfg(test)]
mod tests {
    use super::fake::{FAKE_TOKEN, FakeTransport, credentials_file, reply};
    use super::transport::Reply;
    use super::{OAuthError, OAuthUsageSource, interpret};
    use crate::domain::clock::Timestamp;
    use crate::domain::limit::LimitKind;
    use crate::sources::oauth::credentials::CredentialsError;
    use std::fs;
    use std::time::Duration;

    const NOW: Timestamp = Timestamp::from_unix_millis(1_790_300_000_000);
    const USAGE: &str =
        r#"{"limits":[{"kind":"session","percent":13},{"kind":"weekly_all","percent":14}]}"#;

    #[tokio::test]
    async fn fetches_snapshots_with_the_current_token() {
        let path = credentials_file("fetch");
        let source = OAuthUsageSource::new(
            FakeTransport::replying(vec![reply(200, USAGE)]),
            path.clone(),
        );
        let snapshots = source.fetch(NOW).await.expect("snapshots");
        assert_eq!(
            snapshots
                .iter()
                .map(|snapshot| snapshot.kind)
                .collect::<Vec<_>>(),
            vec![LimitKind::FiveHour, LimitKind::SevenDay]
        );
        assert_eq!(source.transport.seen_tokens(), vec![FAKE_TOKEN.to_owned()]);
        fs::remove_file(path).expect("cleanup");
    }

    #[tokio::test]
    async fn skips_the_request_without_credentials() {
        let source = OAuthUsageSource::new(
            FakeTransport::default(),
            std::env::temp_dir().join("plimsoll-oauth-absent.json"),
        );
        assert!(matches!(
            source.fetch(NOW).await,
            Err(OAuthError::Credentials(CredentialsError::Missing))
        ));
        assert!(source.transport.seen_tokens().is_empty());
    }

    #[test]
    fn classifies_http_statuses() {
        assert!(matches!(
            interpret(&reply(401, "")),
            Err(OAuthError::Unauthorized)
        ));
        assert!(matches!(
            interpret(&reply(403, "")),
            Err(OAuthError::Unauthorized)
        ));
        assert!(matches!(
            interpret(&reply(404, "")),
            Err(OAuthError::EndpointGone)
        ));
        assert!(matches!(
            interpret(&reply(500, "")),
            Err(OAuthError::Status(500))
        ));
        assert!(matches!(
            interpret(&reply(200, r#"{"surprise":true}"#)),
            Err(OAuthError::SchemaChanged)
        ));
        assert_eq!(interpret(&reply(200, USAGE)).map(|s| s.len()).ok(), Some(2));
    }

    #[test]
    fn rate_limits_carry_the_retry_hint() {
        let error = interpret(&Reply {
            status: 429,
            retry_after: Some(Duration::from_secs(300)),
            body: Vec::new(),
        })
        .expect_err("rate limited");
        assert_eq!(error.retry_after(), Some(Duration::from_secs(300)));
        assert_eq!(OAuthError::Unauthorized.retry_after(), None);
    }

    #[test]
    fn error_messages_carry_the_http_status_for_the_log() {
        assert_eq!(
            OAuthError::RateLimited {
                retry_after: Some(Duration::from_secs(300))
            }
            .to_string(),
            "oauth usage endpoint is rate limiting requests (http 429, retry after 300 s)"
        );
        assert_eq!(
            OAuthError::RateLimited { retry_after: None }.to_string(),
            "oauth usage endpoint is rate limiting requests (http 429)"
        );
        assert_eq!(
            OAuthError::Status(503).to_string(),
            "oauth usage endpoint returned http 503"
        );
    }
}
