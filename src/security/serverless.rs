use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use serde_json::Value;
use crate::security::types::{SecurityEntity, SecurityEntityType, SecurityRelationship, SecurityVulnerability};
use crate::security::cloudformation;

/// Analyze Serverless Framework files
pub fn analyze_serverless(
    content: &str,
    path: &Path,
    normalized_path: &str,
    entity_map: &mut HashMap<String, String>,
) -> Result<(Vec<SecurityEntity>, Vec<SecurityRelationship>, Vec<SecurityVulnerability>)> {
    let mut entities = Vec::new();
    let relationships = Vec::new();
    let vulnerabilities = Vec::new();

    let yaml: Value = match serde_yaml::from_str(content) {
        Ok(v) => v,
        Err(_) => return Ok((entities, relationships, vulnerabilities)),
    };

    // Extract functions
    if let Some(functions) = yaml.get("functions").and_then(|f| f.as_object()) {
        for (func_name, func_def) in functions {
            log::info!("Analyzing Serverless Framework function: {}", func_name);
            let id = format!("{}:{}", normalized_path, func_name);
            entity_map.insert(func_name.clone(), id.clone());

            let mut config = HashMap::new();
            config.insert("name".to_string(), Value::String(func_name.clone()));
            if let Some(handler) = func_def.get("handler").and_then(|h| h.as_str()) {
                config.insert("handler".to_string(), Value::String(handler.to_string()));
            }
            if let Some(runtime) = func_def.get("runtime").and_then(|r| r.as_str()) {
                config.insert("runtime".to_string(), Value::String(runtime.to_string()));
            }

            entities.push(SecurityEntity {
                id: id.clone(),
                entity_type: SecurityEntityType::LambdaFunction,
                name: func_name.clone(),
                provider: "aws".to_string(),
                configuration: config,
                file_path: normalized_path.to_string(),
                line_number: None,
                arn: None,
                region: None,
            });
        }
    }

    Ok((entities, relationships, vulnerabilities))
}

/// Analyze AWS SAM templates
pub fn analyze_sam(
    content: &str,
    path: &Path,
    normalized_path: &str,
    entity_map: &mut HashMap<String, String>,
) -> Result<(Vec<SecurityEntity>, Vec<SecurityRelationship>, Vec<SecurityVulnerability>)> {
    // Similar to CloudFormation analysis
    cloudformation::analyze_cloudformation(content, path, normalized_path, entity_map)
}

