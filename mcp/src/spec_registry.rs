//! Specification registry module
//!
//! Loads and manages BuckOS specifications from REGISTRY.json

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Specification metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecInfo {
    pub id: String,
    pub title: String,
    pub status: String,
    pub version: String,
    pub category: String,
    pub path: String,
    pub created: String,
    pub updated: String,
    pub authors: Vec<Author>,
    pub maintainers: Vec<String>,
    pub tags: Vec<String>,
    pub related: Vec<String>,
    pub implementation: Implementation,
    pub compatibility: Compatibility,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Implementation {
    pub status: String,
    pub completeness: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Compatibility {
    pub buck2_version: String,
    pub buckos_version: String,
    pub breaking_changes: bool,
}

/// Specification registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecRegistry {
    pub version: String,
    pub generated: String,
    pub total_specs: usize,
    pub specs: Vec<SpecInfo>,
    pub by_category: HashMap<String, Vec<String>>,
    pub by_status: HashMap<String, Vec<String>>,
    pub by_tag: HashMap<String, Vec<String>>,
}

impl SpecRegistry {
    /// Load specification registry from REGISTRY.json
    pub fn load(specs_path: &Path) -> Result<Self, String> {
        let registry_path = specs_path.join("REGISTRY.json");

        if !registry_path.exists() {
            return Err(format!(
                "Specification registry not found at {}",
                registry_path.display()
            ));
        }

        let content = fs::read_to_string(&registry_path)
            .map_err(|e| format!("Failed to read registry: {}", e))?;

        serde_json::from_str(&content).map_err(|e| format!("Failed to parse registry JSON: {}", e))
    }

    /// Get a specification by ID
    pub fn get_spec(&self, spec_id: &str) -> Option<&SpecInfo> {
        self.specs.iter().find(|s| s.id == spec_id)
    }

    /// Get specs by category
    pub fn get_specs_by_category(&self, category: &str) -> Vec<&SpecInfo> {
        if let Some(ids) = self.by_category.get(category) {
            ids.iter().filter_map(|id| self.get_spec(id)).collect()
        } else {
            Vec::new()
        }
    }

    /// Get specs by status
    pub fn get_specs_by_status(&self, status: &str) -> Vec<&SpecInfo> {
        if let Some(ids) = self.by_status.get(status) {
            ids.iter().filter_map(|id| self.get_spec(id)).collect()
        } else {
            Vec::new()
        }
    }

    /// Get specs by tag
    pub fn get_specs_by_tag(&self, tag: &str) -> Vec<&SpecInfo> {
        if let Some(ids) = self.by_tag.get(tag) {
            ids.iter().filter_map(|id| self.get_spec(id)).collect()
        } else {
            Vec::new()
        }
    }

    /// List all specs with optional filters
    pub fn list_specs(&self, category: Option<&str>, status: Option<&str>) -> Vec<&SpecInfo> {
        let mut specs: Vec<&SpecInfo> = self.specs.iter().collect();

        if let Some(cat) = category {
            specs.retain(|s| s.category == cat);
        }

        if let Some(st) = status {
            specs.retain(|s| s.status == st);
        }

        specs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_load_registry() {
        // This test assumes the specs directory is at ../buckos-build/specs/
        let specs_path = PathBuf::from("../../buckos-build/specs");
        if specs_path.exists() {
            let result = SpecRegistry::load(&specs_path);
            assert!(
                result.is_ok(),
                "Failed to load registry: {:?}",
                result.err()
            );

            let registry = result.unwrap();
            assert!(registry.total_specs > 0);
            assert!(!registry.specs.is_empty());
        }
    }
}
