use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("tauri runtime error: {0}")]
    Tauri(#[from] tauri::Error),
    #[error("tray icon could not be rendered")]
    IconRender,
    #[error("tray icon is missing")]
    MissingTray,
    #[error("window {0} is missing")]
    MissingWindow(&'static str),
}
