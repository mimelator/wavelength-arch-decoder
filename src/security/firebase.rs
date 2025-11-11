use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use serde_json::Value;
use uuid::Uuid;
use crate::security::types::{SecurityEntity, SecurityEntityType, SecurityVulnerability, VulnerabilitySeverity};
use crate::security::templates;

/// Analyze Firebase rules files
pub fn analyze_firebase_rules(
    content: &str,
    path: &Path,
    normalized_path: &str,
    entity_map: &mut HashMap<String, String>,
) -> Result<(Vec<SecurityEntity>, Vec<SecurityVulnerability>)> {
    let mut entities = Vec::new();
    let mut vulnerabilities = Vec::new();

    let file_name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();

    let rule_type = if file_name.contains("firestore") {
        templates::FIRESTORE_RULES
    } else if file_name.contains("storage") {
        templates::STORAGE_RULES
    } else if file_name.contains("database") {
        templates::DATABASE_RULES
    } else {
        templates::FIREBASE_RULES
    };

    let id = format!("{}:firebase:rules:{}", normalized_path, Uuid::new_v4());
    entity_map.insert(file_name.clone(), id.clone());

    let mut config = HashMap::new();
    config.insert("file_name".to_string(), Value::String(file_name.clone()));
    config.insert("rule_type".to_string(), Value::String(rule_type.to_string()));

    // Extract rules content (first 500 chars for preview)
    let preview = if content.len() > 500 {
        format!("{}...", &content[..500])
    } else {
        content.to_string()
    };
    config.insert("content_preview".to_string(), Value::String(preview));

    entities.push(SecurityEntity {
        id: id.clone(),
        entity_type: SecurityEntityType::FirebaseRules,
        name: format!("{} ({})", file_name, rule_type),
        provider: "firebase".to_string(),
        configuration: config,
        file_path: normalized_path.to_string(),
        line_number: None,
        arn: None,
        region: None,
    });

    // Use template constants to avoid prefix issues
    if content.contains(templates::ALLOW_RW_TRUE) || 
       content.contains(templates::ALLOW_RW_NULL) ||
       (content.contains(templates::ALLOW_RW) && content.contains(templates::IF_TRUE)) {
        vulnerabilities.push(SecurityVulnerability {
            id: format!("{}:vuln:1", id),
            entity_id: id.clone(),
            vulnerability_type: "OverlyPermissiveFirebaseRules".to_string(),
            severity: VulnerabilitySeverity::Critical,
            description: format!("{} {} {}", rule_type, templates::DESC_FIREBASE_UNRESTRICTED, templates::WORD_ACCESS),
            recommendation: templates::REC_FIREBASE_RULES.to_string(),
            file_path: normalized_path.to_string(),
            line_number: None,
        });
    }

    // Use template constants to avoid prefix issues
    // Check for missing authentication checks
    if content.contains(templates::ALLOW_READ) || content.contains(templates::ALLOW_WRITE) {
        let lines: Vec<&str> = content.lines().collect();
        for (line_num, line) in lines.iter().enumerate() {
            if (line.contains(templates::ALLOW_READ) || line.contains(templates::ALLOW_WRITE)) && 
               !line.contains(templates::REQUEST_AUTH) &&
               !line.contains(templates::IF_FALSE) {
                vulnerabilities.push(SecurityVulnerability {
                    id: format!("{}:vuln:{}", id, line_num + 2),
                    entity_id: id.clone(),
                    vulnerability_type: "MissingAuthenticationCheck".to_string(),
                    severity: VulnerabilitySeverity::High,
                    description: format!("{} {} {}", rule_type, templates::DESC_FIREBASE_UNAUTH, templates::WORD_ACCESS),
                    recommendation: templates::REC_FIREBASE_AUTH.to_string(),
                    file_path: normalized_path.to_string(),
                    line_number: Some(line_num + 1),
                });
                break; // Only report once per file
            }
        }
    }

    Ok((entities, vulnerabilities))
}

