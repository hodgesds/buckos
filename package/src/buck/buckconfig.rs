//! Buck2 configuration file parsing and management
//!
//! This module provides support for parsing and manipulating .buckconfig files,
//! as well as managing custom Buck configuration options.

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

/// Represents a parsed .buckconfig file
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BuckConfigFile {
    /// Sections in the config file
    pub sections: BTreeMap<String, BuckConfigSection>,
    /// Source path of the config file
    pub source_path: Option<PathBuf>,
}

/// A section within a .buckconfig file
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BuckConfigSection {
    /// Key-value pairs in the section
    pub values: BTreeMap<String, String>,
}

/// Custom Buck configuration options for builds
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BuckConfigOptions {
    /// Path to custom .buckconfig file to use
    pub config_file: Option<PathBuf>,
    /// Configuration overrides (section.key = value)
    pub overrides: HashMap<String, String>,
    /// Cell overrides (cell_name = path)
    pub cell_overrides: HashMap<String, PathBuf>,
    /// Build mode configuration
    pub build_mode: Option<String>,
    /// Execution platform
    pub execution_platform: Option<String>,
    /// Target platform
    pub target_platform: Option<String>,
    /// Modifier options
    pub modifiers: Vec<String>,
    /// Configuration files to include
    pub config_includes: Vec<PathBuf>,
}

impl BuckConfigFile {
    /// Create a new empty config file
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse a .buckconfig file from the given path
    pub fn parse(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| Error::ConfigError(format!("Failed to read {}: {}", path.display(), e)))?;

        let mut config = Self::parse_str(&content)?;
        config.source_path = Some(path.to_path_buf());
        Ok(config)
    }

    /// Parse a .buckconfig from a string
    pub fn parse_str(content: &str) -> Result<Self> {
        let mut config = BuckConfigFile::default();
        let mut current_section = String::new();

        for (line_num, line) in content.lines().enumerate() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }

            // Check for section header
            if line.starts_with('[') && line.ends_with(']') {
                current_section = line[1..line.len() - 1].trim().to_string();
                config
                    .sections
                    .entry(current_section.clone())
                    .or_insert_with(BuckConfigSection::default);
                continue;
            }

            // Parse key-value pair
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim().to_string();
                let value = value.trim().to_string();

                if current_section.is_empty() {
                    return Err(Error::ConfigError(format!(
                        "Line {}: Key-value pair outside of section: {}",
                        line_num + 1,
                        line
                    )));
                }

                config
                    .sections
                    .get_mut(&current_section)
                    .unwrap()
                    .values
                    .insert(key, value);
            } else {
                return Err(Error::ConfigError(format!(
                    "Line {}: Invalid line in .buckconfig: {}",
                    line_num + 1,
                    line
                )));
            }
        }

        Ok(config)
    }

    /// Get a value from the config
    pub fn get(&self, section: &str, key: &str) -> Option<&str> {
        self.sections
            .get(section)
            .and_then(|s| s.values.get(key))
            .map(|s| s.as_str())
    }

    /// Set a value in the config
    pub fn set(&mut self, section: &str, key: &str, value: &str) {
        self.sections
            .entry(section.to_string())
            .or_insert_with(BuckConfigSection::default)
            .values
            .insert(key.to_string(), value.to_string());
    }

    /// Merge another config into this one (other takes precedence)
    pub fn merge(&mut self, other: &BuckConfigFile) {
        for (section_name, section) in &other.sections {
            let target_section = self
                .sections
                .entry(section_name.clone())
                .or_insert_with(BuckConfigSection::default);

            for (key, value) in &section.values {
                target_section.values.insert(key.clone(), value.clone());
            }
        }
    }

    /// Write the config to a file
    pub fn write(&self, path: &Path) -> Result<()> {
        let content = self.to_string();
        std::fs::write(path, content).map_err(|e| {
            Error::ConfigError(format!("Failed to write {}: {}", path.display(), e))
        })?;
        Ok(())
    }

    /// Convert config to string representation
    pub fn to_config_string(&self) -> String {
        let mut output = String::new();

        for (section_name, section) in &self.sections {
            output.push_str(&format!("[{}]\n", section_name));

            for (key, value) in &section.values {
                output.push_str(&format!("{} = {}\n", key, value));
            }

            output.push('\n');
        }

        output
    }

    /// Get all cells defined in the config
    pub fn get_cells(&self) -> HashMap<String, String> {
        let mut cells = HashMap::new();

        if let Some(section) = self.sections.get("cells") {
            for (key, value) in &section.values {
                cells.insert(key.clone(), value.clone());
            }
        }

        cells
    }

    /// Get the build execution platform
    pub fn get_execution_platform(&self) -> Option<&str> {
        self.get("build", "execution_platforms")
    }

    /// Get Rust default edition
    pub fn get_rust_edition(&self) -> Option<&str> {
        self.get("rust", "default_edition")
    }
}

impl std::fmt::Display for BuckConfigFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_config_string())
    }
}

impl BuckConfigOptions {
    /// Create new default options
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a configuration override
    pub fn set_override(&mut self, section: &str, key: &str, value: &str) -> &mut Self {
        self.overrides
            .insert(format!("{}.{}", section, key), value.to_string());
        self
    }

    /// Set build mode
    pub fn set_build_mode(&mut self, mode: &str) -> &mut Self {
        self.build_mode = Some(mode.to_string());
        self
    }

    /// Set execution platform
    pub fn set_execution_platform(&mut self, platform: &str) -> &mut Self {
        self.execution_platform = Some(platform.to_string());
        self
    }

    /// Set target platform
    pub fn set_target_platform(&mut self, platform: &str) -> &mut Self {
        self.target_platform = Some(platform.to_string());
        self
    }

    /// Add a modifier
    pub fn add_modifier(&mut self, modifier: &str) -> &mut Self {
        self.modifiers.push(modifier.to_string());
        self
    }

    /// Add a cell override
    pub fn add_cell_override(&mut self, name: &str, path: PathBuf) -> &mut Self {
        self.cell_overrides.insert(name.to_string(), path);
        self
    }

    /// Add a config include
    pub fn add_config_include(&mut self, path: PathBuf) -> &mut Self {
        self.config_includes.push(path);
        self
    }

    /// Convert options to Buck2 command line arguments
    pub fn to_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        // Add config file if specified
        if let Some(config_file) = &self.config_file {
            args.push("--config-file".to_string());
            args.push(config_file.display().to_string());
        }

        // Add config includes
        for include in &self.config_includes {
            args.push("--config-file".to_string());
            args.push(include.display().to_string());
        }

        // Add configuration overrides
        for (key, value) in &self.overrides {
            args.push("--config".to_string());
            args.push(format!("{}={}", key, value));
        }

        // Add build mode
        if let Some(mode) = &self.build_mode {
            args.push("--config".to_string());
            args.push(format!("build.mode={}", mode));
        }

        // Add execution platform
        if let Some(platform) = &self.execution_platform {
            args.push("--config".to_string());
            args.push(format!("build.execution_platforms={}", platform));
        }

        // Add target platform
        if let Some(platform) = &self.target_platform {
            args.push("--target-platforms".to_string());
            args.push(platform.clone());
        }

        // Add modifiers
        for modifier in &self.modifiers {
            args.push("--modifier".to_string());
            args.push(modifier.clone());
        }

        // Add cell overrides
        for (name, path) in &self.cell_overrides {
            args.push("--config".to_string());
            args.push(format!("cells.{}={}", name, path.display()));
        }

        args
    }

    /// Check if any custom options are set
    pub fn has_options(&self) -> bool {
        self.config_file.is_some()
            || !self.overrides.is_empty()
            || !self.cell_overrides.is_empty()
            || self.build_mode.is_some()
            || self.execution_platform.is_some()
            || self.target_platform.is_some()
            || !self.modifiers.is_empty()
            || !self.config_includes.is_empty()
    }

    /// Merge another set of options into this one
    pub fn merge(&mut self, other: &BuckConfigOptions) {
        if other.config_file.is_some() {
            self.config_file = other.config_file.clone();
        }

        for (key, value) in &other.overrides {
            self.overrides.insert(key.clone(), value.clone());
        }

        for (name, path) in &other.cell_overrides {
            self.cell_overrides.insert(name.clone(), path.clone());
        }

        if other.build_mode.is_some() {
            self.build_mode = other.build_mode.clone();
        }

        if other.execution_platform.is_some() {
            self.execution_platform = other.execution_platform.clone();
        }

        if other.target_platform.is_some() {
            self.target_platform = other.target_platform.clone();
        }

        self.modifiers.extend(other.modifiers.clone());
        self.config_includes.extend(other.config_includes.clone());
    }
}

/// Load Buck configuration from the repository
pub fn load_repo_config(repo_path: &Path) -> Result<BuckConfigFile> {
    let config_path = repo_path.join(".buckconfig");
    if config_path.exists() {
        BuckConfigFile::parse(&config_path)
    } else {
        Ok(BuckConfigFile::new())
    }
}

/// Find all .buckconfig files in the repository hierarchy
pub fn find_config_files(start_path: &Path) -> Vec<PathBuf> {
    let mut configs = Vec::new();
    let mut current = start_path;

    loop {
        let config_path = current.join(".buckconfig");
        if config_path.exists() {
            configs.push(config_path);
        }

        match current.parent() {
            Some(parent) => current = parent,
            None => break,
        }
    }

    // Reverse so that root configs come first
    configs.reverse();
    configs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_buckconfig() {
        let content = r#"
[cells]
root = .
prelude = prelude

[build]
execution_platforms = root//platforms:default

[rust]
default_edition = 2021
"#;

        let config = BuckConfigFile::parse_str(content).unwrap();

        assert_eq!(config.get("cells", "root"), Some("."));
        assert_eq!(config.get("cells", "prelude"), Some("prelude"));
        assert_eq!(
            config.get("build", "execution_platforms"),
            Some("root//platforms:default")
        );
        assert_eq!(config.get("rust", "default_edition"), Some("2021"));
    }

    #[test]
    fn test_config_options_to_args() {
        let mut opts = BuckConfigOptions::new();
        opts.set_build_mode("release");
        opts.set_override("cxx", "compiler", "/usr/bin/clang++");
        opts.add_modifier("opt");

        let args = opts.to_args();

        assert!(args.contains(&"--config".to_string()));
        assert!(args.contains(&"build.mode=release".to_string()));
        assert!(args.contains(&"cxx.compiler=/usr/bin/clang++".to_string()));
        assert!(args.contains(&"--modifier".to_string()));
        assert!(args.contains(&"opt".to_string()));
    }

    #[test]
    fn test_merge_configs() {
        let mut config1 = BuckConfigFile::new();
        config1.set("section1", "key1", "value1");
        config1.set("section1", "key2", "value2");

        let mut config2 = BuckConfigFile::new();
        config2.set("section1", "key2", "overridden");
        config2.set("section2", "key3", "value3");

        config1.merge(&config2);

        assert_eq!(config1.get("section1", "key1"), Some("value1"));
        assert_eq!(config1.get("section1", "key2"), Some("overridden"));
        assert_eq!(config1.get("section2", "key3"), Some("value3"));
    }
}
