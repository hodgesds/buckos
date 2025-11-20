//! Package repository management
//!
//! Handles syncing and querying package repositories.

use crate::config::{Config, RepositoryConfig, SyncType};
use crate::{
    Dependency, Error, PackageId, PackageInfo, Result, UseCondition, UseFlag, VersionSpec,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;
use tracing::{info, warn};

/// Repository manager
pub struct RepositoryManager {
    repos: Vec<RepositoryConfig>,
    cache_dir: PathBuf,
}

impl RepositoryManager {
    /// Create a new repository manager
    pub fn new(config: &Config) -> Result<Self> {
        let cache_dir = config.cache_dir.join("repos");
        std::fs::create_dir_all(&cache_dir)?;

        Ok(Self {
            repos: config.repositories.clone(),
            cache_dir,
        })
    }

    /// Sync all repositories
    pub async fn sync_all(&self) -> Result<()> {
        for repo in &self.repos {
            if repo.auto_sync {
                self.sync_repo_config(repo).await?;
            }
        }
        Ok(())
    }

    /// Sync a single repository by name
    pub async fn sync_repo(&self, repo_name: &str) -> Result<()> {
        let repo = self
            .repos
            .iter()
            .find(|r| r.name == repo_name)
            .ok_or_else(|| Error::RepositoryNotFound(repo_name.to_string()))?;

        self.sync_repo_config(repo).await
    }

    /// Sync a single repository by config
    async fn sync_repo_config(&self, repo: &RepositoryConfig) -> Result<()> {
        info!("Syncing repository: {}", repo.name);

        match repo.sync_type {
            SyncType::Git => self.sync_git(repo).await,
            SyncType::Rsync => self.sync_rsync(repo).await,
            SyncType::Http => self.sync_http(repo).await,
            SyncType::Local => Ok(()), // No sync needed
        }
    }

    async fn sync_git(&self, repo: &RepositoryConfig) -> Result<()> {
        let repo_path = &repo.location;

        if repo_path.exists() {
            // Pull updates
            let output = Command::new("git")
                .args(["pull", "--ff-only"])
                .current_dir(repo_path)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .await
                .map_err(|e| Error::RepositoryError(format!("Git pull failed: {}", e)))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(Error::RepositoryError(format!(
                    "Git pull failed: {}",
                    stderr
                )));
            }
        } else {
            // Clone
            let output = Command::new("git")
                .args(["clone", &repo.sync_uri, repo_path.to_str().unwrap()])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .await
                .map_err(|e| Error::RepositoryError(format!("Git clone failed: {}", e)))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(Error::RepositoryError(format!(
                    "Git clone failed: {}",
                    stderr
                )));
            }
        }

        info!("Repository {} synced successfully", repo.name);
        Ok(())
    }

    async fn sync_rsync(&self, repo: &RepositoryConfig) -> Result<()> {
        let repo_path = &repo.location;
        std::fs::create_dir_all(repo_path)?;

        let output = Command::new("rsync")
            .args([
                "-av",
                "--delete",
                &repo.sync_uri,
                repo_path.to_str().unwrap(),
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| Error::RepositoryError(format!("Rsync failed: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::RepositoryError(format!("Rsync failed: {}", stderr)));
        }

        Ok(())
    }

    async fn sync_http(&self, repo: &RepositoryConfig) -> Result<()> {
        // Download repository index
        let client = reqwest::Client::new();
        let index_url = format!("{}/index.json", repo.sync_uri);

        let response = client
            .get(&index_url)
            .send()
            .await
            .map_err(|e| Error::RepositoryError(format!("HTTP sync failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::RepositoryError(format!(
                "HTTP sync failed: {}",
                response.status()
            )));
        }

        let index_data = response
            .bytes()
            .await
            .map_err(|e| Error::RepositoryError(format!("Failed to read index: {}", e)))?;

        // Save index
        let index_path = self.cache_dir.join(format!("{}.json", repo.name));
        std::fs::write(&index_path, &index_data)?;

        Ok(())
    }

    /// Search for packages
    pub async fn search(&self, query: &str) -> Result<Vec<PackageInfo>> {
        let mut results = Vec::new();

        for repo in &self.repos {
            let packages = self.search_repo(repo, query).await?;
            results.extend(packages);
        }

        // Sort by relevance (name match first)
        results.sort_by(|a, b| {
            let a_name_match = a.id.name.contains(query);
            let b_name_match = b.id.name.contains(query);

            match (a_name_match, b_name_match) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.id.name.cmp(&b.id.name),
            }
        });

        Ok(results)
    }

    async fn search_repo(&self, repo: &RepositoryConfig, query: &str) -> Result<Vec<PackageInfo>> {
        let packages = self.load_repo_packages(repo).await?;
        let query_lower = query.to_lowercase();

        Ok(packages
            .into_iter()
            .filter(|pkg| {
                pkg.id.name.to_lowercase().contains(&query_lower)
                    || pkg.id.category.to_lowercase().contains(&query_lower)
                    || pkg.description.to_lowercase().contains(&query_lower)
            })
            .collect())
    }

    /// Get package information
    pub async fn get_info(&self, name: &str) -> Result<Option<PackageInfo>> {
        for repo in &self.repos {
            let packages = self.load_repo_packages(repo).await?;
            if let Some(pkg) = packages.into_iter().find(|p| p.id.name == name) {
                return Ok(Some(pkg));
            }
        }
        Ok(None)
    }

    /// Get latest version of a package
    pub async fn get_latest(&self, name: &str) -> Result<Option<PackageInfo>> {
        let mut best: Option<PackageInfo> = None;

        for repo in &self.repos {
            let packages = self.load_repo_packages(repo).await?;
            for pkg in packages {
                if pkg.id.name == name {
                    if let Some(ref current) = best {
                        if pkg.version > current.version {
                            best = Some(pkg);
                        }
                    } else {
                        best = Some(pkg);
                    }
                }
            }
        }

        Ok(best)
    }

    /// Get all available packages
    pub async fn get_all_packages(&self) -> Result<Vec<PackageInfo>> {
        let mut all_packages = Vec::new();

        for repo in &self.repos {
            let packages = self.load_repo_packages(repo).await?;
            all_packages.extend(packages);
        }

        Ok(all_packages)
    }

    /// Load packages from a repository
    async fn load_repo_packages(&self, repo: &RepositoryConfig) -> Result<Vec<PackageInfo>> {
        // Look for package metadata in the repository
        let packages_dir = repo.location.join("packages");

        if !packages_dir.exists() {
            return Ok(Vec::new());
        }

        let mut packages = Vec::new();

        // Walk through category directories
        for category_entry in std::fs::read_dir(&packages_dir)? {
            let category_entry = category_entry?;
            if !category_entry.file_type()?.is_dir() {
                continue;
            }

            let category = category_entry.file_name().to_string_lossy().to_string();

            // Walk through package directories
            for pkg_entry in std::fs::read_dir(category_entry.path())? {
                let pkg_entry = pkg_entry?;
                if !pkg_entry.file_type()?.is_dir() {
                    continue;
                }

                let pkg_name = pkg_entry.file_name().to_string_lossy().to_string();
                let metadata_path = pkg_entry.path().join("metadata.json");

                if metadata_path.exists() {
                    match self.load_package_metadata(&metadata_path, &category, &pkg_name) {
                        Ok(pkg) => packages.push(pkg),
                        Err(e) => {
                            warn!("Failed to load {}/{}: {}", category, pkg_name, e);
                        }
                    }
                }
            }
        }

        Ok(packages)
    }

    fn load_package_metadata(
        &self,
        path: &Path,
        category: &str,
        name: &str,
    ) -> Result<PackageInfo> {
        let content = std::fs::read_to_string(path)?;
        let metadata: PackageMetadata = serde_json::from_str(&content)?;

        Ok(PackageInfo {
            id: PackageId::new(category, name),
            version: semver::Version::parse(&metadata.version)
                .map_err(|_| Error::InvalidVersion(metadata.version.clone()))?,
            slot: metadata.slot.unwrap_or_else(|| "0".to_string()),
            description: metadata.description,
            homepage: metadata.homepage,
            license: metadata.license,
            keywords: metadata.keywords,
            use_flags: metadata
                .use_flags
                .into_iter()
                .map(|(name, desc)| UseFlag {
                    name,
                    description: desc,
                    default: false,
                })
                .collect(),
            dependencies: self.parse_dependencies(&metadata.dependencies)?,
            build_dependencies: self.parse_dependencies(&metadata.build_dependencies)?,
            runtime_dependencies: self.parse_dependencies(&metadata.runtime_dependencies)?,
            source_url: metadata.source_url,
            source_hash: metadata.source_hash,
            buck_target: format!("//packages/{}/{}:package", category, name),
            size: metadata.size.unwrap_or(0),
            installed_size: metadata.installed_size.unwrap_or(0),
        })
    }

    fn parse_dependencies(&self, deps: &[String]) -> Result<Vec<Dependency>> {
        let mut result = Vec::new();

        for dep_str in deps {
            let pkg_id = PackageId::parse(dep_str)
                .ok_or_else(|| Error::InvalidPackageSpec(dep_str.clone()))?;

            result.push(Dependency {
                package: pkg_id,
                version: VersionSpec::Any,
                slot: None,
                use_flags: UseCondition::Always,
                optional: false,
                build_time: true,
                run_time: true,
            });
        }

        Ok(result)
    }
}

/// Package metadata from repository
#[derive(Debug, serde::Deserialize)]
struct PackageMetadata {
    version: String,
    description: String,
    slot: Option<String>,
    homepage: Option<String>,
    license: String,
    keywords: Vec<String>,
    #[serde(default)]
    use_flags: HashMap<String, String>,
    #[serde(default)]
    dependencies: Vec<String>,
    #[serde(default)]
    build_dependencies: Vec<String>,
    #[serde(default)]
    runtime_dependencies: Vec<String>,
    source_url: Option<String>,
    source_hash: Option<String>,
    size: Option<u64>,
    installed_size: Option<u64>,
}
