use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("tauri runtime error: {0}")]
    Tauri(#[from] tauri::Error),
    #[error("default window icon is missing")]
    MissingIcon,
    #[error("window {0} is missing")]
    MissingWindow(&'static str),
}
