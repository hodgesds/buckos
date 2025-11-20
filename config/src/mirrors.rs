//! Mirror configuration
//!
//! Implements mirror handling for distfiles:
//! - GENTOO_MIRRORS equivalent
//! - Mirror selection and prioritization
//! - Mirror health checking

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Mirror configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirrorConfig {
    /// Primary mirrors (in order of preference)
    pub mirrors: Vec<Mirror>,
    /// Fetch command template
    pub fetch_command: String,
    /// Resume command template
    pub resume_command: String,
    /// Number of retries
    pub fetch_retries: u32,
    /// Timeout in seconds
    pub fetch_timeout: u32,
    /// Minimum size for resume
    pub resume_min_size: String,
}

impl Default for MirrorConfig {
    fn default() -> Self {
        Self {
            mirrors: vec![
                Mirror::new("https://distfiles.buckos.org", "Buckos Primary"),
                Mirror::new(
                    "https://mirror.rackspace.com/gentoo/distfiles",
                    "Rackspace US",
                ),
                Mirror::new("https://gentoo.osuosl.org/distfiles", "OSU OSL"),
                Mirror::new("https://mirrors.mit.edu/gentoo-distfiles", "MIT"),
            ],
            fetch_command: "wget -t 3 -T 60 --passive-ftp -O \"${DISTDIR}/${FILE}\" \"${URI}\""
                .to_string(),
            resume_command: "wget -c -t 3 -T 60 --passive-ftp -O \"${DISTDIR}/${FILE}\" \"${URI}\""
                .to_string(),
            fetch_retries: 3,
            fetch_timeout: 60,
            resume_min_size: "350K".to_string(),
        }
    }
}

impl MirrorConfig {
    /// Create a new mirror configuration
    pub fn new() -> Self {
        Self {
            mirrors: Vec::new(),
            ..Default::default()
        }
    }

    /// Add a mirror
    pub fn add_mirror(&mut self, mirror: Mirror) {
        self.mirrors.push(mirror);
    }

    /// Insert a mirror at the front (highest priority)
    pub fn prepend_mirror(&mut self, mirror: Mirror) {
        self.mirrors.insert(0, mirror);
    }

    /// Remove a mirror by URL
    pub fn remove_mirror(&mut self, url: &str) -> bool {
        let before = self.mirrors.len();
        self.mirrors.retain(|m| m.url != url);
        self.mirrors.len() < before
    }

    /// Get mirror URLs
    pub fn urls(&self) -> Vec<&str> {
        self.mirrors.iter().map(|m| m.url.as_str()).collect()
    }

    /// Get mirrors as GENTOO_MIRRORS string
    pub fn to_mirrors_string(&self) -> String {
        self.mirrors
            .iter()
            .map(|m| m.url.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Parse a GENTOO_MIRRORS string
    pub fn from_mirrors_string(s: &str) -> Self {
        let mut config = Self::new();

        for url in s.split_whitespace() {
            let url = url.trim();
            if !url.is_empty() {
                config.add_mirror(Mirror::new(url, ""));
            }
        }

        config
    }

    /// Get the best mirror for a region
    pub fn best_for_region(&self, region: &str) -> Option<&Mirror> {
        self.mirrors
            .iter()
            .find(|m| m.region.as_deref() == Some(region))
            .or_else(|| self.mirrors.first())
    }

    /// Construct a fetch URL for a distfile
    pub fn fetch_url(&self, filename: &str) -> Option<String> {
        self.mirrors
            .first()
            .map(|m| format!("{}/{}", m.url, filename))
    }

    /// Get all fetch URLs for a distfile (for fallback)
    pub fn all_fetch_urls(&self, filename: &str) -> Vec<String> {
        self.mirrors
            .iter()
            .map(|m| format!("{}/{}", m.url, filename))
            .collect()
    }
}

/// A single mirror
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mirror {
    /// Mirror URL
    pub url: String,
    /// Mirror name/description
    pub name: String,
    /// Geographic region
    pub region: Option<String>,
    /// Country code
    pub country: Option<String>,
    /// Priority (higher = better)
    pub priority: i32,
    /// Whether this mirror supports IPv6
    pub ipv6: bool,
    /// Protocols supported
    pub protocols: Vec<String>,
}

impl Mirror {
    /// Create a new mirror
    pub fn new(url: impl Into<String>, name: impl Into<String>) -> Self {
        let url = url.into();
        let protocols = if url.starts_with("https://") {
            vec!["https".to_string()]
        } else if url.starts_with("http://") {
            vec!["http".to_string()]
        } else if url.starts_with("ftp://") {
            vec!["ftp".to_string()]
        } else if url.starts_with("rsync://") {
            vec!["rsync".to_string()]
        } else {
            vec![]
        };

        Self {
            url,
            name: name.into(),
            region: None,
            country: None,
            priority: 0,
            ipv6: false,
            protocols,
        }
    }

    /// Set the region
    pub fn with_region(mut self, region: impl Into<String>) -> Self {
        self.region = Some(region.into());
        self
    }

    /// Set the country
    pub fn with_country(mut self, country: impl Into<String>) -> Self {
        self.country = Some(country.into());
        self
    }

    /// Set the priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Set IPv6 support
    pub fn with_ipv6(mut self, ipv6: bool) -> Self {
        self.ipv6 = ipv6;
        self
    }
}

/// Known public mirrors
pub fn public_mirrors() -> Vec<Mirror> {
    vec![
        // North America
        Mirror::new("https://mirrors.mit.edu/gentoo-distfiles", "MIT")
            .with_region("North America")
            .with_country("US")
            .with_priority(100),
        Mirror::new("https://gentoo.osuosl.org/distfiles", "OSU OSL")
            .with_region("North America")
            .with_country("US")
            .with_priority(100),
        Mirror::new(
            "https://mirror.rackspace.com/gentoo/distfiles",
            "Rackspace US",
        )
        .with_region("North America")
        .with_country("US")
        .with_priority(90),
        Mirror::new(
            "https://gentoo.ussg.indiana.edu/distfiles",
            "Indiana University",
        )
        .with_region("North America")
        .with_country("US")
        .with_priority(80),
        // Europe
        Mirror::new("https://ftp.fau.de/gentoo/distfiles", "FAU Germany")
            .with_region("Europe")
            .with_country("DE")
            .with_priority(100),
        Mirror::new(
            "https://mirror.eu.oneandone.net/linux/distributions/gentoo/gentoo/distfiles",
            "1&1",
        )
        .with_region("Europe")
        .with_country("DE")
        .with_priority(90),
        Mirror::new(
            "https://ftp.snt.utwente.nl/pub/os/linux/gentoo/distfiles",
            "SNT Netherlands",
        )
        .with_region("Europe")
        .with_country("NL")
        .with_priority(90),
        Mirror::new(
            "https://mirror.bytemark.co.uk/gentoo/distfiles",
            "Bytemark UK",
        )
        .with_region("Europe")
        .with_country("GB")
        .with_priority(90),
        // Asia
        Mirror::new(
            "https://ftp.iij.ad.jp/pub/linux/gentoo/distfiles",
            "IIJ Japan",
        )
        .with_region("Asia")
        .with_country("JP")
        .with_priority(100),
        Mirror::new(
            "https://ftp.kaist.ac.kr/pub/gentoo/distfiles",
            "KAIST Korea",
        )
        .with_region("Asia")
        .with_country("KR")
        .with_priority(90),
        Mirror::new(
            "https://mirrors.tuna.tsinghua.edu.cn/gentoo/distfiles",
            "Tsinghua China",
        )
        .with_region("Asia")
        .with_country("CN")
        .with_priority(90),
        // Oceania
        Mirror::new(
            "https://mirror.aarnet.edu.au/pub/gentoo/distfiles",
            "AARNet Australia",
        )
        .with_region("Oceania")
        .with_country("AU")
        .with_priority(100),
    ]
}

/// Mirror selection strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MirrorStrategy {
    /// Use mirrors in order
    Ordered,
    /// Randomly select mirror
    Random,
    /// Select based on latency
    Latency,
    /// Select based on bandwidth
    Bandwidth,
    /// Round-robin selection
    RoundRobin,
}

impl Default for MirrorStrategy {
    fn default() -> Self {
        Self::Ordered
    }
}

/// Thirdparty mirror sources for specific packages
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThirdpartyMirrors {
    /// Mirror definitions by name
    pub mirrors: HashMap<String, Vec<String>>,
}

impl ThirdpartyMirrors {
    /// Create a new thirdparty mirrors config
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a mirror source
    pub fn add(&mut self, name: impl Into<String>, urls: Vec<String>) {
        self.mirrors.insert(name.into(), urls);
    }

    /// Get mirrors for a source
    pub fn get(&self, name: &str) -> Option<&Vec<String>> {
        self.mirrors.get(name)
    }

    /// Create with common thirdparty mirrors
    pub fn with_defaults() -> Self {
        let mut mirrors = Self::new();

        mirrors.add(
            "gnu",
            vec![
                "https://ftp.gnu.org/gnu/".to_string(),
                "https://mirrors.kernel.org/gnu/".to_string(),
            ],
        );

        mirrors.add(
            "kernel",
            vec![
                "https://www.kernel.org/pub/".to_string(),
                "https://mirrors.edge.kernel.org/pub/".to_string(),
            ],
        );

        mirrors.add(
            "sourceforge",
            vec!["https://downloads.sourceforge.net/".to_string()],
        );

        mirrors.add("github", vec!["https://github.com/".to_string()]);

        mirrors.add("gitlab", vec!["https://gitlab.com/".to_string()]);

        mirrors.add(
            "pypi",
            vec!["https://files.pythonhosted.org/packages/".to_string()],
        );

        mirrors.add(
            "crates",
            vec!["https://static.crates.io/crates/".to_string()],
        );

        mirrors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mirror_config() {
        let mut config = MirrorConfig::default();
        assert!(!config.mirrors.is_empty());

        config.add_mirror(Mirror::new("https://example.com/distfiles", "Example"));
        assert!(config.urls().contains(&"https://example.com/distfiles"));
    }

    #[test]
    fn test_from_mirrors_string() {
        let config = MirrorConfig::from_mirrors_string(
            "https://mirror1.com https://mirror2.com https://mirror3.com",
        );
        assert_eq!(config.mirrors.len(), 3);
    }

    #[test]
    fn test_fetch_urls() {
        let config = MirrorConfig::default();
        let urls = config.all_fetch_urls("package-1.0.tar.gz");
        assert!(!urls.is_empty());
        assert!(urls[0].ends_with("package-1.0.tar.gz"));
    }

    #[test]
    fn test_thirdparty_mirrors() {
        let mirrors = ThirdpartyMirrors::with_defaults();
        assert!(mirrors.get("gnu").is_some());
        assert!(mirrors.get("pypi").is_some());
    }
}
