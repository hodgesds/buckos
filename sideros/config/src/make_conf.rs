//! Global make.conf configuration
//!
//! Implements the main system configuration equivalent to Gentoo's make.conf:
//! - Compiler flags (CFLAGS, CXXFLAGS, LDFLAGS)
//! - USE flags
//! - FEATURES
//! - System paths
//! - Architecture settings

use crate::{
    FeaturesConfig, KeywordConfig, LicenseConfig, MirrorConfig, Result, UseConfig,
};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

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

        env
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
}
