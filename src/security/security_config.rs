use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use serde_json::Value;
use uuid::Uuid;
use crate::security::types::{SecurityEntity, SecurityEntityType};
use crate::security::templates;

/// Analyze security config files
pub fn analyze_security_config(
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

    let id = format!("{}:security:config:{}", normalized_path, Uuid::new_v4());
    entity_map.insert(file_name.clone(), id.clone());

    let mut config = HashMap::new();
    config.insert("file_name".to_string(), Value::String(file_name.clone()));
    
    // Try to parse as JSON or YAML
    if let Ok(json_value) = serde_json::from_str::<Value>(content) {
        config.insert("config_type".to_string(), Value::String(templates::CONFIG_TYPE_JSON.to_string()));
        config.insert("content".to_string(), json_value);
    } else if let Ok(yaml_value) = serde_yaml::from_str::<Value>(content) {
        config.insert("config_type".to_string(), Value::String(templates::CONFIG_TYPE_YAML.to_string()));
        config.insert("content".to_string(), yaml_value);
    } else {
        // Plain text - store preview
        let preview = if content.len() > 500 {
            format!("{}...", &content[..500])
        } else {
            content.to_string()
        };
        config.insert("content_preview".to_string(), Value::String(preview));
    }

    entities.push(SecurityEntity {
        id: id.clone(),
        entity_type: SecurityEntityType::SecurityConfig,
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

