use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use crate::storage::{Database, RepositoryRepository, DependencyRepository, ServiceRepository, ToolRepository, CodeRelationshipRepository};
use crate::storage::{Repository, StoredDependency, StoredService, StoredTool};
use crate::analysis::RelationshipTargetType;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum NodeType {
    Repository,
    Dependency,
    Service,
    PackageManager,
    ServiceProvider,
    Tool,
    CodeElement,
    SecurityEntity,
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
    UsesTool,           // Repository -> Tool
    ToolUsesDependency, // Tool -> Dependency
    ToolUsesService,    // Tool -> Service
    ToolGenerates,      // Tool -> CodeElement
    CodeUsesService,     // CodeElement -> Service
    CodeUsesDependency, // CodeElement -> Dependency
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
    tool_repo: ToolRepository,
    code_relationship_repo: CodeRelationshipRepository,
}

impl GraphBuilder {
    pub fn new(
        db: Database,
        repo_repo: RepositoryRepository,
        dep_repo: DependencyRepository,
        service_repo: ServiceRepository,
        tool_repo: ToolRepository,
        code_relationship_repo: CodeRelationshipRepository,
    ) -> Self {
        GraphBuilder {
            db,
            repo_repo,
            dep_repo,
            service_repo,
            tool_repo,
            code_relationship_repo,
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
        
        // Track dependency node IDs for code relationships
        let mut dep_node_ids: HashMap<String, String> = HashMap::new();
        
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
            
            // Track dependency node ID by dep.id for code relationships
            dep_node_ids.insert(dep.id.clone(), dep_node_id.clone());

            // Skip creating "has dependency" edges - they're too generic and clutter the graph
            // Dependencies are already connected to package managers, which is more informative
            // Repository has dependency - REMOVED (too generic)
            // edges.push(GraphEdge {
            //     id: Uuid::new_v4().to_string(),
            //     source_node_id: repo_node_id.clone(),
            //     target_node_id: dep_node_id.clone(),
            //     edge_type: EdgeType::HasDependency,
            //     properties: HashMap::new(),
            // });

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

        // Track service and dependency node IDs for code relationships
        let mut service_node_ids: HashMap<String, String> = HashMap::new();
        let mut dep_node_ids: HashMap<String, String> = HashMap::new();
        
        // Create service nodes
        for service in &services {
            let service_node_id = Uuid::new_v4().to_string();
            service_node_ids.insert(service.id.clone(), service_node_id.clone());
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

        // Get tools
        let tools = self.tool_repo.get_tools_by_repository(repository_id)?;
        
        // Create tool nodes and relationships
        for tool in &tools {
            let tool_node_id = Uuid::new_v4().to_string();
            let mut tool_props = HashMap::new();
            tool_props.insert("tool_type".to_string(), tool.tool_type.clone());
            tool_props.insert("category".to_string(), tool.category.clone());
            tool_props.insert("detection_method".to_string(), tool.detection_method.clone());
            tool_props.insert("confidence".to_string(), tool.confidence.to_string());
            tool_props.insert("file_path".to_string(), tool.file_path.clone());
            if let Some(line) = tool.line_number {
                tool_props.insert("line_number".to_string(), line.to_string());
            }
            if let Some(ref version) = tool.version {
                tool_props.insert("version".to_string(), version.clone());
            }
            
            // Parse configuration JSON if present
            if let Ok(config) = serde_json::from_str::<serde_json::Value>(&tool.configuration) {
                for (key, value) in config.as_object().unwrap_or(&serde_json::Map::new()) {
                    if let Some(val_str) = value.as_str() {
                        tool_props.insert(format!("config_{}", key), val_str.to_string());
                    }
                }
            }

            nodes.push(GraphNode {
                id: tool_node_id.clone(),
                node_type: NodeType::Tool,
                name: tool.name.clone(),
                properties: tool_props,
                repository_id: Some(repository_id.to_string()),
            });

            // Repository uses tool
            edges.push(GraphEdge {
                id: Uuid::new_v4().to_string(),
                source_node_id: repo_node_id.clone(),
                target_node_id: tool_node_id.clone(),
                edge_type: EdgeType::UsesTool,
                properties: HashMap::new(),
            });

            // Try to link tool to dependencies it uses
            // Check if tool name matches a dependency name
            for dep in &dependencies {
                let dep_name_lower = dep.name.to_lowercase();
                let tool_name_lower = tool.name.to_lowercase();
                
                // Check if tool uses this dependency (e.g., "webpack" tool uses "webpack" dependency)
                if dep_name_lower.contains(&tool_name_lower) || tool_name_lower.contains(&dep_name_lower) {
                    if let Some(dep_node_id) = node_map.get(&format!("{}:{}", dep.package_manager, dep.name)) {
                        edges.push(GraphEdge {
                            id: Uuid::new_v4().to_string(),
                            source_node_id: tool_node_id.clone(),
                            target_node_id: dep_node_id.clone(),
                            edge_type: EdgeType::ToolUsesDependency,
                            properties: HashMap::new(),
                        });
                    }
                }
            }

            // Try to link tool to services it interacts with
            // Check tool configuration for service references
            if let Ok(config) = serde_json::from_str::<serde_json::Value>(&tool.configuration) {
                if let Some(service_name) = config.get("service").and_then(|v| v.as_str()) {
                    // Find matching service node
                    for service in &services {
                        if service.name.to_lowercase().contains(&service_name.to_lowercase()) {
                            // Find the service node ID (we need to track this)
                            // For now, we'll create a simple match
                            // In a full implementation, we'd track service node IDs
                        }
                    }
                }
            }
        }

        // Add code relationships (code elements to services/dependencies)
        use crate::storage::CodeElementRepository;
        let code_repo = CodeElementRepository::new(self.db.clone());
        if let Ok(code_elements) = code_repo.get_by_repository(repository_id) {
            // Track code element nodes we create
            let mut code_element_nodes: HashMap<String, String> = HashMap::new();
            
            // For each code element with relationships, create edges
            for code_element in &code_elements {
                if let Ok(relationships) = self.code_relationship_repo.get_by_code_element(&code_element.id) {
                    if relationships.is_empty() {
                        continue; // Skip elements without relationships
                    }
                    
                    // Create code element node if it has relationships
                    let code_node_id = format!("code:{}", code_element.id);
                    if !code_element_nodes.contains_key(&code_element.id) {
                        nodes.push(GraphNode {
                            id: code_node_id.clone(),
                            node_type: NodeType::CodeElement,
                            name: code_element.name.clone(),
                            properties: {
                                let mut props = HashMap::new();
                                props.insert("file_path".to_string(), code_element.file_path.clone());
                                props.insert("line_number".to_string(), code_element.line_number.to_string());
                                props.insert("element_type".to_string(), format!("{:?}", code_element.element_type));
                                props.insert("language".to_string(), code_element.language.clone());
                                props
                            },
                            repository_id: Some(repository_id.to_string()),
                        });
                        code_element_nodes.insert(code_element.id.clone(), code_node_id.clone());
                    } else {
                        // Get existing node ID
                        let code_node_id = code_element_nodes.get(&code_element.id).unwrap().clone();
                    }
                    
                    let code_node_id = code_element_nodes.get(&code_element.id).unwrap().clone();
                    
                    for rel in &relationships {
                        match rel.target_type {
                            RelationshipTargetType::Service => {
                                if let Some(service_node_id) = service_node_ids.get(&rel.target_id) {
                                    edges.push(GraphEdge {
                                        id: Uuid::new_v4().to_string(),
                                        source_node_id: code_node_id.clone(),
                                        target_node_id: service_node_id.clone(),
                                        edge_type: EdgeType::CodeUsesService,
                                        properties: {
                                            let mut props = HashMap::new();
                                            props.insert("confidence".to_string(), rel.confidence.to_string());
                                            props.insert("evidence".to_string(), rel.evidence.clone());
                                            props
                                        },
                                    });
                                }
                            },
                            RelationshipTargetType::Dependency => {
                                if let Some(dep_node_id) = dep_node_ids.get(&rel.target_id) {
                                    edges.push(GraphEdge {
                                        id: Uuid::new_v4().to_string(),
                                        source_node_id: code_node_id.clone(),
                                        target_node_id: dep_node_id.clone(),
                                        edge_type: EdgeType::CodeUsesDependency,
                                        properties: {
                                            let mut props = HashMap::new();
                                            props.insert("confidence".to_string(), rel.confidence.to_string());
                                            props.insert("evidence".to_string(), rel.evidence.clone());
                                            props
                                        },
                                    });
                                }
                            }
                        }
                    }
                }
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

        // Get edges - only include edges where BOTH source and target nodes belong to this repository
        // This prevents showing data from other repositories
        let mut stmt = conn.prepare(
            "SELECT e.id, e.source_node_id, e.target_node_id, e.edge_type, e.properties
             FROM graph_edges e
             INNER JOIN graph_nodes source_nodes ON e.source_node_id = source_nodes.id
             INNER JOIN graph_nodes target_nodes ON e.target_node_id = target_nodes.id
             WHERE source_nodes.repository_id = ?1 AND target_nodes.repository_id = ?1"
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
        
        log::debug!("Retrieved {} nodes and {} edges for repository {}", nodes.len(), edges.len(), repository_id);

        Ok(KnowledgeGraph { nodes, edges })
    }

    fn node_type_to_string(&self, node_type: &NodeType) -> String {
        match node_type {
            NodeType::Repository => "repository",
            NodeType::Dependency => "dependency",
            NodeType::Service => "service",
            NodeType::PackageManager => "package_manager",
            NodeType::ServiceProvider => "service_provider",
            NodeType::Tool => "tool",
            NodeType::CodeElement => "code_element",
            NodeType::SecurityEntity => "security_entity",
        }.to_string()
    }

    fn string_to_node_type(&self, s: &str) -> NodeType {
        match s {
            "repository" => NodeType::Repository,
            "dependency" => NodeType::Dependency,
            "service" => NodeType::Service,
            "package_manager" => NodeType::PackageManager,
            "service_provider" => NodeType::ServiceProvider,
            "tool" => NodeType::Tool,
            "code_element" => NodeType::CodeElement,
            "security_entity" => NodeType::SecurityEntity,
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
            EdgeType::UsesTool => "uses_tool",
            EdgeType::ToolUsesDependency => "tool_uses_dependency",
            EdgeType::ToolUsesService => "tool_uses_service",
            EdgeType::ToolGenerates => "tool_generates",
            EdgeType::CodeUsesService => "code_uses_service",
            EdgeType::CodeUsesDependency => "code_uses_dependency",
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
            "uses_tool" => EdgeType::UsesTool,
            "tool_uses_dependency" => EdgeType::ToolUsesDependency,
            "tool_uses_service" => EdgeType::ToolUsesService,
            "tool_generates" => EdgeType::ToolGenerates,
            "code_uses_service" => EdgeType::CodeUsesService,
            "code_uses_dependency" => EdgeType::CodeUsesDependency,
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

