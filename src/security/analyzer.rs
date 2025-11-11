use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use walkdir::WalkDir;
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

pub struct SecurityAnalyzer;

impl SecurityAnalyzer {
    pub fn new() -> Self {
        SecurityAnalyzer
    }

    /// Analyze security configuration in a repository
    pub fn analyze_repository(&self, repo_path: &Path) -> Result<SecurityAnalysis> {
        let mut entities = Vec::new();
        let mut relationships = Vec::new();
        let mut vulnerabilities = Vec::new();
        let mut entity_map: HashMap<String, String> = HashMap::new(); // name -> id

        // Walk through infrastructure and configuration files
        for entry in WalkDir::new(repo_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            let file_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_lowercase();

            // Skip hidden files and common ignore patterns
            if file_name.starts_with('.') || 
               path.to_string_lossy().contains("node_modules") ||
               path.to_string_lossy().contains("target") ||
               path.to_string_lossy().contains(".git") {
                continue;
            }

            // Analyze Terraform files
            if file_name.ends_with(".tf") || file_name.ends_with(".tfvars") {
                if let Ok(content) = std::fs::read_to_string(path) {
                    let (tf_entities, tf_relationships, tf_vulns) = 
                        self.analyze_terraform(&content, path, &mut entity_map)?;
                    entities.extend(tf_entities);
                    relationships.extend(tf_relationships);
                    vulnerabilities.extend(tf_vulns);
                }
            }

            // Analyze CloudFormation files
            if file_name.ends_with(".yaml") || file_name.ends_with(".yml") {
                if let Ok(content) = std::fs::read_to_string(path) {
                    if self.is_cloudformation(&content) {
                        let (cf_entities, cf_relationships, cf_vulns) = 
                            self.analyze_cloudformation(&content, path, &mut entity_map)?;
                        entities.extend(cf_entities);
                        relationships.extend(cf_relationships);
                        vulnerabilities.extend(cf_vulns);
                    }
                }
            }

            // Analyze serverless.yml files
            if file_name == "serverless.yml" || file_name == "serverless.yaml" {
                if let Ok(content) = std::fs::read_to_string(path) {
                    let (sls_entities, sls_relationships, sls_vulns) = 
                        self.analyze_serverless(&content, path, &mut entity_map)?;
                    entities.extend(sls_entities);
                    relationships.extend(sls_relationships);
                    vulnerabilities.extend(sls_vulns);
                }
            }

            // Analyze AWS SAM templates
            if file_name.contains("template") && (file_name.ends_with(".yaml") || file_name.ends_with(".yml")) {
                if let Ok(content) = std::fs::read_to_string(path) {
                    if self.is_sam_template(&content) {
                        let (sam_entities, sam_relationships, sam_vulns) = 
                            self.analyze_sam(&content, path, &mut entity_map)?;
                        entities.extend(sam_entities);
                        relationships.extend(sam_relationships);
                        vulnerabilities.extend(sam_vulns);
                    }
                }
            }
        }

        Ok(SecurityAnalysis {
            entities,
            relationships,
            vulnerabilities,
        })
    }

    fn is_cloudformation(&self, content: &str) -> bool {
        content.contains("AWSTemplateFormatVersion") || 
        content.contains("Resources:") && content.contains("Type:")
    }

    fn is_sam_template(&self, content: &str) -> bool {
        content.contains("Transform: AWS::Serverless") ||
        content.contains("AWS::Serverless::Function")
    }

    /// Analyze Terraform files
    fn analyze_terraform(
        &self,
        content: &str,
        path: &Path,
        entity_map: &mut HashMap<String, String>,
    ) -> Result<(Vec<SecurityEntity>, Vec<SecurityRelationship>, Vec<SecurityVulnerability>)> {
        let mut entities = Vec::new();
        let mut relationships = Vec::new();
        let mut vulnerabilities = Vec::new();

        let lines: Vec<&str> = content.lines().collect();

        // Extract IAM roles
        for (line_num, line) in lines.iter().enumerate() {
            if line.contains("resource \"aws_iam_role\"") || line.contains("aws_iam_role") {
                if let Some((name, config)) = self.extract_terraform_resource(content, line_num, "aws_iam_role") {
                    let id = format!("{}:{}:{}", path.to_string_lossy(), line_num, name);
                    entity_map.insert(name.clone(), id.clone());

                    let mut entity_config = HashMap::new();
                    entity_config.insert("name".to_string(), Value::String(name.clone()));
                    if let Some(arn) = self.extract_arn_from_config(&config) {
                        entity_config.insert("arn".to_string(), Value::String(arn.clone()));
                    }

                    entities.push(SecurityEntity {
                        id: id.clone(),
                        entity_type: SecurityEntityType::IamRole,
                        name,
                        provider: "aws".to_string(),
                        configuration: entity_config,
                        file_path: path.to_string_lossy().to_string(),
                        line_number: Some(line_num + 1),
                        arn: self.extract_arn_from_config(&config),
                        region: self.extract_region_from_config(&config),
                    });

                    // Check for vulnerabilities
                    if let Some(vuln) = self.check_iam_role_vulnerabilities(&config, &id, path, line_num) {
                        vulnerabilities.push(vuln);
                    }
                }
            }

            // Extract IAM policies
            if line.contains("resource \"aws_iam_policy\"") || line.contains("aws_iam_policy") {
                if let Some((name, config)) = self.extract_terraform_resource(content, line_num, "aws_iam_policy") {
                    let id = format!("{}:{}:{}", path.to_string_lossy(), line_num, name);
                    entity_map.insert(name.clone(), id.clone());

                    let mut entity_config = HashMap::new();
                    entity_config.insert("name".to_string(), Value::String(name.clone()));
                    if let Some(policy_doc) = self.extract_policy_document(&config) {
                        entity_config.insert("policy_document".to_string(), Value::String(policy_doc.clone()));
                    }

                    entities.push(SecurityEntity {
                        id: id.clone(),
                        entity_type: SecurityEntityType::IamPolicy,
                        name,
                        provider: "aws".to_string(),
                        configuration: entity_config,
                        file_path: path.to_string_lossy().to_string(),
                        line_number: Some(line_num + 1),
                        arn: self.extract_arn_from_config(&config),
                        region: None,
                    });

                    // Check for overly permissive policies
                    if let Some(vuln) = self.check_iam_policy_vulnerabilities(&config, &id, path, line_num) {
                        vulnerabilities.push(vuln);
                    }
                }
            }

            // Extract Lambda functions
            if line.contains("resource \"aws_lambda_function\"") || line.contains("aws_lambda_function") {
                if let Some((name, config)) = self.extract_terraform_resource(content, line_num, "aws_lambda_function") {
                    let id = format!("{}:{}:{}", path.to_string_lossy(), line_num, name);
                    entity_map.insert(name.clone(), id.clone());

                    let mut entity_config = HashMap::new();
                    entity_config.insert("name".to_string(), Value::String(name.clone()));
                    if let Some(runtime) = self.extract_lambda_runtime(&config) {
                        entity_config.insert("runtime".to_string(), Value::String(runtime));
                    }
                    if let Some(handler) = self.extract_lambda_handler(&config) {
                        entity_config.insert("handler".to_string(), Value::String(handler));
                    }

                    entities.push(SecurityEntity {
                        id: id.clone(),
                        entity_type: SecurityEntityType::LambdaFunction,
                        name,
                        provider: "aws".to_string(),
                        configuration: entity_config,
                        file_path: path.to_string_lossy().to_string(),
                        line_number: Some(line_num + 1),
                        arn: self.extract_arn_from_config(&config),
                        region: self.extract_region_from_config(&config),
                    });

                    // Extract Lambda IAM role relationship
                    if let Some(role_name) = self.extract_lambda_role(&config) {
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
            if line.contains("resource \"aws_s3_bucket\"") || line.contains("aws_s3_bucket") {
                if let Some((name, config)) = self.extract_terraform_resource(content, line_num, "aws_s3_bucket") {
                    let name_clone = name.clone();
                    let id = format!("{}:{}:{}", path.to_string_lossy(), line_num, name_clone);
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
                        file_path: path.to_string_lossy().to_string(),
                        line_number: Some(line_num + 1),
                        arn: Some(arn_str),
                        region: self.extract_region_from_config(&config),
                    });

                    // Check for public access vulnerabilities
                    if let Some(vuln) = self.check_s3_bucket_vulnerabilities(&config, &id, path, line_num) {
                        vulnerabilities.push(vuln);
                    }
                }
            }

            // Extract Security Groups
            if line.contains("resource \"aws_security_group\"") || line.contains("aws_security_group") {
                if let Some((name, config)) = self.extract_terraform_resource(content, line_num, "aws_security_group") {
                    let id = format!("{}:{}:{}", path.to_string_lossy(), line_num, name);
                    entity_map.insert(name.clone(), id.clone());

                    let mut entity_config = HashMap::new();
                    entity_config.insert("name".to_string(), Value::String(name.clone()));

                    entities.push(SecurityEntity {
                        id: id.clone(),
                        entity_type: SecurityEntityType::SecurityGroup,
                        name,
                        provider: "aws".to_string(),
                        configuration: entity_config,
                        file_path: path.to_string_lossy().to_string(),
                        line_number: Some(line_num + 1),
                        arn: None,
                        region: self.extract_region_from_config(&config),
                    });

                    // Check for overly permissive security groups
                    if let Some(vuln) = self.check_security_group_vulnerabilities(&config, &id, path, line_num) {
                        vulnerabilities.push(vuln);
                    }
                }
            }
        }

        Ok((entities, relationships, vulnerabilities))
    }

    /// Analyze CloudFormation templates
    fn analyze_cloudformation(
        &self,
        content: &str,
        path: &Path,
        entity_map: &mut HashMap<String, String>,
    ) -> Result<(Vec<SecurityEntity>, Vec<SecurityRelationship>, Vec<SecurityVulnerability>)> {
        let mut entities = Vec::new();
        let mut relationships = Vec::new();
        let mut vulnerabilities = Vec::new();

        // Parse YAML
        let yaml: Value = match serde_yaml::from_str(content) {
            Ok(v) => v,
            Err(_) => return Ok((entities, relationships, vulnerabilities)),
        };

        if let Some(resources) = yaml.get("Resources").and_then(|r| r.as_object()) {
            for (resource_name, resource_def) in resources {
                if let Some(resource_type) = resource_def.get("Type").and_then(|t| t.as_str()) {
                    let id = format!("{}:{}", path.to_string_lossy(), resource_name);
                    entity_map.insert(resource_name.clone(), id.clone());

                    match resource_type {
                        "AWS::IAM::Role" => {
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
                                file_path: path.to_string_lossy().to_string(),
                                line_number: None,
                                arn: None,
                                region: None,
                            });
                        }
                        "AWS::IAM::Policy" => {
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
                                file_path: path.to_string_lossy().to_string(),
                                line_number: None,
                                arn: None,
                                region: None,
                            });
                        }
                        "AWS::Lambda::Function" => {
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
                                file_path: path.to_string_lossy().to_string(),
                                line_number: None,
                                arn: None,
                                region: None,
                            });
                        }
                        "AWS::S3::Bucket" => {
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
                                file_path: path.to_string_lossy().to_string(),
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

    /// Analyze Serverless Framework files
    fn analyze_serverless(
        &self,
        content: &str,
        path: &Path,
        entity_map: &mut HashMap<String, String>,
    ) -> Result<(Vec<SecurityEntity>, Vec<SecurityRelationship>, Vec<SecurityVulnerability>)> {
        let mut entities = Vec::new();
        let mut relationships = Vec::new();
        let mut vulnerabilities = Vec::new();

        let yaml: Value = match serde_yaml::from_str(content) {
            Ok(v) => v,
            Err(_) => return Ok((entities, relationships, vulnerabilities)),
        };

        // Extract functions
        if let Some(functions) = yaml.get("functions").and_then(|f| f.as_object()) {
            for (func_name, func_def) in functions {
                let id = format!("{}:{}", path.to_string_lossy(), func_name);
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
                    file_path: path.to_string_lossy().to_string(),
                    line_number: None,
                    arn: None,
                    region: None,
                });
            }
        }

        Ok((entities, relationships, vulnerabilities))
    }

    /// Analyze AWS SAM templates
    fn analyze_sam(
        &self,
        content: &str,
        path: &Path,
        entity_map: &mut HashMap<String, String>,
    ) -> Result<(Vec<SecurityEntity>, Vec<SecurityRelationship>, Vec<SecurityVulnerability>)> {
        // Similar to CloudFormation analysis
        self.analyze_cloudformation(content, path, entity_map)
    }

    // Helper functions for extracting Terraform resources
    fn extract_terraform_resource(&self, content: &str, start_line: usize, resource_type: &str) -> Option<(String, String)> {
        let lines: Vec<&str> = content.lines().collect();
        let mut in_resource = false;
        let mut resource_name = None;
        let mut resource_content = Vec::new();
        let mut brace_count = 0;

        for (i, line) in lines.iter().enumerate().skip(start_line) {
            if i > start_line + 100 {
                break; // Safety limit
            }

            if line.contains(&format!("resource \"{}\"", resource_type)) || 
               line.contains(&format!("resource \"{}\"", resource_type.replace("aws_", ""))) {
                in_resource = true;
                // Extract resource name
                if let Some(start) = line.find('"') {
                    if let Some(end) = line[start+1..].find('"') {
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

        resource_name.map(|name| (name, resource_content.join("\n")))
    }

    fn extract_arn_from_config(&self, config: &str) -> Option<String> {
        // Look for ARN patterns
        if let Some(start) = config.find("arn:aws:") {
            if let Some(end) = config[start..].find(|c: char| c == '"' || c == '\'' || c == '\n' || c == ' ') {
                return Some(config[start..start+end].to_string());
            }
        }
        None
    }

    fn extract_region_from_config(&self, config: &str) -> Option<String> {
        // Look for region patterns
        if let Some(start) = config.find("region") {
            let after_region = &config[start..];
            if let Some(start_val) = after_region.find('=') {
                let value_part = &after_region[start_val+1..];
                if let Some(end) = value_part.find(|c: char| c == '"' || c == '\'' || c == '\n') {
                    return Some(value_part[..end].trim_matches(|c| c == '"' || c == '\'' || c == ' ').to_string());
                }
            }
        }
        None
    }

    fn extract_policy_document(&self, config: &str) -> Option<String> {
        // Look for policy document patterns
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

    fn extract_lambda_runtime(&self, config: &str) -> Option<String> {
        if let Some(start) = config.find("runtime") {
            let after_runtime = &config[start..];
            if let Some(start_val) = after_runtime.find('=') {
                let value_part = &after_runtime[start_val+1..];
                if let Some(end) = value_part.find(|c: char| c == '"' || c == '\'' || c == '\n') {
                    return Some(value_part[..end].trim_matches(|c| c == '"' || c == '\'' || c == ' ').to_string());
                }
            }
        }
        None
    }

    fn extract_lambda_handler(&self, config: &str) -> Option<String> {
        if let Some(start) = config.find("handler") {
            let after_handler = &config[start..];
            if let Some(start_val) = after_handler.find('=') {
                let value_part = &after_handler[start_val+1..];
                if let Some(end) = value_part.find(|c: char| c == '"' || c == '\'' || c == '\n') {
                    return Some(value_part[..end].trim_matches(|c| c == '"' || c == '\'' || c == ' ').to_string());
                }
            }
        }
        None
    }

    fn extract_lambda_role(&self, config: &str) -> Option<String> {
        if let Some(start) = config.find("role") {
            let after_role = &config[start..];
            if let Some(start_val) = after_role.find('=') {
                let value_part = &after_role[start_val+1..];
                if let Some(end) = value_part.find(|c: char| c == '"' || c == '\'' || c == '\n') {
                    let role_ref = value_part[..end].trim_matches(|c| c == '"' || c == '\'' || c == ' ');
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

    // Vulnerability detection functions
    fn check_iam_role_vulnerabilities(&self, config: &str, entity_id: &str, path: &Path, line_num: usize) -> Option<SecurityVulnerability> {
        // Check for overly permissive assume role policies
        if config.contains("*") && config.contains("Effect") && config.contains("Allow") {
            return Some(SecurityVulnerability {
                id: format!("{}:vuln:1", entity_id),
                entity_id: entity_id.to_string(),
                vulnerability_type: "OverlyPermissiveAssumeRolePolicy".to_string(),
                severity: VulnerabilitySeverity::High,
                description: "IAM role has overly permissive assume role policy".to_string(),
                recommendation: "Restrict assume role policy to specific principals".to_string(),
                file_path: path.to_string_lossy().to_string(),
                line_number: Some(line_num + 1),
            });
        }
        None
    }

    fn check_iam_policy_vulnerabilities(&self, config: &str, entity_id: &str, path: &Path, line_num: usize) -> Option<SecurityVulnerability> {
        // Check for wildcard actions
        if config.contains("\"Action\": \"*\"") || config.contains("Action: *") {
            return Some(SecurityVulnerability {
                id: format!("{}:vuln:1", entity_id),
                entity_id: entity_id.to_string(),
                vulnerability_type: "WildcardAction".to_string(),
                severity: VulnerabilitySeverity::Critical,
                description: "IAM policy uses wildcard action (*)".to_string(),
                recommendation: "Replace wildcard actions with specific actions".to_string(),
                file_path: path.to_string_lossy().to_string(),
                line_number: Some(line_num + 1),
            });
        }

        // Check for wildcard resources
        if config.contains("\"Resource\": \"*\"") || config.contains("Resource: *") {
            return Some(SecurityVulnerability {
                id: format!("{}:vuln:2", entity_id),
                entity_id: entity_id.to_string(),
                vulnerability_type: "WildcardResource".to_string(),
                severity: VulnerabilitySeverity::High,
                description: "IAM policy uses wildcard resource (*)".to_string(),
                recommendation: "Replace wildcard resources with specific ARNs".to_string(),
                file_path: path.to_string_lossy().to_string(),
                line_number: Some(line_num + 1),
            });
        }

        None
    }

    fn check_s3_bucket_vulnerabilities(&self, config: &str, entity_id: &str, path: &Path, line_num: usize) -> Option<SecurityVulnerability> {
        // Check for public access
        if config.contains("public_access_block") && 
           (config.contains("block_public_acls = false") || 
            config.contains("block_public_policy = false") ||
            config.contains("ignore_public_acls = false") ||
            config.contains("restrict_public_buckets = false")) {
            return Some(SecurityVulnerability {
                id: format!("{}:vuln:1", entity_id),
                entity_id: entity_id.to_string(),
                vulnerability_type: "PublicS3Bucket".to_string(),
                severity: VulnerabilitySeverity::Critical,
                description: "S3 bucket allows public access".to_string(),
                recommendation: "Enable public access block settings".to_string(),
                file_path: path.to_string_lossy().to_string(),
                line_number: Some(line_num + 1),
            });
        }

        // Check for missing encryption
        if !config.contains("server_side_encryption") && !config.contains("encryption") {
            return Some(SecurityVulnerability {
                id: format!("{}:vuln:2", entity_id),
                entity_id: entity_id.to_string(),
                vulnerability_type: "UnencryptedS3Bucket".to_string(),
                severity: VulnerabilitySeverity::Medium,
                description: "S3 bucket does not have encryption enabled".to_string(),
                recommendation: "Enable server-side encryption for S3 bucket".to_string(),
                file_path: path.to_string_lossy().to_string(),
                line_number: Some(line_num + 1),
            });
        }

        None
    }

    fn check_security_group_vulnerabilities(&self, config: &str, entity_id: &str, path: &Path, line_num: usize) -> Option<SecurityVulnerability> {
        // Check for open ingress rules (0.0.0.0/0)
        if config.contains("0.0.0.0/0") || config.contains("::/0") {
            return Some(SecurityVulnerability {
                id: format!("{}:vuln:1", entity_id),
                entity_id: entity_id.to_string(),
                vulnerability_type: "OpenSecurityGroup".to_string(),
                severity: VulnerabilitySeverity::High,
                description: "Security group allows access from anywhere (0.0.0.0/0)".to_string(),
                recommendation: "Restrict security group rules to specific IP ranges".to_string(),
                file_path: path.to_string_lossy().to_string(),
                line_number: Some(line_num + 1),
            });
        }

        None
    }
}

