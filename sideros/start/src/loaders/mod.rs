//! Service loaders for different configuration formats.
//!
//! This module provides a trait-based architecture for loading service
//! definitions from various configuration formats, including:
//!
//! - TOML (native sideros format)
//! - systemd unit files (.service)
//!
//! # Migration Support
//!
//! The systemd loader also provides utilities to convert systemd unit files
//! to the native TOML format for easier management and migration.

pub mod systemd;
pub mod toml;

use crate::error::Result;
use crate::service::ServiceDefinition;
use std::path::Path;

/// Trait for service configuration loaders.
///
/// Implement this trait to add support for new configuration formats.
pub trait ServiceLoader: Send + Sync {
    /// Load a service definition from the given path.
    fn load(&self, path: &Path) -> Result<ServiceDefinition>;

    /// Check if this loader supports the given file extension.
    fn supports_extension(&self, ext: &str) -> bool;

    /// Get a description of the loader for logging purposes.
    fn name(&self) -> &'static str;
}

/// Registry of service loaders.
pub struct LoaderRegistry {
    loaders: Vec<Box<dyn ServiceLoader>>,
}

impl Default for LoaderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl LoaderRegistry {
    /// Create a new loader registry with default loaders.
    pub fn new() -> Self {
        let mut registry = Self {
            loaders: Vec::new(),
        };

        // Register default loaders
        registry.register(Box::new(toml::TomlLoader));
        registry.register(Box::new(systemd::SystemdLoader));

        registry
    }

    /// Register a new loader.
    pub fn register(&mut self, loader: Box<dyn ServiceLoader>) {
        self.loaders.push(loader);
    }

    /// Find a loader that supports the given file extension.
    pub fn find_loader(&self, ext: &str) -> Option<&dyn ServiceLoader> {
        self.loaders
            .iter()
            .find(|loader| loader.supports_extension(ext))
            .map(|b| b.as_ref())
    }

    /// Load a service definition from the given path.
    ///
    /// Automatically selects the appropriate loader based on the file extension.
    pub fn load(&self, path: &Path) -> Result<ServiceDefinition> {
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        let loader = self
            .find_loader(ext)
            .ok_or_else(|| crate::error::Error::ConfigError(
                format!("No loader found for extension: {}", ext)
            ))?;

        loader.load(path)
    }

    /// Get all supported file extensions.
    pub fn supported_extensions(&self) -> Vec<&'static str> {
        let mut exts = Vec::new();
        for loader in &self.loaders {
            if loader.supports_extension("toml") && !exts.contains(&"toml") {
                exts.push("toml");
            }
            if loader.supports_extension("service") && !exts.contains(&"service") {
                exts.push("service");
            }
        }
        exts
    }
}

// Re-export main types
pub use systemd::SystemdLoader;
pub use toml::TomlLoader;
