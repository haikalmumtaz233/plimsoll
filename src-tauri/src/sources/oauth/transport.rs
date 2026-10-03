use std::future::Future;
use std::time::Duration;

use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue, RETRY_AFTER};
use reqwest::redirect::Policy;
use reqwest::{Client, Response};
use thiserror::Error;
use zeroize::Zeroizing;

use super::credentials::AccessToken;

pub const USAGE_URL: &str = "https://api.anthropic.com/api/oauth/usage";
pub const MAX_BODY_BYTES: usize = 256 * 1024;

const BETA_HEADER: &str = "anthropic-beta";
const BETA_VALUE: &str = "oauth-2025-04-20";
const USER_AGENT: &str = concat!("plimsoll/", env!("CARGO_PKG_VERSION"));
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reply {
    pub status: u16,
    pub retry_after: Option<Duration>,
    pub body: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("oauth usage request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("oauth access token contains characters that are not valid in a header")]
    InvalidToken,
    #[error("oauth usage response is larger than {MAX_BODY_BYTES} bytes")]
    BodyTooLarge,
}

pub trait Transport {
    fn get_usage(
        &self,
        token: &AccessToken,
    ) -> impl Future<Output = Result<Reply, TransportError>> + Send;
}

#[derive(Debug, Clone)]
pub struct HttpsTransport {
    client: Client,
}

impl HttpsTransport {
    pub fn new() -> Result<Self, TransportError> {
        install_crypto_provider();
        let client = Client::builder()
            .https_only(true)
            .redirect(Policy::none())
            .tls_version_min(reqwest::tls::Version::TLS_1_2)
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .pool_max_idle_per_host(0)
            .user_agent(USER_AGENT)
            .build()?;
        Ok(Self { client })
    }
}

impl Transport for HttpsTransport {
    async fn get_usage(&self, token: &AccessToken) -> Result<Reply, TransportError> {
        let response = self
            .client
            .get(USAGE_URL)
            .header(AUTHORIZATION, bearer(token)?)
            .header(BETA_HEADER, BETA_VALUE)
            .header(ACCEPT, "application/json")
            .send()
            .await?;
        let status = response.status().as_u16();
        let retry_after = retry_after(response.headers());
        let body = read_capped(response).await?;
        Ok(Reply {
            status,
            retry_after,
            body,
        })
    }
}

fn install_crypto_provider() {
    if rustls::crypto::CryptoProvider::get_default().is_none() {
        let _ = rustls::crypto::ring::default_provider().install_default();
    }
}

fn bearer(token: &AccessToken) -> Result<HeaderValue, TransportError> {
    let text = Zeroizing::new(format!("Bearer {}", token.secret()));
    let mut value = HeaderValue::from_str(&text).map_err(|_| TransportError::InvalidToken)?;
    value.set_sensitive(true);
    Ok(value)
}

fn retry_after(headers: &HeaderMap) -> Option<Duration> {
    headers
        .get(RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .and_then(parse_retry_after)
}

#[must_use]
pub fn parse_retry_after(text: &str) -> Option<Duration> {
    text.trim().parse().ok().map(Duration::from_secs)
}

async fn read_capped(mut response: Response) -> Result<Vec<u8>, TransportError> {
    if response
        .content_length()
        .is_some_and(|length| length > MAX_BODY_BYTES as u64)
    {
        return Err(TransportError::BodyTooLarge);
    }
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        if body.len() + chunk.len() > MAX_BODY_BYTES {
            return Err(TransportError::BodyTooLarge);
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::{HttpsTransport, bearer, parse_retry_after};
    use crate::domain::clock::Timestamp;
    use crate::sources::oauth::credentials::parse_access_token;
    use std::time::Duration;

    #[test]
    fn builds_a_client_with_the_ring_provider() {
        assert!(HttpsTransport::new().is_ok());
        assert!(HttpsTransport::new().is_ok());
    }

    #[test]
    fn bearer_header_is_marked_sensitive() {
        let token = parse_access_token(
            br#"{"claudeAiOauth":{"accessToken":"fixture"}}"#,
            Timestamp::from_unix_millis(0),
        )
        .expect("token");
        let value = bearer(&token).expect("header");
        assert!(value.is_sensitive());
        assert_eq!(value.to_str().ok(), Some("Bearer fixture"));
    }

    #[test]
    fn rejects_tokens_that_cannot_be_sent_as_headers() {
        let token = parse_access_token(
            br#"{"claudeAiOauth":{"accessToken":"line\nbreak"}}"#,
            Timestamp::from_unix_millis(0),
        )
        .expect("token");
        assert!(bearer(&token).is_err());
    }

    #[test]
    fn parses_retry_after_seconds_only() {
        assert_eq!(parse_retry_after("120"), Some(Duration::from_secs(120)));
        assert_eq!(parse_retry_after(" 5 "), Some(Duration::from_secs(5)));
        assert_eq!(parse_retry_after("Wed, 21 Oct 2026 07:28:00 GMT"), None);
        assert_eq!(parse_retry_after("-1"), None);
        assert_eq!(parse_retry_after(""), None);
    }
}
