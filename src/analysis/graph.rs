use anyhow::Result;
use std::collections::{HashMap, HashSet};
use crate::analysis::dependencies::{PackageDependency, PackageManager};

#[derive(Debug, Clone)]
pub struct DependencyNode {
    pub name: String,
    pub version: String,
    pub package_manager: PackageManager,
    pub dependencies: Vec<String>, // Names of direct dependencies
}

#[derive(Debug, Clone)]
pub struct DependencyGraph {
    nodes: HashMap<String, DependencyNode>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        DependencyGraph {
            nodes: HashMap::new(),
        }
    }

    /// Add a dependency node to the graph
    pub fn add_node(&mut self, dependency: PackageDependency, direct_deps: Vec<String>) {
        let key = format!("{}@{}", dependency.name, dependency.version);
        let node = DependencyNode {
            name: dependency.name.clone(),
            version: dependency.version,
            package_manager: dependency.package_manager,
            dependencies: direct_deps,
        };
        self.nodes.insert(key, node);
    }

    /// Get all nodes in the graph
    pub fn get_nodes(&self) -> Vec<&DependencyNode> {
        self.nodes.values().collect()
    }

    /// Get a specific node by name
    pub fn get_node(&self, name: &str) -> Option<&DependencyNode> {
        self.nodes.values().find(|n| n.name == name)
    }

    /// Resolve transitive dependencies for a given package
    pub fn resolve_transitive(&self, package_name: &str) -> Result<HashSet<String>> {
        let mut visited = HashSet::new();
        let mut result = HashSet::new();
        
        self.dfs(package_name, &mut visited, &mut result);
        
        Ok(result)
    }

    /// Depth-first search to find all transitive dependencies
    fn dfs(&self, package_name: &str, visited: &mut HashSet<String>, result: &mut HashSet<String>) {
        if visited.contains(package_name) {
            return;
        }
        visited.insert(package_name.to_string());

        if let Some(node) = self.get_node(package_name) {
            for dep_name in &node.dependencies {
                if !result.contains(dep_name) {
                    result.insert(dep_name.clone());
                    self.dfs(dep_name, visited, result);
                }
            }
        }
    }

    /// Detect version conflicts in the dependency graph
    pub fn detect_conflicts(&self) -> Vec<VersionConflict> {
        let mut conflicts = Vec::new();
        let mut package_versions: HashMap<String, HashSet<String>> = HashMap::new();

        // Collect all versions for each package
        for node in self.nodes.values() {
            package_versions
                .entry(node.name.clone())
                .or_insert_with(HashSet::new)
                .insert(node.version.clone());
        }

        // Find packages with multiple versions
        for (package_name, versions) in package_versions {
            if versions.len() > 1 {
                conflicts.push(VersionConflict {
                    package_name,
                    versions: versions.into_iter().collect(),
                });
            }
        }

        conflicts
    }

    /// Get all root dependencies (dependencies not depended upon by others)
    pub fn get_root_dependencies(&self) -> Vec<&DependencyNode> {
        let mut depended_upon = HashSet::new();
        
        for node in self.nodes.values() {
            for dep in &node.dependencies {
                depended_upon.insert(dep.clone());
            }
        }

        self.nodes
            .values()
            .filter(|node| !depended_upon.contains(&node.name))
            .collect()
    }

    /// Get dependency count statistics
    pub fn get_statistics(&self) -> DependencyStatistics {
        let total_dependencies = self.nodes.len();
        let unique_packages: HashSet<String> = self.nodes.values()
            .map(|n| n.name.clone())
            .collect();
        let conflicts = self.detect_conflicts();
        
        DependencyStatistics {
            total_dependencies,
            unique_packages: unique_packages.len(),
            conflicts: conflicts.len(),
            package_managers: self.get_package_manager_counts(),
        }
    }

    fn get_package_manager_counts(&self) -> HashMap<PackageManager, usize> {
        let mut counts = HashMap::new();
        for node in self.nodes.values() {
            *counts.entry(node.package_manager.clone()).or_insert(0) += 1;
        }
        counts
    }
}

#[derive(Debug, Clone)]
pub struct VersionConflict {
    pub package_name: String,
    pub versions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DependencyStatistics {
    pub total_dependencies: usize,
    pub unique_packages: usize,
    pub conflicts: usize,
    pub package_managers: HashMap<PackageManager, usize>,
}

