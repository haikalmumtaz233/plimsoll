mod app;
pub mod domain;
mod error;
pub mod sources;
pub mod store;
mod tray;

pub use app::run;
pub use error::AppError;
