use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use serde_json::Value;
use uuid::Uuid;
use crate::security::types::{SecurityEntity, SecurityEntityType};
use crate::security::templates;

/// Analyze environment template files
pub fn analyze_env_template(
    content: &str,
    path: &Path,
    normalized_path: &str,
    entity_map: &mut HashMap<String, String>,
) -> Result<Vec<SecurityEntity>> {
    let mut entities = Vec::new();

    let file_name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();

    let id = format!("{}:env:template:{}", normalized_path, Uuid::new_v4());
    entity_map.insert(file_name.clone(), id.clone());

    // Extract environment variables
    let mut env_vars = Vec::new();
    let mut has_secrets = false;
    let lines: Vec<&str> = content.lines().collect();

    for line in &lines {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some(equal_pos) = line.find('=') {
            let var_name = line[..equal_pos].trim();
            let _var_value = line[equal_pos+1..].trim();
            
            // Check for secret-related variable names
            if var_name.to_lowercase().contains("secret") ||
               var_name.to_lowercase().contains("key") ||
               var_name.to_lowercase().contains("password") ||
               var_name.to_lowercase().contains("token") ||
               var_name.to_lowercase().contains("api_key") {
                has_secrets = true;
            }

            env_vars.push(var_name.to_string());
        }
    }

    let mut config = HashMap::new();
    config.insert("file_name".to_string(), Value::String(file_name.clone()));
    config.insert("variable_count".to_string(), Value::Number(serde_json::Number::from(env_vars.len())));
    config.insert("has_secrets".to_string(), Value::Bool(has_secrets));
    if !env_vars.is_empty() {
        config.insert("variables".to_string(), Value::Array(
            env_vars.iter().take(20).map(|v| Value::String(v.clone())).collect()
        ));
    }

    entities.push(SecurityEntity {
        id: id.clone(),
        entity_type: SecurityEntityType::EnvironmentConfig,
        name: file_name,
        provider: templates::PROVIDER_GENERIC.to_string(),
        configuration: config,
        file_path: normalized_path.to_string(),
        line_number: None,
        arn: None,
        region: None,
    });

    Ok(entities)
}

