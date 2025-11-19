//! Error types for the package manager

use thiserror::Error;

/// Result type alias for package manager operations
pub type Result<T> = std::result::Result<T, Error>;

/// Package manager errors
#[derive(Error, Debug)]
pub enum Error {
    #[error("Package not found: {0}")]
    PackageNotFound(String),

    #[error("Package not installed: {0}")]
    PackageNotInstalled(String),

    #[error("Package already installed: {0}")]
    PackageAlreadyInstalled(String),

    #[error("Package {package} has dependents: {dependents:?}")]
    HasDependents {
        package: String,
        dependents: Vec<String>,
    },

    #[error("Dependency resolution failed: {0}")]
    ResolutionFailed(String),

    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),

    #[error("Version conflict: {package} requires {required} but {installed} is installed")]
    VersionConflict {
        package: String,
        required: String,
        installed: String,
    },

    #[error("Build failed for {package}: {message}")]
    BuildFailed { package: String, message: String },

    #[error("Buck error: {0}")]
    BuckError(String),

    #[error("Download failed for {url}: {message}")]
    DownloadFailed { url: String, message: String },

    #[error("Checksum mismatch for {path}: expected {expected}, got {actual}")]
    ChecksumMismatch {
        path: String,
        expected: String,
        actual: String,
    },

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    #[error("Transaction rolled back: {0}")]
    TransactionRolledBack(String),

    #[error("Repository error: {0}")]
    RepositoryError(String),

    #[error("Repository not found: {0}")]
    RepositoryNotFound(String),

    #[error("Invalid package specification: {0}")]
    InvalidPackageSpec(String),

    #[error("Invalid version: {0}")]
    InvalidVersion(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("TOML error: {0}")]
    TomlError(#[from] toml::de::Error),

    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("SQLite error: {0}")]
    SqliteError(#[from] rusqlite::Error),

    #[error("Walk directory error: {0}")]
    WalkDirError(#[from] walkdir::Error),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Operation cancelled")]
    Cancelled,

    #[error("Unsupported architecture: {0}")]
    UnsupportedArch(String),

    #[error("Missing required USE flag: {0}")]
    MissingUseFlag(String),

    #[error("Blocked USE flag: {0}")]
    BlockedUseFlag(String),

    #[error("Slot conflict: {0}")]
    SlotConflict(String),

    #[error("{0}")]
    Other(String),
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::Other(err.to_string())
    }
}

impl From<dialoguer::Error> for Error {
    fn from(err: dialoguer::Error) -> Self {
        Error::Other(format!("User input error: {}", err))
    }
}
