//! USE flag configuration system
//!
//! Implements Gentoo-style USE flag handling including:
//! - Global USE flags
//! - Per-package USE flags
//! - USE flag expansion and dependencies
//! - USE_EXPAND variables (CPU_FLAGS_*, VIDEO_CARDS, etc.)

use crate::PackageAtom;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Complete USE flag configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UseConfig {
    /// Global USE flags (from make.conf USE="...")
    pub global: HashSet<String>,
    /// Per-package USE flags (from package.use)
    pub package: Vec<PackageUseEntry>,
    /// USE_EXPAND variables (e.g., CPU_FLAGS_X86, VIDEO_CARDS)
    pub expand: IndexMap<String, HashSet<String>>,
    /// USE flag masks (disabled flags)
    pub mask: HashSet<String>,
    /// USE flag force (forced flags)
    pub force: HashSet<String>,
    /// Stable USE mask
    pub stable_mask: HashSet<String>,
    /// Stable USE force
    pub stable_force: HashSet<String>,
}

impl UseConfig {
    /// Create a new empty USE configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a global USE flag
    pub fn add_global(&mut self, flag: impl Into<String>) {
        self.global.insert(flag.into());
    }

    /// Remove a global USE flag
    pub fn remove_global(&mut self, flag: &str) {
        self.global.remove(flag);
    }

    /// Check if a global USE flag is enabled
    pub fn is_global_enabled(&self, flag: &str) -> bool {
        self.global.contains(flag)
    }

    /// Add a per-package USE flag entry
    pub fn add_package_use(&mut self, atom: PackageAtom, flags: Vec<UseFlag>) {
        self.package.push(PackageUseEntry { atom, flags });
    }

    /// Get effective USE flags for a package
    pub fn effective_flags(&self, category: &str, name: &str) -> HashSet<String> {
        let mut flags = self.global.clone();

        // Add expanded flags
        for (prefix, values) in &self.expand {
            for value in values {
                flags.insert(format!("{}_{}", prefix.to_lowercase(), value.to_lowercase()));
            }
        }

        // Apply per-package settings
        for entry in &self.package {
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

        // Apply masks
        for flag in &self.mask {
            flags.remove(flag);
        }

        // Apply force
        for flag in &self.force {
            flags.insert(flag.clone());
        }

        flags
    }

    /// Parse a USE string (e.g., "X wayland -gtk systemd")
    pub fn parse_use_string(s: &str) -> Vec<UseFlag> {
        s.split_whitespace()
            .filter(|s| !s.is_empty())
            .map(|s| UseFlag::parse(s))
            .collect()
    }

    /// Merge another USE configuration into this one
    pub fn merge(&mut self, other: &UseConfig) {
        // Merge global flags
        self.global.extend(other.global.iter().cloned());

        // Merge package flags
        self.package.extend(other.package.iter().cloned());

        // Merge expand variables
        for (key, values) in &other.expand {
            self.expand
                .entry(key.clone())
                .or_default()
                .extend(values.iter().cloned());
        }

        // Merge masks and forces
        self.mask.extend(other.mask.iter().cloned());
        self.force.extend(other.force.iter().cloned());
        self.stable_mask.extend(other.stable_mask.iter().cloned());
        self.stable_force.extend(other.stable_force.iter().cloned());
    }

    /// Add a USE_EXPAND variable
    pub fn add_expand(&mut self, variable: impl Into<String>, value: impl Into<String>) {
        self.expand
            .entry(variable.into())
            .or_default()
            .insert(value.into());
    }

    /// Set CPU flags
    pub fn set_cpu_flags(&mut self, arch: &str, flags: Vec<String>) {
        let key = format!("CPU_FLAGS_{}", arch.to_uppercase());
        self.expand.insert(key, flags.into_iter().collect());
    }

    /// Set video cards
    pub fn set_video_cards(&mut self, cards: Vec<String>) {
        self.expand.insert("VIDEO_CARDS".to_string(), cards.into_iter().collect());
    }

    /// Set input devices
    pub fn set_input_devices(&mut self, devices: Vec<String>) {
        self.expand.insert("INPUT_DEVICES".to_string(), devices.into_iter().collect());
    }
}

/// A single USE flag with enable/disable state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UseFlag {
    /// The flag name
    pub name: String,
    /// Whether the flag is enabled
    pub enabled: bool,
}

impl UseFlag {
    /// Create a new enabled USE flag
    pub fn enabled(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            enabled: true,
        }
    }

    /// Create a new disabled USE flag
    pub fn disabled(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            enabled: false,
        }
    }

    /// Parse a USE flag string (e.g., "-gtk" or "systemd")
    pub fn parse(s: &str) -> Self {
        let s = s.trim();
        if s.starts_with('-') {
            Self::disabled(&s[1..])
        } else {
            Self::enabled(s)
        }
    }
}

impl std::fmt::Display for UseFlag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.enabled {
            write!(f, "-")?;
        }
        write!(f, "{}", self.name)
    }
}

/// Per-package USE flag entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageUseEntry {
    /// The package atom
    pub atom: PackageAtom,
    /// USE flags for this package
    pub flags: Vec<UseFlag>,
}

/// USE flag description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UseFlagDescription {
    /// The flag name
    pub name: String,
    /// Description of what the flag does
    pub description: String,
    /// Whether this is a global flag
    pub global: bool,
    /// Which packages this flag applies to (if not global)
    pub packages: Vec<String>,
}

/// USE_EXPAND variable definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UseExpandVariable {
    /// Variable name (e.g., "CPU_FLAGS_X86")
    pub name: String,
    /// Possible values
    pub values: HashSet<String>,
    /// Description
    pub description: String,
    /// Whether values are unprefixed in USE
    pub unprefixed: bool,
    /// Whether this is an implicit variable
    pub implicit: bool,
}

/// Common USE_EXPAND variables
pub fn default_use_expand() -> HashMap<String, UseExpandVariable> {
    let mut vars = HashMap::new();

    vars.insert(
        "CPU_FLAGS_X86".to_string(),
        UseExpandVariable {
            name: "CPU_FLAGS_X86".to_string(),
            values: [
                "aes", "avx", "avx2", "avx512f", "avx512dq", "avx512cd", "avx512bw", "avx512vl",
                "mmx", "mmxext", "pclmul", "popcnt", "sse", "sse2", "sse3", "ssse3",
                "sse4_1", "sse4_2", "sse4a", "f16c", "fma", "fma4", "xop",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
            description: "CPU instruction set extensions for x86/amd64".to_string(),
            unprefixed: false,
            implicit: false,
        },
    );

    vars.insert(
        "VIDEO_CARDS".to_string(),
        UseExpandVariable {
            name: "VIDEO_CARDS".to_string(),
            values: [
                "amdgpu", "ast", "dummy", "fbdev", "i915", "i965", "intel",
                "mga", "nouveau", "nvidia", "r128", "r600", "radeon", "radeonsi",
                "vesa", "via", "virtualbox", "vmware",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
            description: "Video card drivers to build".to_string(),
            unprefixed: false,
            implicit: false,
        },
    );

    vars.insert(
        "INPUT_DEVICES".to_string(),
        UseExpandVariable {
            name: "INPUT_DEVICES".to_string(),
            values: [
                "evdev", "joystick", "keyboard", "libinput", "mouse", "synaptics",
                "vmmouse", "wacom",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
            description: "Input device drivers to build".to_string(),
            unprefixed: false,
            implicit: false,
        },
    );

    vars.insert(
        "L10N".to_string(),
        UseExpandVariable {
            name: "L10N".to_string(),
            values: HashSet::new(), // Language codes are dynamic
            description: "Localization support".to_string(),
            unprefixed: false,
            implicit: false,
        },
    );

    vars.insert(
        "PYTHON_TARGETS".to_string(),
        UseExpandVariable {
            name: "PYTHON_TARGETS".to_string(),
            values: [
                "python3_10", "python3_11", "python3_12", "python3_13",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
            description: "Python implementations to build against".to_string(),
            unprefixed: false,
            implicit: false,
        },
    );

    vars.insert(
        "RUBY_TARGETS".to_string(),
        UseExpandVariable {
            name: "RUBY_TARGETS".to_string(),
            values: ["ruby31", "ruby32", "ruby33"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            description: "Ruby implementations to build against".to_string(),
            unprefixed: false,
            implicit: false,
        },
    );

    vars
}

/// Well-known global USE flags
pub fn common_use_flags() -> Vec<UseFlagDescription> {
    vec![
        UseFlagDescription {
            name: "X".to_string(),
            description: "Add support for X11".to_string(),
            global: true,
            packages: vec![],
        },
        UseFlagDescription {
            name: "wayland".to_string(),
            description: "Enable Wayland support".to_string(),
            global: true,
            packages: vec![],
        },
        UseFlagDescription {
            name: "systemd".to_string(),
            description: "Enable systemd integration".to_string(),
            global: true,
            packages: vec![],
        },
        UseFlagDescription {
            name: "elogind".to_string(),
            description: "Enable elogind support (standalone logind)".to_string(),
            global: true,
            packages: vec![],
        },
        UseFlagDescription {
            name: "pulseaudio".to_string(),
            description: "Add support for PulseAudio".to_string(),
            global: true,
            packages: vec![],
        },
        UseFlagDescription {
            name: "pipewire".to_string(),
            description: "Add support for PipeWire".to_string(),
            global: true,
            packages: vec![],
        },
        UseFlagDescription {
            name: "dbus".to_string(),
            description: "Enable D-Bus support".to_string(),
            global: true,
            packages: vec![],
        },
        UseFlagDescription {
            name: "gtk".to_string(),
            description: "Add support for GTK+".to_string(),
            global: true,
            packages: vec![],
        },
        UseFlagDescription {
            name: "qt5".to_string(),
            description: "Add support for Qt5".to_string(),
            global: true,
            packages: vec![],
        },
        UseFlagDescription {
            name: "qt6".to_string(),
            description: "Add support for Qt6".to_string(),
            global: true,
            packages: vec![],
        },
        UseFlagDescription {
            name: "ssl".to_string(),
            description: "Add support for SSL/TLS".to_string(),
            global: true,
            packages: vec![],
        },
        UseFlagDescription {
            name: "gnutls".to_string(),
            description: "Use GnuTLS instead of OpenSSL".to_string(),
            global: true,
            packages: vec![],
        },
        UseFlagDescription {
            name: "zstd".to_string(),
            description: "Enable zstd compression".to_string(),
            global: true,
            packages: vec![],
        },
        UseFlagDescription {
            name: "lz4".to_string(),
            description: "Enable LZ4 compression".to_string(),
            global: true,
            packages: vec![],
        },
        UseFlagDescription {
            name: "doc".to_string(),
            description: "Build documentation".to_string(),
            global: true,
            packages: vec![],
        },
        UseFlagDescription {
            name: "examples".to_string(),
            description: "Install examples".to_string(),
            global: true,
            packages: vec![],
        },
        UseFlagDescription {
            name: "test".to_string(),
            description: "Build and run tests".to_string(),
            global: true,
            packages: vec![],
        },
        UseFlagDescription {
            name: "debug".to_string(),
            description: "Enable debug symbols and features".to_string(),
            global: true,
            packages: vec![],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_use_flag() {
        let flag = UseFlag::parse("systemd");
        assert_eq!(flag.name, "systemd");
        assert!(flag.enabled);

        let flag = UseFlag::parse("-gtk");
        assert_eq!(flag.name, "gtk");
        assert!(!flag.enabled);
    }

    #[test]
    fn test_effective_flags() {
        let mut config = UseConfig::new();
        config.add_global("X");
        config.add_global("wayland");
        config.add_global("systemd");

        // Add per-package override
        let atom = PackageAtom::new("app-editors", "vim");
        config.add_package_use(atom, vec![UseFlag::disabled("X")]);

        let flags = config.effective_flags("app-editors", "vim");
        assert!(!flags.contains("X"));
        assert!(flags.contains("wayland"));
        assert!(flags.contains("systemd"));

        let flags = config.effective_flags("app-misc", "neofetch");
        assert!(flags.contains("X"));
    }

    #[test]
    fn test_use_expand() {
        let mut config = UseConfig::new();
        config.set_cpu_flags("x86", vec!["avx2".to_string(), "sse4_2".to_string()]);

        let flags = config.effective_flags("some", "package");
        assert!(flags.contains("cpu_flags_x86_avx2"));
        assert!(flags.contains("cpu_flags_x86_sse4_2"));
    }
}
