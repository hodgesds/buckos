//! Package manager configuration

use crate::buck::BuckConfigOptions;
use crate::{Error, Result, UseConfig, WorldSet};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Main configuration for the package manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Root directory for the system (usually "/")
    pub root: PathBuf,
    /// Path to package database
    pub db_path: PathBuf,
    /// Cache directory for downloads and builds
    pub cache_dir: PathBuf,
    /// Path to Buck targets repository
    pub buck_repo: PathBuf,
    /// Path to Buck executable
    pub buck_path: PathBuf,
    /// Number of parallel jobs
    pub parallelism: usize,
    /// Repository configurations
    pub repositories: Vec<RepositoryConfig>,
    /// USE flag configuration
    pub use_flags: UseConfig,
    /// World set
    pub world: WorldSet,
    /// Architecture
    pub arch: String,
    /// CHOST
    pub chost: String,
    /// CFLAGS
    pub cflags: String,
    /// CXXFLAGS
    pub cxxflags: String,
    /// LDFLAGS
    pub ldflags: String,
    /// MAKEOPTS
    pub makeopts: String,
    /// Features
    pub features: HashSet<String>,
    /// Accept keywords
    pub accept_keywords: HashSet<String>,
    /// License acceptance
    pub accept_license: String,
    /// Custom Buck configuration options
    #[serde(default)]
    pub buck_config: BuckConfigOptions,
}

impl Default for Config {
    fn default() -> Self {
        let parallelism = std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(4);

        Self {
            root: PathBuf::from("/"),
            db_path: PathBuf::from("/var/db/buckos"),
            cache_dir: PathBuf::from("/var/cache/buckos"),
            buck_repo: PathBuf::from("/var/db/repos/buckos"),
            buck_path: PathBuf::from("/usr/bin/buck2"),
            parallelism,
            repositories: vec![RepositoryConfig::default()],
            use_flags: UseConfig::default(),
            world: WorldSet::default(),
            arch: detect_arch(),
            chost: detect_chost(),
            cflags: "-O2 -pipe".to_string(),
            cxxflags: "${CFLAGS}".to_string(),
            ldflags: "-Wl,-O1 -Wl,--as-needed".to_string(),
            makeopts: format!("-j{}", parallelism),
            features: default_features(),
            accept_keywords: HashSet::new(),
            accept_license: "@FREE".to_string(),
            buck_config: BuckConfigOptions::default(),
        }
    }
}

impl Config {
    /// Load configuration from default locations
    pub fn load() -> Result<Self> {
        let config_path = PathBuf::from("/etc/buckos/buckos.toml");
        if config_path.exists() {
            Self::load_from(&config_path)
        } else {
            Ok(Self::default())
        }
    }

    /// Load configuration from a specific path
    pub fn load_from(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to a path
    pub fn save_to(&self, path: &Path) -> Result<()> {
        let content =
            toml::to_string_pretty(self).map_err(|e| Error::ConfigError(e.to_string()))?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Get the full path for a system path
    pub fn system_path(&self, path: impl AsRef<Path>) -> PathBuf {
        self.root
            .join(path.as_ref().strip_prefix("/").unwrap_or(path.as_ref()))
    }

    /// Get the download cache directory
    pub fn download_cache(&self) -> PathBuf {
        self.cache_dir.join("distfiles")
    }

    /// Get the build directory
    pub fn build_dir(&self) -> PathBuf {
        self.cache_dir.join("build")
    }

    /// Get the packages cache directory
    pub fn packages_dir(&self) -> PathBuf {
        self.cache_dir.join("packages")
    }
}

/// Repository configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryConfig {
    pub name: String,
    pub location: PathBuf,
    pub sync_type: SyncType,
    pub sync_uri: String,
    pub priority: i32,
    pub auto_sync: bool,
}

impl Default for RepositoryConfig {
    fn default() -> Self {
        Self {
            name: "buckos".to_string(),
            location: PathBuf::from("/var/db/repos/buckos"),
            sync_type: SyncType::Git,
            sync_uri: "https://github.com/hodgesds/packages.git".to_string(),
            priority: 0,
            auto_sync: true,
        }
    }
}

/// Repository sync type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncType {
    Git,
    Rsync,
    Http,
    Local,
}

fn detect_arch() -> String {
    #[cfg(target_arch = "x86_64")]
    return "amd64".to_string();

    #[cfg(target_arch = "aarch64")]
    return "arm64".to_string();

    #[cfg(target_arch = "x86")]
    return "x86".to_string();

    #[cfg(target_arch = "arm")]
    return "arm".to_string();

    #[cfg(not(any(
        target_arch = "x86_64",
        target_arch = "aarch64",
        target_arch = "x86",
        target_arch = "arm"
    )))]
    return "unknown".to_string();
}

fn detect_chost() -> String {
    #[cfg(target_arch = "x86_64")]
    return "x86_64-pc-linux-gnu".to_string();

    #[cfg(target_arch = "aarch64")]
    return "aarch64-unknown-linux-gnu".to_string();

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    return "unknown-unknown-linux-gnu".to_string();
}

fn default_features() -> HashSet<String> {
    let mut features = HashSet::new();
    features.insert("parallel-fetch".to_string());
    features.insert("parallel-install".to_string());
    features.insert("candy".to_string());
    features.insert("buildpkg".to_string());
    features.insert("clean-logs".to_string());
    features
}
