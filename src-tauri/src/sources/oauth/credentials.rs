use std::ffi::OsString;
use std::fmt;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Deserializer};
use thiserror::Error;
use zeroize::Zeroizing;

use crate::domain::clock::Timestamp;
use crate::sources::claude_home_from;

pub const MAX_CREDENTIALS_BYTES: usize = 64 * 1024;

const FILE_NAME: &str = ".credentials.json";

#[derive(Debug, Error)]
pub enum CredentialsError {
    #[error("claude credentials file was not found")]
    Missing,
    #[error("failed to read claude credentials file: {0}")]
    Io(#[from] io::Error),
    #[error("claude credentials file is larger than {MAX_CREDENTIALS_BYTES} bytes")]
    TooLarge,
    #[error("claude credentials file is not valid json")]
    Malformed,
    #[error("claude credentials file has no oauth access token")]
    NoToken,
    #[error("oauth access token has expired")]
    Expired,
}

pub struct AccessToken(Zeroizing<String>);

impl AccessToken {
    #[must_use]
    pub fn secret(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for AccessToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("AccessToken(<redacted>)")
    }
}

#[derive(Deserialize)]
struct CredentialsFile {
    #[serde(rename = "claudeAiOauth", default)]
    oauth: Option<OAuthEntry>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct OAuthEntry {
    #[serde(default, deserialize_with = "zeroizing_string")]
    access_token: Option<Zeroizing<String>>,
    #[serde(default)]
    expires_at: Option<i64>,
}

fn zeroizing_string<'de, D>(deserializer: D) -> Result<Option<Zeroizing<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer).map(|value| value.map(Zeroizing::new))
}

#[must_use]
pub fn credentials_path() -> Option<PathBuf> {
    credentials_path_from(std::env::var_os)
}

#[must_use]
pub fn credentials_path_from<F>(lookup: F) -> Option<PathBuf>
where
    F: Fn(&'static str) -> Option<OsString>,
{
    claude_home_from(lookup).map(|home| home.join(FILE_NAME))
}

pub fn read_access_token(path: &Path, now: Timestamp) -> Result<AccessToken, CredentialsError> {
    let bytes = read_limited(path)?;
    parse_access_token(&bytes, now)
}

pub fn parse_access_token(bytes: &[u8], now: Timestamp) -> Result<AccessToken, CredentialsError> {
    let file: CredentialsFile =
        serde_json::from_slice(bytes).map_err(|_| CredentialsError::Malformed)?;
    let entry = file.oauth.ok_or(CredentialsError::NoToken)?;
    let token = entry
        .access_token
        .filter(|token| !token.is_empty())
        .ok_or(CredentialsError::NoToken)?;
    if entry
        .expires_at
        .is_some_and(|expires| expires <= now.unix_millis())
    {
        return Err(CredentialsError::Expired);
    }
    Ok(AccessToken(token))
}

fn read_limited(path: &Path) -> Result<Zeroizing<Vec<u8>>, CredentialsError> {
    let file = File::open(path).map_err(|error| match error.kind() {
        io::ErrorKind::NotFound => CredentialsError::Missing,
        _ => CredentialsError::Io(error),
    })?;
    let mut bytes = Zeroizing::new(Vec::with_capacity(MAX_CREDENTIALS_BYTES + 1));
    file.take(u64::try_from(MAX_CREDENTIALS_BYTES + 1).unwrap_or(u64::MAX))
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_CREDENTIALS_BYTES {
        return Err(CredentialsError::TooLarge);
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::{
        CredentialsError, MAX_CREDENTIALS_BYTES, credentials_path_from, parse_access_token,
        read_access_token,
    };
    use crate::domain::clock::Timestamp;
    use crate::sources::test_env::lookup;
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;

    const NOW: Timestamp = Timestamp::from_unix_millis(1_790_300_000_000);
    const FAKE_TOKEN: &str = "fixture-access-token";
    const FAKE_REFRESH_TOKEN: &str = "fixture-refresh-token";

    fn credentials(expires_at: Option<i64>) -> Vec<u8> {
        json!({
            "claudeAiOauth": {
                "accessToken": FAKE_TOKEN,
                "refreshToken": FAKE_REFRESH_TOKEN,
                "expiresAt": expires_at,
                "scopes": ["user:inference"],
                "subscriptionType": "pro"
            }
        })
        .to_string()
        .into_bytes()
    }

    fn scratch_file(name: &str, bytes: &[u8]) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "plimsoll-credentials-{name}-{}.json",
            std::process::id()
        ));
        fs::write(&path, bytes).expect("write scratch credentials");
        path
    }

    #[test]
    fn credentials_live_in_the_claude_home() {
        assert_eq!(
            credentials_path_from(lookup(&[("USERPROFILE", "C:\\Users\\dev")])),
            Some(
                PathBuf::from("C:\\Users\\dev")
                    .join(".claude")
                    .join(".credentials.json")
            )
        );
        assert_eq!(credentials_path_from(lookup(&[])), None);
    }

    #[test]
    fn reads_the_access_token_while_it_is_valid() {
        let token =
            parse_access_token(&credentials(Some(1_790_300_000_001)), NOW).expect("valid token");
        assert_eq!(token.secret(), FAKE_TOKEN);

        let without_expiry =
            parse_access_token(&credentials(None), NOW).expect("token without expiry");
        assert_eq!(without_expiry.secret(), FAKE_TOKEN);
    }

    #[test]
    fn debug_output_never_contains_the_token() {
        let token = parse_access_token(&credentials(None), NOW).expect("token");
        let printed = format!("{token:?}");
        assert!(!printed.contains(FAKE_TOKEN));
        assert!(printed.contains("redacted"));
    }

    #[test]
    fn rejects_expired_tokens() {
        assert!(matches!(
            parse_access_token(&credentials(Some(NOW.unix_millis())), NOW),
            Err(CredentialsError::Expired)
        ));
    }

    #[test]
    fn reports_missing_tokens_and_malformed_files() {
        for body in [
            json!({}),
            json!({ "claudeAiOauth": {} }),
            json!({ "claudeAiOauth": { "accessToken": "" } }),
            json!({ "claudeAiOauth": { "accessToken": null } }),
        ] {
            assert!(
                matches!(
                    parse_access_token(body.to_string().as_bytes(), NOW),
                    Err(CredentialsError::NoToken)
                ),
                "{body}"
            );
        }
        for bytes in [&b"not json"[..], b"", b"{", br#"{"claudeAiOauth": 7}"#] {
            assert!(matches!(
                parse_access_token(bytes, NOW),
                Err(CredentialsError::Malformed)
            ));
        }
    }

    #[test]
    fn reads_tokens_from_disk() {
        let path = scratch_file("valid", &credentials(None));
        let token = read_access_token(&path, NOW).expect("token from disk");
        assert_eq!(token.secret(), FAKE_TOKEN);
        fs::remove_file(&path).expect("cleanup");
    }

    #[test]
    fn missing_files_are_reported_as_missing() {
        let path = std::env::temp_dir().join("plimsoll-credentials-absent.json");
        assert!(matches!(
            read_access_token(&path, NOW),
            Err(CredentialsError::Missing)
        ));
    }

    #[test]
    fn oversized_files_are_rejected_before_parsing() {
        let path = scratch_file("oversized", &vec![b' '; MAX_CREDENTIALS_BYTES + 1]);
        assert!(matches!(
            read_access_token(&path, NOW),
            Err(CredentialsError::TooLarge)
        ));
        fs::remove_file(&path).expect("cleanup");
    }
}
