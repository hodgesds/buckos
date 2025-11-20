//! Complete Portage-style configuration
//!
//! This module brings together all configuration components into a
//! unified system configuration that mirrors /etc/portage structure.

use crate::{
    EnvConfig, KeywordConfig, LicenseConfig, MakeConf, MaskConfig, PackageAtom, ProfileConfig,
    ReposConfig, Result, SetsConfig, UseConfig,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Complete system configuration
///
/// This mirrors the structure of /etc/portage in Gentoo:
/// - make.conf -> Global settings
/// - package.use -> Per-package USE flags
/// - package.accept_keywords -> Per-package keywords
/// - package.license -> Per-package licenses
/// - package.mask -> Package masks
/// - package.unmask -> Package unmasks
/// - package.env -> Per-package environment
/// - repos.conf -> Repository configuration
/// - profile -> System profile
/// - sets -> Custom package sets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortageConfig {
    /// Global make.conf settings
    pub make_conf: MakeConf,

    /// Repository configuration (repos.conf)
    pub repos: ReposConfig,

    /// Profile configuration
    pub profile: ProfileConfig,

    /// Package sets
    pub sets: SetsConfig,

    /// Per-package USE flags (package.use)
    pub package_use: UseConfig,

    /// Per-package keywords (package.accept_keywords)
    pub package_keywords: KeywordConfig,

    /// Per-package licenses (package.license)
    pub package_license: LicenseConfig,

    /// Package masks (package.mask)
    pub package_mask: MaskConfig,

    /// Per-package environment (package.env)
    pub package_env: EnvConfig,

    /// Configuration root path
    pub config_root: PathBuf,
}

impl Default for PortageConfig {
    fn default() -> Self {
        Self {
            make_conf: MakeConf::default(),
            repos: ReposConfig::default_buckos(),
            profile: ProfileConfig::default(),
            sets: SetsConfig::with_defaults(),
            package_use: UseConfig::default(),
            package_keywords: KeywordConfig::new("amd64"),
            package_license: LicenseConfig::default(),
            package_mask: MaskConfig::default(),
            package_env: EnvConfig::default(),
            config_root: PathBuf::from("/etc/buckos"),
        }
    }
}

impl PortageConfig {
    /// Create a new configuration with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Load configuration from the filesystem
    pub fn load(config_root: &Path) -> Result<Self> {
        let mut config = Self::default();
        config.config_root = config_root.to_path_buf();

        // Load make.conf
        let make_conf_path = config_root.join("make.conf");
        if make_conf_path.exists() {
            config.make_conf = MakeConf::load(&make_conf_path)?;
        }

        // Load repos.conf
        let repos_conf_path = config_root.join("repos.conf");
        if repos_conf_path.exists() {
            config.repos = crate::repos::parse_repos_conf(&repos_conf_path)?;
        }

        // Load package.use
        let package_use_path = config_root.join("package.use");
        if package_use_path.exists() {
            config.package_use = load_package_use(&package_use_path)?;
        }

        // Load package.accept_keywords
        let keywords_path = config_root.join("package.accept_keywords");
        if keywords_path.exists() {
            config.package_keywords = load_package_keywords(&keywords_path)?;
        }

        // Load package.license
        let license_path = config_root.join("package.license");
        if license_path.exists() {
            config.package_license = load_package_license(&license_path)?;
        }

        // Load package.mask
        let mask_path = config_root.join("package.mask");
        if mask_path.exists() {
            let content = read_config_path(&mask_path)?;
            config.package_mask.masked = crate::mask::parse_mask_file(&content);
        }

        // Load package.unmask
        let unmask_path = config_root.join("package.unmask");
        if unmask_path.exists() {
            let content = read_config_path(&unmask_path)?;
            config.package_mask.unmasked = crate::mask::parse_mask_file(&content);
        }

        // Load sets
        let sets_path = config_root.join("sets");
        if sets_path.exists() {
            config.sets = SetsConfig::load_from_dir(&sets_path)?;
        }

        // Load world file
        let world_path = config_root.join("world");
        if world_path.exists() {
            let content = std::fs::read_to_string(&world_path)?;
            if let Ok(world_set) = crate::sets::PackageSet::parse("world", &content) {
                config.sets.sets.insert("world".to_string(), world_set);
            }
        }

        Ok(config)
    }

    /// Save configuration to the filesystem
    pub fn save(&self) -> Result<()> {
        // Ensure directories exist
        std::fs::create_dir_all(&self.config_root)?;
        std::fs::create_dir_all(self.config_root.join("repos.conf"))?;
        std::fs::create_dir_all(self.config_root.join("sets"))?;

        // Save make.conf
        self.make_conf.save(&self.config_root.join("make.conf"))?;

        // Save world set
        self.sets.save_world(&self.config_root.join("world"))?;

        Ok(())
    }

    /// Get effective USE flags for a package
    pub fn effective_use(&self, category: &str, name: &str) -> std::collections::HashSet<String> {
        // Start with global flags from make.conf
        let mut flags = self.make_conf.use_config.effective_flags(category, name);

        // Apply profile flags
        for flag in self.profile.use_config.effective_flags(category, name) {
            flags.insert(flag);
        }

        // Apply package.use overrides - need to check for disabled flags
        for entry in &self.package_use.package {
            if entry.atom.matches_cpn(category, name) {
                for flag in &entry.flags {
                    if flag.enabled {
                        flags.insert(flag.name.clone());
                    } else {
                        flags.remove(&flag.name);
                    }
                }
            }
        }

        flags
    }

    /// Check if a package is keyword-acceptable
    pub fn is_keyword_acceptable(&self, category: &str, name: &str, keywords: &[&str]) -> bool {
        // First check global keywords from make.conf
        if self
            .make_conf
            .keywords
            .is_acceptable(category, name, keywords)
        {
            return true;
        }

        // Then check per-package keywords
        self.package_keywords
            .is_acceptable(category, name, keywords)
    }

    /// Check if a license is acceptable for a package
    pub fn is_license_acceptable(&self, category: &str, name: &str, license: &str) -> bool {
        // First check global license from make.conf
        if self
            .make_conf
            .license
            .is_accepted_for(category, name, license)
        {
            return true;
        }

        // Then check per-package license
        self.package_license
            .is_accepted_for(category, name, license)
    }

    /// Check if a package is masked
    pub fn is_masked(&self, category: &str, name: &str, version: Option<&str>) -> bool {
        // Check profile masks
        if self.profile.mask_config.is_masked(category, name, version) {
            // But allow user unmask
            if self
                .package_mask
                .unmasked
                .iter()
                .any(|e| e.atom.matches_cpn(category, name))
            {
                return false;
            }
            return true;
        }

        // Check user masks
        self.package_mask.is_masked(category, name, version)
    }

    /// Get environment for building a package
    pub fn build_env(&self, category: &str, name: &str) -> indexmap::IndexMap<String, String> {
        let mut env = self.make_conf.build_env();

        // Apply per-package environment
        let pkg_env = self.package_env.effective_env(category, name);
        for (key, value) in pkg_env {
            env.insert(key, value);
        }

        env
    }

    /// Add a package to the world set
    pub fn add_to_world(&mut self, atom: PackageAtom) -> Result<()> {
        self.sets.add_to_set("world", atom)
    }

    /// Remove a package from the world set
    pub fn remove_from_world(&mut self, category: &str, name: &str) -> Result<bool> {
        self.sets.remove_from_set("world", category, name)
    }

    /// Check if a package is in the world set
    pub fn is_in_world(&self, category: &str, name: &str) -> bool {
        self.sets
            .get("world")
            .map(|w| w.contains(category, name))
            .unwrap_or(false)
    }

    /// Accept testing keywords for a package
    pub fn accept_testing(&mut self, atom: PackageAtom) {
        let arch = self.make_conf.arch.clone();
        self.package_keywords
            .add_package_keywords(atom, vec![crate::keywords::Keyword::testing(arch)]);
    }

    /// Unmask a package
    pub fn unmask(&mut self, atom: PackageAtom, reason: Option<String>) {
        self.package_mask.add_unmask(atom, reason);
    }

    /// Accept a license for a package
    pub fn accept_license(&mut self, atom: PackageAtom, licenses: Vec<String>) {
        self.package_license.add_package_license(atom, licenses);
    }

    /// Set per-package USE flags
    pub fn set_package_use(&mut self, atom: PackageAtom, flags: Vec<crate::use_flags::UseFlag>) {
        self.package_use.add_package_use(atom, flags);
    }

    /// Get all repositories
    pub fn repositories(&self) -> Vec<&crate::repos::Repository> {
        self.repos.repos_by_priority()
    }

    /// Get the main repository
    pub fn main_repo(&self) -> Option<&crate::repos::Repository> {
        self.repos.main_repo()
    }
}

/// Load package.use from a path (file or directory)
fn load_package_use(path: &Path) -> Result<UseConfig> {
    let mut config = UseConfig::default();
    let content = read_config_path(path)?;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            if let Ok(atom) = parts[0].parse::<PackageAtom>() {
                let flags = UseConfig::parse_use_string(&parts[1..].join(" "));
                config.add_package_use(atom, flags);
            }
        }
    }

    Ok(config)
}

/// Load package.accept_keywords from a path
fn load_package_keywords(path: &Path) -> Result<KeywordConfig> {
    let mut config = KeywordConfig::new("amd64"); // Default arch
    let content = read_config_path(path)?;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if !parts.is_empty() {
            if let Ok(atom) = parts[0].parse::<PackageAtom>() {
                let keywords = if parts.len() > 1 {
                    KeywordConfig::parse_keywords_string(&parts[1..].join(" "))
                } else {
                    // Default to ~arch
                    vec![crate::keywords::Keyword::testing(&config.arch)]
                };
                config.add_package_keywords(atom, keywords);
            }
        }
    }

    Ok(config)
}

/// Load package.license from a path
fn load_package_license(path: &Path) -> Result<LicenseConfig> {
    let mut config = LicenseConfig::default();
    let content = read_config_path(path)?;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            if let Ok(atom) = parts[0].parse::<PackageAtom>() {
                let licenses: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();
                config.add_package_license(atom, licenses);
            }
        }
    }

    Ok(config)
}

/// Read content from a path (file or directory)
fn read_config_path(path: &Path) -> Result<String> {
    if path.is_dir() {
        // Read all files in directory
        let mut content = String::new();
        let mut entries: Vec<_> = std::fs::read_dir(path)?.filter_map(|e| e.ok()).collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let file_path = entry.path();
            if file_path.is_file() {
                content.push_str(&std::fs::read_to_string(file_path)?);
                content.push('\n');
            }
        }
        Ok(content)
    } else {
        Ok(std::fs::read_to_string(path)?)
    }
}

/// Configuration builder for creating configurations programmatically
pub struct PortageConfigBuilder {
    config: PortageConfig,
}

impl PortageConfigBuilder {
    /// Create a new builder with default configuration
    pub fn new() -> Self {
        Self {
            config: PortageConfig::default(),
        }
    }

    /// Set the configuration root
    pub fn config_root(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.config_root = path.into();
        self
    }

    /// Set compiler flags
    pub fn cflags(mut self, flags: impl Into<String>) -> Self {
        self.config.make_conf.cflags = flags.into();
        self
    }

    /// Set make options
    pub fn makeopts(mut self, opts: impl Into<String>) -> Self {
        self.config.make_conf.makeopts = opts.into();
        self
    }

    /// Add global USE flags
    pub fn use_flags(mut self, flags: &[&str]) -> Self {
        for flag in flags {
            self.config.make_conf.use_config.add_global(*flag);
        }
        self
    }

    /// Enable a feature
    pub fn enable_feature(mut self, feature: impl Into<String>) -> Self {
        self.config.make_conf.features.enable(feature);
        self
    }

    /// Disable a feature
    pub fn disable_feature(mut self, feature: impl Into<String>) -> Self {
        self.config.make_conf.features.disable(feature);
        self
    }

    /// Accept testing keywords
    pub fn accept_testing(mut self) -> Self {
        self.config.make_conf.keywords.accept_testing();
        self
    }

    /// Set profile
    pub fn profile(mut self, profile: impl Into<String>) -> Self {
        self.config.profile = ProfileConfig::new(profile);
        self
    }

    /// Add a repository
    pub fn add_repo(mut self, repo: crate::repos::Repository) -> Self {
        self.config.repos.add_repo(repo);
        self
    }

    /// Build the configuration
    pub fn build(self) -> PortageConfig {
        self.config
    }
}

impl Default for PortageConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_portage_config_default() {
        let config = PortageConfig::default();
        assert!(!config.make_conf.cflags.is_empty());
        assert!(config.repos.has_repo("buckos"));
    }

    #[test]
    fn test_effective_use() {
        let mut config = PortageConfig::default();
        config.make_conf.use_config.add_global("X");

        let atom = PackageAtom::new("app-editors", "vim");
        config
            .package_use
            .add_package_use(atom, vec![crate::use_flags::UseFlag::disabled("X")]);

        let flags = config.effective_use("app-editors", "vim");
        assert!(!flags.contains("X"));

        let flags = config.effective_use("app-editors", "emacs");
        assert!(flags.contains("X"));
    }

    #[test]
    fn test_builder() {
        let config = PortageConfigBuilder::new()
            .cflags("-O3 -march=native")
            .use_flags(&["X", "wayland", "systemd"])
            .enable_feature("ccache")
            .accept_testing()
            .build();

        assert_eq!(config.make_conf.cflags, "-O3 -march=native");
        assert!(config.make_conf.use_config.global.contains("X"));
        assert!(config.make_conf.features.is_enabled("ccache"));
    }
}
