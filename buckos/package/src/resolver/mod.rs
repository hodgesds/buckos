//! Dependency resolution using SAT solver
//!
//! Uses the varisat SAT solver for optimal dependency resolution.

use crate::db::PackageDb;
use crate::repository::RepositoryManager;
use crate::{Error, InstallOptions, PackageId, PackageInfo, Result};
use petgraph::algo::toposort;
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use varisat::{ExtendFormula, Lit, Solver};

/// Internal resolution result (uses PackageInfo)
#[derive(Debug, Clone)]
pub struct InternalResolution {
    pub packages: Vec<PackageInfo>,
    pub build_order: Vec<usize>,
    pub download_size: u64,
    pub install_size: u64,
}

/// Dependency resolver
pub struct DependencyResolver {
    db: Arc<RwLock<PackageDb>>,
    repos: Arc<RepositoryManager>,
}

impl DependencyResolver {
    /// Create a new dependency resolver
    pub fn new(db: Arc<RwLock<PackageDb>>, repos: Arc<RepositoryManager>) -> Self {
        Self { db, repos }
    }

    /// Resolve dependencies for packages
    pub async fn resolve(
        &self,
        packages: &[String],
        opts: &InstallOptions,
    ) -> Result<InternalResolution> {
        info!("Resolving dependencies for {} packages", packages.len());

        // Parse package specifications
        let mut requested: Vec<PackageId> = Vec::new();
        for pkg in packages {
            let id = PackageId::parse(pkg)
                .or_else(|| {
                    // Try to find by name only
                    Some(PackageId::new("unknown", pkg))
                })
                .ok_or_else(|| Error::InvalidPackageSpec(pkg.clone()))?;
            requested.push(id);
        }

        // Get all available packages
        let mut available = self.repos.get_all_packages().await?;
        let db = self.db.read().await;

        // Filter out already installed packages (unless forcing)
        if !opts.force {
            available.retain(|pkg| !db.is_installed(&pkg.id.name).unwrap_or(false));
        }
        drop(db);

        // Build dependency graph
        let mut graph: DiGraph<PackageInfo, ()> = DiGraph::new();
        let mut node_map: HashMap<PackageId, NodeIndex> = HashMap::new();
        let mut pkg_map: HashMap<PackageId, PackageInfo> = HashMap::new();

        // First pass: add all packages to map
        for pkg in &available {
            pkg_map.insert(pkg.id.clone(), pkg.clone());
        }

        // Collect all packages we need
        let mut to_install: HashSet<PackageId> = HashSet::new();
        let mut queue: Vec<PackageId> = requested.clone();
        let mut visited: HashSet<PackageId> = HashSet::new();

        while let Some(pkg_id) = queue.pop() {
            if visited.contains(&pkg_id) {
                continue;
            }
            visited.insert(pkg_id.clone());

            // Find package info
            let pkg_info = if let Some(info) = pkg_map.get(&pkg_id) {
                info.clone()
            } else {
                // Try to find by name in repos
                match self.repos.get_info(&pkg_id.name).await? {
                    Some(info) => {
                        pkg_map.insert(info.id.clone(), info.clone());
                        info
                    }
                    None => {
                        return Err(Error::PackageNotFound(pkg_id.to_string()));
                    }
                }
            };

            to_install.insert(pkg_id.clone());

            // Add dependencies to queue
            if !opts.no_deps {
                for dep in &pkg_info.dependencies {
                    if !visited.contains(&dep.package) {
                        queue.push(dep.package.clone());
                    }
                }
                for dep in &pkg_info.runtime_dependencies {
                    if !visited.contains(&dep.package) {
                        queue.push(dep.package.clone());
                    }
                }
                if opts.build {
                    for dep in &pkg_info.build_dependencies {
                        if !visited.contains(&dep.package) {
                            queue.push(dep.package.clone());
                        }
                    }
                }
            }
        }

        // Build the graph with actual packages
        for pkg_id in &to_install {
            if let Some(pkg) = pkg_map.get(pkg_id) {
                let node = graph.add_node(pkg.clone());
                node_map.insert(pkg_id.clone(), node);
            }
        }

        // Add edges for dependencies
        for pkg_id in &to_install {
            if let Some(pkg) = pkg_map.get(pkg_id) {
                let pkg_node = node_map[pkg_id];

                for dep in &pkg.dependencies {
                    if let Some(&dep_node) = node_map.get(&dep.package) {
                        graph.add_edge(dep_node, pkg_node, ());
                    }
                }
                for dep in &pkg.runtime_dependencies {
                    if let Some(&dep_node) = node_map.get(&dep.package) {
                        graph.add_edge(dep_node, pkg_node, ());
                    }
                }
            }
        }

        // Topological sort for build order
        let sorted = toposort(&graph, None).map_err(|_| {
            Error::CircularDependency("Circular dependency detected".to_string())
        })?;

        // Build resolution
        let mut packages = Vec::new();
        let mut build_order = Vec::new();
        let mut download_size = 0u64;
        let mut install_size = 0u64;

        for (idx, node) in sorted.iter().enumerate() {
            let pkg = &graph[*node];
            packages.push(pkg.clone());
            build_order.push(idx);
            download_size += pkg.size;
            install_size += pkg.installed_size;
        }

        info!(
            "Resolution complete: {} packages, {} download, {} install",
            packages.len(),
            format_size(download_size),
            format_size(install_size)
        );

        Ok(InternalResolution {
            packages,
            build_order,
            download_size,
            install_size,
        })
    }

    /// Resolve dependencies using SAT solver for complex constraints
    pub async fn resolve_sat(
        &self,
        packages: &[String],
        opts: &InstallOptions,
    ) -> Result<InternalResolution> {
        info!("Using SAT solver for dependency resolution");

        let mut solver = Solver::new();

        // Get all available package versions
        let all_packages = self.repos.get_all_packages().await?;

        // Map packages to SAT variables
        let mut var_map: HashMap<(PackageId, semver::Version), Lit> = HashMap::new();
        let mut reverse_map: HashMap<Lit, (PackageId, semver::Version)> = HashMap::new();
        let mut next_var = 1isize;

        for pkg in &all_packages {
            let lit = Lit::from_dimacs(next_var);
            var_map.insert((pkg.id.clone(), pkg.version.clone()), lit);
            reverse_map.insert(lit, (pkg.id.clone(), pkg.version.clone()));
            next_var += 1;
        }

        // Add constraints

        // 1. At most one version of each package
        let mut versions_by_pkg: HashMap<PackageId, Vec<Lit>> = HashMap::new();
        for ((id, _ver), &lit) in &var_map {
            versions_by_pkg.entry(id.clone()).or_default().push(lit);
        }

        for (_pkg_id, versions) in &versions_by_pkg {
            if versions.len() > 1 {
                // At most one: for each pair, !a || !b
                for i in 0..versions.len() {
                    for j in (i + 1)..versions.len() {
                        solver.add_clause(&[!versions[i], !versions[j]]);
                    }
                }
            }
        }

        // 2. Requested packages must be installed
        for pkg_name in packages {
            let pkg_id = PackageId::parse(pkg_name).ok_or_else(|| {
                Error::InvalidPackageSpec(pkg_name.clone())
            })?;

            if let Some(versions) = versions_by_pkg.get(&pkg_id) {
                // At least one version must be selected
                solver.add_clause(versions);
            } else {
                return Err(Error::PackageNotFound(pkg_name.clone()));
            }
        }

        // 3. Dependencies
        for pkg in &all_packages {
            let pkg_lit = var_map[&(pkg.id.clone(), pkg.version.clone())];

            for dep in &pkg.dependencies {
                // Find versions that satisfy the dependency
                let satisfying: Vec<Lit> = all_packages
                    .iter()
                    .filter(|p| p.id == dep.package && dep.version.matches(&p.version))
                    .map(|p| var_map[&(p.id.clone(), p.version.clone())])
                    .collect();

                if satisfying.is_empty() {
                    // If package is selected, dependency cannot be satisfied
                    solver.add_clause(&[!pkg_lit]);
                } else {
                    // pkg => (dep_v1 || dep_v2 || ...)
                    let mut clause = vec![!pkg_lit];
                    clause.extend(satisfying);
                    solver.add_clause(&clause);
                }
            }
        }

        // Solve
        let solution = solver.solve().map_err(|e| {
            Error::ResolutionFailed(format!("SAT solver error: {:?}", e))
        })?;

        if !solution {
            return Err(Error::ResolutionFailed(
                "No solution found for dependencies".to_string(),
            ));
        }

        // Extract solution
        let model = solver.model().ok_or_else(|| {
            Error::ResolutionFailed("No model available".to_string())
        })?;

        let mut selected: Vec<PackageInfo> = Vec::new();
        for lit in model {
            if lit.is_positive() {
                // Get the positive literal to look up in our map
                let pos_lit = if lit.is_positive() { lit } else { !lit };
                if let Some((pkg_id, version)) = reverse_map.get(&pos_lit) {
                    if let Some(pkg) = all_packages.iter().find(|p| {
                        p.id == *pkg_id && p.version == *version
                    }) {
                        selected.push(pkg.clone());
                    }
                }
            }
        }

        // Sort by dependencies (build order)
        let packages = self.compute_build_order(selected)?;

        let download_size: u64 = packages.iter().map(|p| p.size).sum();
        let install_size: u64 = packages.iter().map(|p| p.installed_size).sum();

        Ok(InternalResolution {
            build_order: (0..packages.len()).collect(),
            packages,
            download_size,
            install_size,
        })
    }

    fn compute_build_order(&self, packages: Vec<PackageInfo>) -> Result<Vec<PackageInfo>> {
        let mut graph: DiGraph<usize, ()> = DiGraph::new();
        let mut node_map: HashMap<PackageId, NodeIndex> = HashMap::new();

        // Add nodes
        for (idx, pkg) in packages.iter().enumerate() {
            let node = graph.add_node(idx);
            node_map.insert(pkg.id.clone(), node);
        }

        // Add edges
        for (idx, pkg) in packages.iter().enumerate() {
            let pkg_node = node_map[&pkg.id];

            for dep in &pkg.dependencies {
                if let Some(&dep_node) = node_map.get(&dep.package) {
                    graph.add_edge(dep_node, pkg_node, ());
                }
            }
        }

        // Topological sort
        let sorted = toposort(&graph, None).map_err(|_| {
            Error::CircularDependency("Circular dependency in packages".to_string())
        })?;

        Ok(sorted.into_iter().map(|n| packages[graph[n]].clone()).collect())
    }
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
