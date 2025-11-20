//! System profile configuration
//!
//! Implements Gentoo-style profile system:
//! - Profile selection and hierarchy
//! - Profile cascading
//! - Profile-provided USE flags and masks

use crate::{ConfigError, MaskConfig, PackageAtom, Result, UseConfig};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Profile configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileConfig {
    /// Current profile path (relative to profiles directory)
    pub current: String,
    /// Profile stack (from parent to current)
    pub stack: Vec<ProfileInfo>,
    /// Accumulated USE configuration from profiles
    pub use_config: UseConfig,
    /// Accumulated mask configuration from profiles
    pub mask_config: MaskConfig,
    /// Package provided atoms
    pub provided: HashSet<PackageAtom>,
    /// Profile variables
    pub variables: HashMap<String, String>,
}

impl Default for ProfileConfig {
    fn default() -> Self {
        Self {
            current: "default/linux/amd64/23.0".to_string(),
            stack: Vec::new(),
            use_config: UseConfig::default(),
            mask_config: MaskConfig::default(),
            provided: HashSet::new(),
            variables: HashMap::new(),
        }
    }
}

impl ProfileConfig {
    /// Create a new profile configuration
    pub fn new(profile: impl Into<String>) -> Self {
        Self {
            current: profile.into(),
            ..Default::default()
        }
    }

    /// Load profile from path
    pub fn load(profile_path: &Path) -> Result<Self> {
        let mut config = Self::default();

        // Build profile stack by following parent links
        let mut stack = Vec::new();
        let mut current = profile_path.to_path_buf();

        while current.exists() {
            let info = ProfileInfo::load(&current)?;
            stack.push(info);

            // Check for parent
            let parent_file = current.join("parent");
            if parent_file.exists() {
                let parent_content = std::fs::read_to_string(&parent_file)?;
                for line in parent_content.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }

                    // Resolve parent path
                    let parent_path = if line.starts_with(':') {
                        // Absolute path within profiles
                        current.parent().unwrap().parent().unwrap().join(&line[1..])
                    } else {
                        current.join(line)
                    };

                    current = parent_path
                        .canonicalize()
                        .map_err(|_| ConfigError::ProfileNotFound(line.to_string()))?;
                    break;
                }
            } else {
                break;
            }
        }

        // Reverse to get parent-first order
        stack.reverse();
        config.stack = stack;

        // Accumulate settings from profile stack
        for profile in &config.stack {
            config.use_config.merge(&profile.use_config);
            config.mask_config.merge(&profile.mask_config);
            config.provided.extend(profile.provided.iter().cloned());
            config.variables.extend(profile.variables.clone());
        }

        Ok(config)
    }

    /// Get the profile name
    pub fn name(&self) -> &str {
        &self.current
    }

    /// Check if this is a stable profile
    pub fn is_stable(&self) -> bool {
        !self.current.contains("/testing")
            && !self.current.contains("/unstable")
            && !self.current.contains("/exp")
    }

    /// Get the arch from profile
    pub fn arch(&self) -> Option<&str> {
        // Extract arch from profile path like "default/linux/amd64/23.0"
        let parts: Vec<&str> = self.current.split('/').collect();
        if parts.len() >= 3 {
            Some(parts[2])
        } else {
            None
        }
    }

    /// Get profile-forced USE flags
    pub fn forced_use(&self) -> &HashSet<String> {
        &self.use_config.force
    }

    /// Get profile-masked USE flags
    pub fn masked_use(&self) -> &HashSet<String> {
        &self.use_config.mask
    }

    /// Check if a package is provided by the profile
    pub fn is_provided(&self, category: &str, name: &str) -> bool {
        self.provided
            .iter()
            .any(|atom| atom.matches_cpn(category, name))
    }

    /// Get a profile variable
    pub fn get_variable(&self, name: &str) -> Option<&String> {
        self.variables.get(name)
    }
}

/// Information about a single profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileInfo {
    /// Profile path
    pub path: PathBuf,
    /// Profile name
    pub name: String,
    /// Profile status (stable, dev, exp)
    pub status: ProfileStatus,
    /// Profile USE configuration
    pub use_config: UseConfig,
    /// Profile mask configuration
    pub mask_config: MaskConfig,
    /// Packages provided by this profile
    pub provided: HashSet<PackageAtom>,
    /// Profile variables
    pub variables: HashMap<String, String>,
    /// EAPI version
    pub eapi: Option<String>,
    /// Whether this profile is deprecated
    pub deprecated: bool,
    /// Deprecation message
    pub deprecation_message: Option<String>,
}

impl ProfileInfo {
    /// Load profile info from a directory
    pub fn load(path: &Path) -> Result<Self> {
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let mut info = Self {
            path: path.to_path_buf(),
            name,
            status: ProfileStatus::Stable,
            use_config: UseConfig::default(),
            mask_config: MaskConfig::default(),
            provided: HashSet::new(),
            variables: HashMap::new(),
            eapi: None,
            deprecated: false,
            deprecation_message: None,
        };

        // Read eapi
        let eapi_file = path.join("eapi");
        if eapi_file.exists() {
            info.eapi = Some(std::fs::read_to_string(eapi_file)?.trim().to_string());
        }

        // Check for deprecation
        let deprecated_file = path.join("deprecated");
        if deprecated_file.exists() {
            info.deprecated = true;
            info.deprecation_message = Some(std::fs::read_to_string(deprecated_file)?);
        }

        // Read USE flags
        let use_file = path.join("use.force");
        if use_file.exists() {
            let content = std::fs::read_to_string(use_file)?;
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if line.starts_with('-') {
                    info.use_config.force.remove(&line[1..].to_string());
                } else {
                    info.use_config.force.insert(line.to_string());
                }
            }
        }

        let use_mask_file = path.join("use.mask");
        if use_mask_file.exists() {
            let content = std::fs::read_to_string(use_mask_file)?;
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if line.starts_with('-') {
                    info.use_config.mask.remove(&line[1..].to_string());
                } else {
                    info.use_config.mask.insert(line.to_string());
                }
            }
        }

        // Read package.use.force
        let pkg_use_force = path.join("package.use.force");
        if pkg_use_force.exists() {
            // Parse per-package USE force
            let content = std::fs::read_to_string(pkg_use_force)?;
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                // Format: atom flag1 flag2 ...
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(atom) = parts[0].parse::<PackageAtom>() {
                        let flags = UseConfig::parse_use_string(&parts[1..].join(" "));
                        info.use_config.add_package_use(atom, flags);
                    }
                }
            }
        }

        // Read provided packages
        let packages_file = path.join("packages");
        if packages_file.exists() {
            let content = std::fs::read_to_string(packages_file)?;
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                // Lines starting with * are system packages
                let line = line.trim_start_matches('*');
                if let Ok(atom) = line.parse::<PackageAtom>() {
                    info.provided.insert(atom);
                }
            }
        }

        // Read make.defaults
        let make_defaults = path.join("make.defaults");
        if make_defaults.exists() {
            let content = std::fs::read_to_string(make_defaults)?;
            parse_make_defaults(&content, &mut info.variables)?;
        }

        Ok(info)
    }
}

/// Parse make.defaults content
fn parse_make_defaults(content: &str, variables: &mut HashMap<String, String>) -> Result<()> {
    let mut current_var: Option<String> = None;
    let mut current_value = String::new();

    for line in content.lines() {
        let line = line.trim();

        // Skip comments
        if line.starts_with('#') {
            continue;
        }

        // Check for continuation
        if let Some(ref var) = current_var {
            if line.ends_with('\\') {
                current_value.push_str(&line[..line.len() - 1]);
                current_value.push(' ');
                continue;
            } else {
                current_value.push_str(line);
                variables.insert(var.clone(), current_value.trim().to_string());
                current_var = None;
                current_value.clear();
                continue;
            }
        }

        // Skip empty lines
        if line.is_empty() {
            continue;
        }

        // Parse variable assignment
        if let Some(eq_idx) = line.find('=') {
            let var = line[..eq_idx].trim();
            let value = line[eq_idx + 1..].trim();

            // Remove quotes
            let value = value.trim_matches('"').trim_matches('\'');

            if value.ends_with('\\') {
                current_var = Some(var.to_string());
                current_value = value[..value.len() - 1].to_string();
                current_value.push(' ');
            } else {
                variables.insert(var.to_string(), value.to_string());
            }
        }
    }

    Ok(())
}

/// Profile status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProfileStatus {
    /// Stable profile
    Stable,
    /// Development profile
    Dev,
    /// Experimental profile
    Exp,
}

/// Available profiles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailableProfiles {
    /// List of profiles
    pub profiles: Vec<ProfileEntry>,
}

impl AvailableProfiles {
    /// Load available profiles from a profiles directory
    pub fn load(profiles_dir: &Path) -> Result<Self> {
        let mut profiles = Vec::new();

        let profiles_desc = profiles_dir.join("profiles.desc");
        if profiles_desc.exists() {
            let content = std::fs::read_to_string(profiles_desc)?;
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }

                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let arch = parts[0].to_string();
                    let path = parts[1].to_string();
                    let status = match parts[2] {
                        "stable" => ProfileStatus::Stable,
                        "dev" => ProfileStatus::Dev,
                        "exp" => ProfileStatus::Exp,
                        _ => ProfileStatus::Stable,
                    };

                    profiles.push(ProfileEntry { arch, path, status });
                }
            }
        }

        Ok(Self { profiles })
    }

    /// Get profiles for an architecture
    pub fn for_arch(&self, arch: &str) -> Vec<&ProfileEntry> {
        self.profiles.iter().filter(|p| p.arch == arch).collect()
    }

    /// Get stable profiles for an architecture
    pub fn stable_for_arch(&self, arch: &str) -> Vec<&ProfileEntry> {
        self.profiles
            .iter()
            .filter(|p| p.arch == arch && p.status == ProfileStatus::Stable)
            .collect()
    }
}

/// A profile entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileEntry {
    /// Architecture
    pub arch: String,
    /// Profile path
    pub path: String,
    /// Profile status
    pub status: ProfileStatus,
}

/// Common Buckos profiles
pub fn default_profiles() -> Vec<ProfileEntry> {
    vec![
        ProfileEntry {
            arch: "amd64".to_string(),
            path: "default/linux/amd64/23.0".to_string(),
            status: ProfileStatus::Stable,
        },
        ProfileEntry {
            arch: "amd64".to_string(),
            path: "default/linux/amd64/23.0/systemd".to_string(),
            status: ProfileStatus::Stable,
        },
        ProfileEntry {
            arch: "amd64".to_string(),
            path: "default/linux/amd64/23.0/desktop".to_string(),
            status: ProfileStatus::Stable,
        },
        ProfileEntry {
            arch: "amd64".to_string(),
            path: "default/linux/amd64/23.0/desktop/gnome".to_string(),
            status: ProfileStatus::Stable,
        },
        ProfileEntry {
            arch: "amd64".to_string(),
            path: "default/linux/amd64/23.0/desktop/kde".to_string(),
            status: ProfileStatus::Stable,
        },
        ProfileEntry {
            arch: "arm64".to_string(),
            path: "default/linux/arm64/23.0".to_string(),
            status: ProfileStatus::Stable,
        },
        ProfileEntry {
            arch: "arm64".to_string(),
            path: "default/linux/arm64/23.0/systemd".to_string(),
            status: ProfileStatus::Stable,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_config() {
        let config = ProfileConfig::new("default/linux/amd64/23.0");
        assert!(config.is_stable());
        assert_eq!(config.arch(), Some("amd64"));
    }

    #[test]
    fn test_profile_status() {
        let config = ProfileConfig::new("default/linux/amd64/23.0/testing");
        assert!(!config.is_stable());
    }

    #[test]
    fn test_parse_make_defaults() {
        let content = r#"
# Comment
USE="X wayland"
CFLAGS="-O2 -pipe \
    -march=native"
"#;

        let mut vars = HashMap::new();
        parse_make_defaults(content, &mut vars).unwrap();

        assert_eq!(vars.get("USE"), Some(&"X wayland".to_string()));
        assert!(vars.get("CFLAGS").unwrap().contains("-march=native"));
    }
}
