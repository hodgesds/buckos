//! Configuration loading utilities
//!
//! Provides utilities for loading and parsing configuration files
//! from the filesystem in various formats.

use crate::{ConfigError, PortageConfig, Result};
use std::path::{Path, PathBuf};

/// Configuration loader for loading system configuration
pub struct ConfigLoader {
    /// Root path for configuration
    root: PathBuf,
    /// Whether to use default values for missing configs
    use_defaults: bool,
    /// Whether to validate configuration after loading
    validate: bool,
}

impl ConfigLoader {
    /// Create a new configuration loader
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            use_defaults: true,
            validate: true,
        }
    }

    /// Create a loader for the default system configuration
    pub fn system() -> Self {
        Self::new("/etc/buckos")
    }

    /// Create a loader for user configuration
    pub fn user() -> Result<Self> {
        let home =
            std::env::var("HOME").map_err(|_| ConfigError::Invalid("HOME not set".to_string()))?;
        Ok(Self::new(PathBuf::from(home).join(".config/buckos")))
    }

    /// Set whether to use defaults for missing configs
    pub fn use_defaults(mut self, use_defaults: bool) -> Self {
        self.use_defaults = use_defaults;
        self
    }

    /// Set whether to validate configuration
    pub fn validate(mut self, validate: bool) -> Self {
        self.validate = validate;
        self
    }

    /// Load the complete configuration
    pub fn load(&self) -> Result<PortageConfig> {
        if !self.root.exists() {
            if self.use_defaults {
                return Ok(PortageConfig::default());
            } else {
                return Err(ConfigError::NotFound(self.root.clone()));
            }
        }

        let config = PortageConfig::load(&self.root)?;

        if self.validate {
            validate_config(&config)?;
        }

        Ok(config)
    }

    /// Load configuration with overlay from another path
    pub fn load_with_overlay(&self, overlay: &Path) -> Result<PortageConfig> {
        let mut config = self.load()?;

        // Load overlay configuration
        if overlay.exists() {
            let overlay_config = PortageConfig::load(overlay)?;

            // Merge overlay into base config
            merge_configs(&mut config, &overlay_config);
        }

        Ok(config)
    }

    /// Get the configuration root path
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Check if a configuration file exists
    pub fn has_config(&self, name: &str) -> bool {
        self.root.join(name).exists()
    }

    /// Get path to a configuration file
    pub fn config_path(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }

    /// List all configuration files
    pub fn list_configs(&self) -> Result<Vec<String>> {
        if !self.root.exists() {
            return Ok(vec![]);
        }

        let mut configs = Vec::new();

        for entry in std::fs::read_dir(&self.root)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            configs.push(name);
        }

        configs.sort();
        Ok(configs)
    }
}

/// Validate configuration for common issues
fn validate_config(config: &PortageConfig) -> Result<()> {
    // Check CHOST format
    if !config.make_conf.chost.contains('-') {
        return Err(ConfigError::Invalid(format!(
            "Invalid CHOST format: {}",
            config.make_conf.chost
        )));
    }

    // Check paths exist or can be created
    let paths = [
        &config.make_conf.distdir,
        &config.make_conf.pkgdir,
        &config.make_conf.tmpdir,
    ];

    for path in paths {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                tracing::warn!("Parent directory does not exist: {}", parent.display());
            }
        }
    }

    // Check for conflicting USE flags
    let global_use = &config.make_conf.use_config.global;
    let conflicts = [
        ("gtk", "gtk2"),
        ("qt5", "qt6"), // May conflict in some cases
    ];

    for (a, b) in conflicts {
        if global_use.contains(a) && global_use.contains(b) {
            tracing::warn!("Potentially conflicting USE flags: {} and {}", a, b);
        }
    }

    // Validate repository configurations
    for repo in config.repos.repos.values() {
        if repo.location.as_os_str().is_empty() {
            return Err(ConfigError::Invalid(format!(
                "Repository '{}' has empty location",
                repo.name
            )));
        }
    }

    Ok(())
}

/// Merge overlay configuration into base
fn merge_configs(base: &mut PortageConfig, overlay: &PortageConfig) {
    // Merge USE flags
    base.make_conf
        .use_config
        .merge(&overlay.make_conf.use_config);
    base.package_use.merge(&overlay.package_use);

    // Merge features
    base.make_conf.features.merge(&overlay.make_conf.features);

    // Merge masks
    base.package_mask.merge(&overlay.package_mask);

    // Merge environment
    base.package_env.merge(&overlay.package_env);

    // Merge repositories
    for (name, repo) in &overlay.repos.repos {
        base.repos.repos.insert(name.clone(), repo.clone());
    }

    // Merge package keywords
    base.package_keywords
        .package
        .extend(overlay.package_keywords.package.iter().cloned());

    // Merge package licenses
    base.package_license
        .package
        .extend(overlay.package_license.package.iter().cloned());
}

/// Default configuration paths
pub mod paths {
    use std::path::PathBuf;

    /// System configuration root
    pub fn system_config() -> PathBuf {
        PathBuf::from("/etc/buckos")
    }

    /// System make.conf
    pub fn make_conf() -> PathBuf {
        system_config().join("make.conf")
    }

    /// Package database
    pub fn package_db() -> PathBuf {
        PathBuf::from("/var/db/buckos")
    }

    /// Cache directory
    pub fn cache() -> PathBuf {
        PathBuf::from("/var/cache/buckos")
    }

    /// Distfiles directory
    pub fn distfiles() -> PathBuf {
        cache().join("distfiles")
    }

    /// Binary packages directory
    pub fn binpkgs() -> PathBuf {
        cache().join("binpkgs")
    }

    /// Repository root
    pub fn repos() -> PathBuf {
        PathBuf::from("/var/db/repos")
    }

    /// World file
    pub fn world() -> PathBuf {
        system_config().join("world")
    }

    /// User configuration
    pub fn user_config() -> Option<PathBuf> {
        std::env::var("HOME")
            .ok()
            .map(|h| PathBuf::from(h).join(".config/buckos"))
    }
}

/// Environment variable names used by the configuration system
pub mod env_vars {
    /// Configuration root override
    pub const CONFIG_ROOT: &str = "BUCKOS_CONFIG_ROOT";
    /// Repository root override
    pub const REPO_ROOT: &str = "BUCKOS_REPO_ROOT";
    /// Cache directory override
    pub const CACHE_DIR: &str = "BUCKOS_CACHE_DIR";
    /// Portage compatibility variables
    pub const PORTAGE_CONFIGROOT: &str = "PORTAGE_CONFIGROOT";
    pub const PORTDIR: &str = "PORTDIR";
    pub const DISTDIR: &str = "DISTDIR";
    pub const PKGDIR: &str = "PKGDIR";
}

/// Get configuration root from environment or default
pub fn get_config_root() -> PathBuf {
    std::env::var(env_vars::CONFIG_ROOT)
        .or_else(|_| std::env::var(env_vars::PORTAGE_CONFIGROOT))
        .map(PathBuf::from)
        .unwrap_or_else(|_| paths::system_config())
}

/// Load the default system configuration
pub fn load_system_config() -> Result<PortageConfig> {
    let root = get_config_root();
    ConfigLoader::new(root).load()
}

/// Load user configuration (with system as base)
pub fn load_user_config() -> Result<PortageConfig> {
    let system_root = get_config_root();
    let user_root = paths::user_config()
        .ok_or_else(|| ConfigError::Invalid("Could not determine user config path".to_string()))?;

    ConfigLoader::new(system_root).load_with_overlay(&user_root)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_loader_defaults() {
        let loader = ConfigLoader::new("/nonexistent/path");
        let config = loader.load().unwrap();
        assert!(!config.make_conf.cflags.is_empty());
    }

    #[test]
    fn test_config_loader_no_defaults() {
        let loader = ConfigLoader::new("/nonexistent/path").use_defaults(false);
        let result = loader.load();
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_config() {
        let config = PortageConfig::default();
        validate_config(&config).unwrap();
    }

    #[test]
    fn test_paths() {
        assert_eq!(paths::system_config(), PathBuf::from("/etc/buckos"));
        assert_eq!(paths::package_db(), PathBuf::from("/var/db/buckos"));
    }

    #[test]
    fn test_merge_configs() {
        let mut base = PortageConfig::default();
        let mut overlay = PortageConfig::default();

        base.make_conf.use_config.add_global("X");
        overlay.make_conf.use_config.add_global("wayland");

        merge_configs(&mut base, &overlay);

        assert!(base.make_conf.use_config.global.contains("X"));
        assert!(base.make_conf.use_config.global.contains("wayland"));
    }
}
