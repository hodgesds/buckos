//! TOML service loader.
//!
//! This is the native configuration format for buckos services.

use crate::error::{Error, Result};
use crate::service::ServiceDefinition;
use std::path::Path;

/// Loader for TOML service configuration files.
pub struct TomlLoader;

impl super::ServiceLoader for TomlLoader {
    fn load(&self, path: &Path) -> Result<ServiceDefinition> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| Error::ConfigError(format!("Failed to read {}: {}", path.display(), e)))?;

        let def: ServiceDefinition = toml::from_str(&content).map_err(|e| {
            Error::ConfigError(format!("Failed to parse TOML {}: {}", path.display(), e))
        })?;

        Ok(def)
    }

    fn supports_extension(&self, ext: &str) -> bool {
        ext == "toml"
    }

    fn name(&self) -> &'static str {
        "TOML"
    }
}

impl TomlLoader {
    /// Create a new TOML loader.
    pub fn new() -> Self {
        Self
    }
}

impl Default for TomlLoader {
    fn default() -> Self {
        Self::new()
    }
}
