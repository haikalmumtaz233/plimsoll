pub mod app;
mod commands;
pub mod diagnostics;
pub mod domain;
mod error;
pub mod i18n;
pub mod sources;
pub mod store;
pub mod toast;
pub mod tray;

pub use app::run;
pub use error::AppError;
