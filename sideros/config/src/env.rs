//! Environment variable configuration
//!
//! Implements Gentoo-style environment configuration:
//! - Global environment variables
//! - Per-package environment (package.env)
//! - Environment file definitions (env.d)

use crate::PackageAtom;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Environment configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EnvConfig {
    /// Global environment variables
    pub global: IndexMap<String, String>,
    /// Per-package environment settings
    pub package: Vec<PackageEnvEntry>,
    /// Named environment file definitions
    pub env_files: HashMap<String, EnvFile>,
    /// bashrc snippets for phases
    pub bashrc: Vec<BashrcEntry>,
}

impl EnvConfig {
    /// Create a new environment configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a global environment variable
    pub fn set_global(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.global.insert(key.into(), value.into());
    }

    /// Get a global environment variable
    pub fn get_global(&self, key: &str) -> Option<&String> {
        self.global.get(key)
    }

    /// Remove a global environment variable
    pub fn remove_global(&mut self, key: &str) {
        self.global.shift_remove(key);
    }

    /// Add a per-package environment entry
    pub fn add_package_env(&mut self, atom: PackageAtom, env_names: Vec<String>) {
        self.package.push(PackageEnvEntry { atom, env_names });
    }

    /// Define an environment file
    pub fn add_env_file(&mut self, name: impl Into<String>, file: EnvFile) {
        self.env_files.insert(name.into(), file);
    }

    /// Get effective environment for a package
    pub fn effective_env(&self, category: &str, name: &str) -> IndexMap<String, String> {
        let mut env = self.global.clone();

        // Apply per-package environment
        for entry in &self.package {
            if entry.atom.matches_cpn(category, name) {
                for env_name in &entry.env_names {
                    if let Some(env_file) = self.env_files.get(env_name) {
                        for (key, value) in &env_file.variables {
                            env.insert(key.clone(), value.clone());
                        }
                    }
                }
            }
        }

        env
    }

    /// Expand variables in a value (e.g., "${CFLAGS}" -> actual value)
    pub fn expand_value(&self, value: &str) -> String {
        let mut result = value.to_string();

        // Simple variable expansion
        for (key, val) in &self.global {
            let pattern = format!("${{{}}}", key);
            result = result.replace(&pattern, val);

            let pattern = format!("${}", key);
            if !result.contains(&format!("${{{}}}",  key)) {
                result = result.replace(&pattern, val);
            }
        }

        result
    }

    /// Get environment for a specific build phase
    pub fn phase_env(&self, category: &str, name: &str, phase: &str) -> IndexMap<String, String> {
        let mut env = self.effective_env(category, name);

        // Add phase-specific bashrc settings
        for entry in &self.bashrc {
            if entry.phase.as_deref() == Some(phase) || entry.phase.is_none() {
                if let Some(ref atom) = entry.atom {
                    if !atom.matches_cpn(category, name) {
                        continue;
                    }
                }

                for (key, value) in &entry.variables {
                    env.insert(key.clone(), value.clone());
                }
            }
        }

        env
    }

    /// Merge another environment configuration
    pub fn merge(&mut self, other: &EnvConfig) {
        // Merge global (other takes precedence)
        for (key, value) in &other.global {
            self.global.insert(key.clone(), value.clone());
        }

        // Merge package entries
        self.package.extend(other.package.iter().cloned());

        // Merge env files
        for (name, file) in &other.env_files {
            self.env_files.insert(name.clone(), file.clone());
        }

        // Merge bashrc
        self.bashrc.extend(other.bashrc.iter().cloned());
    }
}

/// Per-package environment entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageEnvEntry {
    /// The package atom
    pub atom: PackageAtom,
    /// Names of environment files to apply
    pub env_names: Vec<String>,
}

/// An environment file definition (from /etc/portage/env/)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EnvFile {
    /// Variables defined in this file
    pub variables: IndexMap<String, String>,
    /// Description
    pub description: Option<String>,
}

impl EnvFile {
    /// Create a new environment file
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a variable
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.variables.insert(key.into(), value.into());
        self
    }

    /// Set the description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Create from key-value pairs
    pub fn from_pairs(pairs: Vec<(impl Into<String>, impl Into<String>)>) -> Self {
        let mut file = Self::new();
        for (key, value) in pairs {
            file.set(key, value);
        }
        file
    }
}

/// Bashrc entry for build customization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BashrcEntry {
    /// Optional package atom (applies to all if None)
    pub atom: Option<PackageAtom>,
    /// Optional phase restriction
    pub phase: Option<String>,
    /// Variables to set
    pub variables: IndexMap<String, String>,
    /// Shell commands to execute
    pub commands: Vec<String>,
}

/// Common environment presets
pub fn preset_envs() -> HashMap<String, EnvFile> {
    let mut presets = HashMap::new();

    // No optimization (for debugging)
    presets.insert(
        "no-optimization".to_string(),
        EnvFile::from_pairs(vec![
            ("CFLAGS", "-O0 -g"),
            ("CXXFLAGS", "${CFLAGS}"),
        ]).with_description("Disable optimization for debugging".to_string()),
    );

    // Maximum optimization
    presets.insert(
        "max-optimization".to_string(),
        EnvFile::from_pairs(vec![
            ("CFLAGS", "-O3 -march=native -flto"),
            ("CXXFLAGS", "${CFLAGS}"),
            ("LDFLAGS", "-Wl,-O3 -flto"),
        ]).with_description("Maximum optimization with LTO".to_string()),
    );

    // Disable parallel make
    presets.insert(
        "single-job".to_string(),
        EnvFile::from_pairs(vec![
            ("MAKEOPTS", "-j1"),
        ]).with_description("Build with single job (for problematic packages)".to_string()),
    );

    // Disable tests
    presets.insert(
        "no-test".to_string(),
        EnvFile::from_pairs(vec![
            ("FEATURES", "-test"),
            ("RESTRICT", "test"),
        ]).with_description("Skip tests during build".to_string()),
    );

    // Enable ccache
    presets.insert(
        "ccache".to_string(),
        EnvFile::from_pairs(vec![
            ("FEATURES", "ccache"),
            ("CCACHE_DIR", "/var/cache/ccache"),
        ]).with_description("Enable ccache for faster rebuilds".to_string()),
    );

    // Disable sandboxing (for problematic builds)
    presets.insert(
        "no-sandbox".to_string(),
        EnvFile::from_pairs(vec![
            ("FEATURES", "-sandbox -usersandbox -network-sandbox"),
        ]).with_description("Disable sandboxing (use with caution)".to_string()),
    );

    // Keep work directory
    presets.insert(
        "keep-work".to_string(),
        EnvFile::from_pairs(vec![
            ("FEATURES", "keepwork"),
        ]).with_description("Keep work directory after build".to_string()),
    );

    // Clang compiler
    presets.insert(
        "clang".to_string(),
        EnvFile::from_pairs(vec![
            ("CC", "clang"),
            ("CXX", "clang++"),
            ("AR", "llvm-ar"),
            ("NM", "llvm-nm"),
            ("RANLIB", "llvm-ranlib"),
        ]).with_description("Use Clang instead of GCC".to_string()),
    );

    presets
}

/// Build phases where environment can be customized
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildPhase {
    Pretend,
    Setup,
    Unpack,
    Prepare,
    Configure,
    Compile,
    Test,
    Install,
    Preinst,
    Postinst,
    Prerm,
    Postrm,
}

impl BuildPhase {
    /// Get the phase name
    pub fn as_str(&self) -> &'static str {
        match self {
            BuildPhase::Pretend => "pretend",
            BuildPhase::Setup => "setup",
            BuildPhase::Unpack => "unpack",
            BuildPhase::Prepare => "prepare",
            BuildPhase::Configure => "configure",
            BuildPhase::Compile => "compile",
            BuildPhase::Test => "test",
            BuildPhase::Install => "install",
            BuildPhase::Preinst => "preinst",
            BuildPhase::Postinst => "postinst",
            BuildPhase::Prerm => "prerm",
            BuildPhase::Postrm => "postrm",
        }
    }

    /// All phases in order
    pub fn all() -> &'static [BuildPhase] {
        &[
            BuildPhase::Pretend,
            BuildPhase::Setup,
            BuildPhase::Unpack,
            BuildPhase::Prepare,
            BuildPhase::Configure,
            BuildPhase::Compile,
            BuildPhase::Test,
            BuildPhase::Install,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_env() {
        let mut config = EnvConfig::new();
        config.set_global("CFLAGS", "-O2 -pipe");

        let value = config.expand_value("${CFLAGS}");
        assert_eq!(value, "-O2 -pipe");

        // Test that we can get the global value
        assert_eq!(config.get_global("CFLAGS"), Some(&"-O2 -pipe".to_string()));
    }

    #[test]
    fn test_package_env() {
        let mut config = EnvConfig::new();
        config.set_global("CFLAGS", "-O2");

        // Add env file
        let mut env_file = EnvFile::new();
        env_file.set("CFLAGS", "-O0 -g");
        config.add_env_file("debug", env_file);

        // Apply to package
        let atom = PackageAtom::new("app-editors", "vim");
        config.add_package_env(atom, vec!["debug".to_string()]);

        let env = config.effective_env("app-editors", "vim");
        assert_eq!(env.get("CFLAGS"), Some(&"-O0 -g".to_string()));

        // Other packages use global
        let env = config.effective_env("app-editors", "emacs");
        assert_eq!(env.get("CFLAGS"), Some(&"-O2".to_string()));
    }

    #[test]
    fn test_preset_envs() {
        let presets = preset_envs();
        assert!(presets.contains_key("no-optimization"));
        assert!(presets.contains_key("ccache"));
        assert!(presets.contains_key("clang"));
    }
}
