use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use crate::storage::{Database, RepositoryRepository, DependencyRepository, ServiceRepository};
use crate::storage::{Repository, StoredDependency, StoredService};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum NodeType {
    Repository,
    Dependency,
    Service,
    PackageManager,
    ServiceProvider,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub node_type: NodeType,
    pub name: String,
    pub properties: HashMap<String, String>,
    pub repository_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EdgeType {
    DependsOn,           // Dependency -> Dependency
    UsesService,        // Repository -> Service
    HasDependency,      // Repository -> Dependency
    UsesPackageManager, // Repository -> PackageManager
    ProvidedBy,         // Service -> ServiceProvider
    RelatedTo,          // Generic relationship
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub id: String,
    pub source_node_id: String,
    pub target_node_id: String,
    pub edge_type: EdgeType,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

pub struct GraphBuilder {
    db: Database,
    repo_repo: RepositoryRepository,
    dep_repo: DependencyRepository,
    service_repo: ServiceRepository,
}

impl GraphBuilder {
    pub fn new(
        db: Database,
        repo_repo: RepositoryRepository,
        dep_repo: DependencyRepository,
        service_repo: ServiceRepository,
    ) -> Self {
        GraphBuilder {
            db,
            repo_repo,
            dep_repo,
            service_repo,
        }
    }

    /// Build knowledge graph for a specific repository
    pub fn build_for_repository(&self, repository_id: &str) -> Result<KnowledgeGraph> {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut node_map: HashMap<String, String> = HashMap::new(); // name -> node_id

        // Get repository
        let repo = match self.repo_repo.find_by_id(repository_id)? {
            Some(r) => r,
            None => return Err(anyhow::anyhow!("Repository not found")),
        };

        // Create repository node
        let repo_node_id = Uuid::new_v4().to_string();
        let mut repo_properties = HashMap::new();
        repo_properties.insert("url".to_string(), repo.url.clone());
        repo_properties.insert("branch".to_string(), repo.branch.clone());
        
        nodes.push(GraphNode {
            id: repo_node_id.clone(),
            node_type: NodeType::Repository,
            name: repo.name.clone(),
            properties: repo_properties,
            repository_id: Some(repo.id.clone()),
        });

        // Get dependencies
        let dependencies = self.dep_repo.get_by_repository(repository_id)?;
        
        // Group dependencies by package manager
        let mut package_managers: HashSet<String> = HashSet::new();
        for dep in &dependencies {
            package_managers.insert(dep.package_manager.clone());
        }

        // Create package manager nodes and edges
        for pm in &package_managers {
            let pm_node_id = if let Some(id) = node_map.get(pm) {
                id.clone()
            } else {
                let id = Uuid::new_v4().to_string();
                let mut pm_props = HashMap::new();
                pm_props.insert("type".to_string(), "package_manager".to_string());
                
                nodes.push(GraphNode {
                    id: id.clone(),
                    node_type: NodeType::PackageManager,
                    name: pm.clone(),
                    properties: pm_props,
                    repository_id: Some(repository_id.to_string()),
                });
                node_map.insert(pm.clone(), id.clone());
                id
            };

            // Repository uses package manager
            edges.push(GraphEdge {
                id: Uuid::new_v4().to_string(),
                source_node_id: repo_node_id.clone(),
                target_node_id: pm_node_id.clone(),
                edge_type: EdgeType::UsesPackageManager,
                properties: HashMap::new(),
            });
        }

        // Create dependency nodes
        for dep in &dependencies {
            let dep_node_id = if let Some(id) = node_map.get(&dep.name) {
                id.clone()
            } else {
                let id = Uuid::new_v4().to_string();
                let mut dep_props = HashMap::new();
                dep_props.insert("version".to_string(), dep.version.clone());
                dep_props.insert("package_manager".to_string(), dep.package_manager.clone());
                dep_props.insert("is_dev".to_string(), dep.is_dev.to_string());
                dep_props.insert("is_optional".to_string(), dep.is_optional.to_string());
                
                nodes.push(GraphNode {
                    id: id.clone(),
                    node_type: NodeType::Dependency,
                    name: dep.name.clone(),
                    properties: dep_props,
                    repository_id: Some(repository_id.to_string()),
                });
                node_map.insert(dep.name.clone(), id.clone());
                id
            };

            // Repository has dependency
            edges.push(GraphEdge {
                id: Uuid::new_v4().to_string(),
                source_node_id: repo_node_id.clone(),
                target_node_id: dep_node_id.clone(),
                edge_type: EdgeType::HasDependency,
                properties: HashMap::new(),
            });

            // Dependency uses package manager
            if let Some(pm_node_id) = node_map.get(&dep.package_manager) {
                edges.push(GraphEdge {
                    id: Uuid::new_v4().to_string(),
                    source_node_id: dep_node_id.clone(),
                    target_node_id: pm_node_id.clone(),
                    edge_type: EdgeType::UsesPackageManager,
                    properties: HashMap::new(),
                });
            }
        }

        // Get services
        let services = self.service_repo.get_by_repository(repository_id)?;

        // Group services by provider
        let mut service_providers: HashSet<String> = HashSet::new();
        for service in &services {
            service_providers.insert(service.provider.clone());
        }

        // Create service provider nodes
        for provider in &service_providers {
            let provider_node_id = if let Some(id) = node_map.get(provider) {
                id.clone()
            } else {
                let id = Uuid::new_v4().to_string();
                let mut provider_props = HashMap::new();
                provider_props.insert("type".to_string(), "service_provider".to_string());
                
                nodes.push(GraphNode {
                    id: id.clone(),
                    node_type: NodeType::ServiceProvider,
                    name: provider.clone(),
                    properties: provider_props,
                    repository_id: Some(repository_id.to_string()),
                });
                node_map.insert(provider.clone(), id.clone());
                id
            };
        }

        // Create service nodes
        for service in &services {
            let service_node_id = Uuid::new_v4().to_string();
            let mut service_props = HashMap::new();
            service_props.insert("service_type".to_string(), service.service_type.clone());
            service_props.insert("confidence".to_string(), service.confidence.to_string());
            service_props.insert("file_path".to_string(), service.file_path.clone());
            if let Some(line) = service.line_number {
                service_props.insert("line_number".to_string(), line.to_string());
            }
            
            // Parse configuration JSON if present
            if let Ok(config) = serde_json::from_str::<serde_json::Value>(&service.configuration) {
                for (key, value) in config.as_object().unwrap_or(&serde_json::Map::new()) {
                    if let Some(val_str) = value.as_str() {
                        service_props.insert(format!("config_{}", key), val_str.to_string());
                    }
                }
            }

            nodes.push(GraphNode {
                id: service_node_id.clone(),
                node_type: NodeType::Service,
                name: service.name.clone(),
                properties: service_props,
                repository_id: Some(repository_id.to_string()),
            });

            // Repository uses service
            edges.push(GraphEdge {
                id: Uuid::new_v4().to_string(),
                source_node_id: repo_node_id.clone(),
                target_node_id: service_node_id.clone(),
                edge_type: EdgeType::UsesService,
                properties: HashMap::new(),
            });

            // Service provided by provider
            if let Some(provider_node_id) = node_map.get(&service.provider) {
                edges.push(GraphEdge {
                    id: Uuid::new_v4().to_string(),
                    source_node_id: service_node_id.clone(),
                    target_node_id: provider_node_id.clone(),
                    edge_type: EdgeType::ProvidedBy,
                    properties: HashMap::new(),
                });
            }
        }

        Ok(KnowledgeGraph { nodes, edges })
    }

    /// Store graph in database
    pub fn store_graph(&self, repository_id: &str, graph: &KnowledgeGraph) -> Result<()> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();

        // Delete existing graph nodes and edges for this repository
        conn.execute(
            "DELETE FROM graph_edges WHERE source_node_id IN (
                SELECT id FROM graph_nodes WHERE repository_id = ?1
            ) OR target_node_id IN (
                SELECT id FROM graph_nodes WHERE repository_id = ?1
            )",
            [repository_id],
        )?;

        conn.execute(
            "DELETE FROM graph_nodes WHERE repository_id = ?1",
            [repository_id],
        )?;

        // Insert nodes
        for node in &graph.nodes {
            let node_type_str = self.node_type_to_string(&node.node_type);
            let properties_json = serde_json::to_string(&node.properties)?;

            conn.execute(
                "INSERT INTO graph_nodes (id, repository_id, node_type, name, properties, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'))",
                rusqlite::params![
                    node.id,
                    repository_id,
                    node_type_str,
                    node.name,
                    properties_json
                ],
            )?;
        }

        // Insert edges
        for edge in &graph.edges {
            let edge_type_str = self.edge_type_to_string(&edge.edge_type);
            let properties_json = serde_json::to_string(&edge.properties)?;

            conn.execute(
                "INSERT INTO graph_edges (id, source_node_id, target_node_id, edge_type, properties, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'))",
                rusqlite::params![
                    edge.id,
                    edge.source_node_id,
                    edge.target_node_id,
                    edge_type_str,
                    properties_json
                ],
            )?;
        }

        Ok(())
    }

    /// Get graph from database
    pub fn get_graph(&self, repository_id: &str) -> Result<KnowledgeGraph> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();

        // Get nodes
        let mut stmt = conn.prepare(
            "SELECT id, node_type, name, properties FROM graph_nodes WHERE repository_id = ?1"
        )?;

        let nodes = stmt.query_map([repository_id], |row| {
            let node_type_str: String = row.get(1)?;
            let properties_json: String = row.get(3)?;
            let properties: HashMap<String, String> = serde_json::from_str(&properties_json)
                .unwrap_or_default();

            Ok(GraphNode {
                id: row.get(0)?,
                node_type: self.string_to_node_type(&node_type_str),
                name: row.get(2)?,
                properties,
                repository_id: Some(repository_id.to_string()),
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        // Get edges
        let mut stmt = conn.prepare(
            "SELECT id, source_node_id, target_node_id, edge_type, properties
             FROM graph_edges
             WHERE source_node_id IN (
                 SELECT id FROM graph_nodes WHERE repository_id = ?1
             ) OR target_node_id IN (
                 SELECT id FROM graph_nodes WHERE repository_id = ?1
             )"
        )?;

        let edges = stmt.query_map([repository_id], |row| {
            let edge_type_str: String = row.get(3)?;
            let properties_json: String = row.get(4)?;
            let properties: HashMap<String, String> = serde_json::from_str(&properties_json)
                .unwrap_or_default();

            Ok(GraphEdge {
                id: row.get(0)?,
                source_node_id: row.get(1)?,
                target_node_id: row.get(2)?,
                edge_type: self.string_to_edge_type(&edge_type_str),
                properties,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(KnowledgeGraph { nodes, edges })
    }

    fn node_type_to_string(&self, node_type: &NodeType) -> String {
        match node_type {
            NodeType::Repository => "repository",
            NodeType::Dependency => "dependency",
            NodeType::Service => "service",
            NodeType::PackageManager => "package_manager",
            NodeType::ServiceProvider => "service_provider",
        }.to_string()
    }

    fn string_to_node_type(&self, s: &str) -> NodeType {
        match s {
            "repository" => NodeType::Repository,
            "dependency" => NodeType::Dependency,
            "service" => NodeType::Service,
            "package_manager" => NodeType::PackageManager,
            "service_provider" => NodeType::ServiceProvider,
            _ => NodeType::Repository,
        }
    }

    fn edge_type_to_string(&self, edge_type: &EdgeType) -> String {
        match edge_type {
            EdgeType::DependsOn => "depends_on",
            EdgeType::UsesService => "uses_service",
            EdgeType::HasDependency => "has_dependency",
            EdgeType::UsesPackageManager => "uses_package_manager",
            EdgeType::ProvidedBy => "provided_by",
            EdgeType::RelatedTo => "related_to",
        }.to_string()
    }

    fn string_to_edge_type(&self, s: &str) -> EdgeType {
        match s {
            "depends_on" => EdgeType::DependsOn,
            "uses_service" => EdgeType::UsesService,
            "has_dependency" => EdgeType::HasDependency,
            "uses_package_manager" => EdgeType::UsesPackageManager,
            "provided_by" => EdgeType::ProvidedBy,
            "related_to" => EdgeType::RelatedTo,
            _ => EdgeType::RelatedTo,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStatistics {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub nodes_by_type: HashMap<String, usize>,
    pub edges_by_type: HashMap<String, usize>,
    pub most_connected_nodes: Vec<(String, usize)>,
}

impl KnowledgeGraph {
    /// Get statistics about the graph
    pub fn get_statistics(&self) -> GraphStatistics {
        let mut nodes_by_type: HashMap<String, usize> = HashMap::new();
        let mut edges_by_type: HashMap<String, usize> = HashMap::new();
        let mut node_connections: HashMap<String, usize> = HashMap::new();

        for node in &self.nodes {
            let type_str = format!("{:?}", node.node_type);
            *nodes_by_type.entry(type_str).or_insert(0) += 1;
            node_connections.insert(node.id.clone(), 0);
        }

        for edge in &self.edges {
            let type_str = format!("{:?}", edge.edge_type);
            *edges_by_type.entry(type_str).or_insert(0) += 1;
            
            *node_connections.entry(edge.source_node_id.clone()).or_insert(0) += 1;
            *node_connections.entry(edge.target_node_id.clone()).or_insert(0) += 1;
        }

        let mut most_connected: Vec<(String, usize)> = node_connections
            .into_iter()
            .collect();
        most_connected.sort_by(|a, b| b.1.cmp(&a.1));
        most_connected.truncate(10);

        GraphStatistics {
            total_nodes: self.nodes.len(),
            total_edges: self.edges.len(),
            nodes_by_type,
            edges_by_type,
            most_connected_nodes: most_connected,
        }
    }

    /// Find nodes by type
    pub fn find_nodes_by_type(&self, node_type: &NodeType) -> Vec<&GraphNode> {
        self.nodes.iter()
            .filter(|n| n.node_type == *node_type)
            .collect()
    }

    /// Find edges connected to a node
    pub fn find_edges_for_node(&self, node_id: &str) -> Vec<&GraphEdge> {
        self.edges.iter()
            .filter(|e| e.source_node_id == node_id || e.target_node_id == node_id)
            .collect()
    }

    /// Get neighbors of a node
    pub fn get_neighbors(&self, node_id: &str) -> Vec<&GraphNode> {
        let neighbor_ids: HashSet<String> = self.edges.iter()
            .filter_map(|e| {
                if e.source_node_id == node_id {
                    Some(e.target_node_id.clone())
                } else if e.target_node_id == node_id {
                    Some(e.source_node_id.clone())
                } else {
                    None
                }
            })
            .collect();

        self.nodes.iter()
            .filter(|n| neighbor_ids.contains(&n.id))
            .collect()
    }
}

