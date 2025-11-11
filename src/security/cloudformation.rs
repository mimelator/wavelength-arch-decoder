use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use serde_json::Value;
use crate::security::types::{SecurityEntity, SecurityEntityType, SecurityRelationship, SecurityVulnerability};
use crate::security::templates;

/// Analyze CloudFormation templates
pub fn analyze_cloudformation(
    content: &str,
    _path: &Path,
    normalized_path: &str,
    entity_map: &mut HashMap<String, String>,
) -> Result<(Vec<SecurityEntity>, Vec<SecurityRelationship>, Vec<SecurityVulnerability>)> {
    let mut entities = Vec::new();
    let relationships = Vec::new();
    let vulnerabilities = Vec::new();

    // Parse YAML
    let yaml: Value = match serde_yaml::from_str(content) {
        Ok(v) => v,
        Err(_) => return Ok((entities, relationships, vulnerabilities)),
    };

    if let Some(resources) = yaml.get("Resources").and_then(|r| r.as_object()) {
        for (resource_name, resource_def) in resources {
            if let Some(resource_type) = resource_def.get("Type").and_then(|t| t.as_str()) {
                let id = format!("{}:{}", normalized_path, resource_name);
                entity_map.insert(resource_name.clone(), id.clone());

                // Use template constants to avoid quote/prefix issues with AWS:: types
                match resource_type {
                    x if x == templates::AWS_IAM_ROLE => {
                        let mut config = HashMap::new();
                        config.insert("name".to_string(), Value::String(resource_name.clone()));
                        if let Some(props) = resource_def.get("Properties") {
                            config.insert("properties".to_string(), props.clone());
                        }

                        entities.push(SecurityEntity {
                            id: id.clone(),
                            entity_type: SecurityEntityType::IamRole,
                            name: resource_name.clone(),
                            provider: "aws".to_string(),
                            configuration: config.clone(),
                            file_path: normalized_path.to_string(),
                            line_number: None,
                            arn: None,
                            region: None,
                        });
                    }
                    x if x == templates::AWS_IAM_POLICY => {
                        let mut config = HashMap::new();
                        config.insert("name".to_string(), Value::String(resource_name.clone()));
                        if let Some(props) = resource_def.get("Properties") {
                            config.insert("properties".to_string(), props.clone());
                        }

                        entities.push(SecurityEntity {
                            id: id.clone(),
                            entity_type: SecurityEntityType::IamPolicy,
                            name: resource_name.clone(),
                            provider: "aws".to_string(),
                            configuration: config.clone(),
                            file_path: normalized_path.to_string(),
                            line_number: None,
                            arn: None,
                            region: None,
                        });
                    }
                    x if x == templates::AWS_LAMBDA_FUNCTION => {
                        let mut config = HashMap::new();
                        config.insert("name".to_string(), Value::String(resource_name.clone()));
                        if let Some(props) = resource_def.get("Properties") {
                            config.insert("properties".to_string(), props.clone());
                        }

                        entities.push(SecurityEntity {
                            id: id.clone(),
                            entity_type: SecurityEntityType::LambdaFunction,
                            name: resource_name.clone(),
                            provider: "aws".to_string(),
                            configuration: config.clone(),
                            file_path: normalized_path.to_string(),
                            line_number: None,
                            arn: None,
                            region: None,
                        });
                    }
                    x if x == templates::AWS_S3_BUCKET => {
                        let mut config = HashMap::new();
                        config.insert("name".to_string(), Value::String(resource_name.clone()));
                        if let Some(props) = resource_def.get("Properties") {
                            config.insert("properties".to_string(), props.clone());
                        }

                        entities.push(SecurityEntity {
                            id: id.clone(),
                            entity_type: SecurityEntityType::S3Bucket,
                            name: resource_name.clone(),
                            provider: "aws".to_string(),
                            configuration: config.clone(),
                            file_path: normalized_path.to_string(),
                            line_number: None,
                            arn: None,
                            region: None,
                        });
                    }
                    _ => {}
                }
            }
        }
    }

    Ok((entities, relationships, vulnerabilities))
}

