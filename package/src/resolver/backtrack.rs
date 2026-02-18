//! Backtracking dependency resolver
//!
//! When dependency conflicts occur, this resolver can backtrack and try
//! alternative versions or configurations.

use crate::{Error, InstallOptions, PackageId, PackageInfo, Result};
use std::collections::{HashMap, VecDeque};

/// Configuration for backtracking behavior
#[derive(Debug, Clone)]
pub struct BacktrackConfig {
    /// Maximum number of backtrack attempts
    pub max_backtracks: usize,
    /// Whether to prefer newer versions
    pub prefer_newer: bool,
    /// Whether to allow slot conflicts
    pub allow_slot_conflicts: bool,
}

impl Default for BacktrackConfig {
    fn default() -> Self {
        Self {
            max_backtracks: 10,
            prefer_newer: true,
            allow_slot_conflicts: false,
        }
    }
}

/// A choice point in the resolution process
#[derive(Debug, Clone)]
struct ChoicePoint {
    /// Package being selected
    package: PackageId,
    /// Available versions to try
    versions: Vec<semver::Version>,
    /// Current version index
    current_index: usize,
    /// State at this choice point
    state: ResolutionState,
}

/// State of the resolution process
#[derive(Debug, Clone)]
struct ResolutionState {
    /// Selected packages and their versions
    selected: HashMap<PackageId, semver::Version>,
    /// Packages still to process
    remaining: VecDeque<PackageId>,
    /// Decisions made (for tracking)
    decisions: Vec<Decision>,
}

/// A decision made during resolution
#[derive(Debug, Clone)]
struct Decision {
    package: PackageId,
    version: semver::Version,
    reason: String,
}

/// A decision made during resolution (public version)
#[derive(Debug, Clone)]
pub struct ResolutionDecision {
    /// Package that was selected
    pub package: PackageId,
    /// Version that was selected
    pub version: semver::Version,
    /// Human-readable reason for this decision
    pub reason: String,
}

/// Result of backtracking resolution
#[derive(Debug, Clone)]
pub struct BacktrackResult {
    /// Resolved packages
    pub packages: Vec<(PackageId, semver::Version)>,
    /// Number of backtracks performed
    pub backtracks: usize,
    /// Decisions made during resolution
    pub decisions: Vec<ResolutionDecision>,
}

/// Backtracking dependency resolver
pub struct BacktrackResolver {
    /// Available packages (all versions)
    available: HashMap<PackageId, Vec<PackageInfo>>,
    /// Currently installed packages
    installed: HashMap<PackageId, semver::Version>,
    /// Configuration
    config: BacktrackConfig,
    /// Choice points for backtracking
    choice_points: Vec<ChoicePoint>,
    /// Number of backtracks performed
    backtrack_count: usize,
}

impl BacktrackResolver {
    /// Create a new backtracking resolver
    pub fn new(config: BacktrackConfig) -> Self {
        Self {
            available: HashMap::new(),
            installed: HashMap::new(),
            config,
            choice_points: Vec::new(),
            backtrack_count: 0,
        }
    }

    /// Add available packages
    pub fn add_available(&mut self, packages: Vec<PackageInfo>) {
        for pkg in packages {
            self.available.entry(pkg.id.clone()).or_default().push(pkg);
        }

        // Sort versions (newest first if preferred)
        for versions in self.available.values_mut() {
            if self.config.prefer_newer {
                versions.sort_by(|a, b| b.version.cmp(&a.version));
            } else {
                versions.sort_by(|a, b| a.version.cmp(&b.version));
            }
        }
    }

    /// Set installed packages
    pub fn set_installed(&mut self, installed: Vec<(PackageId, semver::Version)>) {
        self.installed = installed.into_iter().collect();
    }

    /// Resolve dependencies with backtracking
    pub fn resolve(
        &mut self,
        requested: &[PackageId],
        opts: &InstallOptions,
    ) -> Result<BacktrackResult> {
        self.choice_points.clear();
        self.backtrack_count = 0;

        let initial_state = ResolutionState {
            selected: HashMap::new(),
            remaining: requested.iter().cloned().collect(),
            decisions: Vec::new(),
        };

        let mut state = initial_state;

        loop {
            match self.step(&mut state, opts) {
                StepResult::Continue => continue,
                StepResult::Done => {
                    return Ok(BacktrackResult {
                        packages: state.selected.into_iter().collect(),
                        backtracks: self.backtrack_count,
                        decisions: state
                            .decisions
                            .into_iter()
                            .map(|d| ResolutionDecision {
                                package: d.package,
                                version: d.version,
                                reason: d.reason,
                            })
                            .collect(),
                    });
                }
                StepResult::Conflict(reason) => {
                    if !self.backtrack(&mut state)? {
                        return Err(Error::ResolutionFailed(format!(
                            "Could not resolve dependencies after {} backtracks: {}",
                            self.backtrack_count, reason
                        )));
                    }
                }
            }
        }
    }

    /// Take one step in the resolution process
    fn step(&mut self, state: &mut ResolutionState, opts: &InstallOptions) -> StepResult {
        // Get next package to process
        let pkg_id = match state.remaining.pop_front() {
            Some(id) => id,
            None => return StepResult::Done,
        };

        // Skip if already selected
        if state.selected.contains_key(&pkg_id) {
            return StepResult::Continue;
        }

        // Get available versions
        let versions = match self.available.get(&pkg_id) {
            Some(pkgs) => pkgs.clone(),
            None => {
                return StepResult::Conflict(format!("Package not found: {}", pkg_id));
            }
        };

        if versions.is_empty() {
            return StepResult::Conflict(format!("No versions available for: {}", pkg_id));
        }

        // Try to select a version
        for (idx, pkg) in versions.iter().enumerate() {
            // Check if this version satisfies all constraints
            if let Err(reason) = self.check_constraints(&pkg_id, &pkg.version, state) {
                if idx == versions.len() - 1 {
                    return StepResult::Conflict(reason.to_string());
                }
                continue;
            }

            // Create choice point for backtracking
            if versions.len() > 1 {
                self.choice_points.push(ChoicePoint {
                    package: pkg_id.clone(),
                    versions: versions.iter().map(|p| p.version.clone()).collect(),
                    current_index: idx,
                    state: state.clone(),
                });
            }

            // Select this version
            state.selected.insert(pkg_id.clone(), pkg.version.clone());
            state.decisions.push(Decision {
                package: pkg_id.clone(),
                version: pkg.version.clone(),
                reason: format!("Selected {}={}", pkg_id, pkg.version),
            });

            // Add dependencies to remaining
            if !opts.no_deps {
                for dep in &pkg.dependencies {
                    if !state.selected.contains_key(&dep.package)
                        && !state.remaining.contains(&dep.package)
                    {
                        state.remaining.push_back(dep.package.clone());
                    }
                }
                for dep in &pkg.runtime_dependencies {
                    if !state.selected.contains_key(&dep.package)
                        && !state.remaining.contains(&dep.package)
                    {
                        state.remaining.push_back(dep.package.clone());
                    }
                }
            }

            return StepResult::Continue;
        }

        StepResult::Conflict(format!("No suitable version for: {}", pkg_id))
    }

    /// Check if a version satisfies all constraints
    fn check_constraints(
        &self,
        pkg_id: &PackageId,
        version: &semver::Version,
        state: &ResolutionState,
    ) -> Result<()> {
        // Check against constraints from selected packages
        for (selected_id, selected_version) in &state.selected {
            let selected_pkg = self
                .available
                .get(selected_id)
                .and_then(|pkgs| pkgs.iter().find(|p| &p.version == selected_version));

            if let Some(pkg) = selected_pkg {
                // Check if this package has a dependency on the package we're selecting
                for dep in &pkg.dependencies {
                    if &dep.package == pkg_id && !dep.version.matches(version) {
                        return Err(Error::ResolutionFailed(format!(
                            "{} requires {} {:?} but got {}",
                            selected_id, pkg_id, dep.version, version
                        )));
                    }
                }
            }
        }

        // Check slot conflicts
        if !self.config.allow_slot_conflicts {
            let pkg = self
                .available
                .get(pkg_id)
                .and_then(|pkgs| pkgs.iter().find(|p| &p.version == version));

            if let Some(pkg) = pkg {
                for (selected_id, selected_version) in &state.selected {
                    if selected_id == pkg_id {
                        continue;
                    }

                    let selected_pkg = self
                        .available
                        .get(selected_id)
                        .and_then(|pkgs| pkgs.iter().find(|p| &p.version == selected_version));

                    if let Some(selected) = selected_pkg {
                        if selected.id.name == pkg.id.name && selected.slot == pkg.slot {
                            return Err(Error::ResolutionFailed(format!(
                                "Slot conflict: {} and {} both use slot {}",
                                selected_id, pkg_id, pkg.slot
                            )));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Backtrack to a previous choice point
    fn backtrack(&mut self, state: &mut ResolutionState) -> Result<bool> {
        if self.backtrack_count >= self.config.max_backtracks {
            return Ok(false);
        }

        while let Some(mut choice) = self.choice_points.pop() {
            // Try next version at this choice point
            choice.current_index += 1;

            if choice.current_index < choice.versions.len() {
                // Restore state and try next version
                *state = choice.state.clone();
                self.backtrack_count += 1;

                let version = &choice.versions[choice.current_index];
                state
                    .selected
                    .insert(choice.package.clone(), version.clone());
                state.decisions.push(Decision {
                    package: choice.package.clone(),
                    version: version.clone(),
                    reason: format!(
                        "Backtrack: trying {}={} (attempt {})",
                        choice.package,
                        version,
                        choice.current_index + 1
                    ),
                });

                // Re-add dependencies
                if let Some(pkgs) = self.available.get(&choice.package) {
                    if let Some(pkg) = pkgs.iter().find(|p| &p.version == version) {
                        for dep in &pkg.dependencies {
                            if !state.selected.contains_key(&dep.package) {
                                state.remaining.push_back(dep.package.clone());
                            }
                        }
                    }
                }

                // Put choice point back for further backtracking
                self.choice_points.push(choice);

                return Ok(true);
            }
        }

        Ok(false)
    }
}

/// Result of a resolution step
enum StepResult {
    /// Continue to next step
    Continue,
    /// Resolution complete
    Done,
    /// Conflict encountered
    Conflict(String),
}

impl Default for BacktrackResolver {
    fn default() -> Self {
        Self::new(BacktrackConfig::default())
    }
}

/// Convenience function for simple resolution
pub fn resolve_with_backtracking(
    requested: &[PackageId],
    available: Vec<PackageInfo>,
    installed: Vec<(PackageId, semver::Version)>,
    opts: &InstallOptions,
) -> Result<BacktrackResult> {
    let mut resolver = BacktrackResolver::new(BacktrackConfig::default());
    resolver.add_available(available);
    resolver.set_installed(installed);
    resolver.resolve(requested, opts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = BacktrackConfig::default();
        assert_eq!(config.max_backtracks, 10);
        assert!(config.prefer_newer);
    }

    #[test]
    fn test_resolver_creation() {
        let resolver = BacktrackResolver::default();
        assert_eq!(resolver.backtrack_count, 0);
    }
}
