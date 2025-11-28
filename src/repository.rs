//! Repository detection and management for buckos-build
//!
//! This module handles finding and validating the buckos-build repository location.
//! The buckos-build repository contains all package definitions and build rules.

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};

/// Standard locations for buckos-build repository (in search order)
pub const STANDARD_REPO_LOCATIONS: &[&str] = &[
    "/var/db/repos/buckos-build", // Gentoo-style standard location
    "/usr/share/buckos-build",    // System-wide read-only (live USB)
    "/opt/buckos-build",          // Alternative system location
];

/// Detect buckos-build repository path
///
/// Searches standard locations in order:
/// 1. User-specified path (if provided via --repo-path)
/// 2. BUCKOS_BUILD_PATH environment variable
/// 3. /var/db/repos/buckos-build (standard Gentoo-style location)
/// 4. /usr/share/buckos-build (system-wide, read-only - typical for live USB)
/// 5. /opt/buckos-build (alternative system location)
/// 6. ~/buckos-build (user home directory)
/// 7. ./buckos-build (current directory - for development)
///
/// Returns the canonicalized path to a valid buckos-build repository.
pub fn detect_repository_path(custom_path: Option<&str>) -> Result<PathBuf> {
    // 1. Check user-specified path first
    if let Some(path_str) = custom_path {
        let path = PathBuf::from(path_str);
        return validate_repository(&path);
    }

    // 2. Check BUCKOS_BUILD_PATH environment variable
    if let Ok(env_path) = std::env::var("BUCKOS_BUILD_PATH") {
        let path = PathBuf::from(env_path);
        if path.exists() {
            match validate_repository(&path) {
                Ok(p) => {
                    eprintln!("Using buckos-build from BUCKOS_BUILD_PATH: {}", p.display());
                    return Ok(p);
                }
                Err(e) => {
                    eprintln!("Warning: BUCKOS_BUILD_PATH set but invalid: {}", e);
                }
            }
        }
    }

    // 3. Check standard system locations
    let mut search_paths = STANDARD_REPO_LOCATIONS
        .iter()
        .map(|p| PathBuf::from(p))
        .collect::<Vec<_>>();

    // 4. Add user home directory
    if let Ok(home) = std::env::var("HOME") {
        search_paths.push(PathBuf::from(home).join("buckos-build"));
    }

    // 5. Add current directory (for development)
    search_paths.push(PathBuf::from("./buckos-build"));

    // Try each path
    for path in &search_paths {
        if path.exists() {
            match validate_repository(path) {
                Ok(p) => {
                    eprintln!("Found buckos-build repository at: {}", p.display());
                    return Ok(p);
                }
                Err(_e) => {
                    // Path exists but is invalid, try next
                    continue;
                }
            }
        }
    }

    // No valid repository found
    bail!(
        "Could not find buckos-build repository.\n\
        \n\
        Searched locations:\n{}\n\
        \n\
        Please either:\n\
        1. Install buckos-build to a standard location (recommended: /var/db/repos/buckos-build)\n\
        2. Set BUCKOS_BUILD_PATH environment variable\n\
        3. Use --repo-path option to specify the location\n\
        \n\
        Example:\n\
          export BUCKOS_BUILD_PATH=/path/to/buckos-build\n\
          buckos --repo-path /path/to/buckos-build <command>",
        search_paths
            .iter()
            .map(|p| format!("  - {}", p.display()))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

/// Validate that a path contains a valid buckos-build repository
///
/// Checks for required directories and files:
/// - defs/ directory with build definitions
/// - packages/ directory with package definitions
/// - defs/package_defs.bzl (core build rules)
/// - defs/use_flags.bzl (USE flag system)
pub fn validate_repository(path: &Path) -> Result<PathBuf> {
    // Canonicalize the path to get absolute path
    let canonical_path = path
        .canonicalize()
        .with_context(|| format!("Failed to resolve repository path: {}", path.display()))?;

    if !canonical_path.exists() {
        bail!(
            "Repository path does not exist: {}",
            canonical_path.display()
        );
    }

    if !canonical_path.is_dir() {
        bail!(
            "Repository path is not a directory: {}",
            canonical_path.display()
        );
    }

    // Check for required directories
    let required_dirs = vec!["defs", "packages"];
    for dir in &required_dirs {
        let dir_path = canonical_path.join(dir);
        if !dir_path.exists() || !dir_path.is_dir() {
            bail!(
                "Invalid buckos-build repository at {}: missing required directory '{}'",
                canonical_path.display(),
                dir
            );
        }
    }

    // Check for required build definition files
    let required_files = vec![
        "defs/package_defs.bzl",
        "defs/use_flags.bzl",
        "defs/versions.bzl",
    ];

    for file in &required_files {
        let file_path = canonical_path.join(file);
        if !file_path.exists() || !file_path.is_file() {
            bail!(
                "Invalid buckos-build repository at {}: missing required file '{}'",
                canonical_path.display(),
                file
            );
        }
    }

    Ok(canonical_path)
}

/// Get the default repository path
///
/// Returns the first standard location that exists and is valid,
/// or the primary standard location if none exist yet.
///
/// This function is useful for:
/// - Configuration defaults
/// - Quick checks without full validation
/// - Determining where to install a new repository
pub fn default_repository_path() -> PathBuf {
    // Try to find an existing valid repository
    for location in STANDARD_REPO_LOCATIONS {
        let path = PathBuf::from(location);
        if path.exists() {
            if let Ok(valid_path) = validate_repository(&path) {
                return valid_path;
            }
        }
    }

    // Return primary standard location as default
    PathBuf::from(STANDARD_REPO_LOCATIONS[0])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_locations_defined() {
        assert!(!STANDARD_REPO_LOCATIONS.is_empty());
        assert_eq!(STANDARD_REPO_LOCATIONS[0], "/var/db/repos/buckos-build");
    }

    #[test]
    fn test_default_path() {
        let path = default_repository_path();
        assert!(!path.as_os_str().is_empty());
    }
}
