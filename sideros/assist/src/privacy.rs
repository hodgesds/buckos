//! Privacy controls and data redaction for system diagnostics.

use regex::Regex;
use serde::{Deserialize, Serialize};

/// Privacy settings that control what information is collected and how it's handled.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacySettings {
    /// Whether to collect hardware information.
    pub collect_hardware: bool,
    /// Whether to collect software information.
    pub collect_software: bool,
    /// Whether to collect network information.
    pub collect_network: bool,
    /// Whether to collect process information.
    pub collect_processes: bool,
    /// Whether to redact usernames.
    pub redact_usernames: bool,
    /// Whether to redact IP addresses.
    pub redact_ips: bool,
    /// Whether to redact hostnames.
    pub redact_hostnames: bool,
    /// Whether to redact MAC addresses.
    pub redact_macs: bool,
    /// Whether to redact file paths containing home directories.
    pub redact_home_paths: bool,
    /// Custom patterns to redact (as regex strings).
    pub custom_redact_patterns: Vec<String>,
}

impl Default for PrivacySettings {
    fn default() -> Self {
        Self {
            collect_hardware: true,
            collect_software: true,
            collect_network: true,
            collect_processes: true,
            redact_usernames: true,
            redact_ips: true,
            redact_hostnames: false,
            redact_macs: true,
            redact_home_paths: true,
            custom_redact_patterns: Vec::new(),
        }
    }
}

impl PrivacySettings {
    /// Create settings that collect minimal information.
    pub fn minimal() -> Self {
        Self {
            collect_hardware: true,
            collect_software: false,
            collect_network: false,
            collect_processes: false,
            redact_usernames: true,
            redact_ips: true,
            redact_hostnames: true,
            redact_macs: true,
            redact_home_paths: true,
            custom_redact_patterns: Vec::new(),
        }
    }

    /// Create settings that collect everything without redaction (for local use only).
    pub fn full() -> Self {
        Self {
            collect_hardware: true,
            collect_software: true,
            collect_network: true,
            collect_processes: true,
            redact_usernames: false,
            redact_ips: false,
            redact_hostnames: false,
            redact_macs: false,
            redact_home_paths: false,
            custom_redact_patterns: Vec::new(),
        }
    }
}

/// Redactor that applies privacy settings to collected data.
pub struct Redactor {
    settings: PrivacySettings,
    username: String,
    hostname: String,
    ip_regex: Regex,
    mac_regex: Regex,
}

impl Redactor {
    /// Create a new redactor with the given privacy settings.
    pub fn new(settings: PrivacySettings) -> Self {
        // Get current username and hostname for redaction
        let username = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| String::from("user"));

        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| String::from("localhost"));

        // Compile regex patterns
        let ip_regex = Regex::new(
            r"\b(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\b"
        ).expect("Invalid IP regex");

        let mac_regex = Regex::new(
            r"\b(?:[0-9A-Fa-f]{2}[:-]){5}[0-9A-Fa-f]{2}\b"
        ).expect("Invalid MAC regex");

        Self {
            settings,
            username,
            hostname,
            ip_regex,
            mac_regex,
        }
    }

    /// Redact sensitive information from a string based on privacy settings.
    pub fn redact(&self, input: &str) -> String {
        let mut result = input.to_string();

        // Redact username
        if self.settings.redact_usernames && !self.username.is_empty() {
            result = result.replace(&self.username, "[REDACTED_USER]");
        }

        // Redact hostname
        if self.settings.redact_hostnames && !self.hostname.is_empty() {
            result = result.replace(&self.hostname, "[REDACTED_HOST]");
        }

        // Redact IP addresses
        if self.settings.redact_ips {
            result = self.ip_regex.replace_all(&result, "[REDACTED_IP]").to_string();
        }

        // Redact MAC addresses
        if self.settings.redact_macs {
            result = self.mac_regex.replace_all(&result, "[REDACTED_MAC]").to_string();
        }

        // Redact home directory paths
        if self.settings.redact_home_paths {
            if let Ok(home) = std::env::var("HOME") {
                result = result.replace(&home, "[REDACTED_HOME]");
            }
        }

        // Apply custom redaction patterns
        for pattern in &self.settings.custom_redact_patterns {
            if let Ok(re) = Regex::new(pattern) {
                result = re.replace_all(&result, "[REDACTED]").to_string();
            }
        }

        result
    }

    /// Check if a category should be collected based on privacy settings.
    pub fn should_collect(&self, category: &str) -> bool {
        match category {
            "hardware" => self.settings.collect_hardware,
            "software" => self.settings.collect_software,
            "network" => self.settings.collect_network,
            "processes" => self.settings.collect_processes,
            _ => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ip_redaction() {
        let settings = PrivacySettings::default();
        let redactor = Redactor::new(settings);

        let input = "Server IP is 192.168.1.100 and gateway is 10.0.0.1";
        let result = redactor.redact(input);

        assert!(!result.contains("192.168.1.100"));
        assert!(!result.contains("10.0.0.1"));
        assert!(result.contains("[REDACTED_IP]"));
    }

    #[test]
    fn test_mac_redaction() {
        let settings = PrivacySettings::default();
        let redactor = Redactor::new(settings);

        let input = "MAC address: 00:1A:2B:3C:4D:5E";
        let result = redactor.redact(input);

        assert!(!result.contains("00:1A:2B:3C:4D:5E"));
        assert!(result.contains("[REDACTED_MAC]"));
    }
}
