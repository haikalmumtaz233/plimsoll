use serde::Serialize;

use super::OAuthError;
use super::credentials::CredentialsError;
use super::poll::PollResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OAuthStatus {
    Disabled,
    Pending,
    Active,
    SignedOut,
    TokenExpired,
    Unauthorized,
    Unavailable,
    Retrying,
}

impl OAuthStatus {
    #[must_use]
    pub const fn from_result(result: &PollResult) -> Self {
        match result {
            Ok(_) => Self::Active,
            Err(error) => Self::from_error(error),
        }
    }

    #[must_use]
    pub const fn from_error(error: &OAuthError) -> Self {
        match error {
            OAuthError::Credentials(CredentialsError::Missing | CredentialsError::NoToken) => {
                Self::SignedOut
            }
            OAuthError::Credentials(CredentialsError::Expired) => Self::TokenExpired,
            OAuthError::Unauthorized => Self::Unauthorized,
            OAuthError::Credentials(
                CredentialsError::Io(_) | CredentialsError::TooLarge | CredentialsError::Malformed,
            )
            | OAuthError::EndpointGone
            | OAuthError::SchemaChanged => Self::Unavailable,
            OAuthError::Transport(_) | OAuthError::RateLimited { .. } | OAuthError::Status(_) => {
                Self::Retrying
            }
        }
    }

    #[must_use]
    pub const fn uses_fallback(self) -> bool {
        !matches!(self, Self::Active)
    }
}

#[cfg(test)]
mod tests {
    use super::OAuthStatus;
    use crate::domain::extras::OfficialUsage;
    use crate::sources::oauth::OAuthError;
    use crate::sources::oauth::credentials::CredentialsError;
    use crate::sources::oauth::transport::TransportError;
    use std::io;

    #[test]
    fn successful_polls_are_active_and_skip_the_fallback() {
        let status = OAuthStatus::from_result(&Ok(OfficialUsage {
            limits: Vec::new(),
            models: Vec::new(),
            credits: None,
        }));
        assert_eq!(status, OAuthStatus::Active);
        assert!(!status.uses_fallback());
    }

    #[test]
    fn errors_map_to_user_facing_statuses() {
        let cases = [
            (
                OAuthError::Credentials(CredentialsError::Missing),
                OAuthStatus::SignedOut,
            ),
            (
                OAuthError::Credentials(CredentialsError::NoToken),
                OAuthStatus::SignedOut,
            ),
            (
                OAuthError::Credentials(CredentialsError::Expired),
                OAuthStatus::TokenExpired,
            ),
            (
                OAuthError::Credentials(CredentialsError::Io(io::Error::other("locked"))),
                OAuthStatus::Unavailable,
            ),
            (
                OAuthError::Credentials(CredentialsError::Malformed),
                OAuthStatus::Unavailable,
            ),
            (OAuthError::Unauthorized, OAuthStatus::Unauthorized),
            (OAuthError::EndpointGone, OAuthStatus::Unavailable),
            (OAuthError::SchemaChanged, OAuthStatus::Unavailable),
            (
                OAuthError::RateLimited { retry_after: None },
                OAuthStatus::Retrying,
            ),
            (OAuthError::Status(503), OAuthStatus::Retrying),
            (
                OAuthError::Transport(TransportError::BodyTooLarge),
                OAuthStatus::Retrying,
            ),
        ];
        for (error, expected) in cases {
            let status = OAuthStatus::from_error(&error);
            assert_eq!(status, expected, "{error}");
            assert!(status.uses_fallback());
        }
        assert!(OAuthStatus::Disabled.uses_fallback());
        assert!(OAuthStatus::Pending.uses_fallback());
    }

    #[test]
    fn statuses_serialize_as_snake_case_for_the_frontend() {
        assert_eq!(
            serde_json::to_string(&OAuthStatus::TokenExpired).ok(),
            Some("\"token_expired\"".to_owned())
        );
        assert_eq!(
            serde_json::to_string(&OAuthStatus::SignedOut).ok(),
            Some("\"signed_out\"".to_owned())
        );
    }
}
