//! Repository configuration
//!
//! Implements Gentoo-style repos.conf:
//! - Repository definitions
//! - Sync settings
//! - Repository priorities

use crate::{ConfigError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Repository configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReposConfig {
    /// Default repository settings
    pub defaults: RepoDefaults,
    /// Repository definitions
    pub repos: HashMap<String, Repository>,
}

impl ReposConfig {
    /// Create a new repository configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a repository
    pub fn add_repo(&mut self, repo: Repository) {
        self.repos.insert(repo.name.clone(), repo);
    }

    /// Get a repository by name
    pub fn get_repo(&self, name: &str) -> Option<&Repository> {
        self.repos.get(name)
    }

    /// Get a mutable repository by name
    pub fn get_repo_mut(&mut self, name: &str) -> Option<&mut Repository> {
        self.repos.get_mut(name)
    }

    /// Remove a repository
    pub fn remove_repo(&mut self, name: &str) -> Option<Repository> {
        self.repos.remove(name)
    }

    /// Get all repositories sorted by priority
    pub fn repos_by_priority(&self) -> Vec<&Repository> {
        let mut repos: Vec<&Repository> = self.repos.values().collect();
        repos.sort_by_key(|r| std::cmp::Reverse(r.priority));
        repos
    }

    /// Get the main repository
    pub fn main_repo(&self) -> Option<&Repository> {
        if let Some(name) = &self.defaults.main_repo {
            self.repos.get(name)
        } else {
            // Fall back to highest priority repo
            self.repos_by_priority().first().copied()
        }
    }

    /// Set the main repository
    pub fn set_main_repo(&mut self, name: impl Into<String>) {
        self.defaults.main_repo = Some(name.into());
    }

    /// Check if a repository exists
    pub fn has_repo(&self, name: &str) -> bool {
        self.repos.contains_key(name)
    }

    /// Get repository names
    pub fn repo_names(&self) -> Vec<&str> {
        self.repos.keys().map(|s| s.as_str()).collect()
    }

    /// Create default Buckos repository configuration
    pub fn default_buckos() -> Self {
        let mut config = Self::new();

        config.defaults = RepoDefaults {
            main_repo: Some("buckos".to_string()),
            auto_sync: true,
            sync_type: Some(SyncType::Git),
            clone_depth: Some(1),
        };

        config.add_repo(Repository {
            name: "buckos".to_string(),
            location: PathBuf::from("/var/db/repos/buckos"),
            sync_type: SyncType::Git,
            sync_uri: Some("https://github.com/hodgesds/buckos-packages.git".to_string()),
            priority: 1000,
            auto_sync: true,
            clone_depth: Some(1),
            sync_git_verify_commit_signature: false,
            masters: Vec::new(),
            aliases: Vec::new(),
            eclass_overrides: Vec::new(),
            force: false,
        });

        config
    }
}

/// Default settings for repositories
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RepoDefaults {
    /// Default main repository
    pub main_repo: Option<String>,
    /// Default auto-sync setting
    pub auto_sync: bool,
    /// Default sync type
    pub sync_type: Option<SyncType>,
    /// Default clone depth for git repos
    pub clone_depth: Option<u32>,
}

/// A single repository definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    /// Repository name
    pub name: String,
    /// Local location
    pub location: PathBuf,
    /// Sync type
    pub sync_type: SyncType,
    /// Sync URI
    pub sync_uri: Option<String>,
    /// Priority (higher = more important)
    pub priority: i32,
    /// Auto-sync enabled
    pub auto_sync: bool,
    /// Git clone depth (None = full clone)
    pub clone_depth: Option<u32>,
    /// Verify git commit signatures
    pub sync_git_verify_commit_signature: bool,
    /// Master repositories
    pub masters: Vec<String>,
    /// Repository aliases
    pub aliases: Vec<String>,
    /// Eclass overrides
    pub eclass_overrides: Vec<String>,
    /// Force sync even if unchanged
    pub force: bool,
}

impl Default for Repository {
    fn default() -> Self {
        Self {
            name: String::new(),
            location: PathBuf::new(),
            sync_type: SyncType::Git,
            sync_uri: None,
            priority: 0,
            auto_sync: true,
            clone_depth: None,
            sync_git_verify_commit_signature: false,
            masters: Vec::new(),
            aliases: Vec::new(),
            eclass_overrides: Vec::new(),
            force: false,
        }
    }
}

impl Repository {
    /// Create a new repository
    pub fn new(name: impl Into<String>, location: impl Into<PathBuf>) -> Self {
        Self {
            name: name.into(),
            location: location.into(),
            ..Default::default()
        }
    }

    /// Set sync URI
    pub fn with_sync_uri(mut self, uri: impl Into<String>) -> Self {
        self.sync_uri = Some(uri.into());
        self
    }

    /// Set sync type
    pub fn with_sync_type(mut self, sync_type: SyncType) -> Self {
        self.sync_type = sync_type;
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Set auto-sync
    pub fn with_auto_sync(mut self, auto_sync: bool) -> Self {
        self.auto_sync = auto_sync;
        self
    }

    /// Add a master repository
    pub fn with_master(mut self, master: impl Into<String>) -> Self {
        self.masters.push(master.into());
        self
    }

    /// Check if this is a local repository
    pub fn is_local(&self) -> bool {
        self.sync_type == SyncType::Local || self.sync_uri.is_none()
    }

    /// Get the packages directory
    pub fn packages_dir(&self) -> PathBuf {
        self.location.clone()
    }

    /// Get the profiles directory
    pub fn profiles_dir(&self) -> PathBuf {
        self.location.join("profiles")
    }

    /// Get the eclass directory
    pub fn eclass_dir(&self) -> PathBuf {
        self.location.join("eclass")
    }

    /// Get the metadata directory
    pub fn metadata_dir(&self) -> PathBuf {
        self.location.join("metadata")
    }
}

/// Repository sync type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncType {
    /// Git repository
    Git,
    /// Rsync
    Rsync,
    /// HTTP/HTTPS tarball
    Http,
    /// Local directory (no sync)
    Local,
    /// CVS (legacy)
    Cvs,
    /// SVN (legacy)
    Svn,
    /// Mercurial
    Mercurial,
    /// WebRsync (for initial sync)
    WebRsync,
}

impl Default for SyncType {
    fn default() -> Self {
        Self::Git
    }
}

impl SyncType {
    /// Get the sync type name
    pub fn as_str(&self) -> &'static str {
        match self {
            SyncType::Git => "git",
            SyncType::Rsync => "rsync",
            SyncType::Http => "http",
            SyncType::Local => "local",
            SyncType::Cvs => "cvs",
            SyncType::Svn => "svn",
            SyncType::Mercurial => "mercurial",
            SyncType::WebRsync => "webrsync",
        }
    }
}

impl std::str::FromStr for SyncType {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "git" => Ok(SyncType::Git),
            "rsync" => Ok(SyncType::Rsync),
            "http" | "https" => Ok(SyncType::Http),
            "local" => Ok(SyncType::Local),
            "cvs" => Ok(SyncType::Cvs),
            "svn" => Ok(SyncType::Svn),
            "mercurial" | "hg" => Ok(SyncType::Mercurial),
            "webrsync" => Ok(SyncType::WebRsync),
            _ => Err(ConfigError::Invalid(format!("unknown sync type: {}", s))),
        }
    }
}

/// Parse a repos.conf file or directory
pub fn parse_repos_conf(path: &Path) -> Result<ReposConfig> {
    let mut config = ReposConfig::new();

    if path.is_dir() {
        // Read all .conf files in directory
        let mut entries: Vec<_> = std::fs::read_dir(path)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|s| s == "conf").unwrap_or(false))
            .collect();

        // Sort by name
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let content = std::fs::read_to_string(entry.path())?;
            parse_repos_conf_content(&content, &mut config)?;
        }
    } else {
        let content = std::fs::read_to_string(path)?;
        parse_repos_conf_content(&content, &mut config)?;
    }

    Ok(config)
}

/// Parse repos.conf content (INI-like format)
fn parse_repos_conf_content(content: &str, config: &mut ReposConfig) -> Result<()> {
    let mut current_section: Option<String> = None;
    let mut current_values: HashMap<String, String> = HashMap::new();

    for line in content.lines() {
        let line = line.trim();

        // Skip comments and empty lines
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Section header
        if line.starts_with('[') && line.ends_with(']') {
            // Save previous section
            if let Some(section) = current_section.take() {
                save_section(&section, &current_values, config)?;
            }

            current_section = Some(line[1..line.len() - 1].to_string());
            current_values.clear();
            continue;
        }

        // Key-value pair
        if let Some(eq_idx) = line.find('=') {
            let key = line[..eq_idx].trim().to_string();
            let value = line[eq_idx + 1..].trim().to_string();
            current_values.insert(key, value);
        }
    }

    // Save last section
    if let Some(section) = current_section {
        save_section(&section, &current_values, config)?;
    }

    Ok(())
}

fn save_section(
    section: &str,
    values: &HashMap<String, String>,
    config: &mut ReposConfig,
) -> Result<()> {
    if section == "DEFAULT" {
        // Default settings
        if let Some(v) = values.get("main-repo") {
            config.defaults.main_repo = Some(v.clone());
        }
        if let Some(v) = values.get("auto-sync") {
            config.defaults.auto_sync = v.to_lowercase() == "yes" || v == "true";
        }
        if let Some(v) = values.get("sync-type") {
            config.defaults.sync_type = Some(v.parse()?);
        }
        if let Some(v) = values.get("clone-depth") {
            config.defaults.clone_depth = v.parse().ok();
        }
    } else {
        // Repository definition
        let mut repo = Repository::default();
        repo.name = section.to_string();

        if let Some(v) = values.get("location") {
            repo.location = PathBuf::from(v);
        }
        if let Some(v) = values.get("sync-type") {
            repo.sync_type = v.parse()?;
        }
        if let Some(v) = values.get("sync-uri") {
            repo.sync_uri = Some(v.clone());
        }
        if let Some(v) = values.get("priority") {
            repo.priority = v.parse().unwrap_or(0);
        }
        if let Some(v) = values.get("auto-sync") {
            repo.auto_sync = v.to_lowercase() == "yes" || v == "true";
        }
        if let Some(v) = values.get("clone-depth") {
            repo.clone_depth = v.parse().ok();
        }
        if let Some(v) = values.get("sync-git-verify-commit-signature") {
            repo.sync_git_verify_commit_signature = v.to_lowercase() == "yes" || v == "true";
        }
        if let Some(v) = values.get("masters") {
            repo.masters = v.split_whitespace().map(|s| s.to_string()).collect();
        }

        config.repos.insert(repo.name.clone(), repo);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_buckos() {
        let config = ReposConfig::default_buckos();
        assert!(config.has_repo("buckos"));

        let repo = config.get_repo("buckos").unwrap();
        assert_eq!(repo.sync_type, SyncType::Git);
        assert!(repo.sync_uri.as_ref().unwrap().contains("github.com"));
    }

    #[test]
    fn test_repos_by_priority() {
        let mut config = ReposConfig::new();

        config.add_repo(Repository::new("low", "/var/db/repos/low").with_priority(10));
        config.add_repo(Repository::new("high", "/var/db/repos/high").with_priority(100));
        config.add_repo(Repository::new("mid", "/var/db/repos/mid").with_priority(50));

        let repos = config.repos_by_priority();
        assert_eq!(repos[0].name, "high");
        assert_eq!(repos[1].name, "mid");
        assert_eq!(repos[2].name, "low");
    }

    #[test]
    fn test_parse_repos_conf() {
        let content = r#"
[DEFAULT]
main-repo = gentoo
auto-sync = yes

[gentoo]
location = /var/db/repos/gentoo
sync-type = rsync
sync-uri = rsync://rsync.gentoo.org/gentoo-portage
priority = 100

[custom]
location = /var/db/repos/custom
sync-type = git
sync-uri = https://github.com/example/overlay.git
priority = 50
"#;

        let mut config = ReposConfig::new();
        parse_repos_conf_content(content, &mut config).unwrap();

        assert_eq!(config.defaults.main_repo, Some("gentoo".to_string()));
        assert!(config.has_repo("gentoo"));
        assert!(config.has_repo("custom"));

        let gentoo = config.get_repo("gentoo").unwrap();
        assert_eq!(gentoo.sync_type, SyncType::Rsync);
        assert_eq!(gentoo.priority, 100);
    }
}
