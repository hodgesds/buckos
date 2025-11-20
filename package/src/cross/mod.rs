//! Cross-compilation support
//!
//! Implements cross-compilation for building packages for different architectures.
//! Supports CBUILD, CHOST, CTARGET triplets, sysroot management, and cross-toolchain
//! configuration.

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Target architecture triplet (e.g., x86_64-unknown-linux-gnu)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TargetTriplet {
    /// Architecture (e.g., x86_64, aarch64, arm)
    pub arch: String,
    /// Vendor (e.g., unknown, pc, apple)
    pub vendor: String,
    /// Operating system (e.g., linux, darwin, windows)
    pub os: String,
    /// ABI/environment (e.g., gnu, musl, eabi)
    pub abi: Option<String>,
}

impl TargetTriplet {
    /// Create a new target triplet
    pub fn new(arch: &str, vendor: &str, os: &str, abi: Option<&str>) -> Self {
        Self {
            arch: arch.to_string(),
            vendor: vendor.to_string(),
            os: os.to_string(),
            abi: abi.map(|s| s.to_string()),
        }
    }

    /// Parse a triplet string (e.g., "x86_64-unknown-linux-gnu")
    pub fn parse(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split('-').collect();

        if parts.len() < 3 {
            return Err(Error::Other(format!("Invalid triplet: {}", s)));
        }

        let arch = parts[0].to_string();
        let vendor = parts[1].to_string();
        let os = parts[2].to_string();
        let abi = if parts.len() > 3 {
            Some(parts[3..].join("-"))
        } else {
            None
        };

        Ok(Self {
            arch,
            vendor,
            os,
            abi,
        })
    }

    /// Get the full triplet string
    pub fn to_string(&self) -> String {
        if let Some(ref abi) = self.abi {
            format!("{}-{}-{}-{}", self.arch, self.vendor, self.os, abi)
        } else {
            format!("{}-{}-{}", self.arch, self.vendor, self.os)
        }
    }

    /// Get the host triplet for the current system
    pub fn host() -> Result<Self> {
        // Try to detect from rustc
        let output = Command::new("rustc")
            .args(["--version", "--verbose"])
            .output()
            .map_err(|e| Error::Other(format!("Failed to detect host: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.starts_with("host:") {
                let triplet = line.trim_start_matches("host:").trim();
                return Self::parse(triplet);
            }
        }

        // Fallback to common triplets based on cfg
        #[cfg(all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"))]
        return Ok(Self::new("x86_64", "unknown", "linux", Some("gnu")));

        #[cfg(all(target_arch = "aarch64", target_os = "linux", target_env = "gnu"))]
        return Ok(Self::new("aarch64", "unknown", "linux", Some("gnu")));

        #[cfg(not(any(
            all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"),
            all(target_arch = "aarch64", target_os = "linux", target_env = "gnu")
        )))]
        Err(Error::Other("Could not detect host triplet".to_string()))
    }

    /// Check if this is a native build (CBUILD == CHOST)
    pub fn is_native(&self, other: &Self) -> bool {
        self == other
    }

    /// Get common Linux triplets
    pub fn common_triplets() -> Vec<Self> {
        vec![
            Self::new("x86_64", "unknown", "linux", Some("gnu")),
            Self::new("x86_64", "unknown", "linux", Some("musl")),
            Self::new("i686", "unknown", "linux", Some("gnu")),
            Self::new("aarch64", "unknown", "linux", Some("gnu")),
            Self::new("aarch64", "unknown", "linux", Some("musl")),
            Self::new("arm", "unknown", "linux", Some("gnueabihf")),
            Self::new("arm", "unknown", "linux", Some("musleabihf")),
            Self::new("armv7", "unknown", "linux", Some("gnueabihf")),
            Self::new("powerpc64le", "unknown", "linux", Some("gnu")),
            Self::new("riscv64gc", "unknown", "linux", Some("gnu")),
            Self::new("s390x", "unknown", "linux", Some("gnu")),
        ]
    }
}

impl std::fmt::Display for TargetTriplet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// Cross-compilation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossConfig {
    /// Build host triplet (CBUILD) - where compilation runs
    pub cbuild: TargetTriplet,
    /// Target host triplet (CHOST) - where binaries will run
    pub chost: TargetTriplet,
    /// Target triplet (CTARGET) - for building cross-compilers
    pub ctarget: Option<TargetTriplet>,
    /// Sysroot for target libraries
    pub sysroot: Option<PathBuf>,
    /// Cross-toolchain configuration
    pub toolchain: CrossToolchain,
    /// Environment variables for cross-compilation
    pub env: HashMap<String, String>,
    /// PKG_CONFIG settings
    pub pkg_config: PkgConfigSettings,
}

impl CrossConfig {
    /// Create a native build configuration (no cross-compilation)
    pub fn native() -> Result<Self> {
        let host = TargetTriplet::host()?;
        Ok(Self {
            cbuild: host.clone(),
            chost: host,
            ctarget: None,
            sysroot: None,
            toolchain: CrossToolchain::default(),
            env: HashMap::new(),
            pkg_config: PkgConfigSettings::default(),
        })
    }

    /// Create a cross-compilation configuration
    pub fn cross(target: TargetTriplet) -> Result<Self> {
        let build = TargetTriplet::host()?;
        let sysroot = PathBuf::from(format!("/usr/{}", target.to_string()));

        let mut config = Self {
            cbuild: build,
            chost: target.clone(),
            ctarget: None,
            sysroot: Some(sysroot.clone()),
            toolchain: CrossToolchain::for_target(&target),
            env: HashMap::new(),
            pkg_config: PkgConfigSettings::for_sysroot(&sysroot),
        };

        config.setup_env();
        Ok(config)
    }

    /// Check if this is a cross-compilation setup
    pub fn is_cross(&self) -> bool {
        self.cbuild != self.chost
    }

    /// Set up environment variables for cross-compilation
    pub fn setup_env(&mut self) {
        self.env
            .insert("CBUILD".to_string(), self.cbuild.to_string());
        self.env.insert("CHOST".to_string(), self.chost.to_string());

        if let Some(ref ctarget) = self.ctarget {
            self.env.insert("CTARGET".to_string(), ctarget.to_string());
        }

        if let Some(ref sysroot) = self.sysroot {
            self.env
                .insert("SYSROOT".to_string(), sysroot.to_string_lossy().to_string());
        }

        // Add toolchain environment
        self.env.extend(self.toolchain.get_env(&self.chost));

        // Add pkg-config environment
        self.env.extend(self.pkg_config.get_env());
    }

    /// Get all environment variables for cross-compilation
    pub fn get_env(&self) -> &HashMap<String, String> {
        &self.env
    }

    /// Get the target triplet string
    pub fn target_string(&self) -> String {
        self.chost.to_string()
    }

    /// Validate the cross-compilation setup
    pub fn validate(&self) -> Result<Vec<String>> {
        let mut warnings = Vec::new();

        // Check sysroot exists
        if let Some(ref sysroot) = self.sysroot {
            if !sysroot.exists() {
                warnings.push(format!("Sysroot does not exist: {}", sysroot.display()));
            } else {
                // Check for essential directories
                let lib_dir = sysroot.join("lib");
                let usr_lib_dir = sysroot.join("usr/lib");
                let include_dir = sysroot.join("usr/include");

                if !lib_dir.exists() && !usr_lib_dir.exists() {
                    warnings.push(format!(
                        "No lib directory in sysroot: {}",
                        sysroot.display()
                    ));
                }

                if !include_dir.exists() {
                    warnings.push(format!(
                        "No include directory in sysroot: {}",
                        sysroot.display()
                    ));
                }
            }
        } else if self.is_cross() {
            warnings.push("Cross-compilation without sysroot specified".to_string());
        }

        // Check toolchain
        if self.is_cross() {
            let cc = &self.toolchain.cc;
            if !PathBuf::from(cc).exists() && Command::new("which").arg(cc).output().is_err() {
                warnings.push(format!("Cross compiler not found: {}", cc));
            }
        }

        Ok(warnings)
    }
}

/// Cross-toolchain configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossToolchain {
    /// C compiler
    pub cc: String,
    /// C++ compiler
    pub cxx: String,
    /// Linker
    pub ld: String,
    /// Archiver
    pub ar: String,
    /// Ranlib
    pub ranlib: String,
    /// Strip
    pub strip: String,
    /// Object copy
    pub objcopy: String,
    /// Object dump
    pub objdump: String,
    /// Readelf
    pub readelf: String,
    /// Size tool
    pub size: String,
    /// NM tool
    pub nm: String,
    /// Additional CFLAGS for target
    pub cflags: Vec<String>,
    /// Additional CXXFLAGS for target
    pub cxxflags: Vec<String>,
    /// Additional LDFLAGS for target
    pub ldflags: Vec<String>,
}

impl Default for CrossToolchain {
    fn default() -> Self {
        Self {
            cc: "gcc".to_string(),
            cxx: "g++".to_string(),
            ld: "ld".to_string(),
            ar: "ar".to_string(),
            ranlib: "ranlib".to_string(),
            strip: "strip".to_string(),
            objcopy: "objcopy".to_string(),
            objdump: "objdump".to_string(),
            readelf: "readelf".to_string(),
            size: "size".to_string(),
            nm: "nm".to_string(),
            cflags: Vec::new(),
            cxxflags: Vec::new(),
            ldflags: Vec::new(),
        }
    }
}

impl CrossToolchain {
    /// Create toolchain for a specific target
    pub fn for_target(target: &TargetTriplet) -> Self {
        let prefix = target.to_string();
        Self {
            cc: format!("{}-gcc", prefix),
            cxx: format!("{}-g++", prefix),
            ld: format!("{}-ld", prefix),
            ar: format!("{}-ar", prefix),
            ranlib: format!("{}-ranlib", prefix),
            strip: format!("{}-strip", prefix),
            objcopy: format!("{}-objcopy", prefix),
            objdump: format!("{}-objdump", prefix),
            readelf: format!("{}-readelf", prefix),
            size: format!("{}-size", prefix),
            nm: format!("{}-nm", prefix),
            cflags: Vec::new(),
            cxxflags: Vec::new(),
            ldflags: Vec::new(),
        }
    }

    /// Get environment variables for the toolchain
    pub fn get_env(&self, target: &TargetTriplet) -> HashMap<String, String> {
        let mut env = HashMap::new();

        env.insert("CC".to_string(), self.cc.clone());
        env.insert("CXX".to_string(), self.cxx.clone());
        env.insert("LD".to_string(), self.ld.clone());
        env.insert("AR".to_string(), self.ar.clone());
        env.insert("RANLIB".to_string(), self.ranlib.clone());
        env.insert("STRIP".to_string(), self.strip.clone());
        env.insert("OBJCOPY".to_string(), self.objcopy.clone());
        env.insert("OBJDUMP".to_string(), self.objdump.clone());
        env.insert("READELF".to_string(), self.readelf.clone());
        env.insert("SIZE".to_string(), self.size.clone());
        env.insert("NM".to_string(), self.nm.clone());

        // Target-prefixed versions
        let prefix = target.to_string().to_uppercase().replace("-", "_");
        env.insert(format!("CC_{}", prefix), self.cc.clone());
        env.insert(format!("CXX_{}", prefix), self.cxx.clone());

        // Flags
        if !self.cflags.is_empty() {
            env.insert("CFLAGS".to_string(), self.cflags.join(" "));
        }
        if !self.cxxflags.is_empty() {
            env.insert("CXXFLAGS".to_string(), self.cxxflags.join(" "));
        }
        if !self.ldflags.is_empty() {
            env.insert("LDFLAGS".to_string(), self.ldflags.join(" "));
        }

        env
    }

    /// Add sysroot to flags
    pub fn with_sysroot(&mut self, sysroot: &Path) {
        let sysroot_flag = format!("--sysroot={}", sysroot.display());
        self.cflags.push(sysroot_flag.clone());
        self.cxxflags.push(sysroot_flag.clone());
        self.ldflags.push(sysroot_flag);
    }

    /// Check if the toolchain is available
    pub fn is_available(&self) -> bool {
        Command::new("which")
            .arg(&self.cc)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

/// pkg-config settings for cross-compilation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PkgConfigSettings {
    /// Search path for .pc files
    pub path: Vec<PathBuf>,
    /// Sysroot for pkg-config
    pub sysroot: Option<PathBuf>,
    /// Disable default search paths
    pub disable_default: bool,
}

impl Default for PkgConfigSettings {
    fn default() -> Self {
        Self {
            path: vec![
                PathBuf::from("/usr/lib/pkgconfig"),
                PathBuf::from("/usr/share/pkgconfig"),
            ],
            sysroot: None,
            disable_default: false,
        }
    }
}

impl PkgConfigSettings {
    /// Create pkg-config settings for a sysroot
    pub fn for_sysroot(sysroot: &Path) -> Self {
        Self {
            path: vec![
                sysroot.join("usr/lib/pkgconfig"),
                sysroot.join("usr/share/pkgconfig"),
            ],
            sysroot: Some(sysroot.to_path_buf()),
            disable_default: true,
        }
    }

    /// Get environment variables for pkg-config
    pub fn get_env(&self) -> HashMap<String, String> {
        let mut env = HashMap::new();

        let path_str: Vec<String> = self
            .path
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();
        env.insert("PKG_CONFIG_PATH".to_string(), path_str.join(":"));

        if let Some(ref sysroot) = self.sysroot {
            env.insert(
                "PKG_CONFIG_SYSROOT_DIR".to_string(),
                sysroot.to_string_lossy().to_string(),
            );
        }

        if self.disable_default {
            env.insert("PKG_CONFIG_LIBDIR".to_string(), path_str.join(":"));
        }

        env
    }
}

/// Cross-compilation manager
pub struct CrossManager {
    /// Configuration
    config: CrossConfig,
}

impl CrossManager {
    /// Create a new cross manager for native builds
    pub fn native() -> Result<Self> {
        Ok(Self {
            config: CrossConfig::native()?,
        })
    }

    /// Create a new cross manager for cross-compilation
    pub fn cross(target: TargetTriplet) -> Result<Self> {
        Ok(Self {
            config: CrossConfig::cross(target)?,
        })
    }

    /// Create from explicit configuration
    pub fn with_config(config: CrossConfig) -> Self {
        Self { config }
    }

    /// Get the configuration
    pub fn config(&self) -> &CrossConfig {
        &self.config
    }

    /// Check if this is a cross-compilation
    pub fn is_cross(&self) -> bool {
        self.config.is_cross()
    }

    /// Get target triplet
    pub fn target(&self) -> &TargetTriplet {
        &self.config.chost
    }

    /// Get build triplet
    pub fn build(&self) -> &TargetTriplet {
        &self.config.cbuild
    }

    /// Get environment variables for build
    pub fn get_env(&self) -> &HashMap<String, String> {
        self.config.get_env()
    }

    /// Get the sysroot path
    pub fn sysroot(&self) -> Option<&PathBuf> {
        self.config.sysroot.as_ref()
    }

    /// Set custom sysroot
    pub fn set_sysroot(&mut self, sysroot: PathBuf) {
        self.config.sysroot = Some(sysroot.clone());
        self.config.pkg_config = PkgConfigSettings::for_sysroot(&sysroot);
        self.config.toolchain.with_sysroot(&sysroot);
        self.config.setup_env();
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<Vec<String>> {
        self.config.validate()
    }

    /// Get configure flags for autotools
    pub fn get_configure_flags(&self) -> Vec<String> {
        let mut flags = Vec::new();

        flags.push(format!("--build={}", self.config.cbuild));
        flags.push(format!("--host={}", self.config.chost));

        if let Some(ref ctarget) = self.config.ctarget {
            flags.push(format!("--target={}", ctarget));
        }

        flags
    }

    /// Get CMake toolchain file content
    pub fn get_cmake_toolchain(&self) -> String {
        let mut content = String::new();

        content.push_str("# Cross-compilation toolchain file\n");
        content.push_str(&format!(
            "set(CMAKE_SYSTEM_NAME {})\n",
            self.cmake_system_name()
        ));
        content.push_str(&format!(
            "set(CMAKE_SYSTEM_PROCESSOR {})\n",
            self.config.chost.arch
        ));

        if let Some(ref sysroot) = self.config.sysroot {
            content.push_str(&format!("set(CMAKE_SYSROOT {})\n", sysroot.display()));
            content.push_str(&format!(
                "set(CMAKE_FIND_ROOT_PATH {})\n",
                sysroot.display()
            ));
        }

        content.push_str(&format!(
            "set(CMAKE_C_COMPILER {})\n",
            self.config.toolchain.cc
        ));
        content.push_str(&format!(
            "set(CMAKE_CXX_COMPILER {})\n",
            self.config.toolchain.cxx
        ));

        content.push_str("\n# Search settings\n");
        content.push_str("set(CMAKE_FIND_ROOT_PATH_MODE_PROGRAM NEVER)\n");
        content.push_str("set(CMAKE_FIND_ROOT_PATH_MODE_LIBRARY ONLY)\n");
        content.push_str("set(CMAKE_FIND_ROOT_PATH_MODE_INCLUDE ONLY)\n");
        content.push_str("set(CMAKE_FIND_ROOT_PATH_MODE_PACKAGE ONLY)\n");

        content
    }

    /// Get CMake system name from target OS
    fn cmake_system_name(&self) -> &str {
        match self.config.chost.os.as_str() {
            "linux" => "Linux",
            "darwin" => "Darwin",
            "windows" => "Windows",
            "freebsd" => "FreeBSD",
            "netbsd" => "NetBSD",
            "openbsd" => "OpenBSD",
            _ => "Linux",
        }
    }

    /// Get meson cross file content
    pub fn get_meson_cross_file(&self) -> String {
        let mut content = String::new();

        content.push_str("[binaries]\n");
        content.push_str(&format!("c = '{}'\n", self.config.toolchain.cc));
        content.push_str(&format!("cpp = '{}'\n", self.config.toolchain.cxx));
        content.push_str(&format!("ar = '{}'\n", self.config.toolchain.ar));
        content.push_str(&format!("strip = '{}'\n", self.config.toolchain.strip));

        content.push_str("\n[host_machine]\n");
        content.push_str(&format!("system = '{}'\n", self.config.chost.os));
        content.push_str(&format!("cpu_family = '{}'\n", self.meson_cpu_family()));
        content.push_str(&format!("cpu = '{}'\n", self.config.chost.arch));
        content.push_str(&format!("endian = '{}'\n", self.meson_endian()));

        if let Some(ref sysroot) = self.config.sysroot {
            content.push_str("\n[properties]\n");
            content.push_str(&format!("sys_root = '{}'\n", sysroot.display()));
        }

        content
    }

    /// Get meson CPU family from arch
    fn meson_cpu_family(&self) -> &str {
        match self.config.chost.arch.as_str() {
            "x86_64" => "x86_64",
            "i686" | "i386" => "x86",
            "aarch64" => "aarch64",
            "arm" | "armv7" => "arm",
            "powerpc64le" => "ppc64",
            "riscv64gc" => "riscv64",
            "s390x" => "s390x",
            _ => &self.config.chost.arch,
        }
    }

    /// Get endianness for target
    fn meson_endian(&self) -> &str {
        match self.config.chost.arch.as_str() {
            "powerpc" | "powerpc64" | "s390x" => "big",
            _ => "little",
        }
    }

    /// Install cross-toolchain packages (returns package names to install)
    pub fn get_toolchain_packages(&self) -> Vec<String> {
        let target = self.config.chost.to_string();

        vec![
            format!("cross-{}/binutils", target),
            format!("cross-{}/gcc", target),
            format!("cross-{}/glibc", target),
            format!("cross-{}/linux-headers", target),
        ]
    }

    /// Create sysroot directory structure
    pub fn create_sysroot(&self, path: &Path) -> Result<()> {
        let dirs = [
            "bin",
            "etc",
            "lib",
            "lib64",
            "sbin",
            "usr/bin",
            "usr/include",
            "usr/lib",
            "usr/lib64",
            "usr/sbin",
            "usr/share",
        ];

        for dir in &dirs {
            let full_path = path.join(dir);
            std::fs::create_dir_all(&full_path)?;
        }

        Ok(())
    }
}

/// Architecture information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchInfo {
    /// Architecture name
    pub name: String,
    /// Description
    pub description: String,
    /// Default triplet
    pub triplet: TargetTriplet,
    /// Common variants
    pub variants: Vec<TargetTriplet>,
    /// CPU flags
    pub cpu_flags: Vec<String>,
}

impl ArchInfo {
    /// Get architecture info for common architectures
    pub fn get_info(arch: &str) -> Option<Self> {
        match arch {
            "x86_64" | "amd64" => Some(Self {
                name: "x86_64".to_string(),
                description: "64-bit x86 (AMD64/Intel 64)".to_string(),
                triplet: TargetTriplet::new("x86_64", "unknown", "linux", Some("gnu")),
                variants: vec![
                    TargetTriplet::new("x86_64", "unknown", "linux", Some("gnu")),
                    TargetTriplet::new("x86_64", "unknown", "linux", Some("musl")),
                ],
                cpu_flags: vec!["mmx", "sse", "sse2"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
            }),
            "aarch64" | "arm64" => Some(Self {
                name: "aarch64".to_string(),
                description: "64-bit ARM".to_string(),
                triplet: TargetTriplet::new("aarch64", "unknown", "linux", Some("gnu")),
                variants: vec![
                    TargetTriplet::new("aarch64", "unknown", "linux", Some("gnu")),
                    TargetTriplet::new("aarch64", "unknown", "linux", Some("musl")),
                ],
                cpu_flags: vec!["neon", "vfpv3"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
            }),
            "arm" | "armv7" => Some(Self {
                name: "arm".to_string(),
                description: "32-bit ARM".to_string(),
                triplet: TargetTriplet::new("arm", "unknown", "linux", Some("gnueabihf")),
                variants: vec![
                    TargetTriplet::new("arm", "unknown", "linux", Some("gnueabihf")),
                    TargetTriplet::new("arm", "unknown", "linux", Some("musleabihf")),
                    TargetTriplet::new("armv7", "unknown", "linux", Some("gnueabihf")),
                ],
                cpu_flags: vec!["vfp", "neon"].into_iter().map(String::from).collect(),
            }),
            "riscv64" => Some(Self {
                name: "riscv64".to_string(),
                description: "64-bit RISC-V".to_string(),
                triplet: TargetTriplet::new("riscv64gc", "unknown", "linux", Some("gnu")),
                variants: vec![
                    TargetTriplet::new("riscv64gc", "unknown", "linux", Some("gnu")),
                    TargetTriplet::new("riscv64gc", "unknown", "linux", Some("musl")),
                ],
                cpu_flags: Vec::new(),
            }),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triplet_parsing() {
        let triplet = TargetTriplet::parse("x86_64-unknown-linux-gnu").unwrap();
        assert_eq!(triplet.arch, "x86_64");
        assert_eq!(triplet.vendor, "unknown");
        assert_eq!(triplet.os, "linux");
        assert_eq!(triplet.abi, Some("gnu".to_string()));
    }

    #[test]
    fn test_triplet_to_string() {
        let triplet = TargetTriplet::new("aarch64", "unknown", "linux", Some("gnu"));
        assert_eq!(triplet.to_string(), "aarch64-unknown-linux-gnu");
    }

    #[test]
    fn test_cross_config() {
        let target = TargetTriplet::parse("aarch64-unknown-linux-gnu").unwrap();
        let config = CrossConfig::cross(target).unwrap();

        assert!(config.is_cross());
        assert_eq!(config.chost.arch, "aarch64");
    }

    #[test]
    fn test_toolchain_env() {
        let target = TargetTriplet::parse("aarch64-unknown-linux-gnu").unwrap();
        let toolchain = CrossToolchain::for_target(&target);

        assert_eq!(toolchain.cc, "aarch64-unknown-linux-gnu-gcc");
        assert_eq!(toolchain.cxx, "aarch64-unknown-linux-gnu-g++");
    }

    #[test]
    fn test_configure_flags() {
        let target = TargetTriplet::parse("aarch64-unknown-linux-gnu").unwrap();
        let manager = CrossManager::cross(target).unwrap();
        let flags = manager.get_configure_flags();

        assert!(flags.iter().any(|f| f.contains("--host=")));
        assert!(flags.iter().any(|f| f.contains("--build=")));
    }
}
