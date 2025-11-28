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
            remote_lists: vec!["https://api.gentoo.org/overlays/repositories.xml".to_string()],
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

        self.state.overlays.insert(overlay.name.clone(), overlay);
        self.save_state()?;

        Ok(())
    }

    /// Add a local overlay
    pub fn add_local(&mut self, name: &str, location: &Path, priority: i32) -> Result<()> {
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
            SyncType::Mercurial => {
                warn!("Mercurial sync not yet implemented for overlay {}", name);
            }
            SyncType::Svn => {
                warn!("SVN sync not yet implemented for overlay {}", name);
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
    ///
    /// Supports multiple formats:
    /// - Gentoo repositories.xml format
    /// - JSON format (for custom overlay lists)
    async fn fetch_overlay_list_from_url(&self, url: &str) -> Result<Vec<OverlayInfo>> {
        // Fetch the content
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| Error::NetworkError(format!("Failed to create HTTP client: {}", e)))?;

        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| Error::NetworkError(format!("Failed to fetch {}: {}", url, e)))?;

        if !response.status().is_success() {
            return Err(Error::NetworkError(format!(
                "Failed to fetch {}: HTTP {}",
                url,
                response.status()
            )));
        }

        let content = response
            .text()
            .await
            .map_err(|e| Error::NetworkError(format!("Failed to read response: {}", e)))?;

        // Determine format and parse
        if url.ends_with(".xml")
            || content.trim().starts_with("<?xml")
            || content.contains("<repositories")
        {
            self.parse_gentoo_xml(&content)
        } else if url.ends_with(".json")
            || content.trim().starts_with('{')
            || content.trim().starts_with('[')
        {
            self.parse_json_overlay_list(&content)
        } else {
            // Try XML first, then JSON
            self.parse_gentoo_xml(&content)
                .or_else(|_| self.parse_json_overlay_list(&content))
        }
    }

    /// Parse Gentoo repositories.xml format
    ///
    /// Format example:
    /// ```xml
    /// <repositories>
    ///   <repo quality="experimental" status="unofficial">
    ///     <name>overlay-name</name>
    ///     <description>Description text</description>
    ///     <homepage>https://example.com</homepage>
    ///     <owner><email>owner@example.com</email></owner>
    ///     <source type="git">https://github.com/example/overlay.git</source>
    ///   </repo>
    /// </repositories>
    /// ```
    fn parse_gentoo_xml(&self, content: &str) -> Result<Vec<OverlayInfo>> {
        let mut overlays = Vec::new();

        // Find all <repo> elements
        let mut pos = 0;
        while let Some(repo_start) = content[pos..].find("<repo") {
            let repo_start = pos + repo_start;
            let repo_end = match content[repo_start..].find("</repo>") {
                Some(end) => repo_start + end + "</repo>".len(),
                None => break,
            };
            let repo_content = &content[repo_start..repo_end];

            // Extract repo attributes
            let quality = self
                .extract_xml_attr(repo_content, "repo", "quality")
                .unwrap_or_else(|| "experimental".to_string());

            // Extract name
            let name = match self.extract_xml_element_text(repo_content, "name") {
                Some(n) => n,
                None => {
                    pos = repo_end;
                    continue;
                }
            };

            // Extract description
            let description = self
                .extract_xml_element_text(repo_content, "description")
                .unwrap_or_default();

            // Extract homepage
            let homepage = self.extract_xml_element_text(repo_content, "homepage");

            // Extract owner email
            let owner = self.extract_xml_element_text(repo_content, "email");

            // Extract source (sync URI and type)
            let (sync_type, sync_uri) = self.extract_source_info(repo_content);

            // Determine quality enum
            let quality_enum = match quality.to_lowercase().as_str() {
                "core" | "official" => OverlayQuality::Official,
                "stable" | "community" => OverlayQuality::Community,
                _ => OverlayQuality::Experimental,
            };

            overlays.push(OverlayInfo {
                name: name.clone(),
                description,
                homepage,
                sync_type,
                sync_uri,
                location: self.config.storage_dir.join(&name),
                priority: 50, // Default priority for remote overlays
                enabled: false,
                is_local: false,
                owner,
                quality: quality_enum,
                masters: vec!["gentoo".to_string()],
                auto_sync: true,
                last_sync: None,
            });

            pos = repo_end;
        }

        debug!("Parsed {} overlays from XML", overlays.len());
        Ok(overlays)
    }

    /// Extract text content of an XML element
    fn extract_xml_element_text(&self, content: &str, element: &str) -> Option<String> {
        let start_tag = format!("<{}", element);
        let end_tag = format!("</{}>", element);

        let start_pos = content.find(&start_tag)?;
        let tag_end = content[start_pos..].find('>')? + start_pos + 1;
        let end_pos = content[tag_end..].find(&end_tag)? + tag_end;

        let text = &content[tag_end..end_pos];
        // Clean up whitespace and decode XML entities
        let cleaned = text
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&")
            .replace("&quot;", "\"")
            .replace("&apos;", "'")
            .trim()
            .to_string();

        if cleaned.is_empty() {
            None
        } else {
            Some(cleaned)
        }
    }

    /// Extract XML attribute value
    fn extract_xml_attr(&self, content: &str, element: &str, attr: &str) -> Option<String> {
        let start_tag = format!("<{}", element);
        let start_pos = content.find(&start_tag)?;
        let tag_content = &content[start_pos..];
        let tag_end = tag_content.find('>')?;
        let tag_str = &tag_content[..tag_end];

        // Look for attr="value" or attr='value'
        let patterns = [format!("{}=\"", attr), format!("{}='", attr)];
        let delimiters = ['"', '\''];

        for (pattern, delimiter) in patterns.iter().zip(delimiters.iter()) {
            if let Some(attr_start) = tag_str.find(pattern.as_str()) {
                let value_start = attr_start + pattern.len();
                if let Some(value_end) = tag_str[value_start..].find(*delimiter) {
                    return Some(tag_str[value_start..value_start + value_end].to_string());
                }
            }
        }

        None
    }

    /// Extract source information (sync type and URI)
    fn extract_source_info(&self, content: &str) -> (SyncType, String) {
        // Look for <source type="git">uri</source>
        if let Some(source_start) = content.find("<source") {
            let source_end = content[source_start..]
                .find("</source>")
                .map(|e| source_start + e);

            if let Some(end) = source_end {
                let source_content = &content[source_start..end];

                // Extract type attribute
                let sync_type = self
                    .extract_xml_attr(source_content, "source", "type")
                    .map(|t| match t.to_lowercase().as_str() {
                        "git" => SyncType::Git,
                        "rsync" => SyncType::Rsync,
                        "mercurial" | "hg" => SyncType::Mercurial,
                        "svn" | "subversion" => SyncType::Svn,
                        _ => SyncType::Git,
                    })
                    .unwrap_or(SyncType::Git);

                // Extract URI (text content)
                let tag_end = source_content.find('>').unwrap_or(0) + 1;
                let uri = source_content[tag_end..].trim().to_string();

                return (sync_type, uri);
            }
        }

        (SyncType::Git, String::new())
    }

    /// Parse JSON overlay list format
    ///
    /// Expected format:
    /// ```json
    /// {
    ///   "overlays": [
    ///     {
    ///       "name": "overlay-name",
    ///       "description": "Description",
    ///       "homepage": "https://example.com",
    ///       "sync_type": "git",
    ///       "sync_uri": "https://github.com/example/overlay.git",
    ///       "owner": "owner@example.com",
    ///       "quality": "community"
    ///     }
    ///   ]
    /// }
    /// ```
    fn parse_json_overlay_list(&self, content: &str) -> Result<Vec<OverlayInfo>> {
        #[derive(Deserialize)]
        struct JsonOverlayList {
            overlays: Vec<JsonOverlay>,
        }

        #[derive(Deserialize)]
        struct JsonOverlay {
            name: String,
            #[serde(default)]
            description: String,
            homepage: Option<String>,
            #[serde(default = "default_sync_type")]
            sync_type: String,
            #[serde(default)]
            sync_uri: String,
            owner: Option<String>,
            #[serde(default = "default_quality")]
            quality: String,
            #[serde(default)]
            masters: Vec<String>,
        }

        fn default_sync_type() -> String {
            "git".to_string()
        }

        fn default_quality() -> String {
            "experimental".to_string()
        }

        // Try to parse as array or object with "overlays" key
        let json_overlays: Vec<JsonOverlay> = if content.trim().starts_with('[') {
            serde_json::from_str(content)
                .map_err(|e| Error::ParseError(format!("Failed to parse JSON: {}", e)))?
        } else {
            let list: JsonOverlayList = serde_json::from_str(content)
                .map_err(|e| Error::ParseError(format!("Failed to parse JSON: {}", e)))?;
            list.overlays
        };

        let overlays: Vec<OverlayInfo> = json_overlays
            .into_iter()
            .map(|jo| {
                let sync_type = match jo.sync_type.to_lowercase().as_str() {
                    "git" => SyncType::Git,
                    "rsync" => SyncType::Rsync,
                    "mercurial" | "hg" => SyncType::Mercurial,
                    "svn" | "subversion" => SyncType::Svn,
                    _ => SyncType::Git,
                };

                let quality = match jo.quality.to_lowercase().as_str() {
                    "official" | "core" => OverlayQuality::Official,
                    "community" | "stable" => OverlayQuality::Community,
                    "local" => OverlayQuality::Local,
                    _ => OverlayQuality::Experimental,
                };

                OverlayInfo {
                    name: jo.name.clone(),
                    description: jo.description,
                    homepage: jo.homepage,
                    sync_type,
                    sync_uri: jo.sync_uri,
                    location: self.config.storage_dir.join(&jo.name),
                    priority: 50,
                    enabled: false,
                    is_local: false,
                    owner: jo.owner,
                    quality,
                    masters: if jo.masters.is_empty() {
                        vec!["gentoo".to_string()]
                    } else {
                        jo.masters
                    },
                    auto_sync: true,
                    last_sync: None,
                }
            })
            .collect();

        debug!("Parsed {} overlays from JSON", overlays.len());
        Ok(overlays)
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

        manager.add_local("test-overlay", &local_path, 50).unwrap();

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

        manager.add_local("test-overlay", &local_path, 50).unwrap();

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

        manager.add_local("test-overlay", &local_path, 50).unwrap();
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

        manager.add_local("test-overlay", &local_path, 50).unwrap();
        manager.enable("test-overlay").unwrap();

        let configs = manager.to_repository_configs();
        assert_eq!(configs.len(), 1);
        assert_eq!(configs[0].name, "test-overlay");
        assert_eq!(configs[0].priority, 50);
    }
}
