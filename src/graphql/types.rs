use async_graphql::{SimpleObject, InputObject};
use crate::storage::{Repository, StoredDependency, StoredService};
use crate::analysis::{CodeElement, CodeCall};
use crate::security::{SecurityEntity, SecurityVulnerability, SecurityRelationship};

// GraphQL Types

#[derive(SimpleObject, Clone)]
pub struct RepositoryType {
    pub id: String,
    pub name: String,
    pub url: String,
    pub branch: String,
    pub created_at: String,
    pub last_analyzed_at: Option<String>,
}

impl From<Repository> for RepositoryType {
    fn from(repo: Repository) -> Self {
        RepositoryType {
            id: repo.id,
            name: repo.name,
            url: repo.url,
            branch: repo.branch,
            created_at: repo.created_at.to_rfc3339(),
            last_analyzed_at: repo.last_analyzed_at.map(|dt| dt.to_rfc3339()),
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct DependencyType {
    pub id: String,
    pub name: String,
    pub version: String,
    pub package_manager: String,
    pub is_dev: bool,
    pub is_optional: bool,
    pub file_path: String,
}

impl From<StoredDependency> for DependencyType {
    fn from(dep: StoredDependency) -> Self {
        DependencyType {
            id: dep.id,
            name: dep.name,
            version: dep.version,
            package_manager: dep.package_manager,
            is_dev: dep.is_dev,
            is_optional: dep.is_optional,
            file_path: dep.file_path,
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct ServiceType {
    pub id: String,
    pub name: String,
    pub service_type: String,
    pub provider: String,
    pub configuration: String,
    pub file_path: String,
    pub line_number: Option<i32>,
    pub confidence: f64,
}

impl From<StoredService> for ServiceType {
    fn from(svc: StoredService) -> Self {
        ServiceType {
            id: svc.id,
            name: svc.name,
            service_type: svc.service_type,
            provider: svc.provider,
            configuration: svc.configuration,
            file_path: svc.file_path,
            line_number: svc.line_number.map(|n| n as i32),
            confidence: svc.confidence,
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct CodeElementType {
    pub id: String,
    pub name: String,
    #[graphql(name = "type")]
    pub element_type: String,
    pub file_path: String,
    pub line_number: i32,
    pub language: String,
    pub signature: Option<String>,
    pub doc_comment: Option<String>,
    pub visibility: Option<String>,
    pub parameters: Vec<String>,
    pub return_type: Option<String>,
}

impl From<CodeElement> for CodeElementType {
    fn from(elem: CodeElement) -> Self {
        CodeElementType {
            id: elem.id,
            name: elem.name,
            element_type: format!("{:?}", elem.element_type),
            file_path: elem.file_path,
            line_number: elem.line_number as i32,
            language: elem.language,
            signature: elem.signature,
            doc_comment: elem.doc_comment,
            visibility: elem.visibility,
            parameters: elem.parameters,
            return_type: elem.return_type,
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct CodeCallType {
    pub caller_id: String,
    pub callee_id: String,
    #[graphql(name = "type")]
    pub call_type: String,
    pub line_number: i32,
}

impl From<CodeCall> for CodeCallType {
    fn from(call: CodeCall) -> Self {
        CodeCallType {
            caller_id: call.caller_id,
            callee_id: call.callee_id,
            call_type: call.call_type,
            line_number: call.line_number as i32,
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct SecurityEntityType {
    pub id: String,
    #[graphql(name = "type")]
    pub entity_type: String,
    pub name: String,
    pub provider: String,
    pub configuration: String,
    pub file_path: String,
    pub line_number: Option<i32>,
    pub arn: Option<String>,
    pub region: Option<String>,
}

impl From<SecurityEntity> for SecurityEntityType {
    fn from(entity: SecurityEntity) -> Self {
        SecurityEntityType {
            id: entity.id,
            entity_type: format!("{:?}", entity.entity_type),
            name: entity.name,
            provider: entity.provider,
            configuration: serde_json::to_string(&entity.configuration).unwrap_or_default(),
            file_path: entity.file_path,
            line_number: entity.line_number.map(|n| n as i32),
            arn: entity.arn,
            region: entity.region,
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct SecurityVulnerabilityType {
    pub id: String,
    pub entity_id: String,
    pub vulnerability_type: String,
    pub severity: String,
    pub description: String,
    pub recommendation: String,
    pub file_path: String,
    pub line_number: Option<i32>,
}

impl From<SecurityVulnerability> for SecurityVulnerabilityType {
    fn from(vuln: SecurityVulnerability) -> Self {
        SecurityVulnerabilityType {
            id: vuln.id,
            entity_id: vuln.entity_id,
            vulnerability_type: vuln.vulnerability_type,
            severity: format!("{:?}", vuln.severity),
            description: vuln.description,
            recommendation: vuln.recommendation,
            file_path: vuln.file_path,
            line_number: vuln.line_number.map(|n| n as i32),
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct SecurityRelationshipType {
    pub id: String,
    pub source_entity_id: String,
    pub target_entity_id: String,
    pub relationship_type: String,
    pub permissions: Vec<String>,
    pub condition: Option<String>,
}

impl From<SecurityRelationship> for SecurityRelationshipType {
    fn from(rel: SecurityRelationship) -> Self {
        SecurityRelationshipType {
            id: format!("{}-{}", rel.source_entity_id, rel.target_entity_id),
            source_entity_id: rel.source_entity_id,
            target_entity_id: rel.target_entity_id,
            relationship_type: rel.relationship_type,
            permissions: rel.permissions,
            condition: rel.condition,
        }
    }
}

#[derive(SimpleObject, Clone)]
pub struct GraphNodeType {
    pub id: String,
    #[graphql(name = "type")]
    pub node_type: String,
    pub name: String,
    pub properties: String,
    pub repository_id: Option<String>,
}

#[derive(SimpleObject, Clone)]
pub struct GraphEdgeType {
    pub id: String,
    pub source_node_id: String,
    pub target_node_id: String,
    #[graphql(name = "type")]
    pub edge_type: String,
    pub properties: String,
}

// Input Types

#[derive(InputObject)]
pub struct DependencyFilter {
    pub name: Option<String>,
    pub package_manager: Option<String>,
    pub is_dev: Option<bool>,
}

#[derive(InputObject)]
pub struct ServiceFilter {
    pub provider: Option<String>,
    pub service_type: Option<String>,
}

#[derive(InputObject)]
pub struct SecurityEntityFilter {
    #[graphql(name = "type")]
    pub entity_type: Option<String>,
    pub provider: Option<String>,
}

#[derive(InputObject)]
pub struct VulnerabilityFilter {
    pub severity: Option<String>,
    pub vulnerability_type: Option<String>,
}

#[derive(SimpleObject)]
pub struct GraphType {
    pub nodes: Vec<GraphNodeType>,
    pub edges: Vec<GraphEdgeType>,
}

#[derive(SimpleObject)]
pub struct GraphStatisticsType {
    pub total_nodes: i32,
    pub total_edges: i32,
    pub nodes_by_type: String,
    pub edges_by_type: String,
}

#[derive(SimpleObject)]
pub struct AnalysisResultType {
    pub success: bool,
    pub message: String,
    pub manifests_found: i32,
    pub total_dependencies: i32,
    pub services_found: i32,
    pub code_elements_found: i32,
    pub security_entities_found: i32,
    pub security_vulnerabilities_found: i32,
}

