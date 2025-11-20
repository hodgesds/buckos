//! Global make.conf configuration
//!
//! Implements the main system configuration equivalent to Gentoo's make.conf:
//! - Compiler flags (CFLAGS, CXXFLAGS, LDFLAGS)
//! - USE flags
//! - FEATURES
//! - System paths
//! - Architecture settings
//! - Buck2 build system configuration

use crate::{
    FeaturesConfig, KeywordConfig, LicenseConfig, MirrorConfig, Result, UseConfig,
};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Buck2 execution mode for builds
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum BuckExecutionMode {
    /// Only execute actions locally
    LocalOnly,
    /// Only execute actions remotely
    RemoteOnly,
    /// Prefer local execution, fall back to remote
    PreferLocal,
    /// Prefer remote execution, fall back to local
    PreferRemote,
    /// Auto-select based on action requirements
    Auto,
}

impl Default for BuckExecutionMode {
    fn default() -> Self {
        Self::Auto
    }
}

impl std::fmt::Display for BuckExecutionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuckExecutionMode::LocalOnly => write!(f, "local-only"),
            BuckExecutionMode::RemoteOnly => write!(f, "remote-only"),
            BuckExecutionMode::PreferLocal => write!(f, "prefer-local"),
            BuckExecutionMode::PreferRemote => write!(f, "prefer-remote"),
            BuckExecutionMode::Auto => write!(f, "auto"),
        }
    }
}

/// Buck2 console output mode
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BuckConsoleMode {
    /// Simple console output
    Simple,
    /// Super console with live updates
    Super,
    /// Simple TTY mode
    SimpleTty,
    /// Auto-detect based on terminal
    Auto,
    /// No console output
    None,
}

impl Default for BuckConsoleMode {
    fn default() -> Self {
        Self::Auto
    }
}

impl std::fmt::Display for BuckConsoleMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuckConsoleMode::Simple => write!(f, "simple"),
            BuckConsoleMode::Super => write!(f, "super"),
            BuckConsoleMode::SimpleTty => write!(f, "simpletty"),
            BuckConsoleMode::Auto => write!(f, "auto"),
            BuckConsoleMode::None => write!(f, "none"),
        }
    }
}

/// Buck2 remote execution configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BuckRemoteExecution {
    /// Enable remote execution
    pub enabled: bool,
    /// Remote execution endpoint
    pub endpoint: String,
    /// CAS (Content Addressable Storage) address
    pub cas_address: String,
    /// Action cache address
    pub action_cache_address: String,
    /// Use TLS for connections
    pub use_tls: bool,
    /// Connection timeout in seconds
    pub timeout_secs: u32,
    /// Instance name for RE
    pub instance_name: String,
}

/// Buck2 cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuckCacheConfig {
    /// Enable remote cache
    pub remote_cache_enabled: bool,
    /// Enable local cache
    pub local_cache_enabled: bool,
    /// Write to cache even when reads are disabled
    pub write_to_cache_anyway: bool,
    /// Upload all actions to RE
    pub upload_all_actions: bool,
    /// Local cache directory
    pub local_cache_dir: PathBuf,
    /// Maximum local cache size in GB
    pub local_cache_size_gb: u32,
}

impl Default for BuckCacheConfig {
    fn default() -> Self {
        Self {
            remote_cache_enabled: false,
            local_cache_enabled: true,
            write_to_cache_anyway: false,
            upload_all_actions: false,
            local_cache_dir: PathBuf::from("/var/cache/buckos/buck-cache"),
            local_cache_size_gb: 10,
        }
    }
}

/// Buck2 toolchain configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuckToolchainConfig {
    /// Default Rust edition
    pub rust_edition: String,
    /// Enable Rust incremental compilation
    pub rust_incremental: bool,
    /// Rust toolchain channel (stable, beta, nightly)
    pub rust_channel: String,
    /// Default C++ standard
    pub cxx_standard: String,
    /// C++ compiler
    pub cxx_compiler: String,
    /// C compiler
    pub cc_compiler: String,
    /// Python version
    pub python_version: String,
    /// Go version
    pub go_version: String,
}

impl Default for BuckToolchainConfig {
    fn default() -> Self {
        Self {
            rust_edition: "2021".to_string(),
            rust_incremental: true,
            rust_channel: "stable".to_string(),
            cxx_standard: "c++17".to_string(),
            cxx_compiler: "clang++".to_string(),
            cc_compiler: "clang".to_string(),
            python_version: "3".to_string(),
            go_version: "1.21".to_string(),
        }
    }
}

/// Buck2 cell configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BuckCellConfig {
    /// Cell name to path mappings
    pub cells: IndexMap<String, PathBuf>,
    /// Cell aliases
    pub aliases: IndexMap<String, String>,
    /// Default prelude location
    pub prelude: String,
}

/// Buck2-specific build configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuckConfig {
    // === Execution settings ===
    /// Number of threads for Buck2 execution
    pub threads: u32,
    /// Execution mode (local-only, remote-only, prefer-local, prefer-remote, auto)
    pub execution_mode: BuckExecutionMode,
    /// Console output mode
    pub console_mode: BuckConsoleMode,
    /// Verbosity level (0-10)
    pub verbosity: u8,

    // === Remote execution ===
    /// Remote execution configuration
    pub remote_execution: BuckRemoteExecution,

    // === Cache configuration ===
    /// Cache configuration
    pub cache: BuckCacheConfig,

    // === Toolchain configuration ===
    /// Toolchain configuration
    pub toolchain: BuckToolchainConfig,

    // === Output configuration ===
    /// Buck2 output directory
    pub out_dir: PathBuf,
    /// Isolation directory for builds
    pub isolation_dir: String,

    // === Cell configuration ===
    /// Cell configuration
    pub cell_config: BuckCellConfig,

    // === Platform configuration ===
    /// Target platform
    pub target_platform: String,
    /// Execution platform
    pub execution_platform: String,

    // === Build settings ===
    /// Enable sandboxing
    pub sandbox: bool,
    /// Enable materialization of outputs
    pub materialize_outputs: bool,
    /// Keep going on build failures
    pub keep_going: bool,
    /// Skip missing targets
    pub skip_missing_targets: bool,

    // === File watchers ===
    /// Enable file watcher
    pub file_watcher: bool,
    /// File watcher type (watchman, notify)
    pub file_watcher_type: String,
}

impl Default for BuckConfig {
    fn default() -> Self {
        let parallelism = std::thread::available_parallelism()
            .map(|p| p.get() as u32)
            .unwrap_or(4);

        Self {
            threads: parallelism,
            execution_mode: BuckExecutionMode::Auto,
            console_mode: BuckConsoleMode::Auto,
            verbosity: 1,
            remote_execution: BuckRemoteExecution::default(),
            cache: BuckCacheConfig::default(),
            toolchain: BuckToolchainConfig::default(),
            out_dir: PathBuf::from("buck-out"),
            isolation_dir: String::new(),
            cell_config: BuckCellConfig::default(),
            target_platform: String::new(),
            execution_platform: "root//platforms:default".to_string(),
            sandbox: true,
            materialize_outputs: true,
            keep_going: false,
            skip_missing_targets: false,
            file_watcher: true,
            file_watcher_type: "watchman".to_string(),
        }
    }
}

impl BuckConfig {
    /// Generate Buck2 command-line arguments from configuration
    pub fn to_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        // Parallelism
        args.push(format!("-j{}", self.threads));

        // Execution mode
        match self.execution_mode {
            BuckExecutionMode::LocalOnly => args.push("--local-only".to_string()),
            BuckExecutionMode::RemoteOnly => args.push("--remote-only".to_string()),
            BuckExecutionMode::PreferLocal => args.push("--prefer-local".to_string()),
            BuckExecutionMode::PreferRemote => args.push("--prefer-remote".to_string()),
            BuckExecutionMode::Auto => {}
        }

        // Console mode
        if self.console_mode != BuckConsoleMode::Auto {
            args.push(format!("--console={}", self.console_mode));
        }

        // Cache options
        if !self.cache.remote_cache_enabled {
            args.push("--no-remote-cache".to_string());
        }
        if self.cache.write_to_cache_anyway {
            args.push("--write-to-cache-anyway".to_string());
        }
        if self.cache.upload_all_actions {
            args.push("--upload-all-actions".to_string());
        }

        // Build options
        if self.keep_going {
            args.push("--keep-going".to_string());
        }
        if self.skip_missing_targets {
            args.push("--skip-missing-targets".to_string());
        }

        // Target platform
        if !self.target_platform.is_empty() {
            args.push(format!("--target-platforms={}", self.target_platform));
        }

        args
    }

    /// Generate .buckconfig content from this configuration
    pub fn to_buckconfig(&self) -> String {
        let mut config = String::new();

        // Build section
        config.push_str("[build]\n");
        config.push_str(&format!("execution_platforms = {}\n", self.execution_platform));
        if !self.isolation_dir.is_empty() {
            config.push_str(&format!("isolation_dir = {}\n", self.isolation_dir));
        }
        config.push('\n');

        // Rust section
        config.push_str("[rust]\n");
        config.push_str(&format!("default_edition = {}\n", self.toolchain.rust_edition));
        if self.toolchain.rust_incremental {
            config.push_str("incremental = true\n");
        }
        config.push('\n');

        // Cxx section
        config.push_str("[cxx]\n");
        config.push_str(&format!("cxx_compiler = {}\n", self.toolchain.cxx_compiler));
        config.push_str(&format!("c_compiler = {}\n", self.toolchain.cc_compiler));
        config.push('\n');

        // Python section
        config.push_str("[python]\n");
        config.push_str(&format!("version = {}\n", self.toolchain.python_version));
        config.push('\n');

        // Cells section if configured
        if !self.cell_config.cells.is_empty() {
            config.push_str("[cells]\n");
            for (name, path) in &self.cell_config.cells {
                config.push_str(&format!("{} = {}\n", name, path.display()));
            }
            config.push('\n');
        }

        // Cell aliases
        if !self.cell_config.aliases.is_empty() {
            config.push_str("[cell_aliases]\n");
            for (alias, target) in &self.cell_config.aliases {
                config.push_str(&format!("{} = {}\n", alias, target));
            }
            config.push('\n');
        }

        config
    }

    /// Get environment variables for Buck2 execution
    pub fn build_env(&self) -> IndexMap<String, String> {
        let mut env = IndexMap::new();

        env.insert("BUCK2_THREADS".to_string(), self.threads.to_string());
        env.insert("BUCK2_EXECUTION_MODE".to_string(), self.execution_mode.to_string());
        env.insert("BUCK2_CONSOLE".to_string(), self.console_mode.to_string());
        env.insert("BUCK2_VERBOSITY".to_string(), self.verbosity.to_string());

        // Toolchain settings
        env.insert("BUCK2_RUST_EDITION".to_string(), self.toolchain.rust_edition.clone());
        env.insert("BUCK2_CXX_STANDARD".to_string(), self.toolchain.cxx_standard.clone());

        // Cache settings
        if !self.cache.remote_cache_enabled {
            env.insert("BUCK2_NO_REMOTE_CACHE".to_string(), "1".to_string());
        }

        env
    }

    /// Create a configuration optimized for CI/CD environments
    pub fn ci() -> Self {
        let mut config = Self::default();
        config.console_mode = BuckConsoleMode::Simple;
        config.cache.remote_cache_enabled = true;
        config.cache.upload_all_actions = true;
        config.keep_going = true;
        config.file_watcher = false;
        config
    }

    /// Create a configuration optimized for development
    pub fn development() -> Self {
        let mut config = Self::default();
        config.console_mode = BuckConsoleMode::Super;
        config.toolchain.rust_incremental = true;
        config.file_watcher = true;
        config
    }

    /// Create a configuration for remote execution
    pub fn remote(endpoint: &str) -> Self {
        let mut config = Self::default();
        config.execution_mode = BuckExecutionMode::PreferRemote;
        config.remote_execution.enabled = true;
        config.remote_execution.endpoint = endpoint.to_string();
        config.cache.remote_cache_enabled = true;
        config
    }
}

/// Main make.conf configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MakeConf {
    // === Compiler settings ===
    /// C compiler flags
    pub cflags: String,
    /// C++ compiler flags
    pub cxxflags: String,
    /// Fortran compiler flags
    pub fflags: String,
    /// Fortran 90+ compiler flags
    pub fcflags: String,
    /// Linker flags
    pub ldflags: String,
    /// Rust compiler flags
    pub rustflags: String,
    /// Go compiler flags
    pub goflags: String,

    // === Build settings ===
    /// Make options (e.g., -j8)
    pub makeopts: String,
    /// Ninja options
    pub ninjaopts: String,
    /// Emerge default options
    pub emerge_default_opts: String,

    // === Architecture ===
    /// Target architecture
    pub arch: String,
    /// CHOST for cross-compilation
    pub chost: String,
    /// CPU flags (e.g., CPU_FLAGS_X86)
    pub cpu_flags: IndexMap<String, HashSet<String>>,

    // === USE flags ===
    /// USE flag configuration
    pub use_config: UseConfig,

    // === Features ===
    /// FEATURES configuration
    pub features: FeaturesConfig,

    // === Keywords ===
    /// ACCEPT_KEYWORDS
    pub keywords: KeywordConfig,

    // === Licenses ===
    /// ACCEPT_LICENSE
    pub license: LicenseConfig,

    // === Mirrors ===
    /// Mirror configuration
    pub mirrors: MirrorConfig,

    // === Paths ===
    /// Distribution files directory
    pub distdir: PathBuf,
    /// Binary packages directory
    pub pkgdir: PathBuf,
    /// Log directory
    pub logdir: PathBuf,
    /// Temporary build directory
    pub tmpdir: PathBuf,
    /// Repository directory
    pub repodir: PathBuf,

    // === Binary packages ===
    /// Binary package compression
    pub binpkg_compress: String,
    /// Binary package format
    pub binpkg_format: String,

    // === Config protection ===
    /// Paths to protect
    pub config_protect: Vec<String>,
    /// Paths to not protect
    pub config_protect_mask: Vec<String>,

    // === Portage settings ===
    /// Clean delay in seconds
    pub clean_delay: u32,
    /// Warning delay in seconds
    pub emerge_warning_delay: u32,
    /// Collision ignore patterns
    pub collision_ignore: Vec<String>,
    /// Uninstall ignore patterns
    pub uninstall_ignore: Vec<String>,

    // === Input devices ===
    /// Input devices to support
    pub input_devices: HashSet<String>,
    /// Video cards to support
    pub video_cards: HashSet<String>,

    // === Language settings ===
    /// Localization support
    pub l10n: HashSet<String>,

    // === Miscellaneous ===
    /// Custom variables
    pub custom: IndexMap<String, String>,

    // === Buck2 configuration ===
    /// Buck2-specific build configuration
    pub buck: BuckConfig,
}

impl Default for MakeConf {
    fn default() -> Self {
        let parallelism = std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(4);

        Self {
            // Compiler settings
            cflags: "-O2 -pipe".to_string(),
            cxxflags: "${CFLAGS}".to_string(),
            fflags: "${CFLAGS}".to_string(),
            fcflags: "${CFLAGS}".to_string(),
            ldflags: "-Wl,-O1 -Wl,--as-needed".to_string(),
            rustflags: "-C opt-level=2".to_string(),
            goflags: String::new(),

            // Build settings
            makeopts: format!("-j{}", parallelism),
            ninjaopts: format!("-j{}", parallelism),
            emerge_default_opts: "--ask --verbose --tree".to_string(),

            // Architecture
            arch: detect_arch(),
            chost: detect_chost(),
            cpu_flags: IndexMap::new(),

            // USE flags
            use_config: UseConfig::default(),

            // Features
            features: FeaturesConfig::with_defaults(),

            // Keywords
            keywords: KeywordConfig::new(detect_arch()),

            // Licenses
            license: LicenseConfig::default(),

            // Mirrors
            mirrors: MirrorConfig::default(),

            // Paths
            distdir: PathBuf::from("/var/cache/distfiles"),
            pkgdir: PathBuf::from("/var/cache/binpkgs"),
            logdir: PathBuf::from("/var/log/portage"),
            tmpdir: PathBuf::from("/var/tmp/portage"),
            repodir: PathBuf::from("/var/db/repos"),

            // Binary packages
            binpkg_compress: "zstd".to_string(),
            binpkg_format: "gpkg".to_string(),

            // Config protection
            config_protect: vec!["/etc".to_string()],
            config_protect_mask: vec!["/etc/env.d".to_string()],

            // Portage settings
            clean_delay: 5,
            emerge_warning_delay: 10,
            collision_ignore: vec!["/lib/modules/*".to_string()],
            uninstall_ignore: vec!["/lib/modules/*".to_string()],

            // Input/Video
            input_devices: ["libinput"].iter().map(|s| s.to_string()).collect(),
            video_cards: HashSet::new(),

            // Language
            l10n: ["en".to_string()].into_iter().collect(),

            // Custom
            custom: IndexMap::new(),

            // Buck2
            buck: BuckConfig::default(),
        }
    }
}

impl MakeConf {
    /// Create a new make.conf with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Load from a TOML file
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save to a TOML file
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Get the USE flags as a string
    pub fn use_string(&self) -> String {
        self.use_config
            .global
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Set USE flags from a string
    pub fn set_use(&mut self, use_str: &str) {
        self.use_config.global.clear();
        for flag in UseConfig::parse_use_string(use_str) {
            if flag.enabled {
                self.use_config.global.insert(flag.name);
            } else {
                self.use_config.global.remove(&flag.name);
            }
        }
    }

    /// Get the FEATURES as a string
    pub fn features_string(&self) -> String {
        self.features.format()
    }

    /// Set FEATURES from a string
    pub fn set_features(&mut self, features_str: &str) {
        self.features = FeaturesConfig::parse(features_str);
    }

    /// Get ACCEPT_KEYWORDS as a string
    pub fn accept_keywords_string(&self) -> String {
        self.keywords
            .accept_keywords
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Set ACCEPT_KEYWORDS from a string
    pub fn set_accept_keywords(&mut self, keywords_str: &str) {
        self.keywords.accept_keywords.clear();
        for kw in keywords_str.split_whitespace() {
            self.keywords.add_keyword(kw);
        }
    }

    /// Get GENTOO_MIRRORS as a string
    pub fn mirrors_string(&self) -> String {
        self.mirrors.to_mirrors_string()
    }

    /// Set mirrors from a string
    pub fn set_mirrors(&mut self, mirrors_str: &str) {
        self.mirrors = MirrorConfig::from_mirrors_string(mirrors_str);
    }

    /// Set CPU flags for an architecture
    pub fn set_cpu_flags(&mut self, arch: &str, flags: Vec<String>) {
        let key = format!("CPU_FLAGS_{}", arch.to_uppercase());
        self.cpu_flags.insert(key.clone(), flags.into_iter().collect());
        // Also update USE expand
        self.use_config.set_cpu_flags(arch,
            self.cpu_flags.get(&key).unwrap().iter().cloned().collect()
        );
    }

    /// Expand variable references in a value
    pub fn expand(&self, value: &str) -> String {
        let mut result = value.to_string();

        // Common variable expansions
        result = result.replace("${CFLAGS}", &self.cflags);
        result = result.replace("${CXXFLAGS}", &self.cxxflags);
        result = result.replace("${LDFLAGS}", &self.ldflags);
        result = result.replace("${MAKEOPTS}", &self.makeopts);
        result = result.replace("${DISTDIR}", &self.distdir.to_string_lossy());
        result = result.replace("${PKGDIR}", &self.pkgdir.to_string_lossy());

        // Expand custom variables
        for (key, val) in &self.custom {
            result = result.replace(&format!("${{{}}}", key), val);
        }

        result
    }

    /// Set a custom variable
    pub fn set_custom(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.custom.insert(key.into(), value.into());
    }

    /// Get a custom variable
    pub fn get_custom(&self, key: &str) -> Option<&String> {
        self.custom.get(key)
    }

    /// Get all environment variables for building
    pub fn build_env(&self) -> IndexMap<String, String> {
        let mut env = IndexMap::new();

        env.insert("CFLAGS".to_string(), self.cflags.clone());
        env.insert("CXXFLAGS".to_string(), self.expand(&self.cxxflags));
        env.insert("FFLAGS".to_string(), self.expand(&self.fflags));
        env.insert("FCFLAGS".to_string(), self.expand(&self.fcflags));
        env.insert("LDFLAGS".to_string(), self.ldflags.clone());
        env.insert("RUSTFLAGS".to_string(), self.rustflags.clone());
        env.insert("MAKEOPTS".to_string(), self.makeopts.clone());
        env.insert("NINJAOPTS".to_string(), self.ninjaopts.clone());
        env.insert("USE".to_string(), self.use_string());
        env.insert("FEATURES".to_string(), self.features_string());
        env.insert("ACCEPT_KEYWORDS".to_string(), self.accept_keywords_string());
        env.insert("ACCEPT_LICENSE".to_string(), self.license.accept_license.clone());
        env.insert("CHOST".to_string(), self.chost.clone());
        env.insert("DISTDIR".to_string(), self.distdir.to_string_lossy().to_string());
        env.insert("PKGDIR".to_string(), self.pkgdir.to_string_lossy().to_string());

        // Add CPU flags
        for (key, values) in &self.cpu_flags {
            env.insert(key.clone(), values.iter().cloned().collect::<Vec<_>>().join(" "));
        }

        // Add custom variables
        for (key, value) in &self.custom {
            env.insert(key.clone(), value.clone());
        }

        // Add Buck2 environment variables
        for (key, value) in self.buck.build_env() {
            env.insert(key, value);
        }

        env
    }

    /// Get Buck2 command-line arguments
    pub fn buck_args(&self) -> Vec<String> {
        self.buck.to_args()
    }

    /// Generate .buckconfig content from configuration
    pub fn to_buckconfig(&self) -> String {
        self.buck.to_buckconfig()
    }

    /// Set Buck2 execution mode
    pub fn set_buck_execution_mode(&mut self, mode: BuckExecutionMode) {
        self.buck.execution_mode = mode;
    }

    /// Enable Buck2 remote execution
    pub fn enable_buck_remote(&mut self, endpoint: &str) {
        self.buck.remote_execution.enabled = true;
        self.buck.remote_execution.endpoint = endpoint.to_string();
        self.buck.cache.remote_cache_enabled = true;
    }

    /// Set Buck2 toolchain settings
    pub fn set_buck_rust_edition(&mut self, edition: &str) {
        self.buck.toolchain.rust_edition = edition.to_string();
    }

    /// Configure Buck2 for CI/CD
    pub fn configure_buck_for_ci(&mut self) {
        self.buck = BuckConfig::ci();
    }

    /// Create a desktop-oriented configuration
    pub fn desktop() -> Self {
        let mut config = Self::default();

        // Common desktop USE flags
        let desktop_use = [
            "X", "wayland", "pulseaudio", "pipewire", "dbus", "policykit",
            "udisks", "networkmanager", "bluetooth", "cups", "gtk", "qt5",
        ];
        for flag in desktop_use {
            config.use_config.add_global(flag);
        }

        config
    }

    /// Create a server-oriented configuration
    pub fn server() -> Self {
        let mut config = Self::default();

        // Minimal server USE flags
        let server_use = ["ssl", "threads", "ipv6", "zstd"];
        for flag in server_use {
            config.use_config.add_global(flag);
        }

        // Disable desktop stuff
        config.use_config.global.remove("X");
        config.use_config.global.remove("wayland");

        config
    }
}

/// Detect the current architecture
fn detect_arch() -> String {
    #[cfg(target_arch = "x86_64")]
    return "amd64".to_string();

    #[cfg(target_arch = "aarch64")]
    return "arm64".to_string();

    #[cfg(target_arch = "x86")]
    return "x86".to_string();

    #[cfg(target_arch = "arm")]
    return "arm".to_string();

    #[cfg(target_arch = "riscv64")]
    return "riscv".to_string();

    #[cfg(not(any(
        target_arch = "x86_64",
        target_arch = "aarch64",
        target_arch = "x86",
        target_arch = "arm",
        target_arch = "riscv64"
    )))]
    return "unknown".to_string();
}

/// Detect the CHOST
fn detect_chost() -> String {
    #[cfg(target_arch = "x86_64")]
    return "x86_64-pc-linux-gnu".to_string();

    #[cfg(target_arch = "aarch64")]
    return "aarch64-unknown-linux-gnu".to_string();

    #[cfg(target_arch = "x86")]
    return "i686-pc-linux-gnu".to_string();

    #[cfg(target_arch = "arm")]
    return "armv7a-unknown-linux-gnueabihf".to_string();

    #[cfg(target_arch = "riscv64")]
    return "riscv64-unknown-linux-gnu".to_string();

    #[cfg(not(any(
        target_arch = "x86_64",
        target_arch = "aarch64",
        target_arch = "x86",
        target_arch = "arm",
        target_arch = "riscv64"
    )))]
    return "unknown-unknown-linux-gnu".to_string();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_make_conf() {
        let conf = MakeConf::default();
        assert!(!conf.cflags.is_empty());
        assert!(!conf.makeopts.is_empty());
    }

    #[test]
    fn test_variable_expansion() {
        let conf = MakeConf::default();
        let expanded = conf.expand("${CFLAGS}");
        assert_eq!(expanded, conf.cflags);
    }

    #[test]
    fn test_build_env() {
        let conf = MakeConf::default();
        let env = conf.build_env();

        assert!(env.contains_key("CFLAGS"));
        assert!(env.contains_key("MAKEOPTS"));
        assert!(env.contains_key("USE"));
    }

    #[test]
    fn test_desktop_config() {
        let conf = MakeConf::desktop();
        assert!(conf.use_config.global.contains("X"));
        assert!(conf.use_config.global.contains("wayland"));
    }

    #[test]
    fn test_server_config() {
        let conf = MakeConf::server();
        assert!(!conf.use_config.global.contains("X"));
        assert!(conf.use_config.global.contains("ssl"));
    }

    #[test]
    fn test_set_cpu_flags() {
        let mut conf = MakeConf::default();
        conf.set_cpu_flags("x86", vec!["avx2".to_string(), "sse4_2".to_string()]);

        let env = conf.build_env();
        let cpu_flags = env.get("CPU_FLAGS_X86").unwrap();
        assert!(cpu_flags.contains("avx2"));
        assert!(cpu_flags.contains("sse4_2"));
    }

    #[test]
    fn test_buck_config_default() {
        let conf = MakeConf::default();
        assert!(conf.buck.threads > 0);
        assert_eq!(conf.buck.execution_mode, BuckExecutionMode::Auto);
        assert_eq!(conf.buck.console_mode, BuckConsoleMode::Auto);
        assert_eq!(conf.buck.toolchain.rust_edition, "2021");
    }

    #[test]
    fn test_buck_config_to_args() {
        let mut buck = BuckConfig::default();
        buck.threads = 8;
        buck.execution_mode = BuckExecutionMode::LocalOnly;
        buck.keep_going = true;

        let args = buck.to_args();
        assert!(args.contains(&"-j8".to_string()));
        assert!(args.contains(&"--local-only".to_string()));
        assert!(args.contains(&"--keep-going".to_string()));
    }

    #[test]
    fn test_buck_config_ci() {
        let buck = BuckConfig::ci();
        assert_eq!(buck.console_mode, BuckConsoleMode::Simple);
        assert!(buck.cache.remote_cache_enabled);
        assert!(buck.cache.upload_all_actions);
        assert!(buck.keep_going);
        assert!(!buck.file_watcher);
    }

    #[test]
    fn test_buck_config_development() {
        let buck = BuckConfig::development();
        assert_eq!(buck.console_mode, BuckConsoleMode::Super);
        assert!(buck.toolchain.rust_incremental);
        assert!(buck.file_watcher);
    }

    #[test]
    fn test_buck_config_remote() {
        let buck = BuckConfig::remote("https://re.example.com");
        assert_eq!(buck.execution_mode, BuckExecutionMode::PreferRemote);
        assert!(buck.remote_execution.enabled);
        assert_eq!(buck.remote_execution.endpoint, "https://re.example.com");
        assert!(buck.cache.remote_cache_enabled);
    }

    #[test]
    fn test_buck_config_to_buckconfig() {
        let buck = BuckConfig::default();
        let config = buck.to_buckconfig();

        assert!(config.contains("[build]"));
        assert!(config.contains("[rust]"));
        assert!(config.contains("default_edition = 2021"));
        assert!(config.contains("[cxx]"));
        assert!(config.contains("[python]"));
    }

    #[test]
    fn test_buck_build_env() {
        let conf = MakeConf::default();
        let env = conf.build_env();

        assert!(env.contains_key("BUCK2_THREADS"));
        assert!(env.contains_key("BUCK2_EXECUTION_MODE"));
        assert!(env.contains_key("BUCK2_RUST_EDITION"));
    }

    #[test]
    fn test_make_conf_buck_methods() {
        let mut conf = MakeConf::default();

        conf.set_buck_execution_mode(BuckExecutionMode::PreferLocal);
        assert_eq!(conf.buck.execution_mode, BuckExecutionMode::PreferLocal);

        conf.set_buck_rust_edition("2024");
        assert_eq!(conf.buck.toolchain.rust_edition, "2024");

        conf.enable_buck_remote("https://re.example.com");
        assert!(conf.buck.remote_execution.enabled);
        assert!(conf.buck.cache.remote_cache_enabled);
    }

    #[test]
    fn test_buck_execution_mode_display() {
        assert_eq!(BuckExecutionMode::LocalOnly.to_string(), "local-only");
        assert_eq!(BuckExecutionMode::RemoteOnly.to_string(), "remote-only");
        assert_eq!(BuckExecutionMode::PreferLocal.to_string(), "prefer-local");
        assert_eq!(BuckExecutionMode::PreferRemote.to_string(), "prefer-remote");
        assert_eq!(BuckExecutionMode::Auto.to_string(), "auto");
    }

    #[test]
    fn test_buck_console_mode_display() {
        assert_eq!(BuckConsoleMode::Simple.to_string(), "simple");
        assert_eq!(BuckConsoleMode::Super.to_string(), "super");
        assert_eq!(BuckConsoleMode::SimpleTty.to_string(), "simpletty");
        assert_eq!(BuckConsoleMode::Auto.to_string(), "auto");
        assert_eq!(BuckConsoleMode::None.to_string(), "none");
    }
}
