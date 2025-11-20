//! FEATURES flags for build behavior control
//!
//! Implements Portage-compatible FEATURES flags that control various
//! aspects of the package build process.

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// All available FEATURES flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Feature {
    // Testing features
    /// Run package test suites
    Test,
    /// Fail if test suite fails (requires test)
    TestFailContinue,
    /// Build documentation
    Doc,

    // Compiler cache features
    /// Enable ccache for faster rebuilds
    Ccache,
    /// Enable distcc for distributed compilation
    Distcc,
    /// Use distcc before ccache in the path
    DistccPump,

    // Logging features
    /// Split build logs by phase
    SplitLog,
    /// Keep logs for binary package builds
    BinpkgLogs,
    /// Clean build logs on success
    CleanLogs,

    // Fetch features
    /// Fetch in parallel with building
    ParallelFetch,
    /// Fetch only, don't build
    FetchOnly,
    /// Mirror all distfiles locally
    Mirror,

    // Unmerge features
    /// Remove orphaned packages automatically
    UnmergeOrphans,
    /// Preserve libraries that may be in use
    PreserveLibs,

    // Sandbox features
    /// Enable filesystem sandbox
    Sandbox,
    /// Enable user sandbox
    Usersandbox,
    /// Enable network sandbox
    NetworkSandbox,

    // Binary package features
    /// Build binary packages
    Buildpkg,
    /// Use binary packages if available
    Getbinpkg,
    /// Prefer binary packages over building
    Binpkg,

    // Misc features
    /// Enable strict checking
    Strict,
    /// Keep work directory after build
    KeepWork,
    /// Enable debugging features
    Debug,
    /// Strip binaries
    Strip,
    /// Install EAPI 7+ docs
    InstallSources,
    /// Protect running processes from unmerge
    UnmergeBackup,
    /// Collision protection for file overwrites
    CollisionProtect,
    /// Protect /etc configuration files
    ProtectOwned,
}

impl Feature {
    /// Get all available features
    pub fn all() -> Vec<Feature> {
        vec![
            Feature::Test,
            Feature::TestFailContinue,
            Feature::Doc,
            Feature::Ccache,
            Feature::Distcc,
            Feature::DistccPump,
            Feature::SplitLog,
            Feature::BinpkgLogs,
            Feature::CleanLogs,
            Feature::ParallelFetch,
            Feature::FetchOnly,
            Feature::Mirror,
            Feature::UnmergeOrphans,
            Feature::PreserveLibs,
            Feature::Sandbox,
            Feature::Usersandbox,
            Feature::NetworkSandbox,
            Feature::Buildpkg,
            Feature::Getbinpkg,
            Feature::Binpkg,
            Feature::Strict,
            Feature::KeepWork,
            Feature::Debug,
            Feature::Strip,
            Feature::InstallSources,
            Feature::UnmergeBackup,
            Feature::CollisionProtect,
            Feature::ProtectOwned,
        ]
    }

    /// Get the string name of the feature
    pub fn name(&self) -> &'static str {
        match self {
            Feature::Test => "test",
            Feature::TestFailContinue => "test-fail-continue",
            Feature::Doc => "doc",
            Feature::Ccache => "ccache",
            Feature::Distcc => "distcc",
            Feature::DistccPump => "distcc-pump",
            Feature::SplitLog => "split-log",
            Feature::BinpkgLogs => "binpkg-logs",
            Feature::CleanLogs => "clean-logs",
            Feature::ParallelFetch => "parallel-fetch",
            Feature::FetchOnly => "fetch-only",
            Feature::Mirror => "mirror",
            Feature::UnmergeOrphans => "unmerge-orphans",
            Feature::PreserveLibs => "preserve-libs",
            Feature::Sandbox => "sandbox",
            Feature::Usersandbox => "usersandbox",
            Feature::NetworkSandbox => "network-sandbox",
            Feature::Buildpkg => "buildpkg",
            Feature::Getbinpkg => "getbinpkg",
            Feature::Binpkg => "binpkg",
            Feature::Strict => "strict",
            Feature::KeepWork => "keepwork",
            Feature::Debug => "debug",
            Feature::Strip => "strip",
            Feature::InstallSources => "install-sources",
            Feature::UnmergeBackup => "unmerge-backup",
            Feature::CollisionProtect => "collision-protect",
            Feature::ProtectOwned => "protect-owned",
        }
    }

    /// Get description of the feature
    pub fn description(&self) -> &'static str {
        match self {
            Feature::Test => "Run package test suites during build",
            Feature::TestFailContinue => "Continue even if tests fail",
            Feature::Doc => "Build and install documentation",
            Feature::Ccache => "Enable ccache for faster rebuilds",
            Feature::Distcc => "Enable distcc for distributed compilation",
            Feature::DistccPump => "Use distcc pump mode for preprocessing",
            Feature::SplitLog => "Split build logs by phase",
            Feature::BinpkgLogs => "Keep logs for binary package builds",
            Feature::CleanLogs => "Clean build logs on successful completion",
            Feature::ParallelFetch => "Fetch files in parallel with building",
            Feature::FetchOnly => "Only fetch source files, don't build",
            Feature::Mirror => "Mirror all distfiles locally",
            Feature::UnmergeOrphans => "Remove orphaned packages automatically",
            Feature::PreserveLibs => "Preserve libraries that may be in use",
            Feature::Sandbox => "Enable filesystem sandbox for builds",
            Feature::Usersandbox => "Enable user namespace sandbox",
            Feature::NetworkSandbox => "Enable network isolation for builds",
            Feature::Buildpkg => "Build binary packages after compiling",
            Feature::Getbinpkg => "Use binary packages if available",
            Feature::Binpkg => "Prefer binary packages over building from source",
            Feature::Strict => "Enable strict mode for builds",
            Feature::KeepWork => "Keep work directory after build",
            Feature::Debug => "Enable debug mode for builds",
            Feature::Strip => "Strip debug symbols from binaries",
            Feature::InstallSources => "Install source files for debugging",
            Feature::UnmergeBackup => "Backup files before unmerging",
            Feature::CollisionProtect => "Abort if file collisions are detected",
            Feature::ProtectOwned => "Protect files owned by other packages",
        }
    }

    /// Parse a feature name string to Feature
    pub fn parse(s: &str) -> Option<Feature> {
        match s.to_lowercase().as_str() {
            "test" => Some(Feature::Test),
            "test-fail-continue" => Some(Feature::TestFailContinue),
            "doc" => Some(Feature::Doc),
            "ccache" => Some(Feature::Ccache),
            "distcc" => Some(Feature::Distcc),
            "distcc-pump" => Some(Feature::DistccPump),
            "split-log" => Some(Feature::SplitLog),
            "binpkg-logs" => Some(Feature::BinpkgLogs),
            "clean-logs" => Some(Feature::CleanLogs),
            "parallel-fetch" => Some(Feature::ParallelFetch),
            "fetch-only" => Some(Feature::FetchOnly),
            "mirror" => Some(Feature::Mirror),
            "unmerge-orphans" => Some(Feature::UnmergeOrphans),
            "preserve-libs" => Some(Feature::PreserveLibs),
            "sandbox" => Some(Feature::Sandbox),
            "usersandbox" => Some(Feature::Usersandbox),
            "network-sandbox" => Some(Feature::NetworkSandbox),
            "buildpkg" => Some(Feature::Buildpkg),
            "getbinpkg" => Some(Feature::Getbinpkg),
            "binpkg" => Some(Feature::Binpkg),
            "strict" => Some(Feature::Strict),
            "keepwork" => Some(Feature::KeepWork),
            "debug" => Some(Feature::Debug),
            "strip" => Some(Feature::Strip),
            "install-sources" => Some(Feature::InstallSources),
            "unmerge-backup" => Some(Feature::UnmergeBackup),
            "collision-protect" => Some(Feature::CollisionProtect),
            "protect-owned" => Some(Feature::ProtectOwned),
            _ => None,
        }
    }
}

impl std::fmt::Display for Feature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// FEATURES configuration manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturesConfig {
    /// Enabled features
    enabled: HashSet<Feature>,
    /// Explicitly disabled features (prefixed with -)
    disabled: HashSet<Feature>,
    /// ccache configuration
    pub ccache: CcacheConfig,
    /// distcc configuration
    pub distcc: DistccConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
    /// Binary package configuration
    pub binpkg: BinpkgConfig,
}

impl Default for FeaturesConfig {
    fn default() -> Self {
        let mut enabled = HashSet::new();
        // Default enabled features
        enabled.insert(Feature::Sandbox);
        enabled.insert(Feature::Usersandbox);
        enabled.insert(Feature::Strip);
        enabled.insert(Feature::PreserveLibs);

        Self {
            enabled,
            disabled: HashSet::new(),
            ccache: CcacheConfig::default(),
            distcc: DistccConfig::default(),
            logging: LoggingConfig::default(),
            binpkg: BinpkgConfig::default(),
        }
    }
}

impl FeaturesConfig {
    /// Create a new empty features config
    pub fn new() -> Self {
        Self {
            enabled: HashSet::new(),
            disabled: HashSet::new(),
            ccache: CcacheConfig::default(),
            distcc: DistccConfig::default(),
            logging: LoggingConfig::default(),
            binpkg: BinpkgConfig::default(),
        }
    }

    /// Parse FEATURES string from make.conf
    pub fn parse_features_string(s: &str) -> Result<Self> {
        let mut config = Self::default();

        for token in s.split_whitespace() {
            if let Some(stripped) = token.strip_prefix('-') {
                // Disable feature
                if let Some(feature) = Feature::parse(stripped) {
                    config.disable(feature);
                }
            } else if let Some(feature) = Feature::parse(token) {
                // Enable feature
                config.enable(feature);
            } else {
                tracing::warn!("Unknown FEATURE: {}", token);
            }
        }

        Ok(config)
    }

    /// Check if a feature is enabled
    pub fn is_enabled(&self, feature: Feature) -> bool {
        self.enabled.contains(&feature) && !self.disabled.contains(&feature)
    }

    /// Enable a feature
    pub fn enable(&mut self, feature: Feature) {
        self.disabled.remove(&feature);
        self.enabled.insert(feature);
    }

    /// Disable a feature
    pub fn disable(&mut self, feature: Feature) {
        self.enabled.remove(&feature);
        self.disabled.insert(feature);
    }

    /// Get all enabled features
    pub fn get_enabled(&self) -> Vec<Feature> {
        self.enabled
            .iter()
            .filter(|f| !self.disabled.contains(f))
            .copied()
            .collect()
    }

    /// Get all explicitly disabled features
    pub fn get_disabled(&self) -> Vec<Feature> {
        self.disabled.iter().copied().collect()
    }

    /// Convert to FEATURES string for make.conf
    pub fn to_features_string(&self) -> String {
        let mut parts: Vec<String> = Vec::new();

        for feature in &self.enabled {
            parts.push(feature.name().to_string());
        }

        for feature in &self.disabled {
            parts.push(format!("-{}", feature.name()));
        }

        parts.sort();
        parts.join(" ")
    }
}

/// ccache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CcacheConfig {
    /// ccache directory
    pub dir: PathBuf,
    /// Maximum cache size (e.g., "10G")
    pub max_size: String,
    /// Enable compression
    pub compression: bool,
    /// Compression level (1-19)
    pub compression_level: u8,
    /// Path to ccache binary
    pub binary: PathBuf,
}

impl Default for CcacheConfig {
    fn default() -> Self {
        Self {
            dir: PathBuf::from("/var/cache/ccache"),
            max_size: "10G".to_string(),
            compression: true,
            compression_level: 6,
            binary: PathBuf::from("/usr/bin/ccache"),
        }
    }
}

impl CcacheConfig {
    /// Get environment variables for ccache
    pub fn get_env(&self) -> HashMap<String, String> {
        let mut env = HashMap::new();
        env.insert("CCACHE_DIR".to_string(), self.dir.to_string_lossy().to_string());
        env.insert("CCACHE_MAXSIZE".to_string(), self.max_size.clone());
        if self.compression {
            env.insert("CCACHE_COMPRESS".to_string(), "1".to_string());
            env.insert(
                "CCACHE_COMPRESSLEVEL".to_string(),
                self.compression_level.to_string(),
            );
        }
        env
    }

    /// Check if ccache is available
    pub fn is_available(&self) -> bool {
        self.binary.exists()
    }

    /// Get ccache statistics
    pub fn get_stats(&self) -> Result<CcacheStats> {
        let output = std::process::Command::new(&self.binary)
            .args(["-s"])
            .env("CCACHE_DIR", &self.dir)
            .output()
            .map_err(|e| Error::Other(format!("Failed to run ccache: {}", e)))?;

        if !output.status.success() {
            return Err(Error::Other("ccache stats failed".to_string()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        CcacheStats::parse(&stdout)
    }

    /// Clear ccache
    pub fn clear(&self) -> Result<()> {
        let status = std::process::Command::new(&self.binary)
            .args(["-C"])
            .env("CCACHE_DIR", &self.dir)
            .status()
            .map_err(|e| Error::Other(format!("Failed to run ccache: {}", e)))?;

        if !status.success() {
            return Err(Error::Other("ccache clear failed".to_string()));
        }

        Ok(())
    }
}

/// ccache statistics
#[derive(Debug, Clone, Default)]
pub struct CcacheStats {
    /// Cache hits
    pub hits: u64,
    /// Cache misses
    pub misses: u64,
    /// Cache size in bytes
    pub cache_size: u64,
    /// Number of files in cache
    pub files: u64,
    /// Hit rate as percentage
    pub hit_rate: f64,
}

impl CcacheStats {
    /// Parse ccache -s output
    fn parse(output: &str) -> Result<Self> {
        let mut stats = CcacheStats::default();

        for line in output.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                continue;
            }

            if line.contains("cache hit") && line.contains("direct") {
                if let Ok(n) = parts.last().unwrap_or(&"0").parse::<u64>() {
                    stats.hits += n;
                }
            } else if line.contains("cache miss") {
                if let Ok(n) = parts.last().unwrap_or(&"0").parse::<u64>() {
                    stats.misses = n;
                }
            } else if line.contains("cache size") {
                // Parse size like "1.5 GB"
                if parts.len() >= 2 {
                    let size_str = parts[parts.len() - 2];
                    let unit = parts[parts.len() - 1];
                    if let Ok(size) = size_str.parse::<f64>() {
                        stats.cache_size = match unit {
                            "kB" => (size * 1024.0) as u64,
                            "MB" => (size * 1024.0 * 1024.0) as u64,
                            "GB" => (size * 1024.0 * 1024.0 * 1024.0) as u64,
                            _ => size as u64,
                        };
                    }
                }
            } else if line.contains("files in cache") {
                if let Ok(n) = parts.last().unwrap_or(&"0").parse::<u64>() {
                    stats.files = n;
                }
            }
        }

        // Calculate hit rate
        let total = stats.hits + stats.misses;
        if total > 0 {
            stats.hit_rate = stats.hits as f64 / total as f64 * 100.0;
        }

        Ok(stats)
    }
}

/// distcc configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistccConfig {
    /// List of distcc hosts
    pub hosts: Vec<String>,
    /// Port for distcc daemon
    pub port: u16,
    /// Enable pump mode for preprocessing
    pub pump_mode: bool,
    /// Path to distcc binary
    pub binary: PathBuf,
    /// Log file path
    pub log: PathBuf,
    /// Maximum number of jobs to distribute
    pub max_jobs: Option<usize>,
}

impl Default for DistccConfig {
    fn default() -> Self {
        Self {
            hosts: Vec::new(),
            port: 3632,
            pump_mode: false,
            binary: PathBuf::from("/usr/bin/distcc"),
            log: PathBuf::from("/var/log/distcc.log"),
            max_jobs: None,
        }
    }
}

impl DistccConfig {
    /// Get environment variables for distcc
    pub fn get_env(&self) -> HashMap<String, String> {
        let mut env = HashMap::new();

        if !self.hosts.is_empty() {
            env.insert("DISTCC_HOSTS".to_string(), self.hosts.join(" "));
        }

        env.insert(
            "DISTCC_LOG".to_string(),
            self.log.to_string_lossy().to_string(),
        );

        if let Some(jobs) = self.max_jobs {
            env.insert("DISTCC_JOBS".to_string(), jobs.to_string());
        }

        env
    }

    /// Check if distcc is available
    pub fn is_available(&self) -> bool {
        self.binary.exists()
    }

    /// Get distcc hosts string
    pub fn get_hosts_string(&self) -> String {
        self.hosts.join(" ")
    }

    /// Add a distcc host
    pub fn add_host(&mut self, host: String) {
        if !self.hosts.contains(&host) {
            self.hosts.push(host);
        }
    }

    /// Remove a distcc host
    pub fn remove_host(&mut self, host: &str) {
        self.hosts.retain(|h| h != host);
    }

    /// Calculate optimal job count based on hosts
    pub fn calculate_jobs(&self) -> usize {
        if self.hosts.is_empty() {
            return std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1);
        }

        // Parse hosts and calculate total slots
        let mut total_slots = 0;
        for host in &self.hosts {
            // Format: host/limit,options
            let parts: Vec<&str> = host.split('/').collect();
            if parts.len() > 1 {
                if let Ok(slots) = parts[1].split(',').next().unwrap_or("1").parse::<usize>() {
                    total_slots += slots;
                }
            } else {
                // Default to 4 slots per host
                total_slots += 4;
            }
        }

        total_slots.max(1)
    }
}

/// Logging configuration for builds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log directory
    pub log_dir: PathBuf,
    /// Keep logs for successful builds
    pub keep_success: bool,
    /// Keep logs for failed builds
    pub keep_failure: bool,
    /// Split logs by phase
    pub split_by_phase: bool,
    /// Compress logs
    pub compress: bool,
    /// Maximum log age in days
    pub max_age_days: u32,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            log_dir: PathBuf::from("/var/log/portage"),
            keep_success: false,
            keep_failure: true,
            split_by_phase: false,
            compress: true,
            max_age_days: 30,
        }
    }
}

impl LoggingConfig {
    /// Get log path for a package
    pub fn get_log_path(&self, category: &str, name: &str, version: &str) -> PathBuf {
        self.log_dir.join(format!("{}-{}-{}.log", category, name, version))
    }

    /// Get log path for a specific phase
    pub fn get_phase_log_path(
        &self,
        category: &str,
        name: &str,
        version: &str,
        phase: &str,
    ) -> PathBuf {
        self.log_dir.join(format!(
            "{}-{}-{}-{}.log",
            category, name, version, phase
        ))
    }

    /// Clean old logs
    pub fn clean_old_logs(&self) -> Result<usize> {
        let cutoff = std::time::SystemTime::now()
            - std::time::Duration::from_secs(self.max_age_days as u64 * 24 * 60 * 60);

        let mut removed = 0;

        if !self.log_dir.exists() {
            return Ok(0);
        }

        for entry in std::fs::read_dir(&self.log_dir)? {
            let entry = entry?;
            let metadata = entry.metadata()?;

            if let Ok(modified) = metadata.modified() {
                if modified < cutoff {
                    std::fs::remove_file(entry.path())?;
                    removed += 1;
                }
            }
        }

        Ok(removed)
    }
}

/// Binary package configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinpkgConfig {
    /// Directory for binary packages
    pub pkgdir: PathBuf,
    /// Compression format
    pub compression: BinpkgCompression,
    /// GPG signing key
    pub gpg_key: Option<String>,
    /// Include build logs in packages
    pub include_logs: bool,
}

impl Default for BinpkgConfig {
    fn default() -> Self {
        Self {
            pkgdir: PathBuf::from("/var/cache/binpkgs"),
            compression: BinpkgCompression::Zstd,
            gpg_key: None,
            include_logs: false,
        }
    }
}

/// Binary package compression format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinpkgCompression {
    None,
    Gzip,
    Bzip2,
    Xz,
    Zstd,
}

impl BinpkgCompression {
    pub fn extension(&self) -> &'static str {
        match self {
            BinpkgCompression::None => ".tar",
            BinpkgCompression::Gzip => ".tar.gz",
            BinpkgCompression::Bzip2 => ".tar.bz2",
            BinpkgCompression::Xz => ".tar.xz",
            BinpkgCompression::Zstd => ".tar.zst",
        }
    }
}

/// Build context with features applied
#[derive(Debug, Clone)]
pub struct FeatureContext {
    /// Features configuration
    pub config: FeaturesConfig,
    /// Environment variables to set
    pub env: HashMap<String, String>,
    /// PATH modifications
    pub path_prepend: Vec<PathBuf>,
}

impl FeatureContext {
    /// Create a new feature context from config
    pub fn new(config: FeaturesConfig) -> Self {
        let mut ctx = Self {
            config,
            env: HashMap::new(),
            path_prepend: Vec::new(),
        };
        ctx.apply_features();
        ctx
    }

    /// Apply enabled features to the context
    fn apply_features(&mut self) {
        // Apply ccache
        if self.config.is_enabled(Feature::Ccache) && self.config.ccache.is_available() {
            self.env.extend(self.config.ccache.get_env());
            // Prepend ccache to PATH
            if let Some(parent) = self.config.ccache.binary.parent() {
                self.path_prepend.push(parent.to_path_buf());
            }
        }

        // Apply distcc
        if self.config.is_enabled(Feature::Distcc) && self.config.distcc.is_available() {
            self.env.extend(self.config.distcc.get_env());
            // Prepend distcc to PATH
            if let Some(parent) = self.config.distcc.binary.parent() {
                self.path_prepend.push(parent.to_path_buf());
            }
        }

        // Set test-related env
        if self.config.is_enabled(Feature::Test) {
            self.env.insert("FEATURES_TEST".to_string(), "1".to_string());
        }

        // Set doc-related env
        if self.config.is_enabled(Feature::Doc) {
            self.env.insert("FEATURES_DOC".to_string(), "1".to_string());
        }

        // Set debug-related env
        if self.config.is_enabled(Feature::Debug) {
            self.env.insert("FEATURES_DEBUG".to_string(), "1".to_string());
        }
    }

    /// Get modified PATH with feature binaries prepended
    pub fn get_path(&self) -> String {
        let mut path_parts: Vec<String> = self
            .path_prepend
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();

        if let Ok(current_path) = std::env::var("PATH") {
            path_parts.push(current_path);
        }

        path_parts.join(":")
    }

    /// Get all environment variables for the build
    pub fn get_all_env(&self) -> HashMap<String, String> {
        let mut env = self.env.clone();
        env.insert("PATH".to_string(), self.get_path());
        env
    }
}

/// Features manager for applying features to builds
pub struct FeaturesManager {
    config: FeaturesConfig,
}

impl FeaturesManager {
    /// Create a new features manager
    pub fn new(config: FeaturesConfig) -> Self {
        Self { config }
    }

    /// Get a build context with features applied
    pub fn get_context(&self) -> FeatureContext {
        FeatureContext::new(self.config.clone())
    }

    /// Check if a specific feature is enabled
    pub fn is_enabled(&self, feature: Feature) -> bool {
        self.config.is_enabled(feature)
    }

    /// Enable a feature
    pub fn enable(&mut self, feature: Feature) {
        self.config.enable(feature);
    }

    /// Disable a feature
    pub fn disable(&mut self, feature: Feature) {
        self.config.disable(feature);
    }

    /// Get ccache stats if enabled
    pub fn get_ccache_stats(&self) -> Option<Result<CcacheStats>> {
        if self.is_enabled(Feature::Ccache) {
            Some(self.config.ccache.get_stats())
        } else {
            None
        }
    }

    /// Clean old build logs
    pub fn clean_logs(&self) -> Result<usize> {
        self.config.logging.clean_old_logs()
    }

    /// Get recommended MAKEOPTS based on features
    pub fn get_makeopts(&self) -> String {
        let jobs = if self.is_enabled(Feature::Distcc) {
            self.config.distcc.calculate_jobs()
        } else {
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1)
        };

        format!("-j{}", jobs)
    }

    /// Validate feature configuration
    pub fn validate(&self) -> Result<Vec<String>> {
        let mut warnings = Vec::new();

        // Check ccache
        if self.is_enabled(Feature::Ccache) && !self.config.ccache.is_available() {
            warnings.push(format!(
                "ccache is enabled but not found at {}",
                self.config.ccache.binary.display()
            ));
        }

        // Check distcc
        if self.is_enabled(Feature::Distcc) {
            if !self.config.distcc.is_available() {
                warnings.push(format!(
                    "distcc is enabled but not found at {}",
                    self.config.distcc.binary.display()
                ));
            }
            if self.config.distcc.hosts.is_empty() {
                warnings.push("distcc is enabled but no hosts are configured".to_string());
            }
        }

        // Check for conflicting features
        if self.is_enabled(Feature::KeepWork) && self.is_enabled(Feature::CleanLogs) {
            warnings.push("keepwork and clean-logs are both enabled, logs may be inconsistent".to_string());
        }

        Ok(warnings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_parsing() {
        assert_eq!(Feature::parse("test"), Some(Feature::Test));
        assert_eq!(Feature::parse("ccache"), Some(Feature::Ccache));
        assert_eq!(Feature::parse("parallel-fetch"), Some(Feature::ParallelFetch));
        assert_eq!(Feature::parse("unknown"), None);
    }

    #[test]
    fn test_features_config() {
        let mut config = FeaturesConfig::default();

        assert!(config.is_enabled(Feature::Sandbox));

        config.disable(Feature::Sandbox);
        assert!(!config.is_enabled(Feature::Sandbox));

        config.enable(Feature::Test);
        assert!(config.is_enabled(Feature::Test));
    }

    #[test]
    fn test_parse_features_string() {
        let config = FeaturesConfig::parse_features_string("test ccache -sandbox parallel-fetch").unwrap();

        assert!(config.is_enabled(Feature::Test));
        assert!(config.is_enabled(Feature::Ccache));
        assert!(!config.is_enabled(Feature::Sandbox));
        assert!(config.is_enabled(Feature::ParallelFetch));
    }

    #[test]
    fn test_features_string_roundtrip() {
        let mut config = FeaturesConfig::new();
        config.enable(Feature::Test);
        config.enable(Feature::Ccache);
        config.disable(Feature::Sandbox);

        let s = config.to_features_string();
        let parsed = FeaturesConfig::parse_features_string(&s).unwrap();

        assert_eq!(config.is_enabled(Feature::Test), parsed.is_enabled(Feature::Test));
        assert_eq!(config.is_enabled(Feature::Ccache), parsed.is_enabled(Feature::Ccache));
        assert_eq!(config.is_enabled(Feature::Sandbox), parsed.is_enabled(Feature::Sandbox));
    }

    #[test]
    fn test_distcc_job_calculation() {
        let mut config = DistccConfig::default();
        config.hosts = vec![
            "host1/8".to_string(),
            "host2/4".to_string(),
        ];

        assert_eq!(config.calculate_jobs(), 12);
    }
}
