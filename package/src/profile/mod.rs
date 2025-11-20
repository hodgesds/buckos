//! Profile system for Buckos
//!
//! Implements Gentoo-style cascading profiles for system defaults including:
//! - USE flags
//! - Package masks/unmasks
//! - Architecture-specific settings
//! - Toolchain selection (gcc, llvm/clang)
//! - Libc selection (glibc, musl)

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// A system profile defining defaults and package configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    /// Profile name (e.g., "default/linux/amd64/desktop")
    pub name: String,
    /// Parent profile names for cascading inheritance
    pub parents: Vec<String>,
    /// Profile description
    pub description: String,
    /// Architecture (e.g., "amd64", "arm64")
    pub arch: Option<String>,
    /// Stable keywords for this profile
    pub keywords: Vec<String>,
    /// USE flags enabled by this profile
    pub use_flags: HashSet<String>,
    /// USE flags disabled by this profile
    pub use_mask: HashSet<String>,
    /// USE flags force-enabled by this profile
    pub use_force: HashSet<String>,
    /// Packages masked by this profile
    pub package_mask: Vec<String>,
    /// Packages unmasked by this profile
    pub package_unmask: Vec<String>,
    /// Make.defaults variables (CFLAGS, CXXFLAGS, etc.)
    pub make_defaults: HashMap<String, String>,
    /// Package-specific USE flags
    pub package_use: HashMap<String, Vec<String>>,
    /// Toolchain type (gcc or llvm)
    pub toolchain: Toolchain,
    /// Libc type (glibc or musl)
    pub libc: Libc,
    /// Whether this is a deprecated profile
    pub deprecated: bool,
    /// Deprecation message if deprecated
    pub deprecation_message: Option<String>,
    /// Profile stability (stable, testing, experimental)
    pub stability: ProfileStability,
}

/// Toolchain selection for the profile
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Toolchain {
    /// GNU Compiler Collection
    Gcc,
    /// LLVM/Clang toolchain
    Llvm,
}

impl Default for Toolchain {
    fn default() -> Self {
        Toolchain::Gcc
    }
}

impl std::fmt::Display for Toolchain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Toolchain::Gcc => write!(f, "gcc"),
            Toolchain::Llvm => write!(f, "llvm"),
        }
    }
}

/// Libc selection for the profile
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Libc {
    /// GNU C Library
    Glibc,
    /// musl libc
    Musl,
}

impl Default for Libc {
    fn default() -> Self {
        Libc::Glibc
    }
}

impl std::fmt::Display for Libc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Libc::Glibc => write!(f, "glibc"),
            Libc::Musl => write!(f, "musl"),
        }
    }
}

/// Profile stability level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProfileStability {
    /// Production-ready profile
    Stable,
    /// Testing profile with newer packages
    Testing,
    /// Experimental profile
    Experimental,
    /// Development profile
    Dev,
}

impl Default for ProfileStability {
    fn default() -> Self {
        ProfileStability::Stable
    }
}

impl std::fmt::Display for ProfileStability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProfileStability::Stable => write!(f, "stable"),
            ProfileStability::Testing => write!(f, "testing"),
            ProfileStability::Experimental => write!(f, "exp"),
            ProfileStability::Dev => write!(f, "dev"),
        }
    }
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            name: String::new(),
            parents: Vec::new(),
            description: String::new(),
            arch: None,
            keywords: Vec::new(),
            use_flags: HashSet::new(),
            use_mask: HashSet::new(),
            use_force: HashSet::new(),
            package_mask: Vec::new(),
            package_unmask: Vec::new(),
            make_defaults: HashMap::new(),
            package_use: HashMap::new(),
            toolchain: Toolchain::default(),
            libc: Libc::default(),
            deprecated: false,
            deprecation_message: None,
            stability: ProfileStability::default(),
        }
    }
}

impl Profile {
    /// Create a new profile with the given name
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }

    /// Check if a USE flag is enabled in this profile
    pub fn has_use_flag(&self, flag: &str) -> bool {
        self.use_flags.contains(flag) || self.use_force.contains(flag)
    }

    /// Check if a USE flag is masked
    pub fn is_use_masked(&self, flag: &str) -> bool {
        self.use_mask.contains(flag)
    }

    /// Check if a package is masked
    pub fn is_package_masked(&self, package: &str) -> bool {
        self.package_mask
            .iter()
            .any(|p| package_matches(package, p))
    }

    /// Check if a package is unmasked
    pub fn is_package_unmasked(&self, package: &str) -> bool {
        self.package_unmask
            .iter()
            .any(|p| package_matches(package, p))
    }

    /// Get a make.defaults variable
    pub fn get_var(&self, name: &str) -> Option<&str> {
        self.make_defaults.get(name).map(|s| s.as_str())
    }
}

/// Resolved profile with all inheritance applied
#[derive(Debug, Clone)]
pub struct ResolvedProfile {
    /// The base profile name
    pub name: String,
    /// Full inheritance chain (bottom-up)
    pub chain: Vec<String>,
    /// Merged USE flags (enabled)
    pub use_flags: HashSet<String>,
    /// Merged USE mask
    pub use_mask: HashSet<String>,
    /// Merged USE force
    pub use_force: HashSet<String>,
    /// Merged package masks
    pub package_mask: Vec<String>,
    /// Merged package unmasks
    pub package_unmask: Vec<String>,
    /// Merged make.defaults
    pub make_defaults: HashMap<String, String>,
    /// Merged package USE
    pub package_use: HashMap<String, Vec<String>>,
    /// Final architecture
    pub arch: String,
    /// Final toolchain
    pub toolchain: Toolchain,
    /// Final libc
    pub libc: Libc,
    /// Final keywords
    pub keywords: Vec<String>,
}

impl ResolvedProfile {
    /// Check if a USE flag is enabled
    pub fn is_use_enabled(&self, flag: &str) -> bool {
        (self.use_flags.contains(flag) || self.use_force.contains(flag))
            && !self.use_mask.contains(flag)
    }

    /// Get effective USE flags for a package
    pub fn get_package_use(&self, package: &str) -> HashSet<String> {
        let mut flags = self.use_flags.clone();

        // Apply package-specific USE flags
        for (pattern, pkg_flags) in &self.package_use {
            if package_matches(package, pattern) {
                for flag in pkg_flags {
                    if flag.starts_with('-') {
                        flags.remove(&flag[1..]);
                    } else {
                        flags.insert(flag.clone());
                    }
                }
            }
        }

        // Remove masked flags
        for flag in &self.use_mask {
            flags.remove(flag);
        }

        // Add forced flags
        for flag in &self.use_force {
            flags.insert(flag.clone());
        }

        flags
    }

    /// Check if a package is masked
    pub fn is_masked(&self, package: &str) -> bool {
        // Check if explicitly unmasked first
        for pattern in &self.package_unmask {
            if package_matches(package, pattern) {
                return false;
            }
        }

        // Check if masked
        for pattern in &self.package_mask {
            if package_matches(package, pattern) {
                return true;
            }
        }

        false
    }

    /// Get a configuration variable
    pub fn get_var(&self, name: &str) -> Option<&str> {
        self.make_defaults.get(name).map(|s| s.as_str())
    }

    /// Get CFLAGS
    pub fn cflags(&self) -> &str {
        self.make_defaults
            .get("CFLAGS")
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    /// Get CXXFLAGS
    pub fn cxxflags(&self) -> &str {
        self.make_defaults
            .get("CXXFLAGS")
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    /// Get LDFLAGS
    pub fn ldflags(&self) -> &str {
        self.make_defaults
            .get("LDFLAGS")
            .map(|s| s.as_str())
            .unwrap_or("")
    }
}

/// Profile manager for loading and managing system profiles
pub struct ProfileManager {
    /// Available profiles
    profiles: HashMap<String, Profile>,
    /// Profiles directory
    profiles_dir: PathBuf,
    /// Current profile path
    current_profile_path: PathBuf,
    /// Current resolved profile
    current: Option<ResolvedProfile>,
}

impl ProfileManager {
    /// Create a new profile manager
    pub fn new(profiles_dir: PathBuf, current_profile_path: PathBuf) -> Self {
        Self {
            profiles: HashMap::new(),
            profiles_dir,
            current_profile_path,
            current: None,
        }
    }

    /// Load all profiles from disk
    pub fn load(&mut self) -> Result<()> {
        info!("Loading profiles from {:?}", self.profiles_dir);

        // Load built-in profiles first
        self.load_builtin_profiles();

        // Load profiles from disk
        if self.profiles_dir.exists() {
            self.load_profiles_from_dir(&self.profiles_dir.clone())?;
        }

        // Load current profile selection
        self.load_current_profile()?;

        info!("Loaded {} profiles", self.profiles.len());
        Ok(())
    }

    /// Load profiles from a directory recursively
    fn load_profiles_from_dir(&mut self, dir: &Path) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Check for profile.toml or parent file
                let profile_file = path.join("profile.toml");
                if profile_file.exists() {
                    if let Ok(profile) = self.load_profile_file(&profile_file) {
                        self.profiles.insert(profile.name.clone(), profile);
                    }
                }

                // Recurse into subdirectories
                self.load_profiles_from_dir(&path)?;
            }
        }

        Ok(())
    }

    /// Load a profile from a TOML file
    fn load_profile_file(&self, path: &Path) -> Result<Profile> {
        let content = std::fs::read_to_string(path)?;
        let profile: Profile = toml::from_str(&content)?;
        Ok(profile)
    }

    /// Load the current profile selection
    fn load_current_profile(&mut self) -> Result<()> {
        if !self.current_profile_path.exists() {
            // Default to base desktop profile
            self.set_current("default/linux/amd64/desktop")?;
            return Ok(());
        }

        let profile_name = std::fs::read_to_string(&self.current_profile_path)?
            .trim()
            .to_string();

        if !profile_name.is_empty() {
            self.resolve_profile(&profile_name)?;
        }

        Ok(())
    }

    /// Load built-in default profiles
    fn load_builtin_profiles(&mut self) {
        // Base profile
        let mut base = Profile::new("base");
        base.description = "Base system profile".to_string();
        base.make_defaults
            .insert("CFLAGS".to_string(), "-O2 -pipe".to_string());
        base.make_defaults
            .insert("CXXFLAGS".to_string(), "${CFLAGS}".to_string());
        base.make_defaults
            .insert("LDFLAGS".to_string(), "-Wl,-O1 -Wl,--as-needed".to_string());
        base.make_defaults
            .insert("MAKEOPTS".to_string(), "-j$(nproc)".to_string());
        self.profiles.insert(base.name.clone(), base);

        // Linux base profile
        let mut linux = Profile::new("default/linux");
        linux.description = "Linux base profile".to_string();
        linux.parents = vec!["base".to_string()];
        linux.use_flags.insert("unicode".to_string());
        linux.use_flags.insert("nls".to_string());
        linux.use_flags.insert("threads".to_string());
        self.profiles.insert(linux.name.clone(), linux);

        // AMD64 architecture profile
        let mut amd64 = Profile::new("default/linux/amd64");
        amd64.description = "AMD64/x86_64 architecture".to_string();
        amd64.parents = vec!["default/linux".to_string()];
        amd64.arch = Some("amd64".to_string());
        amd64.keywords = vec!["amd64".to_string()];
        amd64.make_defaults.insert(
            "CFLAGS".to_string(),
            "-O2 -pipe -march=x86-64 -mtune=generic".to_string(),
        );
        amd64.use_flags.insert("abi_x86_64".to_string());
        self.profiles.insert(amd64.name.clone(), amd64);

        // ARM64 architecture profile
        let mut arm64 = Profile::new("default/linux/arm64");
        arm64.description = "ARM64/AArch64 architecture".to_string();
        arm64.parents = vec!["default/linux".to_string()];
        arm64.arch = Some("arm64".to_string());
        arm64.keywords = vec!["arm64".to_string()];
        arm64
            .make_defaults
            .insert("CFLAGS".to_string(), "-O2 -pipe -march=armv8-a".to_string());
        self.profiles.insert(arm64.name.clone(), arm64);

        // Desktop profile
        let mut desktop = Profile::new("default/linux/amd64/desktop");
        desktop.description = "Desktop system with GUI support".to_string();
        desktop.parents = vec!["default/linux/amd64".to_string()];
        desktop.use_flags.extend(
            [
                "X".to_string(),
                "wayland".to_string(),
                "pulseaudio".to_string(),
                "alsa".to_string(),
                "cups".to_string(),
                "dbus".to_string(),
                "gtk".to_string(),
                "qt5".to_string(),
                "bluetooth".to_string(),
                "networkmanager".to_string(),
                "opengl".to_string(),
                "vulkan".to_string(),
            ]
            .iter()
            .cloned(),
        );
        desktop.stability = ProfileStability::Stable;
        self.profiles.insert(desktop.name.clone(), desktop);

        // Desktop GNOME profile
        let mut gnome = Profile::new("default/linux/amd64/desktop/gnome");
        gnome.description = "GNOME desktop environment".to_string();
        gnome.parents = vec!["default/linux/amd64/desktop".to_string()];
        gnome.use_flags.extend(
            [
                "gnome".to_string(),
                "gtk3".to_string(),
                "introspection".to_string(),
                "gstreamer".to_string(),
            ]
            .iter()
            .cloned(),
        );
        self.profiles.insert(gnome.name.clone(), gnome);

        // Desktop KDE/Plasma profile
        let mut kde = Profile::new("default/linux/amd64/desktop/plasma");
        kde.description = "KDE Plasma desktop environment".to_string();
        kde.parents = vec!["default/linux/amd64/desktop".to_string()];
        kde.use_flags.extend(
            [
                "kde".to_string(),
                "qt6".to_string(),
                "qml".to_string(),
                "semantic-desktop".to_string(),
            ]
            .iter()
            .cloned(),
        );
        self.profiles.insert(kde.name.clone(), kde);

        // Server profile
        let mut server = Profile::new("default/linux/amd64/server");
        server.description = "Server system without GUI".to_string();
        server.parents = vec!["default/linux/amd64".to_string()];
        server.use_flags.extend(
            [
                "acl".to_string(),
                "caps".to_string(),
                "crypt".to_string(),
                "ssl".to_string(),
                "pam".to_string(),
                "ipv6".to_string(),
            ]
            .iter()
            .cloned(),
        );
        // Explicitly disable GUI-related flags
        server.use_mask.extend(
            [
                "X".to_string(),
                "wayland".to_string(),
                "gtk".to_string(),
                "qt5".to_string(),
                "pulseaudio".to_string(),
                "opengl".to_string(),
            ]
            .iter()
            .cloned(),
        );
        server.stability = ProfileStability::Stable;
        self.profiles.insert(server.name.clone(), server);

        // Hardened profile
        let mut hardened = Profile::new("default/linux/amd64/hardened");
        hardened.description = "Security-hardened system".to_string();
        hardened.parents = vec!["default/linux/amd64".to_string()];
        hardened.make_defaults.insert(
            "CFLAGS".to_string(),
            "-O2 -pipe -march=x86-64 -mtune=generic -fstack-protector-strong -fPIE -D_FORTIFY_SOURCE=2".to_string()
        );
        hardened.make_defaults.insert(
            "LDFLAGS".to_string(),
            "-Wl,-O1 -Wl,--as-needed -Wl,-z,relro -Wl,-z,now -pie".to_string(),
        );
        hardened.use_flags.extend(
            [
                "hardened".to_string(),
                "pie".to_string(),
                "ssp".to_string(),
                "seccomp".to_string(),
                "caps".to_string(),
            ]
            .iter()
            .cloned(),
        );
        hardened.stability = ProfileStability::Stable;
        self.profiles.insert(hardened.name.clone(), hardened);

        // Hardened server profile
        let mut hardened_server = Profile::new("default/linux/amd64/hardened/server");
        hardened_server.description = "Security-hardened server".to_string();
        hardened_server.parents = vec![
            "default/linux/amd64/hardened".to_string(),
            "default/linux/amd64/server".to_string(),
        ];
        self.profiles
            .insert(hardened_server.name.clone(), hardened_server);

        // LLVM/Clang toolchain profile
        let mut llvm = Profile::new("default/linux/amd64/llvm");
        llvm.description = "LLVM/Clang toolchain".to_string();
        llvm.parents = vec!["default/linux/amd64".to_string()];
        llvm.toolchain = Toolchain::Llvm;
        llvm.make_defaults
            .insert("CC".to_string(), "clang".to_string());
        llvm.make_defaults
            .insert("CXX".to_string(), "clang++".to_string());
        llvm.make_defaults
            .insert("AR".to_string(), "llvm-ar".to_string());
        llvm.make_defaults
            .insert("NM".to_string(), "llvm-nm".to_string());
        llvm.make_defaults
            .insert("RANLIB".to_string(), "llvm-ranlib".to_string());
        llvm.make_defaults.insert(
            "CFLAGS".to_string(),
            "-O2 -pipe -march=x86-64 -mtune=generic -flto=thin".to_string(),
        );
        llvm.make_defaults.insert(
            "LDFLAGS".to_string(),
            "-Wl,-O1 -Wl,--as-needed -fuse-ld=lld -flto=thin".to_string(),
        );
        llvm.use_flags.extend(
            ["clang".to_string(), "lld".to_string(), "llvm".to_string()]
                .iter()
                .cloned(),
        );
        llvm.stability = ProfileStability::Testing;
        self.profiles.insert(llvm.name.clone(), llvm);

        // GCC explicit profile
        let mut gcc = Profile::new("default/linux/amd64/gcc");
        gcc.description = "GCC toolchain (explicit)".to_string();
        gcc.parents = vec!["default/linux/amd64".to_string()];
        gcc.toolchain = Toolchain::Gcc;
        gcc.make_defaults
            .insert("CC".to_string(), "gcc".to_string());
        gcc.make_defaults
            .insert("CXX".to_string(), "g++".to_string());
        gcc.make_defaults.insert("AR".to_string(), "ar".to_string());
        gcc.make_defaults.insert("NM".to_string(), "nm".to_string());
        gcc.make_defaults
            .insert("RANLIB".to_string(), "ranlib".to_string());
        gcc.stability = ProfileStability::Stable;
        self.profiles.insert(gcc.name.clone(), gcc);

        // musl libc profile
        let mut musl = Profile::new("default/linux/amd64/musl");
        musl.description = "musl libc system".to_string();
        musl.parents = vec!["default/linux/amd64".to_string()];
        musl.libc = Libc::Musl;
        musl.use_flags.insert("musl".to_string());
        // Packages that don't work well with musl
        musl.package_mask.extend([
            "sys-libs/glibc".to_string(),
            "dev-util/valgrind".to_string(), // Limited musl support
        ]);
        musl.make_defaults.insert(
            "CFLAGS".to_string(),
            "-O2 -pipe -march=x86-64 -mtune=generic -fstack-clash-protection".to_string(),
        );
        musl.stability = ProfileStability::Testing;
        self.profiles.insert(musl.name.clone(), musl);

        // musl hardened profile
        let mut musl_hardened = Profile::new("default/linux/amd64/musl/hardened");
        musl_hardened.description = "Security-hardened musl system".to_string();
        musl_hardened.parents = vec![
            "default/linux/amd64/musl".to_string(),
            "default/linux/amd64/hardened".to_string(),
        ];
        self.profiles
            .insert(musl_hardened.name.clone(), musl_hardened);

        // musl LLVM profile
        let mut musl_llvm = Profile::new("default/linux/amd64/musl/llvm");
        musl_llvm.description = "musl with LLVM/Clang toolchain".to_string();
        musl_llvm.parents = vec![
            "default/linux/amd64/musl".to_string(),
            "default/linux/amd64/llvm".to_string(),
        ];
        self.profiles.insert(musl_llvm.name.clone(), musl_llvm);

        // Systemd profile
        let mut systemd = Profile::new("default/linux/amd64/systemd");
        systemd.description = "systemd init system".to_string();
        systemd.parents = vec!["default/linux/amd64".to_string()];
        systemd
            .use_flags
            .extend(["systemd".to_string(), "udev".to_string()].iter().cloned());
        systemd.use_mask.insert("elogind".to_string());
        self.profiles.insert(systemd.name.clone(), systemd);

        // OpenRC profile
        let mut openrc = Profile::new("default/linux/amd64/openrc");
        openrc.description = "OpenRC init system".to_string();
        openrc.parents = vec!["default/linux/amd64".to_string()];
        openrc.use_flags.extend(
            ["openrc".to_string(), "elogind".to_string()]
                .iter()
                .cloned(),
        );
        openrc.use_mask.insert("systemd".to_string());
        self.profiles.insert(openrc.name.clone(), openrc);

        // Developer profile
        let mut developer = Profile::new("default/linux/amd64/developer");
        developer.description = "Development workstation".to_string();
        developer.parents = vec!["default/linux/amd64/desktop".to_string()];
        developer.use_flags.extend(
            [
                "debug".to_string(),
                "doc".to_string(),
                "examples".to_string(),
                "test".to_string(),
                "static-libs".to_string(),
            ]
            .iter()
            .cloned(),
        );
        developer.make_defaults.insert(
            "CFLAGS".to_string(),
            "-O2 -pipe -march=x86-64 -mtune=generic -g".to_string(),
        );
        developer.stability = ProfileStability::Dev;
        self.profiles.insert(developer.name.clone(), developer);

        // Minimal/embedded profile
        let mut minimal = Profile::new("default/linux/amd64/minimal");
        minimal.description = "Minimal system".to_string();
        minimal.parents = vec!["default/linux/amd64".to_string()];
        minimal.use_mask.extend(
            [
                "X".to_string(),
                "gtk".to_string(),
                "qt5".to_string(),
                "doc".to_string(),
                "examples".to_string(),
                "nls".to_string(),
                "static-libs".to_string(),
            ]
            .iter()
            .cloned(),
        );
        minimal.use_flags.insert("minimal".to_string());
        minimal
            .make_defaults
            .insert("CFLAGS".to_string(), "-Os -pipe -march=x86-64".to_string());
        self.profiles.insert(minimal.name.clone(), minimal);

        // Container profile
        let mut container = Profile::new("default/linux/amd64/container");
        container.description = "Container/Docker optimized".to_string();
        container.parents = vec!["default/linux/amd64/minimal".to_string()];
        container
            .use_mask
            .extend(["pam".to_string(), "caps".to_string()].iter().cloned());
        self.profiles.insert(container.name.clone(), container);

        // No-multilib profile (pure 64-bit)
        let mut no_multilib = Profile::new("default/linux/amd64/no-multilib");
        no_multilib.description = "Pure 64-bit system (no 32-bit support)".to_string();
        no_multilib.parents = vec!["default/linux/amd64".to_string()];
        no_multilib.use_mask.insert("abi_x86_32".to_string());
        no_multilib
            .package_mask
            .push("app-emulation/wine".to_string());
        self.profiles.insert(no_multilib.name.clone(), no_multilib);

        // Prefix profile (for non-root installations)
        let mut prefix = Profile::new("default/linux/amd64/prefix");
        prefix.description = "Prefix installation (non-root)".to_string();
        prefix.parents = vec!["default/linux/amd64".to_string()];
        prefix.use_flags.insert("prefix".to_string());
        self.profiles.insert(prefix.name.clone(), prefix);
    }

    /// Get a profile by name
    pub fn get(&self, name: &str) -> Option<&Profile> {
        self.profiles.get(name)
    }

    /// Get all available profiles
    pub fn list_all(&self) -> Vec<&Profile> {
        let mut profiles: Vec<_> = self.profiles.values().collect();
        profiles.sort_by(|a, b| a.name.cmp(&b.name));
        profiles
    }

    /// Get profiles filtered by architecture
    pub fn list_by_arch(&self, arch: &str) -> Vec<&Profile> {
        self.profiles
            .values()
            .filter(|p| p.arch.as_deref() == Some(arch) || p.name.contains(arch))
            .collect()
    }

    /// Set the current profile
    pub fn set_current(&mut self, name: &str) -> Result<()> {
        // Validate profile exists
        if !self.profiles.contains_key(name) {
            return Err(Error::ProfileNotFound(name.to_string()));
        }

        // Check for deprecation
        if let Some(profile) = self.profiles.get(name) {
            if profile.deprecated {
                warn!(
                    "Profile '{}' is deprecated: {}",
                    name,
                    profile
                        .deprecation_message
                        .as_deref()
                        .unwrap_or("No reason given")
                );
            }
        }

        // Resolve the profile
        self.resolve_profile(name)?;

        // Save selection
        if let Some(parent) = self.current_profile_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&self.current_profile_path, name)?;

        info!("Set current profile to: {}", name);
        Ok(())
    }

    /// Get the current resolved profile
    pub fn current(&self) -> Option<&ResolvedProfile> {
        self.current.as_ref()
    }

    /// Get the current profile name
    pub fn current_name(&self) -> Option<&str> {
        self.current.as_ref().map(|p| p.name.as_str())
    }

    /// Resolve a profile with all its parent inheritance
    pub fn resolve_profile(&mut self, name: &str) -> Result<&ResolvedProfile> {
        let chain = self.get_inheritance_chain(name)?;

        let mut resolved = ResolvedProfile {
            name: name.to_string(),
            chain: chain.clone(),
            use_flags: HashSet::new(),
            use_mask: HashSet::new(),
            use_force: HashSet::new(),
            package_mask: Vec::new(),
            package_unmask: Vec::new(),
            make_defaults: HashMap::new(),
            package_use: HashMap::new(),
            arch: "amd64".to_string(),
            toolchain: Toolchain::Gcc,
            libc: Libc::Glibc,
            keywords: Vec::new(),
        };

        // Apply profiles from root to leaf
        for profile_name in &chain {
            if let Some(profile) = self.profiles.get(profile_name) {
                // Merge USE flags
                resolved.use_flags.extend(profile.use_flags.clone());
                resolved.use_mask.extend(profile.use_mask.clone());
                resolved.use_force.extend(profile.use_force.clone());

                // Merge package masks (later profiles can override)
                resolved.package_mask.extend(profile.package_mask.clone());
                resolved
                    .package_unmask
                    .extend(profile.package_unmask.clone());

                // Merge make.defaults (later profiles override)
                for (key, value) in &profile.make_defaults {
                    resolved.make_defaults.insert(key.clone(), value.clone());
                }

                // Merge package USE
                for (pkg, flags) in &profile.package_use {
                    resolved
                        .package_use
                        .entry(pkg.clone())
                        .or_insert_with(Vec::new)
                        .extend(flags.clone());
                }

                // Update architecture if set
                if let Some(ref arch) = profile.arch {
                    resolved.arch = arch.clone();
                }

                // Update toolchain and libc
                if profile.toolchain != Toolchain::Gcc || profile_name.contains("llvm") {
                    resolved.toolchain = profile.toolchain;
                }
                if profile.libc != Libc::Glibc || profile_name.contains("musl") {
                    resolved.libc = profile.libc;
                }

                // Merge keywords
                resolved.keywords.extend(profile.keywords.clone());
            }
        }

        self.current = Some(resolved);
        Ok(self.current.as_ref().unwrap())
    }

    /// Get the inheritance chain for a profile (root to leaf order)
    fn get_inheritance_chain(&self, name: &str) -> Result<Vec<String>> {
        let mut chain = Vec::new();
        let mut visited = HashSet::new();

        self.collect_parents(name, &mut chain, &mut visited)?;

        // Reverse to get root-to-leaf order
        chain.reverse();

        Ok(chain)
    }

    /// Recursively collect parent profiles
    fn collect_parents(
        &self,
        name: &str,
        chain: &mut Vec<String>,
        visited: &mut HashSet<String>,
    ) -> Result<()> {
        if visited.contains(name) {
            return Ok(()); // Avoid circular dependencies
        }

        let profile = self
            .profiles
            .get(name)
            .ok_or_else(|| Error::ProfileNotFound(name.to_string()))?;

        visited.insert(name.to_string());
        chain.push(name.to_string());

        for parent in &profile.parents {
            self.collect_parents(parent, chain, visited)?;
        }

        Ok(())
    }

    /// Show profile information
    pub fn show_profile(&self, name: &str) -> Result<ProfileInfo> {
        let profile = self
            .profiles
            .get(name)
            .ok_or_else(|| Error::ProfileNotFound(name.to_string()))?;

        let chain = self.get_inheritance_chain(name)?;

        Ok(ProfileInfo {
            name: profile.name.clone(),
            description: profile.description.clone(),
            parents: profile.parents.clone(),
            chain,
            arch: profile.arch.clone(),
            toolchain: profile.toolchain,
            libc: profile.libc,
            stability: profile.stability,
            deprecated: profile.deprecated,
            use_flags_count: profile.use_flags.len(),
            mask_count: profile.package_mask.len(),
        })
    }

    /// Validate a profile configuration
    pub fn validate(&self, name: &str) -> Result<Vec<String>> {
        let mut warnings = Vec::new();

        let profile = self
            .profiles
            .get(name)
            .ok_or_else(|| Error::ProfileNotFound(name.to_string()))?;

        // Check for missing parents
        for parent in &profile.parents {
            if !self.profiles.contains_key(parent) {
                warnings.push(format!("Parent profile not found: {}", parent));
            }
        }

        // Check for circular dependencies
        if let Err(e) = self.get_inheritance_chain(name) {
            warnings.push(format!("Inheritance error: {}", e));
        }

        // Check for conflicting USE flags
        for flag in &profile.use_flags {
            if profile.use_mask.contains(flag) {
                warnings.push(format!("USE flag '{}' is both enabled and masked", flag));
            }
        }

        Ok(warnings)
    }

    /// Compare two profiles
    pub fn compare(&self, profile1: &str, profile2: &str) -> Result<ProfileComparison> {
        let p1 = self
            .profiles
            .get(profile1)
            .ok_or_else(|| Error::ProfileNotFound(profile1.to_string()))?;
        let p2 = self
            .profiles
            .get(profile2)
            .ok_or_else(|| Error::ProfileNotFound(profile2.to_string()))?;

        let use_only_in_1: Vec<_> = p1.use_flags.difference(&p2.use_flags).cloned().collect();
        let use_only_in_2: Vec<_> = p2.use_flags.difference(&p1.use_flags).cloned().collect();
        let use_common: Vec<_> = p1.use_flags.intersection(&p2.use_flags).cloned().collect();

        Ok(ProfileComparison {
            profile1: profile1.to_string(),
            profile2: profile2.to_string(),
            use_only_in_first: use_only_in_1,
            use_only_in_second: use_only_in_2,
            use_common,
            toolchain_differs: p1.toolchain != p2.toolchain,
            libc_differs: p1.libc != p2.libc,
        })
    }
}

impl Default for ProfileManager {
    fn default() -> Self {
        Self::new(
            PathBuf::from("/var/db/repos/gentoo/profiles"),
            PathBuf::from("/etc/buckos/profile"),
        )
    }
}

/// Profile information for display
#[derive(Debug, Clone)]
pub struct ProfileInfo {
    pub name: String,
    pub description: String,
    pub parents: Vec<String>,
    pub chain: Vec<String>,
    pub arch: Option<String>,
    pub toolchain: Toolchain,
    pub libc: Libc,
    pub stability: ProfileStability,
    pub deprecated: bool,
    pub use_flags_count: usize,
    pub mask_count: usize,
}

/// Result of comparing two profiles
#[derive(Debug, Clone)]
pub struct ProfileComparison {
    pub profile1: String,
    pub profile2: String,
    pub use_only_in_first: Vec<String>,
    pub use_only_in_second: Vec<String>,
    pub use_common: Vec<String>,
    pub toolchain_differs: bool,
    pub libc_differs: bool,
}

/// Check if a package matches a pattern
fn package_matches(package: &str, pattern: &str) -> bool {
    if pattern.contains('*') {
        // Simple glob matching without regex
        glob_match(pattern, package)
    } else if pattern.starts_with(">=")
        || pattern.starts_with("<=")
        || pattern.starts_with('>')
        || pattern.starts_with('<')
        || pattern.starts_with('=')
    {
        // Version comparison - simplified
        let pkg_name = pattern.trim_start_matches(|c| c == '>' || c == '<' || c == '=' || c == '!');
        package.starts_with(pkg_name.split('-').next().unwrap_or(pkg_name))
    } else {
        package == pattern || package.starts_with(&format!("{}-", pattern))
    }
}

/// Simple glob matching without regex
fn glob_match(pattern: &str, text: &str) -> bool {
    let mut pattern_chars = pattern.chars().peekable();
    let mut text_chars = text.chars().peekable();

    while let Some(p) = pattern_chars.next() {
        match p {
            '*' => {
                // Skip multiple consecutive asterisks
                while pattern_chars.peek() == Some(&'*') {
                    pattern_chars.next();
                }

                // If * is at the end, match everything
                if pattern_chars.peek().is_none() {
                    return true;
                }

                // Try matching the rest of the pattern from every position
                let remaining_pattern: String = pattern_chars.collect();
                let mut remaining_text = String::new();
                while let Some(c) = text_chars.next() {
                    remaining_text.push(c);
                    let test_text: String = std::iter::once(c).chain(text_chars.clone()).collect();
                    if glob_match(&remaining_pattern, &test_text) {
                        return true;
                    }
                }
                return glob_match(&remaining_pattern, "");
            }
            '?' => {
                // Match any single character
                if text_chars.next().is_none() {
                    return false;
                }
            }
            c => {
                // Match literal character
                if text_chars.next() != Some(c) {
                    return false;
                }
            }
        }
    }

    // Both pattern and text should be exhausted
    text_chars.peek().is_none()
}

/// Format a profile listing
pub fn format_profile_list(profiles: &[&Profile], current: Option<&str>) -> String {
    let mut output = String::new();

    for profile in profiles {
        let marker = if Some(profile.name.as_str()) == current {
            "*"
        } else {
            " "
        };
        let status = if profile.deprecated {
            " (deprecated)"
        } else {
            ""
        };

        output.push_str(&format!(
            "{} {:<45} {}{}\n",
            marker, profile.name, profile.description, status
        ));
    }

    output
}

/// Format profile information for display
pub fn format_profile_info(info: &ProfileInfo) -> String {
    let mut output = String::new();

    output.push_str(&format!("Profile: {}\n", info.name));
    output.push_str(&format!("Description: {}\n", info.description));

    if let Some(ref arch) = info.arch {
        output.push_str(&format!("Architecture: {}\n", arch));
    }

    output.push_str(&format!("Toolchain: {}\n", info.toolchain));
    output.push_str(&format!("Libc: {}\n", info.libc));
    output.push_str(&format!("Stability: {}\n", info.stability));

    if info.deprecated {
        output.push_str("Status: DEPRECATED\n");
    }

    if !info.parents.is_empty() {
        output.push_str(&format!("Parents: {}\n", info.parents.join(", ")));
    }

    output.push_str(&format!("\nInheritance chain:\n"));
    for (i, name) in info.chain.iter().enumerate() {
        output.push_str(&format!("  {}. {}\n", i + 1, name));
    }

    output.push_str(&format!("\nUSE flags: {}\n", info.use_flags_count));
    output.push_str(&format!("Package masks: {}\n", info.mask_count));

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_creation() {
        let profile = Profile::new("test/profile");
        assert_eq!(profile.name, "test/profile");
        assert!(profile.use_flags.is_empty());
    }

    #[test]
    fn test_toolchain_default() {
        let toolchain = Toolchain::default();
        assert_eq!(toolchain, Toolchain::Gcc);
    }

    #[test]
    fn test_libc_default() {
        let libc = Libc::default();
        assert_eq!(libc, Libc::Glibc);
    }

    #[test]
    fn test_profile_manager_builtin() {
        let mut manager = ProfileManager::default();
        manager.load_builtin_profiles();

        // Check that basic profiles exist
        assert!(manager.get("base").is_some());
        assert!(manager.get("default/linux/amd64").is_some());
        assert!(manager.get("default/linux/amd64/desktop").is_some());
        assert!(manager.get("default/linux/amd64/server").is_some());
        assert!(manager.get("default/linux/amd64/hardened").is_some());
        assert!(manager.get("default/linux/amd64/llvm").is_some());
        assert!(manager.get("default/linux/amd64/musl").is_some());
    }

    #[test]
    fn test_package_matching() {
        assert!(package_matches("sys-apps/systemd", "sys-apps/systemd"));
        assert!(package_matches("sys-apps/systemd-255", "sys-apps/systemd"));
        assert!(!package_matches("sys-apps/openrc", "sys-apps/systemd"));
        assert!(package_matches("dev-libs/foo", "dev-libs/*"));
    }

    #[test]
    fn test_profile_use_flags() {
        let mut profile = Profile::new("test");
        profile.use_flags.insert("X".to_string());
        profile.use_mask.insert("gtk".to_string());

        assert!(profile.has_use_flag("X"));
        assert!(!profile.has_use_flag("gtk"));
        assert!(profile.is_use_masked("gtk"));
    }

    #[test]
    fn test_llvm_profile() {
        let mut manager = ProfileManager::default();
        manager.load_builtin_profiles();

        let llvm = manager.get("default/linux/amd64/llvm").unwrap();
        assert_eq!(llvm.toolchain, Toolchain::Llvm);
        assert!(llvm.use_flags.contains("clang"));
        assert!(llvm.use_flags.contains("lld"));
    }

    #[test]
    fn test_musl_profile() {
        let mut manager = ProfileManager::default();
        manager.load_builtin_profiles();

        let musl = manager.get("default/linux/amd64/musl").unwrap();
        assert_eq!(musl.libc, Libc::Musl);
        assert!(musl.use_flags.contains("musl"));
    }

    #[test]
    fn test_hardened_profile() {
        let mut manager = ProfileManager::default();
        manager.load_builtin_profiles();

        let hardened = manager.get("default/linux/amd64/hardened").unwrap();
        assert!(hardened.use_flags.contains("hardened"));
        assert!(hardened.use_flags.contains("pie"));
        assert!(hardened
            .make_defaults
            .get("CFLAGS")
            .unwrap()
            .contains("-fstack-protector-strong"));
    }
}
