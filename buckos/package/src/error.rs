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

    #[error("Invalid provider {provider} for virtual package {virtual_pkg}")]
    InvalidProvider {
        virtual_pkg: String,
        provider: String,
    },

    #[error("Invalid blocker: {0}")]
    InvalidBlocker(String),

    #[error("Sandbox error: {0}")]
    SandboxError(String),

    #[error("Signing error: {0}")]
    Signing(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Fetch restricted: {filename} - {message}")]
    FetchRestricted {
        filename: String,
        message: String,
    },

    #[error("Download failed for {filename}: {reason}")]
    DistfileDownloadFailed {
        filename: String,
        reason: String,
    },

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Too many config files for: {0}")]
    TooManyConfigFiles(std::path::PathBuf),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("File not found: {0}")]
    FileNotFound(std::path::PathBuf),

    #[error("News not found: {0}")]
    NewsNotFound(String),

    #[error("Profile not found: {0}")]
    ProfileNotFound(String),

    #[error("Invalid profile: {0}")]
    InvalidProfile(String),

    #[error("Profile inheritance cycle detected: {0}")]
    ProfileCycle(String),

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
