use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use serde_json::Value;
use crate::security::types::{SecurityEntity, SecurityEntityType, SecurityRelationship, SecurityVulnerability};
use crate::security::helpers::*;
use crate::security::vulnerabilities::*;
use crate::security::templates;

/// Analyze Terraform files
pub fn analyze_terraform(
    content: &str,
    path: &Path,
    normalized_path: &str,
    entity_map: &mut HashMap<String, String>,
) -> Result<(Vec<SecurityEntity>, Vec<SecurityRelationship>, Vec<SecurityVulnerability>)> {
    let mut entities = Vec::new();
    let mut relationships = Vec::new();
    let mut vulnerabilities = Vec::new();

    let lines: Vec<&str> = content.lines().collect();

    // Extract IAM roles
    for (line_num, line) in lines.iter().enumerate() {
        if line.contains("resource \"aws_iam_role\"") || line.contains("aws_iam_role") {
            if let Some((name, config)) = extract_terraform_resource(content, line_num, "aws_iam_role") {
                let id = format!("{}:{}:{}", normalized_path, line_num, name);
                entity_map.insert(name.clone(), id.clone());

                let mut entity_config = HashMap::new();
                entity_config.insert("name".to_string(), Value::String(name.clone()));
                if let Some(arn) = extract_arn_from_config(&config) {
                    entity_config.insert("arn".to_string(), Value::String(arn.clone()));
                }

                entities.push(SecurityEntity {
                    id: id.clone(),
                    entity_type: SecurityEntityType::IamRole,
                    name,
                    provider: "aws".to_string(),
                    configuration: entity_config,
                    file_path: normalized_path.to_string(),
                    line_number: Some(line_num + 1),
                    arn: extract_arn_from_config(&config),
                    region: extract_region_from_config(&config),
                });

                // Check for vulnerabilities
                if let Some(vuln) = check_iam_role_vulnerabilities(&config, &id, path, normalized_path, line_num) {
                    vulnerabilities.push(vuln);
                }
            }
        }

        // Extract IAM policies
        if line.contains(&templates::terraform_resource_pattern("aws_iam_policy")) || line.contains("aws_iam_policy") {
            if let Some((name, config)) = extract_terraform_resource(content, line_num, "aws_iam_policy") {
                let id = format!("{}:{}:{}", normalized_path, line_num, name);
                entity_map.insert(name.clone(), id.clone());

                let mut entity_config = HashMap::new();
                entity_config.insert("name".to_string(), Value::String(name.clone()));
                if let Some(policy_doc) = extract_policy_document(&config) {
                    entity_config.insert("policy_document".to_string(), Value::String(policy_doc.clone()));
                }

                entities.push(SecurityEntity {
                    id: id.clone(),
                    entity_type: SecurityEntityType::IamPolicy,
                    name,
                    provider: "aws".to_string(),
                    configuration: entity_config,
                    file_path: normalized_path.to_string(),
                    line_number: Some(line_num + 1),
                    arn: extract_arn_from_config(&config),
                    region: None,
                });

                // Check for vulnerabilities
                if let Some(vuln) = check_iam_policy_vulnerabilities(&config, &id, path, normalized_path, line_num) {
                    vulnerabilities.push(vuln);
                }
            }
        }

        // Extract Lambda functions
        if line.contains(&templates::terraform_resource_pattern("aws_lambda_function")) || line.contains("aws_lambda_function") {
            if let Some((name, config)) = extract_terraform_resource(content, line_num, "aws_lambda_function") {
                let id = format!("{}:{}:{}", normalized_path, line_num, name);
                entity_map.insert(name.clone(), id.clone());

                let mut entity_config = HashMap::new();
                entity_config.insert("name".to_string(), Value::String(name.clone()));
                if let Some(runtime) = extract_lambda_runtime(&config) {
                    entity_config.insert("runtime".to_string(), Value::String(runtime));
                }
                if let Some(handler) = extract_lambda_handler(&config) {
                    entity_config.insert("handler".to_string(), Value::String(handler));
                }

                entities.push(SecurityEntity {
                    id: id.clone(),
                    entity_type: SecurityEntityType::LambdaFunction,
                    name,
                    provider: "aws".to_string(),
                    configuration: entity_config,
                    file_path: normalized_path.to_string(),
                    line_number: Some(line_num + 1),
                    arn: extract_arn_from_config(&config),
                    region: extract_region_from_config(&config),
                });

                // Extract Lambda IAM role relationship
                if let Some(role_name) = extract_lambda_role(&config) {
                    if let Some(role_id) = entity_map.get(&role_name) {
                        relationships.push(SecurityRelationship {
                            source_entity_id: id.clone(),
                            target_entity_id: role_id.clone(),
                            relationship_type: "uses".to_string(),
                            permissions: vec!["assume_role".to_string()],
                            condition: None,
                        });
                    }
                }
            }
        }

        // Extract S3 buckets
        if line.contains(&templates::terraform_resource_pattern("aws_s3_bucket")) || line.contains("aws_s3_bucket") {
            if let Some((name, config)) = extract_terraform_resource(content, line_num, "aws_s3_bucket") {
                let name_clone = name.clone();
                let id = format!("{}:{}:{}", normalized_path, line_num, name_clone);
                entity_map.insert(name_clone.clone(), id.clone());

                let mut entity_config = HashMap::new();
                entity_config.insert("name".to_string(), Value::String(name_clone.clone()));

                let arn_str = format!("arn:aws:s3:::{}", name_clone);
                entities.push(SecurityEntity {
                    id: id.clone(),
                    entity_type: SecurityEntityType::S3Bucket,
                    name: name_clone,
                    provider: "aws".to_string(),
                    configuration: entity_config.clone(),
                    file_path: normalized_path.to_string(),
                    line_number: Some(line_num + 1),
                    arn: Some(arn_str),
                    region: extract_region_from_config(&config),
                });

                // Check for public access vulnerabilities
                if let Some(vuln) = check_s3_bucket_vulnerabilities(&config, &id, path, normalized_path, line_num) {
                    vulnerabilities.push(vuln);
                }
            }
        }

        // Extract Security Groups
        if line.contains("resource \"aws_security_group\"") || line.contains("aws_security_group") {
            if let Some((name, config)) = extract_terraform_resource(content, line_num, "aws_security_group") {
                let id = format!("{}:{}:{}", normalized_path, line_num, name);
                entity_map.insert(name.clone(), id.clone());

                let mut entity_config = HashMap::new();
                entity_config.insert("name".to_string(), Value::String(name.clone()));

                entities.push(SecurityEntity {
                    id: id.clone(),
                    entity_type: SecurityEntityType::SecurityGroup,
                    name,
                    provider: "aws".to_string(),
                    configuration: entity_config,
                    file_path: normalized_path.to_string(),
                    line_number: Some(line_num + 1),
                    arn: None,
                    region: extract_region_from_config(&config),
                });

                // Check for overly permissive security groups
                if let Some(vuln) = check_security_group_vulnerabilities(&config, &id, path, normalized_path, line_num) {
                    vulnerabilities.push(vuln);
                }
            }
        }
    }

    Ok((entities, relationships, vulnerabilities))
}

