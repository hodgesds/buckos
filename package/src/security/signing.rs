//! Package signing and verification
//!
//! Provides GPG-based signing and verification for packages, manifests, and repositories.
//! Compatible with Gentoo's gemato and Manifest signing.

use crate::{Error, PackageId, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// GPG key information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningKey {
    /// Key fingerprint (40 hex characters)
    pub fingerprint: String,
    /// Key ID (last 16 characters of fingerprint)
    pub key_id: String,
    /// User ID (name and email)
    pub user_id: String,
    /// Creation date
    pub created: chrono::NaiveDate,
    /// Expiration date (if set)
    pub expires: Option<chrono::NaiveDate>,
    /// Key algorithm (RSA, DSA, etc.)
    pub algorithm: String,
    /// Key size in bits
    pub key_size: u32,
    /// Trust level
    pub trust: TrustLevel,
    /// Whether this is a secret key
    pub is_secret: bool,
}

/// Trust level for a key
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrustLevel {
    /// Unknown trust
    Unknown,
    /// Never trust this key
    Never,
    /// Marginal trust
    Marginal,
    /// Full trust
    Full,
    /// Ultimate trust (own key)
    Ultimate,
}

impl std::fmt::Display for TrustLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrustLevel::Unknown => write!(f, "unknown"),
            TrustLevel::Never => write!(f, "never"),
            TrustLevel::Marginal => write!(f, "marginal"),
            TrustLevel::Full => write!(f, "full"),
            TrustLevel::Ultimate => write!(f, "ultimate"),
        }
    }
}

/// Signature verification result
#[derive(Debug, Clone)]
pub struct SignatureVerification {
    /// Whether the signature is valid
    pub valid: bool,
    /// Key that made the signature
    pub key_id: String,
    /// User ID of the signer
    pub signer: String,
    /// When the signature was made
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,
    /// Trust level of the signing key
    pub trust: TrustLevel,
    /// Any warnings or issues
    pub warnings: Vec<String>,
}

/// Manifest entry with hash information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestEntry {
    /// File type (DIST, EBUILD, MISC, AUX)
    pub file_type: ManifestFileType,
    /// File path relative to package directory
    pub path: String,
    /// File size in bytes
    pub size: u64,
    /// BLAKE2B hash
    pub blake2b: Option<String>,
    /// SHA512 hash
    pub sha512: Option<String>,
}

/// Type of file in manifest
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ManifestFileType {
    /// Distfile (source tarball)
    Dist,
    /// Ebuild file
    Ebuild,
    /// Miscellaneous file
    Misc,
    /// Auxiliary file (patches, etc.)
    Aux,
}

impl std::fmt::Display for ManifestFileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ManifestFileType::Dist => write!(f, "DIST"),
            ManifestFileType::Ebuild => write!(f, "EBUILD"),
            ManifestFileType::Misc => write!(f, "MISC"),
            ManifestFileType::Aux => write!(f, "AUX"),
        }
    }
}

/// Package manifest with signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManifest {
    /// Package ID
    pub package: PackageId,
    /// Manifest entries
    pub entries: Vec<ManifestEntry>,
    /// GPG signature (if signed)
    pub signature: Option<String>,
}

/// Repository signing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositorySigningConfig {
    /// Repository name
    pub repo_name: String,
    /// Signing key fingerprint
    pub signing_key: String,
    /// Signature type
    pub signature_type: SignatureType,
    /// Whether to sign individual packages
    pub sign_packages: bool,
    /// Whether to sign the repository metadata
    pub sign_metadata: bool,
}

/// Type of signature
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignatureType {
    /// Detached GPG signature (.sig file)
    Detached,
    /// Clear-signed (inline signature)
    ClearSign,
    /// OpenPGP signature
    OpenPgp,
}

/// Package signing manager
pub struct SigningManager {
    /// GPG home directory
    gpg_home: PathBuf,
    /// Default signing key
    default_key: Option<String>,
    /// Trusted keys
    trusted_keys: Vec<String>,
    /// Key cache
    key_cache: HashMap<String, SigningKey>,
}

impl SigningManager {
    /// Create a new signing manager with default GPG home
    pub fn new() -> Result<Self> {
        let gpg_home = dirs::home_dir()
            .ok_or_else(|| Error::Config("Cannot find home directory".to_string()))?
            .join(".gnupg");

        Ok(Self {
            gpg_home,
            default_key: None,
            trusted_keys: Vec::new(),
            key_cache: HashMap::new(),
        })
    }

    /// Create with custom GPG home directory
    pub fn with_gpg_home(gpg_home: PathBuf) -> Self {
        Self {
            gpg_home,
            default_key: None,
            trusted_keys: Vec::new(),
            key_cache: HashMap::new(),
        }
    }

    /// Set the default signing key
    pub fn set_default_key(&mut self, key_id: &str) {
        self.default_key = Some(key_id.to_string());
    }

    /// Add a trusted key
    pub fn add_trusted_key(&mut self, key_id: &str) {
        if !self.trusted_keys.contains(&key_id.to_string()) {
            self.trusted_keys.push(key_id.to_string());
        }
    }

    /// Check if GPG is available
    pub fn is_gpg_available(&self) -> bool {
        Command::new("gpg")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// List available signing keys
    pub fn list_keys(&mut self, secret_only: bool) -> Result<Vec<SigningKey>> {
        let mut cmd = Command::new("gpg");
        cmd.args([
            "--homedir",
            self.gpg_home.to_str().unwrap_or("~/.gnupg"),
            "--list-keys",
            "--with-colons",
            "--with-fingerprint",
        ]);

        if secret_only {
            cmd.arg("--list-secret-keys");
        }

        let output = cmd
            .output()
            .map_err(|e| Error::Signing(format!("Failed to run gpg: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Signing(format!("GPG error: {}", stderr)));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let keys = self.parse_key_listing(&stdout, secret_only);

        // Update cache
        for key in &keys {
            self.key_cache.insert(key.fingerprint.clone(), key.clone());
        }

        Ok(keys)
    }

    /// Parse GPG key listing output
    fn parse_key_listing(&self, output: &str, is_secret: bool) -> Vec<SigningKey> {
        let mut keys = Vec::new();
        let mut current_key: Option<SigningKey> = None;

        for line in output.lines() {
            let fields: Vec<&str> = line.split(':').collect();
            if fields.is_empty() {
                continue;
            }

            match fields[0] {
                "pub" | "sec" => {
                    // Save previous key if any
                    if let Some(key) = current_key.take() {
                        keys.push(key);
                    }

                    // Parse key info
                    let key_size = fields.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
                    let algorithm = fields
                        .get(3)
                        .map(|s| match *s {
                            "1" => "RSA",
                            "16" => "ElGamal",
                            "17" => "DSA",
                            "18" => "ECDH",
                            "19" => "ECDSA",
                            "22" => "EdDSA",
                            _ => "Unknown",
                        })
                        .unwrap_or("Unknown")
                        .to_string();

                    let created = fields
                        .get(5)
                        .and_then(|s| s.parse::<i64>().ok())
                        .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0))
                        .map(|dt| dt.date_naive())
                        .unwrap_or_else(|| chrono::Local::now().date_naive());

                    let expires = fields
                        .get(6)
                        .filter(|s| !s.is_empty())
                        .and_then(|s| s.parse::<i64>().ok())
                        .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0))
                        .map(|dt| dt.date_naive());

                    let trust = match fields.get(1).copied() {
                        Some("o") => TrustLevel::Unknown,
                        Some("i") | Some("d") | Some("r") => TrustLevel::Never,
                        Some("n") => TrustLevel::Never,
                        Some("m") => TrustLevel::Marginal,
                        Some("f") => TrustLevel::Full,
                        Some("u") => TrustLevel::Ultimate,
                        _ => TrustLevel::Unknown,
                    };

                    current_key = Some(SigningKey {
                        fingerprint: String::new(),
                        key_id: fields.get(4).unwrap_or(&"").to_string(),
                        user_id: String::new(),
                        created,
                        expires,
                        algorithm,
                        key_size,
                        trust,
                        is_secret,
                    });
                }
                "fpr" => {
                    // Fingerprint
                    if let Some(ref mut key) = current_key {
                        if let Some(fpr) = fields.get(9) {
                            key.fingerprint = fpr.to_string();
                            // Key ID is last 16 characters of fingerprint
                            if fpr.len() >= 16 {
                                key.key_id = fpr[fpr.len() - 16..].to_string();
                            }
                        }
                    }
                }
                "uid" => {
                    // User ID
                    if let Some(ref mut key) = current_key {
                        if key.user_id.is_empty() {
                            if let Some(uid) = fields.get(9) {
                                key.user_id = uid.to_string();
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // Don't forget the last key
        if let Some(key) = current_key {
            keys.push(key);
        }

        keys
    }

    /// Import a key from a file or keyserver
    pub fn import_key(&self, source: &str) -> Result<String> {
        let mut cmd = Command::new("gpg");
        cmd.args(["--homedir", self.gpg_home.to_str().unwrap_or("~/.gnupg")]);

        // Check if source is a file or key ID
        let path = Path::new(source);
        if path.exists() {
            cmd.args(["--import", source]);
        } else if source.starts_with("http://") || source.starts_with("https://") {
            // Fetch from URL
            cmd.args(["--fetch-keys", source]);
        } else {
            // Assume it's a key ID, fetch from keyserver
            cmd.args([
                "--keyserver",
                "hkps://keys.openpgp.org",
                "--recv-keys",
                source,
            ]);
        }

        let output = cmd
            .output()
            .map_err(|e| Error::Signing(format!("Failed to import key: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Signing(format!("Failed to import key: {}", stderr)));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.to_string())
    }

    /// Export a key to a file
    pub fn export_key(&self, key_id: &str, output_path: &Path, armor: bool) -> Result<()> {
        let mut cmd = Command::new("gpg");
        cmd.args([
            "--homedir",
            self.gpg_home.to_str().unwrap_or("~/.gnupg"),
            "--export",
        ]);

        if armor {
            cmd.arg("--armor");
        }

        cmd.arg(key_id);
        cmd.args(["-o", output_path.to_str().unwrap_or("")]);

        let output = cmd
            .output()
            .map_err(|e| Error::Signing(format!("Failed to export key: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Signing(format!("Failed to export key: {}", stderr)));
        }

        Ok(())
    }

    /// Sign a file with detached signature
    pub fn sign_file(&self, file_path: &Path, key_id: Option<&str>) -> Result<PathBuf> {
        let key = key_id
            .map(|s| s.to_string())
            .or_else(|| self.default_key.clone())
            .ok_or_else(|| Error::Signing("No signing key specified".to_string()))?;

        let mut cmd = Command::new("gpg");
        cmd.args([
            "--homedir",
            self.gpg_home.to_str().unwrap_or("~/.gnupg"),
            "--detach-sign",
            "--armor",
            "-u",
            &key,
            file_path.to_str().unwrap_or(""),
        ]);

        let output = cmd
            .output()
            .map_err(|e| Error::Signing(format!("Failed to sign file: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Signing(format!("Failed to sign file: {}", stderr)));
        }

        // Return path to signature file
        let sig_path = file_path.with_extension(format!(
            "{}.asc",
            file_path
                .extension()
                .unwrap_or_default()
                .to_str()
                .unwrap_or("")
        ));

        Ok(sig_path)
    }

    /// Sign data and return signature
    pub fn sign_data(&self, data: &[u8], key_id: Option<&str>) -> Result<String> {
        let key = key_id
            .map(|s| s.to_string())
            .or_else(|| self.default_key.clone())
            .ok_or_else(|| Error::Signing("No signing key specified".to_string()))?;

        let mut cmd = Command::new("gpg");
        cmd.args([
            "--homedir",
            self.gpg_home.to_str().unwrap_or("~/.gnupg"),
            "--detach-sign",
            "--armor",
            "-u",
            &key,
        ]);

        let mut child = cmd
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| Error::Signing(format!("Failed to spawn gpg: {}", e)))?;

        // Write data to stdin
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            stdin
                .write_all(data)
                .map_err(|e| Error::Signing(format!("Failed to write to gpg: {}", e)))?;
        }

        let output = child
            .wait_with_output()
            .map_err(|e| Error::Signing(format!("Failed to wait for gpg: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Signing(format!("Failed to sign data: {}", stderr)));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Verify a file signature
    pub fn verify_file(
        &self,
        file_path: &Path,
        sig_path: Option<&Path>,
    ) -> Result<SignatureVerification> {
        let default_sig = file_path.with_extension(format!(
            "{}.asc",
            file_path
                .extension()
                .unwrap_or_default()
                .to_str()
                .unwrap_or("")
        ));
        let sig = sig_path.unwrap_or(&default_sig);

        let mut cmd = Command::new("gpg");
        cmd.args([
            "--homedir",
            self.gpg_home.to_str().unwrap_or("~/.gnupg"),
            "--verify",
            "--status-fd",
            "1",
            sig.to_str().unwrap_or(""),
            file_path.to_str().unwrap_or(""),
        ]);

        let output = cmd
            .output()
            .map_err(|e| Error::Signing(format!("Failed to verify signature: {}", e)))?;

        self.parse_verification_output(&output)
    }

    /// Verify a signature on data
    pub fn verify_data(&self, data: &[u8], signature: &str) -> Result<SignatureVerification> {
        // Write signature to temp file
        let temp_dir = tempfile::tempdir()
            .map_err(|e| Error::Signing(format!("Failed to create temp dir: {}", e)))?;

        let sig_path = temp_dir.path().join("signature.asc");
        let data_path = temp_dir.path().join("data");

        std::fs::write(&sig_path, signature)
            .map_err(|e| Error::Signing(format!("Failed to write signature: {}", e)))?;
        std::fs::write(&data_path, data)
            .map_err(|e| Error::Signing(format!("Failed to write data: {}", e)))?;

        self.verify_file(&data_path, Some(&sig_path))
    }

    /// Parse GPG verification output
    fn parse_verification_output(
        &self,
        output: &std::process::Output,
    ) -> Result<SignatureVerification> {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let mut verification = SignatureVerification {
            valid: output.status.success(),
            key_id: String::new(),
            signer: String::new(),
            timestamp: None,
            trust: TrustLevel::Unknown,
            warnings: Vec::new(),
        };

        // Parse status output
        for line in stdout.lines().chain(stderr.lines()) {
            if line.contains("[GNUPG:] GOODSIG") {
                verification.valid = true;
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    verification.key_id = parts[2].to_string();
                }
                if parts.len() >= 4 {
                    verification.signer = parts[3..].join(" ");
                }
            } else if line.contains("[GNUPG:] BADSIG") {
                verification.valid = false;
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    verification.key_id = parts[2].to_string();
                }
            } else if line.contains("[GNUPG:] TRUST_") {
                if line.contains("TRUST_ULTIMATE") {
                    verification.trust = TrustLevel::Ultimate;
                } else if line.contains("TRUST_FULLY") {
                    verification.trust = TrustLevel::Full;
                } else if line.contains("TRUST_MARGINAL") {
                    verification.trust = TrustLevel::Marginal;
                } else if line.contains("TRUST_NEVER") {
                    verification.trust = TrustLevel::Never;
                }
            } else if line.contains("[GNUPG:] SIG_CREATED") || line.contains("Signature made") {
                // Try to parse timestamp
                if let Some(ts_str) = line.split_whitespace().nth(2) {
                    if let Ok(ts) = ts_str.parse::<i64>() {
                        verification.timestamp = chrono::DateTime::from_timestamp(ts, 0);
                    }
                }
            } else if line.contains("WARNING") {
                verification.warnings.push(line.to_string());
            }
        }

        // Check trusted keys
        if verification.valid
            && !self.trusted_keys.is_empty()
            && !self
                .trusted_keys
                .iter()
                .any(|k| verification.key_id.ends_with(k) || k.ends_with(&verification.key_id))
        {
            verification
                .warnings
                .push("Signature is valid but key is not in trusted keys list".to_string());
        }

        Ok(verification)
    }

    /// Generate a manifest for a package directory
    pub fn generate_manifest(
        &self,
        package_dir: &Path,
        package: &PackageId,
    ) -> Result<PackageManifest> {
        let mut entries = Vec::new();

        // Find all files in the package directory
        for entry in walkdir::WalkDir::new(package_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            let relative = path
                .strip_prefix(package_dir)
                .map_err(|_| Error::Signing("Invalid path".to_string()))?;

            let metadata = std::fs::metadata(path)
                .map_err(|e| Error::Signing(format!("Failed to read file metadata: {}", e)))?;

            // Determine file type
            let file_type = match relative.to_str() {
                Some(s) if s.ends_with(".ebuild") => ManifestFileType::Ebuild,
                Some(s) if s.starts_with("files/") => ManifestFileType::Aux,
                Some(s) if s.contains("distfiles/") => ManifestFileType::Dist,
                _ => ManifestFileType::Misc,
            };

            // Calculate hashes
            let content = std::fs::read(path)
                .map_err(|e| Error::Signing(format!("Failed to read file: {}", e)))?;

            let blake2b = {
                use blake3::Hasher;
                let mut hasher = Hasher::new();
                hasher.update(&content);
                hex::encode(hasher.finalize().as_bytes())
            };

            let sha512 = {
                use sha2::{Digest, Sha512};
                let mut hasher = Sha512::new();
                hasher.update(&content);
                hex::encode(hasher.finalize())
            };

            entries.push(ManifestEntry {
                file_type,
                path: relative.to_string_lossy().to_string(),
                size: metadata.len(),
                blake2b: Some(blake2b),
                sha512: Some(sha512),
            });
        }

        Ok(PackageManifest {
            package: package.clone(),
            entries,
            signature: None,
        })
    }

    /// Sign a manifest
    pub fn sign_manifest(
        &self,
        manifest: &mut PackageManifest,
        key_id: Option<&str>,
    ) -> Result<()> {
        let manifest_text = self.format_manifest(manifest);
        let signature = self.sign_data(manifest_text.as_bytes(), key_id)?;
        manifest.signature = Some(signature);
        Ok(())
    }

    /// Format manifest for signing/verification
    pub fn format_manifest(&self, manifest: &PackageManifest) -> String {
        let mut output = String::new();

        for entry in &manifest.entries {
            output.push_str(&format!(
                "{} {} {}",
                entry.file_type, entry.path, entry.size
            ));

            if let Some(ref hash) = entry.blake2b {
                output.push_str(&format!(" BLAKE2B {}", hash));
            }
            if let Some(ref hash) = entry.sha512 {
                output.push_str(&format!(" SHA512 {}", hash));
            }

            output.push('\n');
        }

        output
    }

    /// Verify a manifest signature
    pub fn verify_manifest(&self, manifest: &PackageManifest) -> Result<SignatureVerification> {
        let signature = manifest
            .signature
            .as_ref()
            .ok_or_else(|| Error::Signing("Manifest is not signed".to_string()))?;

        let manifest_text = self.format_manifest(manifest);
        self.verify_data(manifest_text.as_bytes(), signature)
    }

    /// Verify manifest entries against actual files
    pub fn verify_manifest_files(
        &self,
        manifest: &PackageManifest,
        base_dir: &Path,
    ) -> Result<Vec<ManifestVerifyResult>> {
        let mut results = Vec::new();

        for entry in &manifest.entries {
            let file_path = base_dir.join(&entry.path);
            let result = self.verify_manifest_entry(entry, &file_path);
            results.push(result);
        }

        Ok(results)
    }

    /// Verify a single manifest entry
    fn verify_manifest_entry(
        &self,
        entry: &ManifestEntry,
        file_path: &Path,
    ) -> ManifestVerifyResult {
        if !file_path.exists() {
            return ManifestVerifyResult {
                path: entry.path.clone(),
                status: ManifestVerifyStatus::Missing,
                message: "File not found".to_string(),
            };
        }

        // Check file size
        let metadata = match std::fs::metadata(file_path) {
            Ok(m) => m,
            Err(e) => {
                return ManifestVerifyResult {
                    path: entry.path.clone(),
                    status: ManifestVerifyStatus::Error,
                    message: format!("Failed to read metadata: {}", e),
                };
            }
        };

        if metadata.len() != entry.size {
            return ManifestVerifyResult {
                path: entry.path.clone(),
                status: ManifestVerifyStatus::SizeMismatch,
                message: format!("Expected {} bytes, found {}", entry.size, metadata.len()),
            };
        }

        // Read file content
        let content = match std::fs::read(file_path) {
            Ok(c) => c,
            Err(e) => {
                return ManifestVerifyResult {
                    path: entry.path.clone(),
                    status: ManifestVerifyStatus::Error,
                    message: format!("Failed to read file: {}", e),
                };
            }
        };

        // Verify BLAKE2B hash
        if let Some(ref expected) = entry.blake2b {
            use blake3::Hasher;
            let mut hasher = Hasher::new();
            hasher.update(&content);
            let actual = hex::encode(hasher.finalize().as_bytes());

            if &actual != expected {
                return ManifestVerifyResult {
                    path: entry.path.clone(),
                    status: ManifestVerifyStatus::HashMismatch,
                    message: "BLAKE2B hash mismatch".to_string(),
                };
            }
        }

        // Verify SHA512 hash
        if let Some(ref expected) = entry.sha512 {
            use sha2::{Digest, Sha512};
            let mut hasher = Sha512::new();
            hasher.update(&content);
            let actual = hex::encode(hasher.finalize());

            if &actual != expected {
                return ManifestVerifyResult {
                    path: entry.path.clone(),
                    status: ManifestVerifyStatus::HashMismatch,
                    message: "SHA512 hash mismatch".to_string(),
                };
            }
        }

        ManifestVerifyResult {
            path: entry.path.clone(),
            status: ManifestVerifyStatus::Ok,
            message: "Verified".to_string(),
        }
    }

    /// Write manifest to file
    pub fn write_manifest(&self, manifest: &PackageManifest, path: &Path) -> Result<()> {
        let content = self.format_manifest(manifest);

        if let Some(ref sig) = manifest.signature {
            // Write clear-signed manifest
            let signed = format!(
                "-----BEGIN PGP SIGNED MESSAGE-----\nHash: SHA512\n\n{}\n{}",
                content, sig
            );
            std::fs::write(path, signed)
                .map_err(|e| Error::Signing(format!("Failed to write manifest: {}", e)))?;
        } else {
            std::fs::write(path, content)
                .map_err(|e| Error::Signing(format!("Failed to write manifest: {}", e)))?;
        }

        Ok(())
    }

    /// Read and parse a manifest file
    pub fn read_manifest(&self, path: &Path) -> Result<PackageManifest> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| Error::Signing(format!("Failed to read manifest: {}", e)))?;

        // Check if it's a signed manifest
        let (manifest_content, signature) =
            if content.contains("-----BEGIN PGP SIGNED MESSAGE-----") {
                self.parse_signed_manifest(&content)?
            } else {
                (content, None)
            };

        let entries = self.parse_manifest_content(&manifest_content)?;

        // Extract package ID from path
        let package = PackageId::new(
            path.parent()
                .and_then(|p| p.parent())
                .and_then(|p| p.file_name())
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string())
                .as_str(),
            path.parent()
                .and_then(|p| p.file_name())
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string())
                .as_str(),
        );

        Ok(PackageManifest {
            package,
            entries,
            signature,
        })
    }

    /// Parse a signed manifest
    fn parse_signed_manifest(&self, content: &str) -> Result<(String, Option<String>)> {
        let mut in_content = false;
        let mut in_signature = false;
        let mut manifest_content = String::new();
        let mut signature = String::new();

        for line in content.lines() {
            if line.starts_with("-----BEGIN PGP SIGNED MESSAGE-----") || line.starts_with("Hash:") {
                continue;
            } else if line.starts_with("-----BEGIN PGP SIGNATURE-----") {
                in_content = false;
                in_signature = true;
                signature.push_str(line);
                signature.push('\n');
            } else if line.starts_with("-----END PGP SIGNATURE-----") {
                signature.push_str(line);
                signature.push('\n');
                break;
            } else if in_signature {
                signature.push_str(line);
                signature.push('\n');
            } else if line.is_empty() && !in_content {
                in_content = true;
            } else if in_content {
                manifest_content.push_str(line);
                manifest_content.push('\n');
            }
        }

        Ok((
            manifest_content,
            if signature.is_empty() {
                None
            } else {
                Some(signature)
            },
        ))
    }

    /// Parse manifest content
    fn parse_manifest_content(&self, content: &str) -> Result<Vec<ManifestEntry>> {
        let mut entries = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 3 {
                continue;
            }

            let file_type = match parts[0] {
                "DIST" => ManifestFileType::Dist,
                "EBUILD" => ManifestFileType::Ebuild,
                "MISC" => ManifestFileType::Misc,
                "AUX" => ManifestFileType::Aux,
                _ => continue,
            };

            let path = parts[1].to_string();
            let size: u64 = parts[2].parse().unwrap_or(0);

            let mut blake2b = None;
            let mut sha512 = None;

            // Parse hash values
            let mut i = 3;
            while i < parts.len() {
                match parts[i] {
                    "BLAKE2B" if i + 1 < parts.len() => {
                        blake2b = Some(parts[i + 1].to_string());
                        i += 2;
                    }
                    "SHA512" if i + 1 < parts.len() => {
                        sha512 = Some(parts[i + 1].to_string());
                        i += 2;
                    }
                    _ => i += 1,
                }
            }

            entries.push(ManifestEntry {
                file_type,
                path,
                size,
                blake2b,
                sha512,
            });
        }

        Ok(entries)
    }

    /// Get key information by ID
    pub fn get_key(&mut self, key_id: &str) -> Result<Option<SigningKey>> {
        // Check cache first
        if let Some(key) = self.key_cache.get(key_id) {
            return Ok(Some(key.clone()));
        }

        // Search by fingerprint suffix
        for (fpr, key) in &self.key_cache {
            if fpr.ends_with(key_id) || key.key_id == key_id {
                return Ok(Some(key.clone()));
            }
        }

        // Refresh cache
        self.list_keys(false)?;

        // Try again
        for (fpr, key) in &self.key_cache {
            if fpr.ends_with(key_id) || key.key_id == key_id {
                return Ok(Some(key.clone()));
            }
        }

        Ok(None)
    }

    /// Delete a key
    pub fn delete_key(&self, key_id: &str, secret: bool) -> Result<()> {
        let mut cmd = Command::new("gpg");
        cmd.args([
            "--homedir",
            self.gpg_home.to_str().unwrap_or("~/.gnupg"),
            "--batch",
            "--yes",
        ]);

        if secret {
            cmd.arg("--delete-secret-keys");
        } else {
            cmd.arg("--delete-keys");
        }

        cmd.arg(key_id);

        let output = cmd
            .output()
            .map_err(|e| Error::Signing(format!("Failed to delete key: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Signing(format!("Failed to delete key: {}", stderr)));
        }

        Ok(())
    }

    /// Set trust level for a key
    pub fn set_key_trust(&self, key_id: &str, trust: TrustLevel) -> Result<()> {
        let trust_value = match trust {
            TrustLevel::Unknown => "1",
            TrustLevel::Never => "2",
            TrustLevel::Marginal => "3",
            TrustLevel::Full => "4",
            TrustLevel::Ultimate => "5",
        };

        let mut cmd = Command::new("gpg");
        cmd.args([
            "--homedir",
            self.gpg_home.to_str().unwrap_or("~/.gnupg"),
            "--batch",
            "--command-fd",
            "0",
            "--edit-key",
            key_id,
            "trust",
        ]);

        let mut child = cmd
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| Error::Signing(format!("Failed to spawn gpg: {}", e)))?;

        // Send trust level and save
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            writeln!(stdin, "{}\ny\nquit", trust_value)
                .map_err(|e| Error::Signing(format!("Failed to write to gpg: {}", e)))?;
        }

        let output = child
            .wait_with_output()
            .map_err(|e| Error::Signing(format!("Failed to wait for gpg: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::Signing(format!("Failed to set trust: {}", stderr)));
        }

        Ok(())
    }

    /// Sign a repository (gemato-style)
    pub fn sign_repository(&self, repo_dir: &Path, key_id: Option<&str>) -> Result<()> {
        // Generate Manifest file for the entire repository
        let manifest_path = repo_dir.join("Manifest");

        // Find all package Manifests
        let mut entries = Vec::new();

        for entry in walkdir::WalkDir::new(repo_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name() == "Manifest" && e.path() != manifest_path)
        {
            let path = entry.path();
            let relative = path
                .strip_prefix(repo_dir)
                .map_err(|_| Error::Signing("Invalid path".to_string()))?;

            let metadata = std::fs::metadata(path)
                .map_err(|e| Error::Signing(format!("Failed to read metadata: {}", e)))?;

            let content = std::fs::read(path)
                .map_err(|e| Error::Signing(format!("Failed to read file: {}", e)))?;

            let blake2b = {
                use blake3::Hasher;
                let mut hasher = Hasher::new();
                hasher.update(&content);
                hex::encode(hasher.finalize().as_bytes())
            };

            let sha512 = {
                use sha2::{Digest, Sha512};
                let mut hasher = Sha512::new();
                hasher.update(&content);
                hex::encode(hasher.finalize())
            };

            entries.push(ManifestEntry {
                file_type: ManifestFileType::Misc,
                path: relative.to_string_lossy().to_string(),
                size: metadata.len(),
                blake2b: Some(blake2b),
                sha512: Some(sha512),
            });
        }

        // Create repository manifest
        let mut manifest = PackageManifest {
            package: PackageId::new("repo", "manifest"),
            entries,
            signature: None,
        };

        // Sign the manifest
        self.sign_manifest(&mut manifest, key_id)?;

        // Write to file
        self.write_manifest(&manifest, &manifest_path)?;

        Ok(())
    }

    /// Verify a repository signature
    pub fn verify_repository(&self, repo_dir: &Path) -> Result<SignatureVerification> {
        let manifest_path = repo_dir.join("Manifest");

        if !manifest_path.exists() {
            return Err(Error::Signing("Repository is not signed".to_string()));
        }

        let manifest = self.read_manifest(&manifest_path)?;

        // Verify signature
        let verification = self.verify_manifest(&manifest)?;

        if !verification.valid {
            return Ok(verification);
        }

        // Verify all referenced manifests
        let results = self.verify_manifest_files(&manifest, repo_dir)?;

        let mut warnings = verification.warnings.clone();
        for result in results {
            if result.status != ManifestVerifyStatus::Ok {
                warnings.push(format!("{}: {}", result.path, result.message));
            }
        }

        Ok(SignatureVerification {
            warnings,
            ..verification
        })
    }
}

impl Default for SigningManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            gpg_home: PathBuf::from("/tmp/.gnupg"),
            default_key: None,
            trusted_keys: Vec::new(),
            key_cache: HashMap::new(),
        })
    }
}

/// Result of manifest file verification
#[derive(Debug, Clone)]
pub struct ManifestVerifyResult {
    /// File path
    pub path: String,
    /// Verification status
    pub status: ManifestVerifyStatus,
    /// Status message
    pub message: String,
}

/// Manifest verification status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManifestVerifyStatus {
    /// File verified successfully
    Ok,
    /// File not found
    Missing,
    /// File size mismatch
    SizeMismatch,
    /// Hash mismatch
    HashMismatch,
    /// Error reading file
    Error,
}

/// Format key information for display
pub fn format_key(key: &SigningKey) -> String {
    let mut output = String::new();

    output.push_str(&format!("Key ID: {}\n", key.key_id));
    output.push_str(&format!("Fingerprint: {}\n", key.fingerprint));
    output.push_str(&format!("User ID: {}\n", key.user_id));
    output.push_str(&format!(
        "Algorithm: {} ({} bits)\n",
        key.algorithm, key.key_size
    ));
    output.push_str(&format!("Created: {}\n", key.created));

    if let Some(ref expires) = key.expires {
        output.push_str(&format!("Expires: {}\n", expires));
    }

    output.push_str(&format!("Trust: {}\n", key.trust));

    if key.is_secret {
        output.push_str("Type: Secret key available\n");
    }

    output
}

/// Format verification result for display
pub fn format_verification(verification: &SignatureVerification) -> String {
    let mut output = String::new();

    if verification.valid {
        output.push_str("Good signature from ");
    } else {
        output.push_str("BAD signature from ");
    }

    output.push_str(&format!(
        "{} ({})\n",
        verification.signer, verification.key_id
    ));

    if let Some(ref ts) = verification.timestamp {
        output.push_str(&format!("Signed: {}\n", ts));
    }

    output.push_str(&format!("Trust: {}\n", verification.trust));

    for warning in &verification.warnings {
        output.push_str(&format!("Warning: {}\n", warning));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_level_display() {
        assert_eq!(TrustLevel::Unknown.to_string(), "unknown");
        assert_eq!(TrustLevel::Full.to_string(), "full");
        assert_eq!(TrustLevel::Ultimate.to_string(), "ultimate");
    }

    #[test]
    fn test_manifest_file_type_display() {
        assert_eq!(ManifestFileType::Dist.to_string(), "DIST");
        assert_eq!(ManifestFileType::Ebuild.to_string(), "EBUILD");
    }

    #[test]
    fn test_signing_manager_default() {
        let manager = SigningManager::default();
        assert!(manager.trusted_keys.is_empty());
    }

    #[test]
    fn test_add_trusted_key() {
        let mut manager = SigningManager::default();
        manager.add_trusted_key("ABCD1234");
        manager.add_trusted_key("ABCD1234"); // Duplicate
        assert_eq!(manager.trusted_keys.len(), 1);
    }

    #[test]
    fn test_format_manifest() {
        let manager = SigningManager::default();
        let manifest = PackageManifest {
            package: PackageId::new("dev-libs", "openssl"),
            entries: vec![ManifestEntry {
                file_type: ManifestFileType::Dist,
                path: "openssl-3.0.0.tar.gz".to_string(),
                size: 12345,
                blake2b: Some("abc123".to_string()),
                sha512: Some("def456".to_string()),
            }],
            signature: None,
        };

        let formatted = manager.format_manifest(&manifest);
        assert!(formatted.contains("DIST"));
        assert!(formatted.contains("openssl-3.0.0.tar.gz"));
        assert!(formatted.contains("12345"));
        assert!(formatted.contains("BLAKE2B abc123"));
        assert!(formatted.contains("SHA512 def456"));
    }

    #[test]
    fn test_parse_manifest_content() {
        let manager = SigningManager::default();
        let content = "DIST test.tar.gz 1000 BLAKE2B abc123 SHA512 def456\nEBUILD test.ebuild 500";

        let entries = manager.parse_manifest_content(content).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].file_type, ManifestFileType::Dist);
        assert_eq!(entries[0].size, 1000);
        assert_eq!(entries[1].file_type, ManifestFileType::Ebuild);
    }
}
