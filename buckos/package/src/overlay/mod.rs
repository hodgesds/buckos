//! Overlay support for additional package repositories
//!
//! This module provides functionality similar to Gentoo's layman and
//! eselect-repository for managing overlay repositories.
//!
//! # Features
//!
//! - Add/remove overlay repositories
//! - Enable/disable overlays
//! - Repository priorities
//! - Local overlays
//! - Remote overlay sources (git, rsync, http)

use crate::config::{RepositoryConfig, SyncType};
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// Configuration for overlays
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayConfig {
    /// Path to overlay configuration file
    pub config_path: PathBuf,
    /// Path to overlay list file
    pub list_path: PathBuf,
    /// Default overlay storage directory
    pub storage_dir: PathBuf,
    /// Remote overlay list URLs
    pub remote_lists: Vec<String>,
}

impl Default for OverlayConfig {
    fn default() -> Self {
        Self {
            config_path: PathBuf::from("/etc/buckos/overlays.toml"),
            list_path: PathBuf::from("/var/db/buckos/overlays.json"),
            storage_dir: PathBuf::from("/var/db/repos"),
            remote_lists: vec![
                "https://api.gentoo.org/overlays/repositories.xml".to_string(),
            ],
        }
    }
}

/// Information about an overlay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayInfo {
    /// Unique overlay name
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Overlay homepage or documentation URL
    pub homepage: Option<String>,
    /// Repository sync type
    pub sync_type: SyncType,
    /// Repository URI for syncing
    pub sync_uri: String,
    /// Local path where overlay is stored
    pub location: PathBuf,
    /// Priority (higher = preferred)
    pub priority: i32,
    /// Whether overlay is currently enabled
    pub enabled: bool,
    /// Whether this is a local (user-defined) overlay
    pub is_local: bool,
    /// Owner/maintainer information
    pub owner: Option<String>,
    /// Quality rating (official, community, experimental)
    pub quality: OverlayQuality,
    /// Master repository (for inheriting eclasses)
    pub masters: Vec<String>,
    /// Whether overlay should auto-sync
    pub auto_sync: bool,
    /// Last sync timestamp
    pub last_sync: Option<u64>,
}

/// Quality classification for overlays
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OverlayQuality {
    /// Official repository
    Official,
    /// Community-maintained
    Community,
    /// Experimental or testing
    Experimental,
    /// User-created local overlay
    Local,
}

impl std::fmt::Display for OverlayQuality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OverlayQuality::Official => write!(f, "official"),
            OverlayQuality::Community => write!(f, "community"),
            OverlayQuality::Experimental => write!(f, "experimental"),
            OverlayQuality::Local => write!(f, "local"),
        }
    }
}

/// Overlay management state
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OverlayState {
    /// All known overlays
    pub overlays: HashMap<String, OverlayInfo>,
    /// Enabled overlay names in priority order
    pub enabled: Vec<String>,
}

/// Manager for overlay operations
pub struct OverlayManager {
    /// Configuration
    config: OverlayConfig,
    /// Current state
    state: OverlayState,
}

impl OverlayManager {
    /// Create a new overlay manager
    pub fn new(config: OverlayConfig) -> Result<Self> {
        let state = Self::load_state(&config.list_path)?;

        Ok(Self { config, state })
    }

    /// Create with default configuration
    pub fn with_defaults() -> Result<Self> {
        Self::new(OverlayConfig::default())
    }

    /// Load state from file
    fn load_state(path: &Path) -> Result<OverlayState> {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            let state: OverlayState = serde_json::from_str(&content)?;
            Ok(state)
        } else {
            Ok(OverlayState::default())
        }
    }

    /// Save state to file
    fn save_state(&self) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.config.list_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(&self.state)?;
        std::fs::write(&self.config.list_path, content)?;

        Ok(())
    }

    /// List all known overlays
    pub fn list_all(&self) -> Vec<&OverlayInfo> {
        self.state.overlays.values().collect()
    }

    /// List enabled overlays in priority order
    pub fn list_enabled(&self) -> Vec<&OverlayInfo> {
        self.state
            .enabled
            .iter()
            .filter_map(|name| self.state.overlays.get(name))
            .collect()
    }

    /// List available (not enabled) overlays
    pub fn list_available(&self) -> Vec<&OverlayInfo> {
        self.state
            .overlays
            .values()
            .filter(|o| !o.enabled)
            .collect()
    }

    /// Get information about a specific overlay
    pub fn get_info(&self, name: &str) -> Option<&OverlayInfo> {
        self.state.overlays.get(name)
    }

    /// Add a new overlay
    pub fn add(&mut self, overlay: OverlayInfo) -> Result<()> {
        if self.state.overlays.contains_key(&overlay.name) {
            return Err(Error::OverlayAlreadyExists(overlay.name));
        }

        info!("Adding overlay: {}", overlay.name);

        // Create the overlay directory if it doesn't exist
        if !overlay.location.exists() {
            std::fs::create_dir_all(&overlay.location)?;
        }

        self.state
            .overlays
            .insert(overlay.name.clone(), overlay);
        self.save_state()?;

        Ok(())
    }

    /// Add a local overlay
    pub fn add_local(
        &mut self,
        name: &str,
        location: &Path,
        priority: i32,
    ) -> Result<()> {
        let overlay = OverlayInfo {
            name: name.to_string(),
            description: format!("Local overlay: {}", name),
            homepage: None,
            sync_type: SyncType::Local,
            sync_uri: location.to_string_lossy().to_string(),
            location: location.to_path_buf(),
            priority,
            enabled: false,
            is_local: true,
            owner: None,
            quality: OverlayQuality::Local,
            masters: vec!["buckos".to_string()],
            auto_sync: false,
            last_sync: None,
        };

        self.add(overlay)
    }

    /// Add a remote overlay (git, rsync, http)
    pub fn add_remote(
        &mut self,
        name: &str,
        sync_uri: &str,
        sync_type: SyncType,
        priority: i32,
    ) -> Result<()> {
        let location = self.config.storage_dir.join(name);

        let overlay = OverlayInfo {
            name: name.to_string(),
            description: format!("Remote overlay: {}", name),
            homepage: None,
            sync_type,
            sync_uri: sync_uri.to_string(),
            location,
            priority,
            enabled: false,
            is_local: false,
            owner: None,
            quality: OverlayQuality::Community,
            masters: vec!["buckos".to_string()],
            auto_sync: true,
            last_sync: None,
        };

        self.add(overlay)
    }

    /// Remove an overlay
    pub fn remove(&mut self, name: &str, delete_files: bool) -> Result<()> {
        let overlay = self
            .state
            .overlays
            .get(name)
            .ok_or_else(|| Error::OverlayNotFound(name.to_string()))?;

        info!("Removing overlay: {}", name);

        // Get necessary info before mutable operations
        let was_enabled = overlay.enabled;
        let location = overlay.location.clone();

        // Disable first if enabled
        if was_enabled {
            self.disable(name)?;
        }

        // Optionally delete the overlay files
        if delete_files && location.exists() {
            warn!("Deleting overlay files at: {:?}", location);
            std::fs::remove_dir_all(&location)?;
        }

        self.state.overlays.remove(name);
        self.save_state()?;

        Ok(())
    }

    /// Enable an overlay
    pub fn enable(&mut self, name: &str) -> Result<()> {
        let overlay = self
            .state
            .overlays
            .get_mut(name)
            .ok_or_else(|| Error::OverlayNotFound(name.to_string()))?;

        if overlay.enabled {
            return Ok(());
        }

        info!("Enabling overlay: {}", name);

        overlay.enabled = true;

        // Add to enabled list in priority order
        let priority = overlay.priority;
        let insert_pos = self
            .state
            .enabled
            .iter()
            .position(|n| {
                self.state
                    .overlays
                    .get(n)
                    .map(|o| o.priority < priority)
                    .unwrap_or(false)
            })
            .unwrap_or(self.state.enabled.len());

        self.state.enabled.insert(insert_pos, name.to_string());
        self.save_state()?;

        Ok(())
    }

    /// Disable an overlay
    pub fn disable(&mut self, name: &str) -> Result<()> {
        let overlay = self
            .state
            .overlays
            .get_mut(name)
            .ok_or_else(|| Error::OverlayNotFound(name.to_string()))?;

        if !overlay.enabled {
            return Ok(());
        }

        info!("Disabling overlay: {}", name);

        overlay.enabled = false;
        self.state.enabled.retain(|n| n != name);
        self.save_state()?;

        Ok(())
    }

    /// Set overlay priority
    pub fn set_priority(&mut self, name: &str, priority: i32) -> Result<()> {
        let overlay = self
            .state
            .overlays
            .get_mut(name)
            .ok_or_else(|| Error::OverlayNotFound(name.to_string()))?;

        info!("Setting priority for {}: {}", name, priority);

        let was_enabled = overlay.enabled;
        overlay.priority = priority;

        // Re-sort enabled list if this overlay is enabled
        if was_enabled {
            self.state.enabled.retain(|n| n != name);
            let insert_pos = self
                .state
                .enabled
                .iter()
                .position(|n| {
                    self.state
                        .overlays
                        .get(n)
                        .map(|o| o.priority < priority)
                        .unwrap_or(false)
                })
                .unwrap_or(self.state.enabled.len());

            self.state.enabled.insert(insert_pos, name.to_string());
        }

        self.save_state()?;

        Ok(())
    }

    /// Sync an overlay
    pub async fn sync(&mut self, name: &str) -> Result<()> {
        let overlay = self
            .state
            .overlays
            .get(name)
            .ok_or_else(|| Error::OverlayNotFound(name.to_string()))?
            .clone();

        info!("Syncing overlay: {}", name);

        match overlay.sync_type {
            SyncType::Git => self.sync_git(&overlay).await?,
            SyncType::Rsync => self.sync_rsync(&overlay).await?,
            SyncType::Http => self.sync_http(&overlay).await?,
            SyncType::Local => {
                debug!("Local overlay {} does not need syncing", name);
            }
        }

        // Update last sync time
        if let Some(o) = self.state.overlays.get_mut(name) {
            o.last_sync = Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            );
        }
        self.save_state()?;

        Ok(())
    }

    /// Sync all enabled overlays
    pub async fn sync_all(&mut self) -> Result<()> {
        let enabled: Vec<String> = self.state.enabled.clone();

        for name in enabled {
            if let Some(overlay) = self.state.overlays.get(&name) {
                if overlay.auto_sync {
                    self.sync(&name).await?;
                }
            }
        }

        Ok(())
    }

    /// Sync using git
    async fn sync_git(&self, overlay: &OverlayInfo) -> Result<()> {
        if overlay.location.exists() && overlay.location.join(".git").exists() {
            // Pull existing repository
            let output = tokio::process::Command::new("git")
                .args(["-C", &overlay.location.to_string_lossy(), "pull"])
                .output()
                .await?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(Error::OverlaySyncFailed {
                    name: overlay.name.clone(),
                    reason: stderr.to_string(),
                });
            }
        } else {
            // Clone new repository
            std::fs::create_dir_all(&overlay.location)?;

            let output = tokio::process::Command::new("git")
                .args([
                    "clone",
                    &overlay.sync_uri,
                    &overlay.location.to_string_lossy(),
                ])
                .output()
                .await?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(Error::OverlaySyncFailed {
                    name: overlay.name.clone(),
                    reason: stderr.to_string(),
                });
            }
        }

        Ok(())
    }

    /// Sync using rsync
    async fn sync_rsync(&self, overlay: &OverlayInfo) -> Result<()> {
        std::fs::create_dir_all(&overlay.location)?;

        let output = tokio::process::Command::new("rsync")
            .args([
                "-av",
                "--delete",
                &overlay.sync_uri,
                &overlay.location.to_string_lossy(),
            ])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::OverlaySyncFailed {
                name: overlay.name.clone(),
                reason: stderr.to_string(),
            });
        }

        Ok(())
    }

    /// Sync using HTTP (download and extract)
    async fn sync_http(&self, overlay: &OverlayInfo) -> Result<()> {
        std::fs::create_dir_all(&overlay.location)?;

        // Download the tarball
        let client = reqwest::Client::new();
        let response = client.get(&overlay.sync_uri).send().await?;

        if !response.status().is_success() {
            return Err(Error::OverlaySyncFailed {
                name: overlay.name.clone(),
                reason: format!("HTTP error: {}", response.status()),
            });
        }

        let bytes = response.bytes().await?;

        // Extract to location
        let tar_gz = std::io::Cursor::new(bytes);
        let tar = flate2::read::GzDecoder::new(tar_gz);
        let mut archive = tar::Archive::new(tar);

        archive.unpack(&overlay.location)?;

        Ok(())
    }

    /// Convert enabled overlays to repository configs
    pub fn to_repository_configs(&self) -> Vec<RepositoryConfig> {
        self.list_enabled()
            .iter()
            .map(|overlay| RepositoryConfig {
                name: overlay.name.clone(),
                location: overlay.location.clone(),
                sync_type: overlay.sync_type.clone(),
                sync_uri: overlay.sync_uri.clone(),
                priority: overlay.priority,
                auto_sync: overlay.auto_sync,
            })
            .collect()
    }

    /// Fetch remote overlay list
    pub async fn fetch_remote_list(&mut self) -> Result<Vec<OverlayInfo>> {
        let mut all_overlays = Vec::new();

        for url in &self.config.remote_lists.clone() {
            match self.fetch_overlay_list_from_url(url).await {
                Ok(overlays) => {
                    all_overlays.extend(overlays);
                }
                Err(e) => {
                    warn!("Failed to fetch overlay list from {}: {}", url, e);
                }
            }
        }

        // Merge with existing state
        for overlay in &all_overlays {
            if !self.state.overlays.contains_key(&overlay.name) {
                self.state
                    .overlays
                    .insert(overlay.name.clone(), overlay.clone());
            }
        }
        self.save_state()?;

        Ok(all_overlays)
    }

    /// Fetch overlay list from a specific URL
    async fn fetch_overlay_list_from_url(&self, _url: &str) -> Result<Vec<OverlayInfo>> {
        // TODO: Parse different formats (XML for Gentoo, JSON for others)
        // For now, return an empty list as placeholder
        Ok(Vec::new())
    }

    /// Search overlays by name or description
    pub fn search(&self, query: &str) -> Vec<&OverlayInfo> {
        let query_lower = query.to_lowercase();

        self.state
            .overlays
            .values()
            .filter(|o| {
                o.name.to_lowercase().contains(&query_lower)
                    || o.description.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    /// Update overlay information
    pub fn update_info(&mut self, name: &str, info: OverlayInfo) -> Result<()> {
        if !self.state.overlays.contains_key(name) {
            return Err(Error::OverlayNotFound(name.to_string()));
        }

        self.state.overlays.insert(name.to_string(), info);
        self.save_state()?;

        Ok(())
    }

    /// Get the configuration
    pub fn config(&self) -> &OverlayConfig {
        &self.config
    }

    /// Check if an overlay exists
    pub fn exists(&self, name: &str) -> bool {
        self.state.overlays.contains_key(name)
    }

    /// Check if an overlay is enabled
    pub fn is_enabled(&self, name: &str) -> bool {
        self.state
            .overlays
            .get(name)
            .map(|o| o.enabled)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_manager() -> (OverlayManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();

        let config = OverlayConfig {
            config_path: temp_dir.path().join("overlays.toml"),
            list_path: temp_dir.path().join("overlays.json"),
            storage_dir: temp_dir.path().join("repos"),
            remote_lists: vec![],
        };

        let manager = OverlayManager::new(config).unwrap();
        (manager, temp_dir)
    }

    #[test]
    fn test_add_local_overlay() {
        let (mut manager, temp_dir) = setup_test_manager();

        let local_path = temp_dir.path().join("my-overlay");
        std::fs::create_dir(&local_path).unwrap();

        manager
            .add_local("test-overlay", &local_path, 50)
            .unwrap();

        assert!(manager.exists("test-overlay"));
        assert!(!manager.is_enabled("test-overlay"));

        let info = manager.get_info("test-overlay").unwrap();
        assert_eq!(info.name, "test-overlay");
        assert_eq!(info.priority, 50);
        assert!(info.is_local);
    }

    #[test]
    fn test_enable_disable_overlay() {
        let (mut manager, temp_dir) = setup_test_manager();

        let local_path = temp_dir.path().join("my-overlay");
        std::fs::create_dir(&local_path).unwrap();

        manager
            .add_local("test-overlay", &local_path, 50)
            .unwrap();

        // Enable
        manager.enable("test-overlay").unwrap();
        assert!(manager.is_enabled("test-overlay"));

        let enabled = manager.list_enabled();
        assert_eq!(enabled.len(), 1);
        assert_eq!(enabled[0].name, "test-overlay");

        // Disable
        manager.disable("test-overlay").unwrap();
        assert!(!manager.is_enabled("test-overlay"));

        let enabled = manager.list_enabled();
        assert!(enabled.is_empty());
    }

    #[test]
    fn test_priority_order() {
        let (mut manager, temp_dir) = setup_test_manager();

        // Create overlays with different priorities
        for (name, priority) in [("low", 10), ("high", 100), ("medium", 50)] {
            let path = temp_dir.path().join(name);
            std::fs::create_dir(&path).unwrap();
            manager.add_local(name, &path, priority).unwrap();
            manager.enable(name).unwrap();
        }

        let enabled = manager.list_enabled();
        assert_eq!(enabled.len(), 3);
        assert_eq!(enabled[0].name, "high");
        assert_eq!(enabled[1].name, "medium");
        assert_eq!(enabled[2].name, "low");
    }

    #[test]
    fn test_remove_overlay() {
        let (mut manager, temp_dir) = setup_test_manager();

        let local_path = temp_dir.path().join("my-overlay");
        std::fs::create_dir(&local_path).unwrap();

        manager
            .add_local("test-overlay", &local_path, 50)
            .unwrap();
        manager.enable("test-overlay").unwrap();

        manager.remove("test-overlay", false).unwrap();

        assert!(!manager.exists("test-overlay"));
        assert!(manager.list_enabled().is_empty());
    }

    #[test]
    fn test_set_priority() {
        let (mut manager, temp_dir) = setup_test_manager();

        // Create two overlays
        for (name, priority) in [("first", 10), ("second", 50)] {
            let path = temp_dir.path().join(name);
            std::fs::create_dir(&path).unwrap();
            manager.add_local(name, &path, priority).unwrap();
            manager.enable(name).unwrap();
        }

        // Change priority of "first" to be higher than "second"
        manager.set_priority("first", 100).unwrap();

        let enabled = manager.list_enabled();
        assert_eq!(enabled[0].name, "first");
        assert_eq!(enabled[1].name, "second");
    }

    #[test]
    fn test_search() {
        let (mut manager, temp_dir) = setup_test_manager();

        let local_path = temp_dir.path().join("my-overlay");
        std::fs::create_dir(&local_path).unwrap();

        let mut overlay = OverlayInfo {
            name: "rust-overlay".to_string(),
            description: "Rust programming language packages".to_string(),
            homepage: None,
            sync_type: SyncType::Local,
            sync_uri: local_path.to_string_lossy().to_string(),
            location: local_path.clone(),
            priority: 50,
            enabled: false,
            is_local: true,
            owner: None,
            quality: OverlayQuality::Local,
            masters: vec![],
            auto_sync: false,
            last_sync: None,
        };

        manager.add(overlay.clone()).unwrap();

        overlay.name = "python-overlay".to_string();
        overlay.description = "Python programming language packages".to_string();
        manager.add(overlay).unwrap();

        let results = manager.search("rust");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "rust-overlay");

        let results = manager.search("programming");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_to_repository_configs() {
        let (mut manager, temp_dir) = setup_test_manager();

        let local_path = temp_dir.path().join("my-overlay");
        std::fs::create_dir(&local_path).unwrap();

        manager
            .add_local("test-overlay", &local_path, 50)
            .unwrap();
        manager.enable("test-overlay").unwrap();

        let configs = manager.to_repository_configs();
        assert_eq!(configs.len(), 1);
        assert_eq!(configs[0].name, "test-overlay");
        assert_eq!(configs[0].priority, 50);
    }
}
