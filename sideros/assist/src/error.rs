//! Error types for the sideros-assist crate.

use thiserror::Error;

/// Result type alias for sideros-assist operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during assist operations.
#[derive(Error, Debug)]
pub enum Error {
    /// Failed to collect system information.
    #[error("Failed to collect system information: {0}")]
    CollectionError(String),

    /// Failed to read file or directory.
    #[error("Failed to read {path}: {reason}")]
    IoError { path: String, reason: String },

    /// Failed to serialize or deserialize data.
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Failed to write report.
    #[error("Failed to write report to {path}: {reason}")]
    ReportWriteError { path: String, reason: String },

    /// Invalid configuration.
    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    /// User cancelled operation.
    #[error("Operation cancelled by user")]
    UserCancelled,

    /// Privacy policy violation.
    #[error("Privacy policy violation: {0}")]
    PrivacyViolation(String),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError {
            path: String::new(),
            reason: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::SerializationError(err.to_string())
    }
}
