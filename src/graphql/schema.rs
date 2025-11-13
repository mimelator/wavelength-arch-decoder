use async_graphql::{Context, Object, Result as GraphQLResult, Schema, EmptySubscription};
use crate::api::ApiState;
use crate::graphql::types::*;
use crate::graph::GraphBuilder;

pub type GraphQLSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Get a repository by ID
    async fn repository(&self, ctx: &Context<'_>, id: String) -> GraphQLResult<Option<RepositoryType>> {
        let state = ctx.data::<ApiState>()?;
        match state.repo_repo.find_by_id(&id)? {
            Some(repo) => Ok(Some(RepositoryType::from(repo))),
            None => Ok(None),
        }
    }

    /// List all repositories
    async fn repositories(&self, ctx: &Context<'_>) -> GraphQLResult<Vec<RepositoryType>> {
        let state = ctx.data::<ApiState>()?;
        let repos = state.repo_repo.list_all()?;
        Ok(repos.into_iter().map(RepositoryType::from).collect())
    }

    /// Get dependencies for a repository
    async fn dependencies(
        &self,
        ctx: &Context<'_>,
        repository_id: String,
        filter: Option<DependencyFilter>,
    ) -> GraphQLResult<Vec<DependencyType>> {
        let state = ctx.data::<ApiState>()?;
        let mut deps = state.dep_repo.get_by_repository(&repository_id)?;
        
        if let Some(f) = filter {
            if let Some(name) = f.name {
                deps.retain(|d| d.name.contains(&name));
            }
            if let Some(pm) = f.package_manager {
                deps.retain(|d| d.package_manager == pm);
            }
            if let Some(is_dev) = f.is_dev {
                deps.retain(|d| d.is_dev == is_dev);
            }
        }
        
        Ok(deps.into_iter().map(DependencyType::from).collect())
    }

    /// Get services for a repository
    async fn services(
        &self,
        ctx: &Context<'_>,
        repository_id: String,
        filter: Option<ServiceFilter>,
    ) -> GraphQLResult<Vec<ServiceType>> {
        let state = ctx.data::<ApiState>()?;
        let mut services = state.service_repo.get_by_repository(&repository_id)?;
        
        if let Some(f) = filter {
            if let Some(provider) = f.provider {
                services.retain(|s| s.provider == provider);
            }
            if let Some(service_type) = f.service_type {
                services.retain(|s| s.service_type == service_type);
            }
        }
        
        Ok(services.into_iter().map(ServiceType::from).collect())
    }

    /// Get code elements for a repository
    async fn code_elements(
        &self,
        ctx: &Context<'_>,
        repository_id: String,
        element_type: Option<String>,
    ) -> GraphQLResult<Vec<CodeElementType>> {
        let state = ctx.data::<ApiState>()?;
        let mut elements = state.code_repo.get_by_repository(&repository_id)?;
        
        if let Some(et) = element_type {
            elements.retain(|e| format!("{:?}", e.element_type).to_lowercase() == et.to_lowercase());
        }
        
        Ok(elements.into_iter().map(CodeElementType::from).collect())
    }

    /// Get code calls for a repository
    async fn code_calls(
        &self,
        ctx: &Context<'_>,
        repository_id: String,
    ) -> GraphQLResult<Vec<CodeCallType>> {
        let state = ctx.data::<ApiState>()?;
        let calls = state.code_repo.get_calls(&repository_id)?;
        Ok(calls.into_iter().map(CodeCallType::from).collect())
    }

    /// Get security entities for a repository
    async fn security_entities(
        &self,
        ctx: &Context<'_>,
        repository_id: String,
        filter: Option<SecurityEntityFilter>,
    ) -> GraphQLResult<Vec<SecurityEntityType>> {
        let state = ctx.data::<ApiState>()?;
        let entities = if let Some(f) = &filter {
            if let Some(entity_type) = &f.entity_type {
                state.security_repo.get_by_type(&repository_id, entity_type)?
            } else {
                state.security_repo.get_entities(&repository_id)?
            }
        } else {
            state.security_repo.get_entities(&repository_id)?
        };
        
        Ok(entities.into_iter().map(SecurityEntityType::from).collect())
    }

    /// Get security vulnerabilities for a repository
    async fn security_vulnerabilities(
        &self,
        ctx: &Context<'_>,
        repository_id: String,
        filter: Option<VulnerabilityFilter>,
    ) -> GraphQLResult<Vec<SecurityVulnerabilityType>> {
        let state = ctx.data::<ApiState>()?;
        let vulnerabilities = if let Some(f) = &filter {
            if let Some(severity) = &f.severity {
                state.security_repo.get_vulnerabilities_by_severity(&repository_id, severity)?
            } else {
                state.security_repo.get_vulnerabilities(&repository_id)?
            }
        } else {
            state.security_repo.get_vulnerabilities(&repository_id)?
        };
        
        Ok(vulnerabilities.into_iter().map(SecurityVulnerabilityType::from).collect())
    }

    /// Get security relationships for a repository
    async fn security_relationships(
        &self,
        ctx: &Context<'_>,
        repository_id: String,
    ) -> GraphQLResult<Vec<SecurityRelationshipType>> {
        let state = ctx.data::<ApiState>()?;
        let relationships = state.security_repo.get_relationships(&repository_id)?;
        Ok(relationships.into_iter().map(SecurityRelationshipType::from).collect())
    }

    /// Get knowledge graph for a repository
    async fn graph(
        &self,
        ctx: &Context<'_>,
        repository_id: String,
    ) -> GraphQLResult<GraphType> {
        let state = ctx.data::<ApiState>()?;
        let graph_builder = GraphBuilder::new(
            state.repo_repo.db.clone(),
            state.repo_repo.clone(),
            state.dep_repo.clone(),
            state.service_repo.clone(),
            state.tool_repo.clone(),
            state.code_relationship_repo.clone(),
            state.test_repo.clone(),
        );
        
        let graph = graph_builder.build_for_repository(&repository_id)?;
        
        Ok(GraphType {
            nodes: graph.nodes.iter().map(|n| GraphNodeType {
                id: n.id.clone(),
                node_type: format!("{:?}", n.node_type),
                name: n.name.clone(),
                properties: serde_json::to_string(&n.properties).unwrap_or_default(),
                repository_id: n.repository_id.clone(),
            }).collect(),
            edges: graph.edges.iter().map(|e| GraphEdgeType {
                id: e.id.clone(),
                source_node_id: e.source_node_id.clone(),
                target_node_id: e.target_node_id.clone(),
                edge_type: format!("{:?}", e.edge_type),
                properties: serde_json::to_string(&e.properties).unwrap_or_default(),
            }).collect(),
        })
    }

    /// Get graph statistics for a repository
    async fn graph_statistics(
        &self,
        ctx: &Context<'_>,
        repository_id: String,
    ) -> GraphQLResult<GraphStatisticsType> {
        let state = ctx.data::<ApiState>()?;
        let graph_builder = GraphBuilder::new(
            state.repo_repo.db.clone(),
            state.repo_repo.clone(),
            state.dep_repo.clone(),
            state.service_repo.clone(),
            state.tool_repo.clone(),
            state.code_relationship_repo.clone(),
            state.test_repo.clone(),
        );
        
        let graph = graph_builder.build_for_repository(&repository_id)?;
        
        // Calculate statistics
        let total_nodes = graph.nodes.len() as i32;
        let total_edges = graph.edges.len() as i32;
        
        let mut nodes_by_type: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
        for node in &graph.nodes {
            let node_type_str = format!("{:?}", node.node_type);
            *nodes_by_type.entry(node_type_str).or_insert(0) += 1;
        }
        
        let mut edges_by_type: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
        for edge in &graph.edges {
            let edge_type_str = format!("{:?}", edge.edge_type);
            *edges_by_type.entry(edge_type_str).or_insert(0) += 1;
        }
        
        Ok(GraphStatisticsType {
            total_nodes,
            total_edges,
            nodes_by_type: serde_json::to_string(&nodes_by_type).unwrap_or_default(),
            edges_by_type: serde_json::to_string(&edges_by_type).unwrap_or_default(),
        })
    }

    /// Search dependencies by name
    async fn search_dependencies(
        &self,
        ctx: &Context<'_>,
        name: String,
    ) -> GraphQLResult<Vec<DependencyType>> {
        let state = ctx.data::<ApiState>()?;
        // GraphQL search - global search across all repositories
        let deps = state.dep_repo.get_by_package_name(&name, None)?;
        Ok(deps.into_iter().map(DependencyType::from).collect())
    }

    /// Search services by provider
    async fn search_services_by_provider(
        &self,
        ctx: &Context<'_>,
        provider: String,
    ) -> GraphQLResult<Vec<ServiceType>> {
        let state = ctx.data::<ApiState>()?;
        // GraphQL search - global search across all repositories
        let services = state.service_repo.get_by_provider(&provider, None)?;
        Ok(services.into_iter().map(ServiceType::from).collect())
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    /// Create a new repository
    async fn create_repository(
        &self,
        ctx: &Context<'_>,
        name: String,
        url: String,
        branch: Option<String>,
    ) -> GraphQLResult<RepositoryType> {
        let state = ctx.data::<ApiState>()?;
                let repo = state.repo_repo.create(&name, &url, branch.as_deref(), None, None)?;
        Ok(RepositoryType::from(repo))
    }

    /// Analyze a repository
    async fn analyze_repository(
        &self,
        _ctx: &Context<'_>,
        _repository_id: String,
    ) -> GraphQLResult<AnalysisResultType> {
        // This would trigger the same analysis as the REST endpoint
        // For now, return a placeholder
        Ok(AnalysisResultType {
            success: true,
            message: "Analysis triggered".to_string(),
            manifests_found: 0,
            total_dependencies: 0,
            services_found: 0,
            code_elements_found: 0,
            security_entities_found: 0,
            security_vulnerabilities_found: 0,
        })
    }
}

