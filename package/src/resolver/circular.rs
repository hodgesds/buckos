//! Circular dependency detection and resolution
//!
//! Detects cycles in the dependency graph and provides strategies to break them.

use crate::{Error, PackageId, PackageInfo, Result};
use petgraph::algo::{kosaraju_scc, tarjan_scc};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::{HashMap, HashSet};

/// A detected circular dependency
#[derive(Debug, Clone)]
pub struct CircularDependency {
    /// Packages involved in the cycle
    pub packages: Vec<PackageId>,
    /// Whether this cycle can be broken via USE flags
    pub breakable: bool,
    /// Suggested way to break the cycle
    pub break_suggestion: Option<CycleBreakSuggestion>,
}

/// Suggestion for breaking a dependency cycle
#[derive(Debug, Clone)]
pub enum CycleBreakSuggestion {
    /// Disable a USE flag to remove a conditional dependency
    DisableUseFlag { package: PackageId, flag: String },
    /// Use a bootstrap version of a package
    UseBootstrap { package: PackageId },
    /// Build in multiple passes
    MultiPassBuild {
        first_pass: Vec<PackageId>,
        second_pass: Vec<PackageId>,
    },
    /// Manual intervention required
    ManualIntervention { reason: String },
}

/// Circular dependency detector
pub struct CircularDepDetector {
    /// Dependency graph
    graph: DiGraph<PackageId, DependencyEdge>,
    /// Node index lookup
    node_map: HashMap<PackageId, NodeIndex>,
}

/// Edge type for dependency graph
#[derive(Debug, Clone)]
struct DependencyEdge {
    /// Whether this dependency is conditional on USE flags
    conditional: bool,
    /// USE flag that enables this dependency (if conditional)
    use_flag: Option<String>,
    /// Whether this is a build-time only dependency
    build_only: bool,
}

impl CircularDepDetector {
    /// Create a new circular dependency detector
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_map: HashMap::new(),
        }
    }

    /// Build the dependency graph from packages
    pub fn build_graph(&mut self, packages: &[PackageInfo]) {
        self.graph.clear();
        self.node_map.clear();

        // Add nodes
        for pkg in packages {
            let node = self.graph.add_node(pkg.id.clone());
            self.node_map.insert(pkg.id.clone(), node);
        }

        // Add edges
        for pkg in packages {
            let pkg_node = match self.node_map.get(&pkg.id) {
                Some(&node) => node,
                None => continue,
            };

            // Process all dependencies
            for dep in &pkg.dependencies {
                if let Some(&dep_node) = self.node_map.get(&dep.package) {
                    let edge = DependencyEdge {
                        conditional: !matches!(dep.use_flags, crate::UseCondition::Always),
                        use_flag: match &dep.use_flags {
                            crate::UseCondition::IfEnabled(flag) => Some(flag.clone()),
                            _ => None,
                        },
                        build_only: dep.build_time && !dep.run_time,
                    };
                    self.graph.add_edge(dep_node, pkg_node, edge);
                }
            }

            for dep in &pkg.build_dependencies {
                if let Some(&dep_node) = self.node_map.get(&dep.package) {
                    let edge = DependencyEdge {
                        conditional: !matches!(dep.use_flags, crate::UseCondition::Always),
                        use_flag: match &dep.use_flags {
                            crate::UseCondition::IfEnabled(flag) => Some(flag.clone()),
                            _ => None,
                        },
                        build_only: true,
                    };
                    self.graph.add_edge(dep_node, pkg_node, edge);
                }
            }
        }
    }

    /// Detect all circular dependencies
    pub fn detect_cycles(&self) -> Vec<CircularDependency> {
        let mut cycles = Vec::new();

        // Use Tarjan's algorithm to find strongly connected components
        let sccs = tarjan_scc(&self.graph);

        for scc in sccs {
            // A cycle exists if SCC has more than one node
            if scc.len() > 1 {
                let packages: Vec<PackageId> =
                    scc.iter().map(|&node| self.graph[node].clone()).collect();

                let (breakable, suggestion) = self.analyze_cycle(&scc);

                cycles.push(CircularDependency {
                    packages,
                    breakable,
                    break_suggestion: suggestion,
                });
            }
        }

        cycles
    }

    /// Analyze a cycle and suggest how to break it
    fn analyze_cycle(&self, cycle_nodes: &[NodeIndex]) -> (bool, Option<CycleBreakSuggestion>) {
        let node_set: HashSet<_> = cycle_nodes.iter().cloned().collect();

        // Find edges within the cycle
        let mut conditional_edges = Vec::new();
        let mut build_only_edges = Vec::new();

        for &node in cycle_nodes {
            for edge in self.graph.edges(node) {
                if node_set.contains(&edge.target()) {
                    let edge_data = edge.weight();
                    if edge_data.conditional {
                        conditional_edges.push((node, edge.target(), edge_data.clone()));
                    }
                    if edge_data.build_only {
                        build_only_edges.push((node, edge.target(), edge_data.clone()));
                    }
                }
            }
        }

        // If there are conditional edges, we can break the cycle by disabling USE flags
        if let Some((source, _target, edge)) = conditional_edges.first() {
            if let Some(flag) = &edge.use_flag {
                return (
                    true,
                    Some(CycleBreakSuggestion::DisableUseFlag {
                        package: self.graph[*source].clone(),
                        flag: flag.clone(),
                    }),
                );
            }
        }

        // If there are build-only edges, we might be able to do a multi-pass build
        if !build_only_edges.is_empty() {
            // Find packages with only build-only deps in the cycle
            let mut first_pass = Vec::new();
            let mut second_pass = Vec::new();

            for &node in cycle_nodes {
                let has_runtime_cycle_dep = self
                    .graph
                    .edges(node)
                    .any(|e| node_set.contains(&e.target()) && !e.weight().build_only);

                if has_runtime_cycle_dep {
                    second_pass.push(self.graph[node].clone());
                } else {
                    first_pass.push(self.graph[node].clone());
                }
            }

            if !first_pass.is_empty() && !second_pass.is_empty() {
                return (
                    true,
                    Some(CycleBreakSuggestion::MultiPassBuild {
                        first_pass,
                        second_pass,
                    }),
                );
            }
        }

        // Check for known bootstrap packages
        for &node in cycle_nodes {
            let pkg_id = &self.graph[node];
            if Self::has_bootstrap_version(pkg_id) {
                return (
                    true,
                    Some(CycleBreakSuggestion::UseBootstrap {
                        package: pkg_id.clone(),
                    }),
                );
            }
        }

        // Cannot automatically break this cycle
        (
            false,
            Some(CycleBreakSuggestion::ManualIntervention {
                reason: format!(
                    "Circular dependency between {} packages requires manual intervention",
                    cycle_nodes.len()
                ),
            }),
        )
    }

    /// Check if a package has a known bootstrap version
    fn has_bootstrap_version(pkg_id: &PackageId) -> bool {
        // Known packages with bootstrap versions
        let bootstrap_packages = [
            ("sys-libs", "glibc"),
            ("sys-devel", "gcc"),
            ("sys-devel", "binutils"),
            ("dev-lang", "python"),
            ("dev-lang", "perl"),
            ("dev-lang", "rust"),
            ("dev-lang", "go"),
        ];

        bootstrap_packages
            .iter()
            .any(|(cat, name)| pkg_id.category == *cat && pkg_id.name == *name)
    }

    /// Attempt to break cycles and return a valid build order
    pub fn break_cycles_and_order(&self, cycles: &[CircularDependency]) -> Result<Vec<PackageId>> {
        if cycles.is_empty() {
            // No cycles, just do topological sort
            return self.topological_sort();
        }

        // Collect packages that need special handling
        let mut bootstrap_packages = HashSet::new();
        let mut disabled_flags: HashMap<PackageId, Vec<String>> = HashMap::new();

        for cycle in cycles {
            if !cycle.breakable {
                return Err(Error::CircularDependency(format!(
                    "Unbreakable circular dependency: {:?}",
                    cycle.packages
                )));
            }

            if let Some(ref suggestion) = cycle.break_suggestion {
                match suggestion {
                    CycleBreakSuggestion::UseBootstrap { package } => {
                        bootstrap_packages.insert(package.clone());
                    }
                    CycleBreakSuggestion::DisableUseFlag { package, flag } => {
                        disabled_flags
                            .entry(package.clone())
                            .or_default()
                            .push(flag.clone());
                    }
                    _ => {}
                }
            }
        }

        // Create modified graph without cycle-creating edges
        let mut modified_graph = self.graph.clone();

        // Remove edges based on disabled flags
        let edges_to_remove: Vec<_> = modified_graph
            .edge_indices()
            .filter(|&edge_idx| {
                let (source, _target) = modified_graph.edge_endpoints(edge_idx).unwrap();
                let edge = &modified_graph[edge_idx];
                let pkg_id = &modified_graph[source];

                if let Some(flags) = disabled_flags.get(pkg_id) {
                    if let Some(ref use_flag) = edge.use_flag {
                        return flags.contains(use_flag);
                    }
                }
                false
            })
            .collect();

        for edge_idx in edges_to_remove.into_iter().rev() {
            modified_graph.remove_edge(edge_idx);
        }

        // Perform topological sort on modified graph
        match petgraph::algo::toposort(&modified_graph, None) {
            Ok(sorted) => {
                let mut order: Vec<PackageId> = sorted
                    .into_iter()
                    .map(|node| modified_graph[node].clone())
                    .collect();

                // Put bootstrap packages first
                order.sort_by_key(|pkg| !bootstrap_packages.contains(pkg));

                Ok(order)
            }
            Err(_) => Err(Error::CircularDependency(
                "Could not break all cycles".to_string(),
            )),
        }
    }

    /// Perform topological sort (when no cycles)
    fn topological_sort(&self) -> Result<Vec<PackageId>> {
        match petgraph::algo::toposort(&self.graph, None) {
            Ok(sorted) => Ok(sorted
                .into_iter()
                .map(|node| self.graph[node].clone())
                .collect()),
            Err(_) => Err(Error::CircularDependency(
                "Unexpected cycle in dependency graph".to_string(),
            )),
        }
    }

    /// Get the strongly connected components
    pub fn get_sccs(&self) -> Vec<Vec<PackageId>> {
        kosaraju_scc(&self.graph)
            .into_iter()
            .map(|scc| {
                scc.into_iter()
                    .map(|node| self.graph[node].clone())
                    .collect()
            })
            .collect()
    }
}

impl Default for CircularDepDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Quick check for cycles in a package list
pub fn has_cycles(packages: &[PackageInfo]) -> bool {
    let mut detector = CircularDepDetector::new();
    detector.build_graph(packages);
    !detector.detect_cycles().is_empty()
}

/// Get all cycles in a package list
pub fn find_cycles(packages: &[PackageInfo]) -> Vec<CircularDependency> {
    let mut detector = CircularDepDetector::new();
    detector.build_graph(packages);
    detector.detect_cycles()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_pkg(category: &str, name: &str) -> PackageInfo {
        PackageInfo {
            id: PackageId::new(category, name),
            version: semver::Version::new(1, 0, 0),
            slot: "0".to_string(),
            description: String::new(),
            homepage: None,
            license: String::new(),
            keywords: vec![],
            use_flags: vec![],
            dependencies: vec![],
            build_dependencies: vec![],
            runtime_dependencies: vec![],
            source_url: None,
            source_hash: None,
            buck_target: String::new(),
            size: 0,
            installed_size: 0,
        }
    }

    #[test]
    fn test_no_cycles() {
        let packages = vec![
            make_pkg("sys-apps", "coreutils"),
            make_pkg("sys-libs", "glibc"),
        ];

        assert!(!has_cycles(&packages));
    }

    #[test]
    fn test_detector_creation() {
        let detector = CircularDepDetector::new();
        assert_eq!(detector.node_map.len(), 0);
    }
}
