//! Binary Package Support
//!
//! Pre-built binary package management for Buckos, compatible with Gentoo's binary package system.
//!
//! Features:
//! - PKGDIR for binary package storage
//! - binpkg-multi-instance support
//! - Binary package signing
//! - --getbinpkg and --usepkg flags

use crate::security::signing::{SignatureVerification, SigningManager};
use crate::{Error, InstalledPackage, PackageId, PackageInfo, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Binary package format version
pub const BINPKG_FORMAT_VERSION: u32 = 2;

/// Default compression for binary packages
pub const DEFAULT_COMPRESSION: BinpkgCompression = BinpkgCompression::Zstd;

/// Binary package compression types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinpkgCompression {
    /// No compression
    None,
    /// Gzip compression
    Gzip,
    /// Bzip2 compression
    Bzip2,
    /// XZ compression
    Xz,
    /// LZ4 compression
    Lz4,
    /// Zstandard compression (default)
    Zstd,
}

impl BinpkgCompression {
    /// Get file extension for this compression type
    pub fn extension(&self) -> &'static str {
        match self {
            BinpkgCompression::None => "tar",
            BinpkgCompression::Gzip => "tar.gz",
            BinpkgCompression::Bzip2 => "tar.bz2",
            BinpkgCompression::Xz => "tar.xz",
            BinpkgCompression::Lz4 => "tar.lz4",
            BinpkgCompression::Zstd => "tar.zst",
        }
    }

    /// Parse compression type from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            "tar" => Some(BinpkgCompression::None),
            "tar.gz" | "tgz" => Some(BinpkgCompression::Gzip),
            "tar.bz2" | "tbz2" => Some(BinpkgCompression::Bzip2),
            "tar.xz" | "txz" => Some(BinpkgCompression::Xz),
            "tar.lz4" => Some(BinpkgCompression::Lz4),
            "tar.zst" | "tzst" => Some(BinpkgCompression::Zstd),
            _ => None,
        }
    }
}

impl Default for BinpkgCompression {
    fn default() -> Self {
        DEFAULT_COMPRESSION
    }
}

impl std::fmt::Display for BinpkgCompression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinpkgCompression::None => write!(f, "none"),
            BinpkgCompression::Gzip => write!(f, "gzip"),
            BinpkgCompression::Bzip2 => write!(f, "bzip2"),
            BinpkgCompression::Xz => write!(f, "xz"),
            BinpkgCompression::Lz4 => write!(f, "lz4"),
            BinpkgCompression::Zstd => write!(f, "zstd"),
        }
    }
}

/// Binary package metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryPackage {
    /// Package identifier
    pub id: PackageId,
    /// Package version
    pub version: semver::Version,
    /// Package slot
    pub slot: String,
    /// Package description
    pub description: String,
    /// Build time timestamp
    pub build_time: chrono::DateTime<chrono::Utc>,
    /// USE flags used in build
    pub use_flags: Vec<String>,
    /// Size of the binary package file
    pub size: u64,
    /// Installed size when unpacked
    pub installed_size: u64,
    /// BLAKE3 hash of the package file
    pub blake3_hash: String,
    /// SHA512 hash of the package file
    pub sha512_hash: String,
    /// Build host information
    pub build_host: String,
    /// Build CFLAGS
    pub cflags: String,
    /// Build CXXFLAGS
    pub cxxflags: String,
    /// Build LDFLAGS
    pub ldflags: String,
    /// Dependencies
    pub dependencies: Vec<String>,
    /// Runtime dependencies
    pub runtime_deps: Vec<String>,
    /// Build dependencies
    pub build_deps: Vec<String>,
    /// Compression type used
    pub compression: BinpkgCompression,
    /// Multi-instance identifier (for binpkg-multi-instance)
    pub instance_id: Option<String>,
    /// GPG signature (if signed)
    pub signature: Option<String>,
    /// Path to the binary package file (relative to PKGDIR)
    pub path: String,
    /// Repository the package came from
    pub repository: String,
    /// License
    pub license: String,
    /// Architecture (e.g., "amd64", "arm64")
    pub arch: String,
    /// EAPI version
    pub eapi: String,
    /// Package format version
    pub format_version: u32,
}

impl BinaryPackage {
    /// Create a new binary package from installed package
    pub fn from_installed(pkg: &InstalledPackage) -> Self {
        Self {
            id: pkg.id.clone(),
            version: pkg.version.clone(),
            slot: pkg.slot.clone(),
            description: String::new(),
            build_time: chrono::Utc::now(),
            use_flags: pkg.use_flags.iter().cloned().collect(),
            size: 0,
            installed_size: pkg.size,
            blake3_hash: String::new(),
            sha512_hash: String::new(),
            build_host: get_hostname(),
            cflags: std::env::var("CFLAGS").unwrap_or_default(),
            cxxflags: std::env::var("CXXFLAGS").unwrap_or_default(),
            ldflags: std::env::var("LDFLAGS").unwrap_or_default(),
            dependencies: Vec::new(),
            runtime_deps: Vec::new(),
            build_deps: Vec::new(),
            compression: DEFAULT_COMPRESSION,
            instance_id: None,
            signature: None,
            path: String::new(),
            repository: "buckos".to_string(),
            license: String::new(),
            arch: get_arch(),
            eapi: "8".to_string(),
            format_version: BINPKG_FORMAT_VERSION,
        }
    }

    /// Create from package info
    pub fn from_package_info(info: &PackageInfo) -> Self {
        Self {
            id: info.id.clone(),
            version: info.version.clone(),
            slot: info.slot.clone(),
            description: info.description.clone(),
            build_time: chrono::Utc::now(),
            use_flags: info
                .use_flags
                .iter()
                .filter(|f| f.default)
                .map(|f| f.name.clone())
                .collect(),
            size: 0,
            installed_size: info.installed_size,
            blake3_hash: String::new(),
            sha512_hash: String::new(),
            build_host: get_hostname(),
            cflags: std::env::var("CFLAGS").unwrap_or_default(),
            cxxflags: std::env::var("CXXFLAGS").unwrap_or_default(),
            ldflags: std::env::var("LDFLAGS").unwrap_or_default(),
            dependencies: info
                .dependencies
                .iter()
                .map(|d| d.package.full_name())
                .collect(),
            runtime_deps: info
                .runtime_dependencies
                .iter()
                .map(|d| d.package.full_name())
                .collect(),
            build_deps: info
                .build_dependencies
                .iter()
                .map(|d| d.package.full_name())
                .collect(),
            compression: DEFAULT_COMPRESSION,
            instance_id: None,
            signature: None,
            path: String::new(),
            repository: "buckos".to_string(),
            license: info.license.clone(),
            arch: get_arch(),
            eapi: "8".to_string(),
            format_version: BINPKG_FORMAT_VERSION,
        }
    }

    /// Get the filename for this binary package
    pub fn filename(&self) -> String {
        let base = if let Some(ref instance_id) = self.instance_id {
            format!(
                "{}-{}-{}.{}",
                self.id.name,
                self.version,
                instance_id,
                self.compression.extension()
            )
        } else {
            format!(
                "{}-{}.{}",
                self.id.name,
                self.version,
                self.compression.extension()
            )
        };
        base
    }

    /// Get the full path for this binary package within PKGDIR
    pub fn full_path(&self, pkgdir: &Path) -> PathBuf {
        pkgdir.join(&self.id.category).join(self.filename())
    }

    /// Check if this package is signed
    pub fn is_signed(&self) -> bool {
        self.signature.is_some()
    }
}

/// Options for binary package operations
#[derive(Debug, Clone, Default)]
pub struct BinaryPackageOptions {
    /// Use binary packages when available (--usepkg)
    pub use_pkg: bool,
    /// Only use binary packages, fail if not available (--usepkgonly)
    pub use_pkg_only: bool,
    /// Fetch binary packages from remote (--getbinpkg)
    pub get_binpkg: bool,
    /// Only fetch binary packages, don't build (--getbinpkgonly)
    pub get_binpkg_only: bool,
    /// Build binary packages after compilation (--buildpkg)
    pub build_pkg: bool,
    /// Only build binary packages (--buildpkgonly)
    pub build_pkg_only: bool,
    /// Enable multi-instance support
    pub multi_instance: bool,
    /// Sign binary packages
    pub sign: bool,
    /// Signing key ID
    pub signing_key: Option<String>,
    /// Compression type
    pub compression: BinpkgCompression,
    /// Remote binary package server URL
    pub binpkg_server: Option<String>,
}

/// Binary package directory (PKGDIR) manager
pub struct BinaryPackageManager {
    /// Root directory for binary packages (PKGDIR)
    pkgdir: PathBuf,
    /// Binary package index
    index: BinaryPackageIndex,
    /// Signing manager for GPG operations
    signing_manager: SigningManager,
    /// Multi-instance support enabled
    multi_instance: bool,
    /// Remote server URL for fetching packages
    remote_server: Option<String>,
}

/// Index of available binary packages
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BinaryPackageIndex {
    /// Map of package ID to available binary packages
    pub packages: HashMap<String, Vec<BinaryPackage>>,
    /// Last update timestamp
    pub last_updated: Option<chrono::DateTime<chrono::Utc>>,
    /// Index format version
    pub version: u32,
}

impl BinaryPackageManager {
    /// Create a new binary package manager
    pub fn new(pkgdir: PathBuf) -> Result<Self> {
        // Ensure PKGDIR exists
        if !pkgdir.exists() {
            std::fs::create_dir_all(&pkgdir)?;
            info!("Created PKGDIR: {}", pkgdir.display());
        }

        let signing_manager = SigningManager::new()?;

        // Load or create index
        let index = Self::load_or_create_index(&pkgdir)?;

        Ok(Self {
            pkgdir,
            index,
            signing_manager,
            multi_instance: false,
            remote_server: None,
        })
    }

    /// Create with multi-instance support
    pub fn with_multi_instance(mut self, enabled: bool) -> Self {
        self.multi_instance = enabled;
        self
    }

    /// Set remote server for fetching packages
    pub fn with_remote_server(mut self, url: Option<String>) -> Self {
        self.remote_server = url;
        self
    }

    /// Get the PKGDIR path
    pub fn pkgdir(&self) -> &Path {
        &self.pkgdir
    }

    /// Load or create the binary package index
    fn load_or_create_index(pkgdir: &Path) -> Result<BinaryPackageIndex> {
        let index_path = pkgdir.join("Packages.json");

        if index_path.exists() {
            let content = std::fs::read_to_string(&index_path)?;
            let index: BinaryPackageIndex = serde_json::from_str(&content)
                .map_err(|e| Error::Other(format!("Failed to parse package index: {}", e)))?;
            Ok(index)
        } else {
            Ok(BinaryPackageIndex {
                packages: HashMap::new(),
                last_updated: Some(chrono::Utc::now()),
                version: 1,
            })
        }
    }

    /// Save the package index
    pub fn save_index(&self) -> Result<()> {
        let index_path = self.pkgdir.join("Packages.json");
        let content = serde_json::to_string_pretty(&self.index)
            .map_err(|e| Error::Other(format!("Failed to serialize index: {}", e)))?;
        std::fs::write(&index_path, content)?;
        debug!("Saved package index to {}", index_path.display());
        Ok(())
    }

    /// Create a binary package from build output
    pub async fn create_package(
        &mut self,
        pkg: &InstalledPackage,
        build_dir: &Path,
        opts: &BinaryPackageOptions,
    ) -> Result<BinaryPackage> {
        info!("Creating binary package for {}-{}", pkg.id, pkg.version);

        let mut binpkg = BinaryPackage::from_installed(pkg);
        binpkg.compression = opts.compression;

        // Generate instance ID if multi-instance is enabled
        if opts.multi_instance || self.multi_instance {
            binpkg.instance_id = Some(generate_instance_id(pkg));
        }

        // Create package directory
        let pkg_category_dir = self.pkgdir.join(&pkg.id.category);
        if !pkg_category_dir.exists() {
            std::fs::create_dir_all(&pkg_category_dir)?;
        }

        // Create the archive
        let pkg_path = binpkg.full_path(&self.pkgdir);
        self.create_archive(build_dir, &pkg_path, opts.compression)
            .await?;

        // Calculate hashes
        let content = std::fs::read(&pkg_path)?;
        binpkg.size = content.len() as u64;
        binpkg.blake3_hash = calculate_blake3(&content);
        binpkg.sha512_hash = calculate_sha512(&content);

        // Sign if requested
        if opts.sign {
            let signature = self
                .signing_manager
                .sign_data(&content, opts.signing_key.as_deref())?;
            binpkg.signature = Some(signature);

            // Also write detached signature file
            let sig_path = pkg_path.with_extension(format!("{}.asc", opts.compression.extension()));
            std::fs::write(&sig_path, binpkg.signature.as_ref().unwrap())?;
            info!("Created signature: {}", sig_path.display());
        }

        // Set relative path
        binpkg.path = format!("{}/{}", pkg.id.category, binpkg.filename());

        // Update index
        let key = pkg.id.full_name();
        self.index
            .packages
            .entry(key)
            .or_default()
            .push(binpkg.clone());
        self.index.last_updated = Some(chrono::Utc::now());
        self.save_index()?;

        info!(
            "Created binary package: {} ({} bytes)",
            pkg_path.display(),
            binpkg.size
        );

        Ok(binpkg)
    }

    /// Create a compressed archive
    async fn create_archive(
        &self,
        source_dir: &Path,
        output_path: &Path,
        compression: BinpkgCompression,
    ) -> Result<()> {
        let tar_cmd = match compression {
            BinpkgCompression::None => "tar -cf",
            BinpkgCompression::Gzip => "tar -czf",
            BinpkgCompression::Bzip2 => "tar -cjf",
            BinpkgCompression::Xz => "tar -cJf",
            BinpkgCompression::Lz4 => "tar -c --lz4 -f",
            BinpkgCompression::Zstd => "tar -c --zstd -f",
        };

        let output = tokio::process::Command::new("sh")
            .args([
                "-c",
                &format!(
                    "{} {} -C {} .",
                    tar_cmd,
                    output_path.display(),
                    source_dir.display()
                ),
            ])
            .output()
            .await
            .map_err(|e| Error::Other(format!("Failed to create archive: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Other(format!("Archive creation failed: {}", stderr)));
        }

        Ok(())
    }

    /// Extract a binary package
    pub async fn extract_package(&self, binpkg: &BinaryPackage, dest_dir: &Path) -> Result<()> {
        let pkg_path = self.pkgdir.join(&binpkg.path);

        if !pkg_path.exists() {
            return Err(Error::PackageNotFound(binpkg.id.full_name()));
        }

        // Create destination directory
        if !dest_dir.exists() {
            std::fs::create_dir_all(dest_dir)?;
        }

        let tar_cmd = match binpkg.compression {
            BinpkgCompression::None => "tar -xf",
            BinpkgCompression::Gzip => "tar -xzf",
            BinpkgCompression::Bzip2 => "tar -xjf",
            BinpkgCompression::Xz => "tar -xJf",
            BinpkgCompression::Lz4 => "tar -x --lz4 -f",
            BinpkgCompression::Zstd => "tar -x --zstd -f",
        };

        let output = tokio::process::Command::new("sh")
            .args([
                "-c",
                &format!(
                    "{} {} -C {}",
                    tar_cmd,
                    pkg_path.display(),
                    dest_dir.display()
                ),
            ])
            .output()
            .await
            .map_err(|e| Error::Other(format!("Failed to extract package: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Other(format!(
                "Package extraction failed: {}",
                stderr
            )));
        }

        info!("Extracted {} to {}", binpkg.path, dest_dir.display());
        Ok(())
    }

    /// Find available binary packages for a package ID
    pub fn find_packages(&self, pkg_id: &PackageId) -> Vec<&BinaryPackage> {
        let key = pkg_id.full_name();
        self.index
            .packages
            .get(&key)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    /// Find a specific version of a binary package
    pub fn find_package_version(
        &self,
        pkg_id: &PackageId,
        version: &semver::Version,
    ) -> Option<&BinaryPackage> {
        let key = pkg_id.full_name();
        self.index
            .packages
            .get(&key)
            .and_then(|packages| packages.iter().find(|p| &p.version == version))
    }

    /// Get the best matching binary package (latest version)
    pub fn get_best_match(&self, pkg_id: &PackageId) -> Option<&BinaryPackage> {
        let packages = self.find_packages(pkg_id);
        packages
            .into_iter()
            .max_by(|a, b| a.version.cmp(&b.version))
    }

    /// Verify a binary package
    pub fn verify_package(&self, binpkg: &BinaryPackage) -> Result<BinaryPackageVerification> {
        let pkg_path = self.pkgdir.join(&binpkg.path);

        if !pkg_path.exists() {
            return Ok(BinaryPackageVerification {
                valid: false,
                hash_valid: false,
                signature_valid: None,
                message: "Package file not found".to_string(),
            });
        }

        let content = std::fs::read(&pkg_path)?;

        // Verify hashes
        let blake3_valid = calculate_blake3(&content) == binpkg.blake3_hash;
        let sha512_valid = calculate_sha512(&content) == binpkg.sha512_hash;
        let hash_valid = blake3_valid && sha512_valid;

        // Verify signature if present
        let signature_valid = if let Some(ref sig) = binpkg.signature {
            self.signing_manager.verify_data(&content, sig).ok()
        } else {
            None
        };

        let valid = hash_valid && signature_valid.as_ref().map(|s| s.valid).unwrap_or(true);

        Ok(BinaryPackageVerification {
            valid,
            hash_valid,
            signature_valid,
            message: if valid {
                "Package verified successfully".to_string()
            } else if !hash_valid {
                "Hash verification failed".to_string()
            } else {
                "Signature verification failed".to_string()
            },
        })
    }

    /// List all binary packages in PKGDIR
    pub fn list_packages(&self) -> Vec<&BinaryPackage> {
        self.index
            .packages
            .values()
            .flat_map(|v| v.iter())
            .collect()
    }

    /// Remove a binary package
    pub fn remove_package(&mut self, binpkg: &BinaryPackage) -> Result<()> {
        let pkg_path = self.pkgdir.join(&binpkg.path);

        if pkg_path.exists() {
            std::fs::remove_file(&pkg_path)?;
            info!("Removed binary package: {}", pkg_path.display());
        }

        // Remove signature file if exists
        let sig_path = pkg_path.with_extension(format!("{}.asc", binpkg.compression.extension()));
        if sig_path.exists() {
            std::fs::remove_file(&sig_path)?;
        }

        // Update index
        let key = binpkg.id.full_name();
        if let Some(packages) = self.index.packages.get_mut(&key) {
            packages.retain(|p| p.version != binpkg.version);
            if packages.is_empty() {
                self.index.packages.remove(&key);
            }
        }
        self.save_index()?;

        Ok(())
    }

    /// Clean old binary packages (keep only latest N versions)
    pub fn clean_old_packages(&mut self, keep_versions: usize) -> Result<Vec<BinaryPackage>> {
        let mut removed = Vec::new();

        for packages in self.index.packages.values_mut() {
            // Sort by version descending
            packages.sort_by(|a, b| b.version.cmp(&a.version));

            // Remove old versions
            while packages.len() > keep_versions {
                if let Some(old) = packages.pop() {
                    let pkg_path = self.pkgdir.join(&old.path);
                    if pkg_path.exists() {
                        std::fs::remove_file(&pkg_path)?;
                    }
                    removed.push(old);
                }
            }
        }

        self.index.last_updated = Some(chrono::Utc::now());
        self.save_index()?;

        info!("Cleaned {} old binary packages", removed.len());
        Ok(removed)
    }

    /// Fetch a binary package from remote server
    pub async fn fetch_package(
        &mut self,
        pkg_id: &PackageId,
        version: Option<&semver::Version>,
    ) -> Result<BinaryPackage> {
        let server = self
            .remote_server
            .as_ref()
            .ok_or_else(|| Error::Other("No remote server configured".to_string()))?;

        // Construct URL
        let filename = if let Some(v) = version {
            format!("{}-{}.{}", pkg_id.name, v, DEFAULT_COMPRESSION.extension())
        } else {
            // Fetch index to find latest version
            format!("{}.{}", pkg_id.name, DEFAULT_COMPRESSION.extension())
        };

        let url = format!("{}/{}/{}", server, pkg_id.category, filename);
        info!("Fetching binary package from {}", url);

        // Create HTTP client and download
        let client = reqwest::Client::new();
        let response = client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(Error::DownloadFailed {
                url: url.clone(),
                message: format!("HTTP {}", response.status()),
            });
        }

        let content = response.bytes().await?;

        // Save to PKGDIR
        let pkg_category_dir = self.pkgdir.join(&pkg_id.category);
        if !pkg_category_dir.exists() {
            std::fs::create_dir_all(&pkg_category_dir)?;
        }

        let pkg_path = pkg_category_dir.join(&filename);
        std::fs::write(&pkg_path, &content)?;

        // Calculate hashes
        let blake3_hash = calculate_blake3(&content);
        let sha512_hash = calculate_sha512(&content);

        // Create package metadata
        let binpkg = BinaryPackage {
            id: pkg_id.clone(),
            version: version
                .cloned()
                .unwrap_or_else(|| semver::Version::new(0, 0, 0)),
            slot: "0".to_string(),
            description: String::new(),
            build_time: chrono::Utc::now(),
            use_flags: Vec::new(),
            size: content.len() as u64,
            installed_size: 0,
            blake3_hash,
            sha512_hash,
            build_host: String::new(),
            cflags: String::new(),
            cxxflags: String::new(),
            ldflags: String::new(),
            dependencies: Vec::new(),
            runtime_deps: Vec::new(),
            build_deps: Vec::new(),
            compression: DEFAULT_COMPRESSION,
            instance_id: None,
            signature: None,
            path: format!("{}/{}", pkg_id.category, filename),
            repository: server.clone(),
            license: String::new(),
            arch: get_arch(),
            eapi: "8".to_string(),
            format_version: BINPKG_FORMAT_VERSION,
        };

        // Update index
        let key = pkg_id.full_name();
        self.index
            .packages
            .entry(key)
            .or_default()
            .push(binpkg.clone());
        self.index.last_updated = Some(chrono::Utc::now());
        self.save_index()?;

        info!("Downloaded binary package: {}", pkg_path.display());
        Ok(binpkg)
    }

    /// Sync remote package index
    pub async fn sync_remote_index(&mut self) -> Result<()> {
        let server = self
            .remote_server
            .as_ref()
            .ok_or_else(|| Error::Other("No remote server configured".to_string()))?;

        let url = format!("{}/Packages.json", server);
        info!("Syncing remote package index from {}", url);

        let client = reqwest::Client::new();
        let response = client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(Error::DownloadFailed {
                url,
                message: format!("HTTP {}", response.status()),
            });
        }

        let content = response.text().await?;
        let remote_index: BinaryPackageIndex = serde_json::from_str(&content)
            .map_err(|e| Error::Other(format!("Failed to parse remote index: {}", e)))?;

        // Merge with local index
        for (key, packages) in remote_index.packages {
            self.index.packages.entry(key).or_default().extend(packages);
        }

        self.index.last_updated = Some(chrono::Utc::now());
        self.save_index()?;

        info!("Synced remote package index");
        Ok(())
    }

    /// Generate Packages file (for repository serving)
    pub fn generate_packages_file(&self) -> Result<String> {
        let mut output = String::new();

        for packages in self.index.packages.values() {
            for pkg in packages {
                output.push_str(&format!(
                    "CPV: {}/{}-{}\n",
                    pkg.id.category, pkg.id.name, pkg.version
                ));
                output.push_str(&format!("SLOT: {}\n", pkg.slot));
                output.push_str(&format!("PATH: {}\n", pkg.path));
                output.push_str(&format!("SIZE: {}\n", pkg.size));
                output.push_str(&format!("BLAKE3: {}\n", pkg.blake3_hash));
                output.push_str(&format!("SHA512: {}\n", pkg.sha512_hash));
                output.push_str(&format!("BUILD_TIME: {}\n", pkg.build_time.timestamp()));
                output.push_str(&format!("USE: {}\n", pkg.use_flags.join(" ")));
                if let Some(ref sig) = pkg.signature {
                    output.push_str(&format!("GPG: {}\n", sig.lines().next().unwrap_or("")));
                }
                output.push('\n');
            }
        }

        Ok(output)
    }

    /// Sign all unsigned packages in PKGDIR
    pub fn sign_all_packages(&mut self, key_id: Option<&str>) -> Result<usize> {
        let mut signed_count = 0;

        for packages in self.index.packages.values_mut() {
            for pkg in packages.iter_mut() {
                if pkg.signature.is_none() {
                    let pkg_path = self.pkgdir.join(&pkg.path);
                    if pkg_path.exists() {
                        let content = std::fs::read(&pkg_path)?;
                        let signature = self.signing_manager.sign_data(&content, key_id)?;
                        pkg.signature = Some(signature.clone());

                        // Write detached signature
                        let sig_path =
                            pkg_path.with_extension(format!("{}.asc", pkg.compression.extension()));
                        std::fs::write(&sig_path, &signature)?;

                        signed_count += 1;
                        debug!("Signed: {}", pkg.path);
                    }
                }
            }
        }

        if signed_count > 0 {
            self.save_index()?;
        }

        info!("Signed {} packages", signed_count);
        Ok(signed_count)
    }

    /// Rebuild the package index by scanning PKGDIR
    pub fn rebuild_index(&mut self) -> Result<()> {
        info!("Rebuilding binary package index");

        let mut new_index = BinaryPackageIndex {
            packages: HashMap::new(),
            last_updated: Some(chrono::Utc::now()),
            version: 1,
        };

        // Scan PKGDIR for packages
        for entry in walkdir::WalkDir::new(&self.pkgdir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

            // Check if it's a package file
            if let Some(compression) = BinpkgCompression::from_extension(filename) {
                // Parse package info from filename
                if let Some(binpkg) = self.parse_package_from_path(path, compression)? {
                    let key = binpkg.id.full_name();
                    new_index.packages.entry(key).or_default().push(binpkg);
                }
            }
        }

        self.index = new_index;
        self.save_index()?;

        info!("Rebuilt index with {} packages", self.index.packages.len());
        Ok(())
    }

    /// Parse package metadata from file path
    fn parse_package_from_path(
        &self,
        path: &Path,
        compression: BinpkgCompression,
    ) -> Result<Option<BinaryPackage>> {
        let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        let category = path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .unwrap_or("");

        // Parse name and version from filename (e.g., "openssl-3.0.0.tar.zst")
        let stem = filename.trim_end_matches(&format!(".{}", compression.extension()));

        // Find last dash followed by digit (version separator)
        let mut last_dash = None;
        for (i, c) in stem.char_indices() {
            if c == '-'
                && stem[i + 1..]
                    .chars()
                    .next()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false)
            {
                last_dash = Some(i);
            }
        }

        let (name, version_str) = if let Some(idx) = last_dash {
            (&stem[..idx], &stem[idx + 1..])
        } else {
            return Ok(None);
        };

        let version = semver::Version::parse(version_str)
            .or_else(|_| parse_simple_version(version_str))
            .map_err(|_| Error::InvalidVersion(version_str.to_string()))?;

        // Read file content for hashes
        let content = std::fs::read(path)?;
        let size = content.len() as u64;

        let pkg_id = PackageId::new(category, name);
        let relative_path = path
            .strip_prefix(&self.pkgdir)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| filename.to_string());

        Ok(Some(BinaryPackage {
            id: pkg_id,
            version,
            slot: "0".to_string(),
            description: String::new(),
            build_time: chrono::Utc::now(),
            use_flags: Vec::new(),
            size,
            installed_size: 0,
            blake3_hash: calculate_blake3(&content),
            sha512_hash: calculate_sha512(&content),
            build_host: String::new(),
            cflags: String::new(),
            cxxflags: String::new(),
            ldflags: String::new(),
            dependencies: Vec::new(),
            runtime_deps: Vec::new(),
            build_deps: Vec::new(),
            compression,
            instance_id: None,
            signature: None,
            path: relative_path,
            repository: "local".to_string(),
            license: String::new(),
            arch: get_arch(),
            eapi: "8".to_string(),
            format_version: BINPKG_FORMAT_VERSION,
        }))
    }
}

/// Result of binary package verification
#[derive(Debug, Clone)]
pub struct BinaryPackageVerification {
    /// Overall validity
    pub valid: bool,
    /// Hash verification passed
    pub hash_valid: bool,
    /// Signature verification result (None if not signed)
    pub signature_valid: Option<SignatureVerification>,
    /// Human-readable message
    pub message: String,
}

// Helper functions

fn get_hostname() -> String {
    std::env::var("HOSTNAME")
        .or_else(|_| std::fs::read_to_string("/etc/hostname").map(|s| s.trim().to_string()))
        .unwrap_or_else(|_| "localhost".to_string())
}

fn get_arch() -> String {
    std::env::consts::ARCH.to_string()
}

fn calculate_blake3(data: &[u8]) -> String {
    let hash = blake3::hash(data);
    hex::encode(hash.as_bytes())
}

fn calculate_sha512(data: &[u8]) -> String {
    use sha2::{Digest, Sha512};
    let mut hasher = Sha512::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

fn generate_instance_id(pkg: &InstalledPackage) -> String {
    // Generate a unique instance ID based on USE flags and build time
    let mut hasher = blake3::Hasher::new();
    hasher.update(pkg.installed_at.to_rfc3339().as_bytes());
    for flag in &pkg.use_flags {
        hasher.update(flag.as_bytes());
    }
    let hash = hasher.finalize();
    hex::encode(&hash.as_bytes()[..8])
}

fn parse_simple_version(s: &str) -> std::result::Result<semver::Version, semver::Error> {
    let parts: Vec<&str> = s.split('.').collect();
    match parts.len() {
        1 => format!("{}.0.0", parts[0]).parse(),
        2 => format!("{}.{}.0", parts[0], parts[1]).parse(),
        _ => s.parse(),
    }
}

/// Format binary package info for display
pub fn format_binary_package(pkg: &BinaryPackage) -> String {
    let mut output = String::new();

    output.push_str(&format!("Package: {}/{}\n", pkg.id.category, pkg.id.name));
    output.push_str(&format!("Version: {}\n", pkg.version));
    output.push_str(&format!("Slot: {}\n", pkg.slot));
    output.push_str(&format!("Size: {} bytes\n", pkg.size));
    output.push_str(&format!("Installed Size: {} bytes\n", pkg.installed_size));
    output.push_str(&format!("Compression: {}\n", pkg.compression));
    output.push_str(&format!("Build Time: {}\n", pkg.build_time));
    output.push_str(&format!("Build Host: {}\n", pkg.build_host));
    output.push_str(&format!("Architecture: {}\n", pkg.arch));

    if !pkg.use_flags.is_empty() {
        output.push_str(&format!("USE Flags: {}\n", pkg.use_flags.join(" ")));
    }

    if pkg.is_signed() {
        output.push_str("Signed: Yes\n");
    } else {
        output.push_str("Signed: No\n");
    }

    if let Some(ref instance_id) = pkg.instance_id {
        output.push_str(&format!("Instance ID: {}\n", instance_id));
    }

    output.push_str(&format!("BLAKE3: {}\n", pkg.blake3_hash));
    output.push_str(&format!("SHA512: {}\n", pkg.sha512_hash));

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binpkg_compression() {
        assert_eq!(BinpkgCompression::Zstd.extension(), "tar.zst");
        assert_eq!(BinpkgCompression::Gzip.extension(), "tar.gz");
        assert_eq!(
            BinpkgCompression::from_extension("tar.zst"),
            Some(BinpkgCompression::Zstd)
        );
    }

    #[test]
    fn test_binary_package_filename() {
        let pkg = BinaryPackage {
            id: PackageId::new("dev-libs", "openssl"),
            version: semver::Version::new(3, 0, 0),
            slot: "0".to_string(),
            description: String::new(),
            build_time: chrono::Utc::now(),
            use_flags: vec!["ssl".to_string()],
            size: 1000,
            installed_size: 5000,
            blake3_hash: String::new(),
            sha512_hash: String::new(),
            build_host: String::new(),
            cflags: String::new(),
            cxxflags: String::new(),
            ldflags: String::new(),
            dependencies: Vec::new(),
            runtime_deps: Vec::new(),
            build_deps: Vec::new(),
            compression: BinpkgCompression::Zstd,
            instance_id: None,
            signature: None,
            path: String::new(),
            repository: String::new(),
            license: String::new(),
            arch: "amd64".to_string(),
            eapi: "8".to_string(),
            format_version: BINPKG_FORMAT_VERSION,
        };

        assert_eq!(pkg.filename(), "openssl-3.0.0.tar.zst");
    }

    #[test]
    fn test_calculate_hashes() {
        let data = b"test data";
        let blake3 = calculate_blake3(data);
        let sha512 = calculate_sha512(data);

        assert!(!blake3.is_empty());
        assert!(!sha512.is_empty());
        assert_eq!(blake3.len(), 64); // BLAKE3 produces 32 bytes = 64 hex chars
        assert_eq!(sha512.len(), 128); // SHA512 produces 64 bytes = 128 hex chars
    }

    #[test]
    fn test_parse_simple_version() {
        assert_eq!(
            parse_simple_version("3").unwrap(),
            semver::Version::new(3, 0, 0)
        );
        assert_eq!(
            parse_simple_version("3.0").unwrap(),
            semver::Version::new(3, 0, 0)
        );
        assert_eq!(
            parse_simple_version("3.0.0").unwrap(),
            semver::Version::new(3, 0, 0)
        );
    }
}
