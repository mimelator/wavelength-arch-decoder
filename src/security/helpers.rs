use std::path::Path;
use crate::security::templates;

/// Check if content is a CloudFormation template
pub fn is_cloudformation(content: &str) -> bool {
    content.contains("AWSTemplateFormatVersion") || 
    (content.contains("Resources:") && content.contains("Type:"))
}

/// Check if content is an AWS SAM template
pub fn is_sam_template(content: &str) -> bool {
    content.contains("AWSTemplateFormatVersion") && 
    (content.contains("Transform: AWS::Serverless") || content.contains("Transform: AWS::Serverless-2016-10-31"))
}

/// Normalize file path to remove cache prefixes
pub fn normalize_path(path: &Path, repo_path: &Path) -> String {
    if let Ok(rel_path) = path.strip_prefix(repo_path) {
        let rel_str = rel_path.to_string_lossy().to_string();
        if rel_str.starts_with("./") {
            rel_str[2..].to_string()
        } else {
            rel_str
        }
    } else {
        // Try to strip common cache prefixes
        let path_str = path.to_string_lossy().to_string();
        if let Some(stripped) = path_str.strip_prefix("./cache/repos/") {
            if let Some(repo_rel) = stripped.find('/') {
                stripped[repo_rel+1..].to_string()
            } else {
                path_str
            }
        } else {
            path_str
        }
    }
}

/// Extract Terraform resource from content
pub fn extract_terraform_resource(content: &str, start_line: usize, resource_type: &str) -> Option<(String, String)> {
    let lines: Vec<&str> = content.lines().collect();
    let mut in_resource = false;
    let mut resource_name = None;
    let mut resource_content = Vec::new();
    let mut brace_count = 0;

    for (i, line) in lines.iter().enumerate().skip(start_line) {
        if i > start_line + 100 {
            break; // Safety limit
        }

        let pattern1 = templates::terraform_resource_pattern(resource_type);
        let pattern2 = templates::terraform_resource_pattern(&resource_type.replace("aws_", ""));
        if line.contains(&pattern1) || line.contains(&pattern2) {
            in_resource = true;
            // Extract resource name
            if let Some(start) = line.find(templates::QUOTE) {
                if let Some(end) = line[start+1..].find(templates::QUOTE) {
                    resource_name = Some(line[start+1..start+1+end].to_string());
                }
            }
        }

        if in_resource {
            resource_content.push(*line);
            brace_count += line.matches('{').count();
            brace_count -= line.matches('}').count();

            if brace_count == 0 && resource_content.len() > 1 {
                break;
            }
        }
    }

    resource_name.map(|name| (name, resource_content.join(&templates::NEWLINE.to_string())))
}

/// Extract ARN from config string
pub fn extract_arn_from_config(config: &str) -> Option<String> {
    if let Some(start) = config.find("arn:aws:") {
        if let Some(end) = config[start..].find(|c: char| c == templates::QUOTE || c == templates::SINGLE_QUOTE || c == templates::NEWLINE || c == templates::SPACE) {
            return Some(config[start..start+end].to_string());
        }
    }
    None
}

/// Extract region from config string
pub fn extract_region_from_config(config: &str) -> Option<String> {
    if let Some(start) = config.find("region") {
        let after_region = &config[start..];
        if let Some(start_val) = after_region.find('=') {
            let value_part = &after_region[start_val+1..];
            if let Some(end) = value_part.find(|c: char| c == templates::QUOTE || c == templates::SINGLE_QUOTE || c == templates::NEWLINE) {
                return Some(value_part[..end].trim_matches(|c| c == templates::QUOTE || c == templates::SINGLE_QUOTE || c == ' ').to_string());
            }
        }
    }
    None
}

/// Extract policy document from config string
pub fn extract_policy_document(config: &str) -> Option<String> {
    if let Some(start) = config.find("policy") {
        let after_policy = &config[start..];
        if let Some(start_doc) = after_policy.find('{') {
            let mut brace_count = 0;
            let mut end = start_doc;
            for (i, c) in after_policy[start_doc..].char_indices() {
                if c == '{' {
                    brace_count += 1;
                } else if c == '}' {
                    brace_count -= 1;
                    if brace_count == 0 {
                        end = start_doc + i + 1;
                        break;
                    }
                }
            }
            return Some(after_policy[start_doc..end].to_string());
        }
    }
    None
}

/// Extract Lambda runtime from config string
pub fn extract_lambda_runtime(config: &str) -> Option<String> {
    if let Some(start) = config.find("runtime") {
        let after_runtime = &config[start..];
        if let Some(start_val) = after_runtime.find('=') {
            let value_part = &after_runtime[start_val+1..];
            if let Some(end) = value_part.find(|c: char| c == templates::QUOTE || c == templates::SINGLE_QUOTE || c == templates::NEWLINE) {
                return Some(value_part[..end].trim_matches(|c| c == templates::QUOTE || c == templates::SINGLE_QUOTE || c == templates::SPACE).to_string());
            }
        }
    }
    None
}

/// Extract Lambda handler from config string
pub fn extract_lambda_handler(config: &str) -> Option<String> {
    if let Some(start) = config.find("handler") {
        let after_handler = &config[start..];
        if let Some(start_val) = after_handler.find('=') {
            let value_part = &after_handler[start_val+1..];
            if let Some(end) = value_part.find(|c: char| c == templates::QUOTE || c == templates::SINGLE_QUOTE || c == templates::NEWLINE) {
                return Some(value_part[..end].trim_matches(|c| c == templates::QUOTE || c == templates::SINGLE_QUOTE || c == templates::SPACE).to_string());
            }
        }
    }
    None
}

/// Extract Lambda role from config string
pub fn extract_lambda_role(config: &str) -> Option<String> {
    if let Some(start) = config.find("role") {
        let after_role = &config[start..];
        if let Some(start_val) = after_role.find('=') {
            let value_part = &after_role[start_val+1..];
            if let Some(end) = value_part.find(|c: char| c == templates::QUOTE || c == templates::SINGLE_QUOTE || c == templates::NEWLINE) {
                let role_ref = value_part[..end].trim_matches(|c| c == templates::QUOTE || c == templates::SINGLE_QUOTE || c == templates::SPACE);
                // Extract role name from reference like aws_iam_role.example.name
                if let Some(name_start) = role_ref.rfind('.') {
                    return Some(role_ref[name_start+1..].to_string());
                }
                return Some(role_ref.to_string());
            }
        }
    }
    None
}

