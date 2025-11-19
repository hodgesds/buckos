//! Package cache for downloads and build artifacts

use crate::{Error, Result};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Package cache manager
pub struct PackageCache {
    /// Base cache directory
    base_dir: PathBuf,
    /// Distfiles directory for source downloads
    distfiles_dir: PathBuf,
    /// Packages directory for built packages
    packages_dir: PathBuf,
    /// Temporary directory for in-progress downloads
    tmp_dir: PathBuf,
}

impl PackageCache {
    /// Create a new cache manager
    pub fn new(base_dir: &Path) -> Result<Self> {
        let distfiles_dir = base_dir.join("distfiles");
        let packages_dir = base_dir.join("packages");
        let tmp_dir = base_dir.join("tmp");

        // Create directories
        std::fs::create_dir_all(&distfiles_dir)?;
        std::fs::create_dir_all(&packages_dir)?;
        std::fs::create_dir_all(&tmp_dir)?;

        Ok(Self {
            base_dir: base_dir.to_path_buf(),
            distfiles_dir,
            packages_dir,
            tmp_dir,
        })
    }

    /// Get path to a distfile
    pub fn distfile_path(&self, filename: &str) -> PathBuf {
        self.distfiles_dir.join(filename)
    }

    /// Get path to a package file
    pub fn package_path(&self, category: &str, name: &str, version: &str) -> PathBuf {
        self.packages_dir
            .join(category)
            .join(format!("{}-{}.tar.zst", name, version))
    }

    /// Check if a distfile exists in cache
    pub fn has_distfile(&self, filename: &str) -> bool {
        self.distfile_path(filename).exists()
    }

    /// Check if a package exists in cache
    pub fn has_package(&self, category: &str, name: &str, version: &str) -> bool {
        self.package_path(category, name, version).exists()
    }

    /// Download a file to the cache
    pub async fn download(
        &self,
        url: &str,
        filename: &str,
        expected_hash: Option<&str>,
    ) -> Result<PathBuf> {
        let dest_path = self.distfile_path(filename);

        // Check if already cached
        if dest_path.exists() {
            if let Some(hash) = expected_hash {
                let actual = compute_sha256(&dest_path)?;
                if actual == hash {
                    info!("Using cached distfile: {}", filename);
                    return Ok(dest_path);
                } else {
                    warn!("Cached file has wrong hash, re-downloading");
                    std::fs::remove_file(&dest_path)?;
                }
            } else {
                return Ok(dest_path);
            }
        }

        info!("Downloading: {}", url);

        // Download to temp file first
        let tmp_path = self.tmp_dir.join(format!("{}.partial", filename));

        let client = reqwest::Client::new();
        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| Error::DownloadFailed {
                url: url.to_string(),
                message: e.to_string(),
            })?;

        if !response.status().is_success() {
            return Err(Error::DownloadFailed {
                url: url.to_string(),
                message: format!("HTTP {}", response.status()),
            });
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| Error::DownloadFailed {
                url: url.to_string(),
                message: e.to_string(),
            })?;

        std::fs::write(&tmp_path, &bytes)?;

        // Verify hash
        if let Some(expected) = expected_hash {
            let actual = compute_sha256(&tmp_path)?;
            if actual != expected {
                std::fs::remove_file(&tmp_path)?;
                return Err(Error::ChecksumMismatch {
                    path: filename.to_string(),
                    expected: expected.to_string(),
                    actual,
                });
            }
        }

        // Move to final location
        std::fs::rename(&tmp_path, &dest_path)?;

        Ok(dest_path)
    }

    /// Store a built package in the cache
    pub fn store_package(
        &self,
        category: &str,
        name: &str,
        version: &str,
        source_path: &Path,
    ) -> Result<PathBuf> {
        let dest_path = self.package_path(category, name, version);

        // Ensure directory exists
        if let Some(parent) = dest_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::copy(source_path, &dest_path)?;
        Ok(dest_path)
    }

    /// Get cache size
    pub fn size(&self) -> Result<u64> {
        let mut total = 0;
        for entry in walkdir::WalkDir::new(&self.base_dir) {
            let entry = entry?;
            if entry.file_type().is_file() {
                total += entry.metadata()?.len();
            }
        }
        Ok(total)
    }

    /// Clean all cache
    pub fn clean_all(&self) -> Result<()> {
        info!("Cleaning all cache");
        if self.distfiles_dir.exists() {
            std::fs::remove_dir_all(&self.distfiles_dir)?;
            std::fs::create_dir_all(&self.distfiles_dir)?;
        }
        if self.packages_dir.exists() {
            std::fs::remove_dir_all(&self.packages_dir)?;
            std::fs::create_dir_all(&self.packages_dir)?;
        }
        if self.tmp_dir.exists() {
            std::fs::remove_dir_all(&self.tmp_dir)?;
            std::fs::create_dir_all(&self.tmp_dir)?;
        }
        Ok(())
    }

    /// Clean only downloads
    pub fn clean_downloads(&self) -> Result<()> {
        info!("Cleaning downloads cache");
        if self.distfiles_dir.exists() {
            std::fs::remove_dir_all(&self.distfiles_dir)?;
            std::fs::create_dir_all(&self.distfiles_dir)?;
        }
        Ok(())
    }

    /// Clean old entries
    pub fn clean_old(&self, max_age_days: u32) -> Result<()> {
        let cutoff = std::time::SystemTime::now()
            - std::time::Duration::from_secs(max_age_days as u64 * 24 * 3600);

        for entry in walkdir::WalkDir::new(&self.base_dir) {
            let entry = entry?;
            if entry.file_type().is_file() {
                let metadata = entry.metadata()?;
                if let Ok(modified) = metadata.modified() {
                    if modified < cutoff {
                        std::fs::remove_file(entry.path())?;
                    }
                }
            }
        }
        Ok(())
    }
}

/// Compute SHA256 hash of a file
pub fn compute_sha256(path: &Path) -> Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Compute BLAKE3 hash of a file
pub fn compute_blake3(path: &Path) -> Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0u8; 8192];

    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    Ok(hasher.finalize().to_hex().to_string())
}

/// Extract a tarball
pub fn extract_tarball(archive_path: &Path, dest_dir: &Path) -> Result<()> {
    let file = std::fs::File::open(archive_path)?;

    // Determine compression based on extension
    let ext = archive_path.extension().and_then(|e| e.to_str());

    match ext {
        Some("zst") => {
            let decoder = zstd::Decoder::new(file)?;
            let mut archive = tar::Archive::new(decoder);
            archive.unpack(dest_dir)?;
        }
        Some("xz") => {
            let decoder = xz2::read::XzDecoder::new(file);
            let mut archive = tar::Archive::new(decoder);
            archive.unpack(dest_dir)?;
        }
        Some("gz") | Some("tgz") => {
            let decoder = flate2::read::GzDecoder::new(file);
            let mut archive = tar::Archive::new(decoder);
            archive.unpack(dest_dir)?;
        }
        Some("tar") => {
            let mut archive = tar::Archive::new(file);
            archive.unpack(dest_dir)?;
        }
        _ => {
            // Try to detect from magic bytes
            let mut archive = tar::Archive::new(file);
            archive.unpack(dest_dir)?;
        }
    }

    Ok(())
}

/// Create a compressed tarball
pub fn create_tarball(source_dir: &Path, dest_path: &Path) -> Result<()> {
    let file = std::fs::File::create(dest_path)?;
    let encoder = zstd::Encoder::new(file, 3)?.auto_finish();
    let mut archive = tar::Builder::new(encoder);

    archive.append_dir_all(".", source_dir)?;
    archive.finish()?;

    Ok(())
}
