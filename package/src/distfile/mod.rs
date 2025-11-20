//! Distfile management
//!
//! Handles source downloads with mirror support, checksum verification,
//! and RESTRICT="fetch" support.

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

/// Distfile manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistfileConfig {
    /// Directory to store downloaded files
    pub distdir: PathBuf,
    /// Mirror list
    pub mirrors: Vec<Mirror>,
    /// Parallel download limit
    pub parallel_downloads: usize,
    /// Connection timeout in seconds
    pub timeout: u64,
    /// Number of retry attempts
    pub retries: usize,
    /// Resume partial downloads
    pub resume: bool,
}

impl Default for DistfileConfig {
    fn default() -> Self {
        Self {
            distdir: PathBuf::from("/var/cache/distfiles"),
            mirrors: vec![
                Mirror {
                    name: "gentoo".to_string(),
                    url: "https://distfiles.gentoo.org/distfiles".to_string(),
                    priority: 100,
                    enabled: true,
                },
                Mirror {
                    name: "kernel".to_string(),
                    url: "https://mirrors.kernel.org/gentoo/distfiles".to_string(),
                    priority: 90,
                    enabled: true,
                },
            ],
            parallel_downloads: 4,
            timeout: 300,
            retries: 3,
            resume: true,
        }
    }
}

/// Mirror definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mirror {
    /// Mirror name
    pub name: String,
    /// Base URL
    pub url: String,
    /// Priority (higher is better)
    pub priority: i32,
    /// Whether this mirror is enabled
    pub enabled: bool,
}

/// Source URI definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceUri {
    /// URI(s) to fetch from
    pub uris: Vec<String>,
    /// Expected filename
    pub filename: String,
    /// Expected size in bytes
    pub size: Option<u64>,
    /// BLAKE2B hash
    pub blake2b: Option<String>,
    /// SHA512 hash
    pub sha512: Option<String>,
    /// Whether this file has fetch restrictions
    pub restrict_fetch: bool,
    /// Rename to this filename after download
    pub rename_to: Option<String>,
}

/// Download status
#[derive(Debug, Clone)]
pub struct DownloadStatus {
    /// Filename
    pub filename: String,
    /// Total size
    pub total: u64,
    /// Downloaded so far
    pub downloaded: u64,
    /// Download speed in bytes/sec
    pub speed: u64,
    /// Current mirror being used
    pub mirror: String,
    /// Status message
    pub status: DownloadState,
}

/// Download state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DownloadState {
    /// Waiting to start
    Pending,
    /// Currently downloading
    Downloading,
    /// Download complete
    Complete,
    /// Verifying checksum
    Verifying,
    /// Download failed
    Failed(String),
}

/// Distfile manager
pub struct DistfileManager {
    /// Configuration
    config: DistfileConfig,
    /// HTTP client
    client: reqwest::Client,
}

impl DistfileManager {
    /// Create a new distfile manager
    pub fn new(config: DistfileConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout))
            .build()
            .map_err(|e| Error::NetworkError(e.to_string()))?;

        // Ensure distdir exists
        std::fs::create_dir_all(&config.distdir)?;

        Ok(Self { config, client })
    }

    /// Fetch a source file
    pub async fn fetch(&self, source: &SourceUri) -> Result<PathBuf> {
        let dest = self.config.distdir.join(&source.filename);

        // Check if file already exists and is valid
        if dest.exists() {
            if self.verify_file(&dest, source).await? {
                tracing::info!("Using cached distfile: {}", source.filename);
                return Ok(dest);
            } else {
                // Invalid cached file, remove it
                std::fs::remove_file(&dest)?;
            }
        }

        // Check for fetch restrictions
        if source.restrict_fetch {
            return Err(Error::FetchRestricted {
                filename: source.filename.clone(),
                message: "This file must be downloaded manually".to_string(),
            });
        }

        // Try each URI
        let mut last_error = None;

        // First try original URIs
        for uri in &source.uris {
            match self.download(uri, &dest, source.size).await {
                Ok(_) => {
                    // Verify checksum
                    if self.verify_file(&dest, source).await? {
                        // Rename if needed
                        if let Some(ref new_name) = source.rename_to {
                            let new_dest = self.config.distdir.join(new_name);
                            std::fs::rename(&dest, &new_dest)?;
                            return Ok(new_dest);
                        }
                        return Ok(dest);
                    } else {
                        std::fs::remove_file(&dest)?;
                        last_error = Some("Checksum verification failed".to_string());
                    }
                }
                Err(e) => {
                    last_error = Some(e.to_string());
                }
            }
        }

        // Try mirrors
        for mirror in self.get_sorted_mirrors() {
            let mirror_url = format!("{}/{}", mirror.url, source.filename);

            match self.download(&mirror_url, &dest, source.size).await {
                Ok(_) => {
                    if self.verify_file(&dest, source).await? {
                        if let Some(ref new_name) = source.rename_to {
                            let new_dest = self.config.distdir.join(new_name);
                            std::fs::rename(&dest, &new_dest)?;
                            return Ok(new_dest);
                        }
                        return Ok(dest);
                    } else {
                        std::fs::remove_file(&dest)?;
                        last_error = Some("Checksum verification failed".to_string());
                    }
                }
                Err(e) => {
                    last_error = Some(e.to_string());
                }
            }
        }

        Err(Error::DistfileDownloadFailed {
            filename: source.filename.clone(),
            reason: last_error.unwrap_or_else(|| "All download attempts failed".to_string()),
        })
    }

    /// Download a file from URL
    async fn download(&self, url: &str, dest: &Path, expected_size: Option<u64>) -> Result<()> {
        tracing::info!("Downloading {} -> {}", url, dest.display());

        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| Error::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(Error::NetworkError(format!(
                "HTTP {}: {}",
                response.status(),
                url
            )));
        }

        let total_size = response.content_length().or(expected_size);

        let mut file = tokio::fs::File::create(dest).await?;
        let mut downloaded = 0u64;

        let mut stream = response.bytes_stream();
        use futures::StreamExt;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| Error::NetworkError(e.to_string()))?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;

            // Progress reporting
            if let Some(total) = total_size {
                let percent = (downloaded as f64 / total as f64 * 100.0) as u8;
                if percent % 10 == 0 {
                    tracing::debug!("Download progress: {}%", percent);
                }
            }
        }

        file.flush().await?;

        Ok(())
    }

    /// Verify a downloaded file
    async fn verify_file(&self, path: &Path, source: &SourceUri) -> Result<bool> {
        // Check size if specified
        if let Some(expected) = source.size {
            let actual = std::fs::metadata(path)?.len();
            if actual != expected {
                return Ok(false);
            }
        }

        // Check BLAKE2B hash
        if let Some(ref expected) = source.blake2b {
            let actual = self.compute_blake2b(path)?;
            if &actual != expected {
                return Ok(false);
            }
        }

        // Check SHA512 hash
        if let Some(ref expected) = source.sha512 {
            let actual = self.compute_sha512(path)?;
            if &actual != expected {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Compute BLAKE2B hash of a file
    fn compute_blake2b(&self, path: &Path) -> Result<String> {
        let data = std::fs::read(path)?;
        let hash = blake3::hash(&data);
        Ok(hash.to_hex().to_string())
    }

    /// Compute SHA512 hash of a file
    fn compute_sha512(&self, path: &Path) -> Result<String> {
        use sha2::{Digest, Sha512};

        let data = std::fs::read(path)?;
        let mut hasher = Sha512::new();
        hasher.update(&data);
        let result = hasher.finalize();
        Ok(hex::encode(result))
    }

    /// Get mirrors sorted by priority
    fn get_sorted_mirrors(&self) -> Vec<&Mirror> {
        let mut mirrors: Vec<_> = self.config.mirrors.iter().filter(|m| m.enabled).collect();
        mirrors.sort_by(|a, b| b.priority.cmp(&a.priority));
        mirrors
    }

    /// Fetch multiple files in parallel
    pub async fn fetch_all(&self, sources: &[SourceUri]) -> Result<Vec<PathBuf>> {
        use futures::stream::{self, StreamExt};

        let results: Vec<Result<PathBuf>> = stream::iter(sources)
            .map(|source| self.fetch(source))
            .buffer_unordered(self.config.parallel_downloads)
            .collect()
            .await;

        let mut paths = Vec::new();
        for result in results {
            paths.push(result?);
        }

        Ok(paths)
    }

    /// Clean old distfiles
    pub fn clean(&self, keep_days: u64) -> Result<CleanResult> {
        let cutoff =
            std::time::SystemTime::now() - std::time::Duration::from_secs(keep_days * 24 * 60 * 60);

        let mut removed = Vec::new();
        let mut freed = 0u64;

        for entry in std::fs::read_dir(&self.config.distdir)? {
            let entry = entry?;
            let metadata = entry.metadata()?;

            if let Ok(modified) = metadata.modified() {
                if modified < cutoff {
                    let size = metadata.len();
                    let path = entry.path();
                    std::fs::remove_file(&path)?;
                    removed.push(path.file_name().unwrap().to_string_lossy().to_string());
                    freed += size;
                }
            }
        }

        Ok(CleanResult { removed, freed })
    }

    /// Get distfile directory usage
    pub fn get_usage(&self) -> Result<DirUsage> {
        let mut total_size = 0u64;
        let mut file_count = 0usize;

        for entry in std::fs::read_dir(&self.config.distdir)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            total_size += metadata.len();
            file_count += 1;
        }

        Ok(DirUsage {
            path: self.config.distdir.clone(),
            total_size,
            file_count,
        })
    }

    /// Add a mirror
    pub fn add_mirror(&mut self, mirror: Mirror) {
        self.config.mirrors.push(mirror);
    }

    /// Remove a mirror by name
    pub fn remove_mirror(&mut self, name: &str) {
        self.config.mirrors.retain(|m| m.name != name);
    }

    /// Enable or disable a mirror
    pub fn set_mirror_enabled(&mut self, name: &str, enabled: bool) {
        for mirror in &mut self.config.mirrors {
            if mirror.name == name {
                mirror.enabled = enabled;
                break;
            }
        }
    }
}

/// Result of clean operation
#[derive(Debug, Clone)]
pub struct CleanResult {
    /// Files that were removed
    pub removed: Vec<String>,
    /// Bytes freed
    pub freed: u64,
}

/// Directory usage information
#[derive(Debug, Clone)]
pub struct DirUsage {
    /// Directory path
    pub path: PathBuf,
    /// Total size in bytes
    pub total_size: u64,
    /// Number of files
    pub file_count: usize,
}

/// Parse SRC_URI string into SourceUri objects
pub fn parse_src_uri(src_uri: &str) -> Vec<SourceUri> {
    let mut sources = Vec::new();
    let mut current_uris = Vec::new();
    let mut current_filename = None;

    for part in src_uri.split_whitespace() {
        if part == "->" {
            // Next token is filename
            continue;
        } else if part.starts_with("http")
            || part.starts_with("ftp")
            || part.starts_with("mirror://")
        {
            if let Some(filename) = current_filename.take() {
                // Previous URI group complete
                if !current_uris.is_empty() {
                    sources.push(SourceUri {
                        uris: current_uris.clone(),
                        filename,
                        size: None,
                        blake2b: None,
                        sha512: None,
                        restrict_fetch: false,
                        rename_to: None,
                    });
                    current_uris.clear();
                }
            }
            current_uris.push(part.to_string());

            // Extract filename from URL
            if let Some(name) = part.rsplit('/').next() {
                current_filename = Some(name.to_string());
            }
        } else if !current_uris.is_empty() {
            // This is a rename target
            if !current_uris.is_empty() {
                sources.push(SourceUri {
                    uris: current_uris.clone(),
                    filename: part.to_string(),
                    size: None,
                    blake2b: None,
                    sha512: None,
                    restrict_fetch: false,
                    rename_to: Some(part.to_string()),
                });
                current_uris.clear();
                current_filename = None;
            }
        }
    }

    // Handle last URI
    if !current_uris.is_empty() {
        if let Some(filename) = current_filename {
            sources.push(SourceUri {
                uris: current_uris,
                filename,
                size: None,
                blake2b: None,
                sha512: None,
                restrict_fetch: false,
                rename_to: None,
            });
        }
    }

    sources
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = DistfileConfig::default();
        assert!(!config.mirrors.is_empty());
        assert_eq!(config.parallel_downloads, 4);
    }

    #[test]
    fn test_parse_src_uri() {
        let uri = "https://example.com/foo-1.0.tar.gz";
        let sources = parse_src_uri(uri);
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].filename, "foo-1.0.tar.gz");
    }
}
