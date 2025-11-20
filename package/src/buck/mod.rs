//! Buck2 build system integration
//!
//! This module provides integration with Buck2 for building packages from source.

pub mod buckconfig;

pub use buckconfig::{BuckConfigFile, BuckConfigOptions, BuckConfigSection};

use crate::config::Config;
use crate::{BuildOptions, BuildResult, Error, Result};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, error, info};

/// Buck2 build system integration
pub struct BuckIntegration {
    /// Path to Buck2 executable
    buck_path: PathBuf,
    /// Path to Buck targets repository
    repo_path: PathBuf,
    /// Build output directory
    output_dir: PathBuf,
    /// Number of parallel jobs
    jobs: usize,
    /// Custom Buck configuration options
    config_options: BuckConfigOptions,
}

impl BuckIntegration {
    /// Create a new Buck integration
    pub fn new(config: &Config) -> Result<Self> {
        Self::with_config_options(config, BuckConfigOptions::default())
    }

    /// Create a new Buck integration with custom config options
    pub fn with_config_options(config: &Config, config_options: BuckConfigOptions) -> Result<Self> {
        let buck_path = config.buck_path.clone();
        let repo_path = config.buck_repo.clone();
        let output_dir = config.cache_dir.join("buck-out");

        // Verify Buck exists
        if !buck_path.exists() {
            // Try to find it in PATH
            if let Ok(found) = which::which("buck2") {
                Ok(Self {
                    buck_path: found,
                    repo_path,
                    output_dir,
                    jobs: config.parallelism,
                    config_options,
                })
            } else {
                Err(Error::BuckError(format!(
                    "Buck2 not found at {:?} or in PATH",
                    buck_path
                )))
            }
        } else {
            Ok(Self {
                buck_path,
                repo_path,
                output_dir,
                jobs: config.parallelism,
                config_options,
            })
        }
    }

    /// Get mutable reference to config options
    pub fn config_options_mut(&mut self) -> &mut BuckConfigOptions {
        &mut self.config_options
    }

    /// Get reference to config options
    pub fn config_options(&self) -> &BuckConfigOptions {
        &self.config_options
    }

    /// Set custom config options
    pub fn set_config_options(&mut self, options: BuckConfigOptions) {
        self.config_options = options;
    }

    /// Load and apply .buckconfig from the repository
    pub fn load_repo_config(&self) -> Result<BuckConfigFile> {
        buckconfig::load_repo_config(&self.repo_path)
    }

    /// Build a target
    pub async fn build(&self, target: &str, opts: &BuildOptions) -> Result<BuildResult> {
        let start = std::time::Instant::now();

        info!("Building Buck target: {}", target);

        let mut cmd = Command::new(&self.buck_path);
        cmd.arg("build")
            .arg(target)
            .current_dir(&self.repo_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Add job count
        let jobs = opts.jobs.unwrap_or(self.jobs);
        cmd.arg(format!("--num-threads={}", jobs));

        // Apply custom config options from BuckIntegration
        for arg in self.config_options.to_args() {
            cmd.arg(arg);
        }

        // Apply build-specific config options
        if let Some(ref build_config) = opts.config_options {
            for arg in build_config.to_args() {
                cmd.arg(arg);
            }
        }

        // Release mode (can be overridden by config options)
        if opts.release && self.config_options.build_mode.is_none() {
            cmd.arg("--config").arg("build.mode=release");
        }

        // Additional arguments
        for arg in &opts.buck_args {
            cmd.arg(arg);
        }

        debug!("Running: {:?}", cmd);

        let output = cmd
            .output()
            .await
            .map_err(|e| Error::BuckError(format!("Failed to execute Buck: {}", e)))?;

        let duration = start.elapsed();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            error!("Build failed for {}", target);
            return Ok(BuildResult {
                target: target.to_string(),
                success: false,
                output_path: None,
                duration,
                stdout,
                stderr,
            });
        }

        // Find output path
        let output_path = self.find_build_output(target).await?;

        info!("Build completed in {:?}", duration);

        Ok(BuildResult {
            target: target.to_string(),
            success: true,
            output_path,
            duration,
            stdout,
            stderr,
        })
    }

    /// Build multiple targets in parallel
    pub async fn build_many(
        &self,
        targets: &[String],
        opts: &BuildOptions,
    ) -> Result<Vec<BuildResult>> {
        let start = std::time::Instant::now();

        info!("Building {} Buck targets", targets.len());

        let mut cmd = Command::new(&self.buck_path);
        cmd.arg("build")
            .current_dir(&self.repo_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Add all targets
        for target in targets {
            cmd.arg(target);
        }

        // Add job count
        let jobs = opts.jobs.unwrap_or(self.jobs);
        cmd.arg(format!("--num-threads={}", jobs));

        // Apply custom config options from BuckIntegration
        for arg in self.config_options.to_args() {
            cmd.arg(arg);
        }

        // Apply build-specific config options
        if let Some(ref build_config) = opts.config_options {
            for arg in build_config.to_args() {
                cmd.arg(arg);
            }
        }

        // Release mode (can be overridden by config options)
        if opts.release && self.config_options.build_mode.is_none() {
            cmd.arg("--config").arg("build.mode=release");
        }

        for arg in &opts.buck_args {
            cmd.arg(arg);
        }

        let output = cmd
            .output()
            .await
            .map_err(|e| Error::BuckError(format!("Failed to execute Buck: {}", e)))?;

        let duration = start.elapsed();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let success = output.status.success();

        // Create results for each target
        let mut results = Vec::new();
        for target in targets {
            let output_path = if success {
                self.find_build_output(target).await.ok().flatten()
            } else {
                None
            };

            results.push(BuildResult {
                target: target.clone(),
                success,
                output_path,
                duration,
                stdout: stdout.clone(),
                stderr: stderr.clone(),
            });
        }

        Ok(results)
    }

    /// Query target information
    pub async fn query(&self, pattern: &str) -> Result<Vec<String>> {
        let mut cmd = Command::new(&self.buck_path);
        cmd.arg("query")
            .arg(pattern)
            .current_dir(&self.repo_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let output = cmd
            .output()
            .await
            .map_err(|e| Error::BuckError(format!("Failed to query Buck: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::BuckError(format!("Query failed: {}", stderr)));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(|s| s.to_string()).collect())
    }

    /// Get target dependencies
    pub async fn deps(&self, target: &str) -> Result<Vec<String>> {
        let mut cmd = Command::new(&self.buck_path);
        cmd.arg("query")
            .arg(format!("deps({})", target))
            .current_dir(&self.repo_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let output = cmd
            .output()
            .await
            .map_err(|e| Error::BuckError(format!("Failed to query deps: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::BuckError(format!("Deps query failed: {}", stderr)));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(|s| s.to_string()).collect())
    }

    /// Get reverse dependencies
    pub async fn rdeps(&self, target: &str) -> Result<Vec<String>> {
        let mut cmd = Command::new(&self.buck_path);
        cmd.arg("query")
            .arg(format!("rdeps(//..., {})", target))
            .current_dir(&self.repo_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let output = cmd
            .output()
            .await
            .map_err(|e| Error::BuckError(format!("Failed to query rdeps: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::BuckError(format!("Rdeps query failed: {}", stderr)));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(|s| s.to_string()).collect())
    }

    /// Clean build outputs
    pub async fn clean(&self) -> Result<()> {
        info!("Cleaning Buck build outputs");

        let mut cmd = Command::new(&self.buck_path);
        cmd.arg("clean")
            .current_dir(&self.repo_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let output = cmd
            .output()
            .await
            .map_err(|e| Error::BuckError(format!("Failed to clean: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::BuckError(format!("Clean failed: {}", stderr)));
        }

        Ok(())
    }

    /// Find build output for a target
    async fn find_build_output(&self, target: &str) -> Result<Option<PathBuf>> {
        let mut cmd = Command::new(&self.buck_path);
        cmd.arg("build")
            .arg("--show-output")
            .arg(target)
            .current_dir(&self.repo_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let output = cmd
            .output()
            .await
            .map_err(|e| Error::BuckError(format!("Failed to get output path: {}", e)))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            debug!("Buck show-output: {}", stdout);

            // Parse output like "//path/to/target <TAB/SPACE> /path/to/output"
            for line in stdout.lines() {
                // Try tab first, then space
                if let Some((_target, path)) = line.split_once('\t').or_else(|| line.split_once(' ')) {
                    let path_str = path.trim();
                    // Path is relative to repo_path, make it absolute
                    let abs_path = self.repo_path.join(path_str);
                    debug!("Found output path: {}", abs_path.display());
                    return Ok(Some(abs_path));
                }
            }
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            debug!("Buck show-output failed: {}", stderr);
        }

        Ok(None)
    }

    /// Get audit information for a target
    pub async fn audit(&self, target: &str) -> Result<String> {
        let mut cmd = Command::new(&self.buck_path);
        cmd.arg("audit")
            .arg("includes")
            .arg(target)
            .current_dir(&self.repo_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let output = cmd
            .output()
            .await
            .map_err(|e| Error::BuckError(format!("Failed to audit: {}", e)))?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Generate project files
    pub async fn project(&self) -> Result<()> {
        let mut cmd = Command::new(&self.buck_path);
        cmd.arg("project")
            .current_dir(&self.repo_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let output = cmd
            .output()
            .await
            .map_err(|e| Error::BuckError(format!("Failed to generate project: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::BuckError(format!(
                "Project generation failed: {}",
                stderr
            )));
        }

        Ok(())
    }
}

/// Convert package ID to Buck target
pub fn package_to_target(category: &str, name: &str) -> String {
    format!("//packages/{}/{}:package", category, name)
}

/// Parse Buck target to package ID components
pub fn target_to_package(target: &str) -> Option<(String, String)> {
    // Parse "//packages/category/name:target"
    let target = target.strip_prefix("//packages/")?;
    let parts: Vec<&str> = target.split('/').collect();
    if parts.len() >= 2 {
        let category = parts[0].to_string();
        let name = parts[1].split(':').next()?.to_string();
        Some((category, name))
    } else {
        None
    }
}

/// Convert package ID to Buck target using BuckTarget type
pub fn package_id_to_target(pkg_id: &crate::PackageId) -> crate::BuckTarget {
    crate::BuckTarget::for_package(&pkg_id.category, &pkg_id.name)
}

/// Parse Buck target string to package ID
pub fn target_string_to_package_id(target: &str) -> Option<crate::PackageId> {
    let (category, name) = target_to_package(target)?;
    Some(crate::PackageId::new(category, name))
}

/// Generate Buck target for package metadata
pub fn package_metadata_target(category: &str, name: &str) -> String {
    format!("//packages/{}/{}:metadata", category, name)
}

/// Generate Buck target for package install script
pub fn package_install_target(category: &str, name: &str) -> String {
    format!("//packages/{}/{}:install", category, name)
}

/// Get all targets for a package (useful for buckos-build)
pub fn package_all_targets(category: &str, name: &str) -> Vec<String> {
    vec![
        package_to_target(category, name),
        package_metadata_target(category, name),
        package_install_target(category, name),
        format!("//packages/{}/{}:{}", category, name, name), // library target
    ]
}
