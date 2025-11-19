//! Error types for configuration operations

use std::path::PathBuf;
use thiserror::Error;

/// Configuration errors
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("TOML serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("Invalid configuration: {0}")]
    Invalid(String),

    #[error("Configuration file not found: {0}")]
    NotFound(PathBuf),

    #[error("Invalid package atom: {0}")]
    InvalidAtom(String),

    #[error("Invalid USE flag: {0}")]
    InvalidUseFlag(String),

    #[error("Invalid keyword: {0}")]
    InvalidKeyword(String),

    #[error("Invalid license: {0}")]
    InvalidLicense(String),

    #[error("Profile not found: {0}")]
    ProfileNotFound(String),

    #[error("Circular profile inheritance: {0}")]
    CircularProfile(String),

    #[error("Invalid glob pattern: {0}")]
    InvalidGlob(String),

    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    #[error("Variable expansion error: {0}")]
    VariableExpansion(String),
}

/// Result type alias for configuration operations
pub type Result<T> = std::result::Result<T, ConfigError>;
