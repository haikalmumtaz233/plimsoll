pub mod jsonl;
pub mod rfc3339;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SourceError {
    #[error("failed to read usage files: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to watch usage files: {0}")]
    Watch(#[from] notify::Error),
}
