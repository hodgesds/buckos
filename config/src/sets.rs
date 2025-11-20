//! Package sets configuration
//!
//! Implements Gentoo-style package sets:
//! - @world - explicitly installed packages
//! - @system - core system packages
//! - Custom user sets

use crate::{ConfigError, PackageAtom, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Package sets configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SetsConfig {
    /// Available package sets
    pub sets: HashMap<String, PackageSet>,
}

impl SetsConfig {
    /// Create a new sets configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Create default configuration with standard sets
    pub fn with_defaults() -> Self {
        let mut config = Self::new();

        // World set (user packages)
        config.sets.insert(
            "world".to_string(),
            PackageSet {
                name: "world".to_string(),
                description: "User-selected packages".to_string(),
                atoms: HashSet::new(),
                is_system: false,
                dependencies: vec![],
            },
        );

        // System set (core packages)
        config.sets.insert(
            "system".to_string(),
            PackageSet {
                name: "system".to_string(),
                description: "Core system packages".to_string(),
                atoms: default_system_packages(),
                is_system: true,
                dependencies: vec![],
            },
        );

        // Selected set (selected packages)
        config.sets.insert(
            "selected".to_string(),
            PackageSet {
                name: "selected".to_string(),
                description: "Explicitly selected packages".to_string(),
                atoms: HashSet::new(),
                is_system: false,
                dependencies: vec!["world".to_string()],
            },
        );

        config
    }

    /// Get a set by name
    pub fn get(&self, name: &str) -> Option<&PackageSet> {
        // Handle @ prefix
        let name = name.strip_prefix('@').unwrap_or(name);
        self.sets.get(name)
    }

    /// Get a mutable set by name
    pub fn get_mut(&mut self, name: &str) -> Option<&mut PackageSet> {
        let name = name.strip_prefix('@').unwrap_or(name);
        self.sets.get_mut(name)
    }

    /// Check if a set exists
    pub fn has(&self, name: &str) -> bool {
        let name = name.strip_prefix('@').unwrap_or(name);
        self.sets.contains_key(name)
    }

    /// Create a new set
    pub fn create(&mut self, name: impl Into<String>, description: impl Into<String>) {
        let name = name.into();
        self.sets.insert(
            name.clone(),
            PackageSet {
                name,
                description: description.into(),
                atoms: HashSet::new(),
                is_system: false,
                dependencies: vec![],
            },
        );
    }

    /// Remove a set
    pub fn remove(&mut self, name: &str) -> Option<PackageSet> {
        let name = name.strip_prefix('@').unwrap_or(name);
        self.sets.remove(name)
    }

    /// Add a package to a set
    pub fn add_to_set(&mut self, set_name: &str, atom: PackageAtom) -> Result<()> {
        let set = self
            .get_mut(set_name)
            .ok_or_else(|| ConfigError::Invalid(format!("set not found: {}", set_name)))?;
        set.atoms.insert(atom);
        Ok(())
    }

    /// Remove a package from a set
    pub fn remove_from_set(&mut self, set_name: &str, category: &str, name: &str) -> Result<bool> {
        let set = self
            .get_mut(set_name)
            .ok_or_else(|| ConfigError::Invalid(format!("set not found: {}", set_name)))?;

        let before = set.atoms.len();
        set.atoms.retain(|atom| !atom.matches_cpn(category, name));
        Ok(set.atoms.len() < before)
    }

    /// Get all packages in a set (including dependencies)
    pub fn resolve(&self, name: &str) -> HashSet<PackageAtom> {
        let mut result = HashSet::new();
        let mut visited = HashSet::new();
        self.resolve_recursive(name, &mut result, &mut visited);
        result
    }

    fn resolve_recursive(
        &self,
        name: &str,
        result: &mut HashSet<PackageAtom>,
        visited: &mut HashSet<String>,
    ) {
        let name = name.strip_prefix('@').unwrap_or(name);

        if visited.contains(name) {
            return;
        }
        visited.insert(name.to_string());

        if let Some(set) = self.sets.get(name) {
            result.extend(set.atoms.iter().cloned());

            for dep in &set.dependencies {
                self.resolve_recursive(dep, result, visited);
            }
        }
    }

    /// List all set names
    pub fn names(&self) -> Vec<&str> {
        self.sets.keys().map(|s| s.as_str()).collect()
    }

    /// Load sets from a directory
    pub fn load_from_dir(dir: &Path) -> Result<Self> {
        let mut config = Self::with_defaults();

        if !dir.exists() {
            return Ok(config);
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                let name = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown");

                let content = std::fs::read_to_string(&path)?;
                let set = PackageSet::parse(name, &content)?;
                config.sets.insert(name.to_string(), set);
            }
        }

        Ok(config)
    }

    /// Save the world set
    pub fn save_world(&self, path: &Path) -> Result<()> {
        if let Some(world) = self.get("world") {
            let content = world.format();
            std::fs::write(path, content)?;
        }
        Ok(())
    }
}

/// A package set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageSet {
    /// Set name
    pub name: String,
    /// Description
    pub description: String,
    /// Packages in this set
    pub atoms: HashSet<PackageAtom>,
    /// Whether this is a system set
    pub is_system: bool,
    /// Dependencies on other sets
    pub dependencies: Vec<String>,
}

impl PackageSet {
    /// Create a new empty set
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            atoms: HashSet::new(),
            is_system: false,
            dependencies: vec![],
        }
    }

    /// Add an atom to the set
    pub fn add(&mut self, atom: PackageAtom) {
        self.atoms.insert(atom);
    }

    /// Remove an atom from the set
    pub fn remove(&mut self, category: &str, name: &str) -> bool {
        let before = self.atoms.len();
        self.atoms.retain(|atom| !atom.matches_cpn(category, name));
        self.atoms.len() < before
    }

    /// Check if the set contains a package
    pub fn contains(&self, category: &str, name: &str) -> bool {
        self.atoms
            .iter()
            .any(|atom| atom.matches_cpn(category, name))
    }

    /// Get all atoms
    pub fn atoms(&self) -> &HashSet<PackageAtom> {
        &self.atoms
    }

    /// Parse a set file
    pub fn parse(name: &str, content: &str) -> Result<Self> {
        let mut set = Self::new(name);

        for line in content.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Check for set reference
            if line.starts_with('@') {
                set.dependencies.push(line[1..].to_string());
                continue;
            }

            // Parse atom
            if let Ok(atom) = line.parse::<PackageAtom>() {
                set.atoms.insert(atom);
            }
        }

        Ok(set)
    }

    /// Format as file content
    pub fn format(&self) -> String {
        let mut output = String::new();

        // Sort atoms for consistent output
        let mut atoms: Vec<_> = self.atoms.iter().collect();
        atoms.sort_by_key(|a| (&a.category, &a.name));

        for atom in atoms {
            output.push_str(&atom.to_string());
            output.push('\n');
        }

        output
    }

    /// Number of packages in the set
    pub fn len(&self) -> usize {
        self.atoms.len()
    }

    /// Check if set is empty
    pub fn is_empty(&self) -> bool {
        self.atoms.is_empty()
    }
}

/// Default system packages
fn default_system_packages() -> HashSet<PackageAtom> {
    let packages = vec![
        // Core system
        "sys-apps/baselayout",
        "sys-apps/coreutils",
        "sys-apps/util-linux",
        "sys-apps/shadow",
        "sys-apps/grep",
        "sys-apps/sed",
        "sys-apps/findutils",
        "sys-apps/gawk",
        "sys-apps/file",
        "sys-apps/which",
        "sys-apps/diffutils",
        "sys-apps/less",
        // Filesystem
        "sys-fs/e2fsprogs",
        // Process management
        "sys-process/procps",
        "sys-process/psmisc",
        // Compression
        "app-arch/gzip",
        "app-arch/bzip2",
        "app-arch/xz-utils",
        "app-arch/tar",
        // Network
        "net-misc/wget",
        "net-misc/curl",
        "net-misc/iputils",
        "sys-apps/iproute2",
        // Kernel
        "sys-kernel/linux-firmware",
        // Development (minimal)
        "sys-devel/gcc",
        "sys-devel/binutils",
        "sys-devel/make",
        "sys-libs/glibc",
        // Shell
        "app-shells/bash",
        // Text
        "app-editors/nano",
        // Init system
        "sys-apps/systemd",
        // Package management
        "app-portage/eix",
    ];

    packages
        .into_iter()
        .filter_map(|p| p.parse().ok())
        .collect()
}

/// Well-known set names
pub mod well_known {
    /// The world set - user-selected packages
    pub const WORLD: &str = "world";
    /// The system set - core system packages
    pub const SYSTEM: &str = "system";
    /// Selected packages (alias for world)
    pub const SELECTED: &str = "selected";
    /// Everything installed
    pub const INSTALLED: &str = "installed";
    /// Build dependencies
    pub const BDEPS: &str = "bdeps";
    /// Runtime dependencies
    pub const RDEPS: &str = "rdeps";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sets_config() {
        let mut config = SetsConfig::with_defaults();

        // Add to world
        let atom = PackageAtom::new("app-editors", "vim");
        config.add_to_set("world", atom).unwrap();

        let world = config.get("world").unwrap();
        assert!(world.contains("app-editors", "vim"));
    }

    #[test]
    fn test_set_reference() {
        let mut config = SetsConfig::with_defaults();

        let atom = PackageAtom::new("app-editors", "vim");
        config.add_to_set("world", atom).unwrap();

        // Selected depends on world
        let selected = config.resolve("selected");
        assert!(selected.iter().any(|a| a.matches_cpn("app-editors", "vim")));
    }

    #[test]
    fn test_parse_set() {
        let content = r#"
# My packages
app-editors/vim
dev-vcs/git
@world
"#;

        let set = PackageSet::parse("custom", content).unwrap();
        assert_eq!(set.atoms.len(), 2);
        assert_eq!(set.dependencies, vec!["world"]);
    }

    #[test]
    fn test_system_packages() {
        let config = SetsConfig::with_defaults();
        let system = config.get("system").unwrap();

        assert!(system.contains("sys-apps", "coreutils"));
        assert!(system.contains("sys-devel", "gcc"));
    }
}
