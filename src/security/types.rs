use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SecurityEntityType {
    IamRole,
    IamPolicy,
    LambdaFunction,
    S3Bucket,
    SecurityGroup,
    Vpc,
    Subnet,
    Ec2Instance,
    RdsInstance,
    ApiGateway,
    FirebaseRules,
    EnvironmentConfig,
    SecurityConfig,
    ApiKey,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VulnerabilitySeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEntity {
    pub id: String,
    pub entity_type: SecurityEntityType,
    pub name: String,
    pub provider: String, // aws, azure, gcp
    pub configuration: HashMap<String, Value>,
    pub file_path: String,
    pub line_number: Option<usize>,
    pub arn: Option<String>,
    pub region: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRelationship {
    pub source_entity_id: String,
    pub target_entity_id: String,
    pub relationship_type: String, // uses, allows_access, depends_on, etc.
    pub permissions: Vec<String>,
    pub condition: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityVulnerability {
    pub id: String,
    pub entity_id: String,
    pub vulnerability_type: String,
    pub severity: VulnerabilitySeverity,
    pub description: String,
    pub recommendation: String,
    pub file_path: String,
    pub line_number: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAnalysis {
    pub entities: Vec<SecurityEntity>,
    pub relationships: Vec<SecurityRelationship>,
    pub vulnerabilities: Vec<SecurityVulnerability>,
}

