//! News system
//!
//! GLEP 42 compatible news items for important notifications to users.

use crate::{Error, PackageId, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// A news item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsItem {
    /// News item name (filename without extension)
    pub name: String,
    /// Title
    pub title: String,
    /// Author
    pub author: String,
    /// Author email
    pub email: Option<String>,
    /// Content type (text/plain or text/restructuredtext)
    pub content_type: String,
    /// Publication date
    pub posted: chrono::NaiveDate,
    /// Revision date
    pub revision: Option<chrono::NaiveDate>,
    /// Display conditions
    pub display_if: Vec<DisplayCondition>,
    /// News content
    pub content: String,
}

/// Condition for displaying a news item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisplayCondition {
    /// Always display
    Always,
    /// Display if package is installed
    Installed(PackageId),
    /// Display if profile is active
    Profile(String),
    /// Display if keyword is accepted
    Keyword(String),
}

/// News item read status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadStatus {
    /// Unread
    Unread,
    /// Read
    Read,
    /// Skipped
    Skipped,
}

/// News manager
pub struct NewsManager {
    /// News items
    items: Vec<NewsItem>,
    /// Read item names
    read_items: HashSet<String>,
    /// News directory
    news_dir: PathBuf,
    /// Read items file
    read_file: PathBuf,
}

impl NewsManager {
    /// Create a new news manager
    pub fn new(news_dir: PathBuf, read_file: PathBuf) -> Self {
        Self {
            items: Vec::new(),
            read_items: HashSet::new(),
            news_dir,
            read_file,
        }
    }

    /// Load news items
    pub fn load(&mut self) -> Result<()> {
        // Load read items
        self.load_read_status()?;

        // Load news items
        if !self.news_dir.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(&self.news_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Repository news directory
                self.load_from_repo(&path)?;
            }
        }

        // Sort by date (newest first)
        self.items.sort_by(|a, b| b.posted.cmp(&a.posted));

        Ok(())
    }

    /// Load news from a repository directory
    fn load_from_repo(&mut self, repo_path: &Path) -> Result<()> {
        let news_path = repo_path.join("metadata/news");
        if !news_path.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(&news_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Each news item is a directory with files inside
                if let Ok(item) = self.parse_news_item(&path) {
                    self.items.push(item);
                }
            }
        }

        Ok(())
    }

    /// Parse a news item directory
    fn parse_news_item(&self, dir: &Path) -> Result<NewsItem> {
        let name = dir.file_name()
            .ok_or_else(|| Error::InvalidPath(dir.to_string_lossy().to_string()))?
            .to_string_lossy()
            .to_string();

        // Read the news file
        let news_file = dir.join(format!("{}.txt", name));
        let content = if news_file.exists() {
            std::fs::read_to_string(&news_file)?
        } else {
            // Try other common names
            let alt_file = dir.join(format!("{}.en.txt", name));
            if alt_file.exists() {
                std::fs::read_to_string(&alt_file)?
            } else {
                return Err(Error::FileNotFound(news_file));
            }
        };

        // Parse the news item
        self.parse_news_content(&name, &content)
    }

    /// Parse news item content
    fn parse_news_content(&self, name: &str, content: &str) -> Result<NewsItem> {
        let mut title = String::new();
        let mut author = String::new();
        let mut email = None;
        let mut content_type = "text/plain".to_string();
        let mut posted = chrono::Local::now().date_naive();
        let mut revision = None;
        let mut display_if = Vec::new();
        let mut body = String::new();
        let mut in_header = true;

        for line in content.lines() {
            if in_header {
                if line.is_empty() {
                    in_header = false;
                    continue;
                }

                if let Some(value) = line.strip_prefix("Title: ") {
                    title = value.trim().to_string();
                } else if let Some(value) = line.strip_prefix("Author: ") {
                    // Parse "Name <email>"
                    if let Some(idx) = value.find('<') {
                        author = value[..idx].trim().to_string();
                        let email_part = &value[idx + 1..];
                        if let Some(end) = email_part.find('>') {
                            email = Some(email_part[..end].to_string());
                        }
                    } else {
                        author = value.trim().to_string();
                    }
                } else if let Some(value) = line.strip_prefix("Content-Type: ") {
                    content_type = value.trim().to_string();
                } else if let Some(value) = line.strip_prefix("Posted: ") {
                    if let Ok(date) = chrono::NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d") {
                        posted = date;
                    }
                } else if let Some(value) = line.strip_prefix("Revision: ") {
                    if let Ok(date) = chrono::NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d") {
                        revision = Some(date);
                    }
                } else if let Some(value) = line.strip_prefix("Display-If-Installed: ") {
                    if let Some(pkg_id) = PackageId::parse(value.trim()) {
                        display_if.push(DisplayCondition::Installed(pkg_id));
                    }
                } else if let Some(value) = line.strip_prefix("Display-If-Profile: ") {
                    display_if.push(DisplayCondition::Profile(value.trim().to_string()));
                } else if let Some(value) = line.strip_prefix("Display-If-Keyword: ") {
                    display_if.push(DisplayCondition::Keyword(value.trim().to_string()));
                }
            } else {
                body.push_str(line);
                body.push('\n');
            }
        }

        // Default to always display if no conditions
        if display_if.is_empty() {
            display_if.push(DisplayCondition::Always);
        }

        Ok(NewsItem {
            name: name.to_string(),
            title,
            author,
            email,
            content_type,
            posted,
            revision,
            display_if,
            content: body,
        })
    }

    /// Load read status from file
    fn load_read_status(&mut self) -> Result<()> {
        if !self.read_file.exists() {
            return Ok(());
        }

        let content = std::fs::read_to_string(&self.read_file)?;
        for line in content.lines() {
            let line = line.trim();
            if !line.is_empty() && !line.starts_with('#') {
                self.read_items.insert(line.to_string());
            }
        }

        Ok(())
    }

    /// Save read status to file
    fn save_read_status(&self) -> Result<()> {
        if let Some(parent) = self.read_file.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content: Vec<String> = self.read_items.iter().cloned().collect();
        std::fs::write(&self.read_file, content.join("\n"))?;

        Ok(())
    }

    /// Get unread news items
    pub fn get_unread(&self) -> Vec<&NewsItem> {
        self.items
            .iter()
            .filter(|item| !self.read_items.contains(&item.name))
            .collect()
    }

    /// Get all news items
    pub fn get_all(&self) -> &[NewsItem] {
        &self.items
    }

    /// Check if news item should be displayed based on conditions
    pub fn should_display(
        &self,
        item: &NewsItem,
        installed: &[PackageId],
        profile: &str,
        keywords: &[String],
    ) -> bool {
        item.display_if.iter().any(|condition| {
            match condition {
                DisplayCondition::Always => true,
                DisplayCondition::Installed(pkg) => installed.contains(pkg),
                DisplayCondition::Profile(p) => profile.contains(p),
                DisplayCondition::Keyword(k) => keywords.contains(k),
            }
        })
    }

    /// Mark a news item as read
    pub fn mark_read(&mut self, name: &str) -> Result<()> {
        self.read_items.insert(name.to_string());
        self.save_read_status()
    }

    /// Mark all news items as read
    pub fn mark_all_read(&mut self) -> Result<()> {
        for item in &self.items {
            self.read_items.insert(item.name.clone());
        }
        self.save_read_status()
    }

    /// Get news item by name
    pub fn get(&self, name: &str) -> Option<&NewsItem> {
        self.items.iter().find(|item| item.name == name)
    }

    /// Get unread count
    pub fn unread_count(&self) -> usize {
        self.items
            .iter()
            .filter(|item| !self.read_items.contains(&item.name))
            .count()
    }

    /// Get read status for an item
    pub fn get_status(&self, name: &str) -> ReadStatus {
        if self.read_items.contains(name) {
            ReadStatus::Read
        } else {
            ReadStatus::Unread
        }
    }

    /// List items by date range
    pub fn by_date_range(
        &self,
        start: chrono::NaiveDate,
        end: chrono::NaiveDate,
    ) -> Vec<&NewsItem> {
        self.items
            .iter()
            .filter(|item| item.posted >= start && item.posted <= end)
            .collect()
    }
}

impl Default for NewsManager {
    fn default() -> Self {
        Self::new(
            PathBuf::from("/var/db/repos"),
            PathBuf::from("/var/lib/buckos/news.read"),
        )
    }
}

/// Format news listing
pub fn format_news_list(items: &[&NewsItem], show_read: bool) -> String {
    if items.is_empty() {
        return "No news items.\n".to_string();
    }

    let mut output = String::new();

    for item in items {
        let date = item.posted.format("%Y-%m-%d");
        output.push_str(&format!(
            "{} [{}] {}\n",
            item.name, date, item.title
        ));
    }

    output
}

/// Format a single news item for display
pub fn format_news_item(item: &NewsItem) -> String {
    let mut output = String::new();

    output.push_str(&format!("Title: {}\n", item.title));
    output.push_str(&format!("Author: {}", item.author));
    if let Some(ref email) = item.email {
        output.push_str(&format!(" <{}>", email));
    }
    output.push('\n');
    output.push_str(&format!("Posted: {}\n", item.posted.format("%Y-%m-%d")));
    if let Some(rev) = item.revision {
        output.push_str(&format!("Revision: {}\n", rev.format("%Y-%m-%d")));
    }
    output.push_str(&format!("\n{}", item.content));

    output
}

/// Show news notification if there are unread items
pub fn news_notification(manager: &NewsManager) -> Option<String> {
    let unread = manager.unread_count();
    if unread > 0 {
        Some(format!(
            ">>> {} unread news item(s). Run 'eselect news read' to view.",
            unread
        ))
    } else {
        None
    }
}

/// eselect news command equivalent
pub fn eselect_news_command(args: &[&str], manager: &mut NewsManager) -> Result<String> {
    match args.first() {
        Some(&"list") => {
            let items: Vec<_> = manager.get_all().iter().collect();
            Ok(format_news_list(&items, true))
        }
        Some(&"count") => {
            Ok(format!("{}\n", manager.unread_count()))
        }
        Some(&"read") => {
            if args.len() > 1 {
                // Read specific item
                let name = args[1];
                if let Some(item) = manager.get(name) {
                    let formatted = format_news_item(item);
                    manager.mark_read(name)?;
                    Ok(formatted)
                } else {
                    Err(Error::NewsNotFound(name.to_string()))
                }
            } else {
                // Read all unread
                let unread = manager.get_unread();
                if unread.is_empty() {
                    Ok("No unread news items.\n".to_string())
                } else {
                    let mut output = String::new();
                    for item in unread {
                        output.push_str(&format_news_item(item));
                        output.push_str("\n---\n\n");
                    }
                    manager.mark_all_read()?;
                    Ok(output)
                }
            }
        }
        Some(&"unread") => {
            let items = manager.get_unread();
            Ok(format_news_list(&items, false))
        }
        Some(&"purge") => {
            manager.mark_all_read()?;
            Ok("All news items marked as read.\n".to_string())
        }
        _ => {
            Ok("Usage: eselect news [list|count|read|unread|purge]\n".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_default() {
        let manager = NewsManager::default();
        assert_eq!(manager.unread_count(), 0);
    }

    #[test]
    fn test_read_status() {
        let mut manager = NewsManager::default();

        // Add a mock item
        manager.items.push(NewsItem {
            name: "test-news".to_string(),
            title: "Test News".to_string(),
            author: "Test Author".to_string(),
            email: None,
            content_type: "text/plain".to_string(),
            posted: chrono::Local::now().date_naive(),
            revision: None,
            display_if: vec![DisplayCondition::Always],
            content: "Test content".to_string(),
        });

        assert_eq!(manager.unread_count(), 1);
        assert_eq!(manager.get_status("test-news"), ReadStatus::Unread);
    }

    #[test]
    fn test_display_conditions() {
        let manager = NewsManager::default();
        let item = NewsItem {
            name: "test".to_string(),
            title: "Test".to_string(),
            author: "Author".to_string(),
            email: None,
            content_type: "text/plain".to_string(),
            posted: chrono::Local::now().date_naive(),
            revision: None,
            display_if: vec![
                DisplayCondition::Installed(PackageId::new("sys-apps", "systemd")),
            ],
            content: String::new(),
        };

        let installed = vec![PackageId::new("sys-apps", "systemd")];
        assert!(manager.should_display(&item, &installed, "", &[]));

        let installed = vec![PackageId::new("sys-apps", "openrc")];
        assert!(!manager.should_display(&item, &installed, "", &[]));
    }
}
