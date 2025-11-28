//! Parser for package_sets.bzl file
//!
//! Reads package sets from the buckos-build repository's package_sets.bzl file
//! and converts Buck targets to package IDs.

use crate::{ConfigError, Result};
use std::collections::HashMap;
use std::path::Path;

/// Package set information
#[derive(Debug, Clone)]
pub struct PackageSetInfo {
    pub name: String,
    pub description: String,
    pub packages: Vec<String>,
    pub inherits: Vec<String>,
}

/// Package sets parsed from package_sets.bzl
#[derive(Debug, Clone)]
pub struct PackageSets {
    /// System packages (from SYSTEM_PACKAGES_GLIBC)
    pub system_packages: Vec<String>,
    /// All package sets (from PACKAGE_SETS)
    pub sets: HashMap<String, PackageSetInfo>,
}

impl PackageSets {
    /// Parse package sets from a bzl file
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| ConfigError::Io(e))?;

        Self::parse(&content)
    }

    /// Parse package sets from bzl file content
    pub fn parse(content: &str) -> Result<Self> {
        let system_packages = Self::parse_system_packages(content)?;
        let sets = Self::parse_package_sets(content)?;

        Ok(Self {
            system_packages,
            sets,
        })
    }

    /// Parse SYSTEM_PACKAGES_GLIBC list
    fn parse_system_packages(content: &str) -> Result<Vec<String>> {
        // Find SYSTEM_PACKAGES_GLIBC = [ ... ]
        let start_marker = "SYSTEM_PACKAGES_GLIBC = [";

        let start_pos = content
            .find(start_marker)
            .ok_or_else(|| ConfigError::Invalid("SYSTEM_PACKAGES_GLIBC not found".to_string()))?;

        // Find the matching closing bracket
        let list_content = &content[start_pos + start_marker.len()..];
        let end_pos = Self::find_matching_bracket(list_content)?;
        let list_str = &list_content[..end_pos];

        Self::parse_string_list(list_str)
    }

    /// Parse PACKAGE_SETS dictionary
    fn parse_package_sets(content: &str) -> Result<HashMap<String, PackageSetInfo>> {
        let mut sets = HashMap::new();

        // Parse each category of sets
        sets.extend(Self::parse_set_category(content, "PROFILE_PACKAGE_SETS")?);
        sets.extend(Self::parse_set_category(content, "TASK_PACKAGE_SETS")?);
        sets.extend(Self::parse_set_category(content, "INIT_SYSTEM_SETS")?);
        sets.extend(Self::parse_set_category(
            content,
            "DESKTOP_ENVIRONMENT_SETS",
        )?);

        Ok(sets)
    }

    /// Parse a set category (e.g., PROFILE_PACKAGE_SETS)
    fn parse_set_category(
        content: &str,
        category: &str,
    ) -> Result<HashMap<String, PackageSetInfo>> {
        let start_marker = format!("{} = {{", category);

        let start_pos = match content.find(&start_marker) {
            Some(pos) => pos,
            None => return Ok(HashMap::new()), // Category not found, return empty
        };

        let dict_content = &content[start_pos + start_marker.len()..];
        let end_pos = Self::find_matching_brace(dict_content)?;
        let dict_str = &dict_content[..end_pos];

        Self::parse_set_dict(dict_str)
    }

    /// Parse a dictionary of sets
    fn parse_set_dict(dict_str: &str) -> Result<HashMap<String, PackageSetInfo>> {
        let mut sets = HashMap::new();

        // Split by set entries (look for "name": {)
        let mut current_pos = 0;

        while let Some(quote_pos) = dict_str[current_pos..].find('"') {
            let abs_quote_pos = current_pos + quote_pos;

            // Extract set name
            let name_start = abs_quote_pos + 1;
            let name_end = match dict_str[name_start..].find('"') {
                Some(pos) => name_start + pos,
                None => break,
            };
            let set_name = dict_str[name_start..name_end].to_string();

            // Find the opening brace for this set
            let brace_pos = match dict_str[name_end..].find('{') {
                Some(pos) => name_end + pos + 1,
                None => break,
            };

            // Find the matching closing brace
            let set_content = &dict_str[brace_pos..];
            let set_end = match Self::find_matching_brace(set_content) {
                Ok(pos) => pos,
                Err(_) => break,
            };
            let set_str = &set_content[..set_end];

            // Parse the set
            if let Ok(set_info) = Self::parse_set_entry(&set_name, set_str) {
                sets.insert(set_name, set_info);
            }

            current_pos = brace_pos + set_end + 1;
        }

        Ok(sets)
    }

    /// Parse a single set entry
    fn parse_set_entry(name: &str, set_str: &str) -> Result<PackageSetInfo> {
        let description = Self::extract_string_field(set_str, "description")?;
        let packages = Self::extract_list_field(set_str, "packages")?;
        let inherits = Self::extract_list_field(set_str, "inherits").unwrap_or_default();

        Ok(PackageSetInfo {
            name: name.to_string(),
            description,
            packages,
            inherits,
        })
    }

    /// Extract a string field from a dict
    fn extract_string_field(content: &str, field_name: &str) -> Result<String> {
        let pattern = format!(r#""{}":\s*""#, field_name);

        let start_pos = content
            .find(&pattern)
            .ok_or_else(|| ConfigError::Invalid(format!("Field '{}' not found", field_name)))?;

        let value_start = start_pos + pattern.len();
        let quote_end = content[value_start..].find('"').ok_or_else(|| {
            ConfigError::Invalid(format!("Unterminated string for field '{}'", field_name))
        })?;

        Ok(content[value_start..value_start + quote_end].to_string())
    }

    /// Extract a list field from a dict
    fn extract_list_field(content: &str, field_name: &str) -> Result<Vec<String>> {
        let pattern = format!(r#""{}":\s*\["#, field_name);

        let start_pos = match content.find(&pattern) {
            Some(pos) => pos,
            None => return Ok(Vec::new()), // Field not found, return empty list
        };

        let list_start = start_pos + content[start_pos..].find('[').unwrap() + 1;
        let list_content = &content[list_start..];
        let list_end = Self::find_matching_bracket(list_content)?;
        let list_str = &list_content[..list_end];

        Self::parse_string_list(list_str)
    }

    /// Parse a list of strings
    fn parse_string_list(list_str: &str) -> Result<Vec<String>> {
        let mut items = Vec::new();

        for line in list_str.lines() {
            let trimmed = line.trim();

            // Skip comments and empty lines
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Extract string literals
            if let Some(quote_start) = trimmed.find('"') {
                if let Some(quote_end) = trimmed[quote_start + 1..].find('"') {
                    let value = &trimmed[quote_start + 1..quote_start + 1 + quote_end];

                    // Convert Buck target to package ID
                    let package_id = Self::buck_target_to_package_id(value);
                    items.push(package_id);
                }
            }
        }

        Ok(items)
    }

    /// Convert Buck target to package ID
    /// Example: "//packages/linux/core:bash" -> "core/bash"
    fn buck_target_to_package_id(target: &str) -> String {
        // Handle @ prefix (set references)
        if target.starts_with('@') {
            return target.to_string();
        }

        // Buck target format: //packages/linux/CATEGORY:NAME
        // or: //packages/linux/CATEGORY/SUBCATEGORY:NAME

        if !target.starts_with("//packages/linux/") {
            // Not a standard Buck target, return as-is
            return target.to_string();
        }

        let path_part = &target["//packages/linux/".len()..];

        // Split by :
        let parts: Vec<&str> = path_part.split(':').collect();
        if parts.len() != 2 {
            // No target name, use directory name
            return path_part.to_string();
        }

        let category = parts[0];
        let name = parts[1];

        // If name matches the last directory component, just use category path
        // e.g., //packages/linux/core:bash -> core/bash (category="core", name="bash", doesn't end with /bash)
        // e.g., //packages/linux/system/apps:coreutils -> system/apps/coreutils
        // e.g., //packages/linux/system/apps/shadow:shadow -> system/apps/shadow (category="system/apps/shadow", ends with /shadow, so return as-is)

        if category.ends_with(&format!("/{}", name)) {
            // The last directory component matches the target name, so use category as-is
            return category.to_string();
        }

        format!("{}/{}", category, name)
    }

    /// Find matching closing bracket ]
    fn find_matching_bracket(content: &str) -> Result<usize> {
        let mut depth = 1;
        let mut in_string = false;
        let mut escape = false;

        for (i, ch) in content.chars().enumerate() {
            if escape {
                escape = false;
                continue;
            }

            match ch {
                '\\' if in_string => escape = true,
                '"' => in_string = !in_string,
                '[' if !in_string => depth += 1,
                ']' if !in_string => {
                    depth -= 1;
                    if depth == 0 {
                        return Ok(i);
                    }
                }
                _ => {}
            }
        }

        Err(ConfigError::Invalid("Unmatched bracket".to_string()))
    }

    /// Find matching closing brace }
    fn find_matching_brace(content: &str) -> Result<usize> {
        let mut depth = 1;
        let mut in_string = false;
        let mut escape = false;

        for (i, ch) in content.chars().enumerate() {
            if escape {
                escape = false;
                continue;
            }

            match ch {
                '\\' if in_string => escape = true,
                '"' => in_string = !in_string,
                '{' if !in_string => depth += 1,
                '}' if !in_string => {
                    depth -= 1;
                    if depth == 0 {
                        return Ok(i);
                    }
                }
                _ => {}
            }
        }

        Err(ConfigError::Invalid("Unmatched brace".to_string()))
    }

    /// Get system packages
    pub fn get_system_packages(&self) -> &[String] {
        &self.system_packages
    }

    /// Get a package set by name
    pub fn get_set(&self, name: &str) -> Option<&PackageSetInfo> {
        // Handle @ prefix
        let name = name.strip_prefix('@').unwrap_or(name);
        self.sets.get(name)
    }

    /// Resolve a package set including inherited sets
    pub fn resolve_set(&self, name: &str) -> Vec<String> {
        let mut packages = Vec::new();
        let mut visited = std::collections::HashSet::new();
        self.resolve_set_recursive(name, &mut packages, &mut visited);
        packages
    }

    fn resolve_set_recursive(
        &self,
        name: &str,
        packages: &mut Vec<String>,
        visited: &mut std::collections::HashSet<String>,
    ) {
        let name = name.strip_prefix('@').unwrap_or(name);

        if visited.contains(name) {
            return; // Avoid cycles
        }
        visited.insert(name.to_string());

        if let Some(set) = self.sets.get(name) {
            // First resolve inherited sets
            for inherited in &set.inherits {
                self.resolve_set_recursive(inherited, packages, visited);
            }

            // Then add this set's packages
            for pkg in &set.packages {
                if !packages.contains(pkg) {
                    packages.push(pkg.clone());
                }
            }
        }
    }

    /// List all available set names
    pub fn list_sets(&self) -> Vec<String> {
        let mut names: Vec<String> = self.sets.keys().cloned().collect();
        names.sort();
        names
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buck_target_conversion() {
        // Simple target where name doesn't match directory
        assert_eq!(
            PackageSets::buck_target_to_package_id("//packages/linux/core:bash"),
            "core/bash"
        );

        // Target where name doesn't match last directory component
        assert_eq!(
            PackageSets::buck_target_to_package_id("//packages/linux/system/apps:coreutils"),
            "system/apps/coreutils"
        );

        // Target where name DOES match last directory component - should keep full path
        assert_eq!(
            PackageSets::buck_target_to_package_id("//packages/linux/system/apps/shadow:shadow"),
            "system/apps/shadow"
        );

        // Set reference
        assert_eq!(PackageSets::buck_target_to_package_id("@world"), "@world");

        // Verify the fix: these should all be different
        let bash = PackageSets::buck_target_to_package_id("//packages/linux/core:bash");
        let shadow =
            PackageSets::buck_target_to_package_id("//packages/linux/system/apps/shadow:shadow");
        let coreutils =
            PackageSets::buck_target_to_package_id("//packages/linux/system/apps:coreutils");

        assert_eq!(bash, "core/bash");
        assert_eq!(shadow, "system/apps/shadow");
        assert_eq!(coreutils, "system/apps/coreutils");
    }

    #[test]
    fn test_parse_string_list() {
        let list_str = r#"
            # Comment
            "//packages/linux/core:bash",
            "//packages/linux/core:zlib",  # inline comment

            "//packages/linux/system/apps:coreutils",
        "#;

        let items = PackageSets::parse_string_list(list_str).unwrap();

        assert_eq!(items.len(), 3);
        assert_eq!(items[0], "core/bash");
        assert_eq!(items[1], "core/zlib");
        assert_eq!(items[2], "system/apps/coreutils");
    }
}
