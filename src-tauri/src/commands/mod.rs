use serde::{Serialize, Serializer};
use tauri::{AppHandle, Runtime};
use thiserror::Error;

use crate::app::runtime;
use crate::app::view::UsageView;

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("usage data is unavailable")]
    Unavailable,
}

impl Serialize for CommandError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

#[tauri::command(async)]
#[allow(clippy::needless_pass_by_value)]
pub fn usage_summary<R: Runtime>(app: AppHandle<R>) -> Result<UsageView, CommandError> {
    runtime::current_view(&app).ok_or(CommandError::Unavailable)
}

#[tauri::command(async)]
#[allow(clippy::needless_pass_by_value)]
pub fn set_accurate_mode<R: Runtime>(
    app: AppHandle<R>,
    enabled: bool,
) -> Result<UsageView, CommandError> {
    runtime::set_accurate_mode(&app, enabled).ok_or(CommandError::Unavailable)
}

#[cfg(test)]
mod tests {
    use super::CommandError;

    #[test]
    fn errors_reach_the_frontend_as_plain_messages() {
        assert_eq!(
            serde_json::to_string(&CommandError::Unavailable).ok(),
            Some("\"usage data is unavailable\"".to_owned())
        );
    }
}
