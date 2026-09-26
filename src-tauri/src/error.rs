use thiserror::Error;

use crate::store::DatabaseError;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("tauri runtime error: {0}")]
    Tauri(#[from] tauri::Error),
    #[error(transparent)]
    Database(#[from] DatabaseError),
    #[error("tray icon could not be rendered")]
    IconRender,
    #[error("tray icon is missing")]
    MissingTray,
    #[error("window {0} is missing")]
    MissingWindow(&'static str),
}
