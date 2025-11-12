use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use regex::Regex;
use crate::ingestion::FileType;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ServiceType {
    CloudProvider,
    SaaS,
    Database,
    Api,
    Cdn,
    Monitoring,
    Auth,
    Payment,
    AI,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ServiceProvider {
    // Cloud Providers
    Aws,
    Azure,
    Gcp,
    Vercel,
    Netlify,
    Heroku,
    DigitalOcean,
    
    // SaaS Services
    Clerk,
    Auth0,
    Stripe,
    Twilio,
    SendGrid,
    Mailgun,
    Slack,
    Discord,
    
    // Databases
    Postgres,
    MySQL,
    MongoDB,
    Redis,
    DynamoDB,
    RDS,
    
    // APIs
    GitHub,
    GitLab,
    Jira,
    Linear,
    
    // CDN
    Cloudflare,
    CloudFront,
    
    // Monitoring
    Datadog,
    NewRelic,
    Sentry,
    LogRocket,
    
    // AI Services
    OpenAI,
    Anthropic,
    GitHubCopilot,
    GoogleAI,
    Cohere,
    HuggingFace,
    Replicate,
    TogetherAI,
    MistralAI,
    Perplexity,
    
    // Other
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedService {
    pub provider: ServiceProvider,
    pub service_type: ServiceType,
    pub name: String,
    pub configuration: HashMap<String, String>,
    pub file_path: String,
    pub line_number: Option<usize>,
    pub confidence: f64, // 0.0 to 1.0
}

pub struct ServiceDetector;

impl ServiceDetector {
    pub fn new() -> Self {
        ServiceDetector
    }

    /// Detect services in a repository
    pub fn detect_services(&self, repo_path: &Path) -> Result<Vec<DetectedService>> {
        let mut services = Vec::new();
        
        // Use direct directory walking instead of FileIndexer
        use walkdir::WalkDir;
        
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
            
            // Determine file type
            let file_type = if file_name.contains("terraform") || 
                             file_name.contains("cloudformation") ||
                             file_name.ends_with(".tf") ||
                             file_name.ends_with(".tfvars") {
                FileType::Infrastructure
            } else if file_name.ends_with(".json") ||
                      file_name.ends_with(".yaml") ||
                      file_name.ends_with(".yml") ||
                      file_name.ends_with(".toml") ||
                      file_name.ends_with(".env") ||
                      file_name.contains("config") {
                FileType::Config
            } else if file_name.ends_with(".js") ||
                      file_name.ends_with(".ts") ||
                      file_name.ends_with(".jsx") ||
                      file_name.ends_with(".tsx") ||
                      file_name.ends_with(".py") ||
                      file_name.ends_with(".rs") ||
                      file_name.ends_with(".go") {
                FileType::Code
            } else {
                continue;
            };
            
            // Detect services based on file type
            match file_type {
                FileType::Config | FileType::Infrastructure => {
                    if let Ok(detected) = self.detect_in_file(path, &file_type) {
                        services.extend(detected);
                    }
                }
                FileType::Code => {
                    let language = path.extension()
                        .and_then(|e| e.to_str())
                        .map(|s| s.to_string());
                    if let Ok(detected) = self.detect_in_code(path, &language) {
                        services.extend(detected);
                    }
                }
                _ => {}
            }
        }
        
        // Deduplicate services: combine identical services (same name, provider, type)
        // Keep the one with highest confidence and merge file paths
        let mut deduplicated: Vec<DetectedService> = Vec::new();
        let mut seen: HashMap<(String, ServiceProvider, ServiceType), usize> = HashMap::new();
        
        for service in services {
            let key = (service.name.clone(), service.provider.clone(), service.service_type.clone());
            
            if let Some(&existing_idx) = seen.get(&key) {
                // Service already exists, merge if this one has higher confidence
                let existing = &mut deduplicated[existing_idx];
                if service.confidence > existing.confidence {
                    existing.confidence = service.confidence;
                    existing.line_number = service.line_number;
                }
                // Merge file paths in configuration
                let file_paths = existing.configuration
                    .get("file_paths")
                    .map(|s| s.clone())
                    .unwrap_or_else(|| existing.file_path.clone());
                if !file_paths.contains(&service.file_path) {
                    existing.configuration.insert(
                        "file_paths".to_string(),
                        format!("{}, {}", file_paths, service.file_path)
                    );
                }
            } else {
                // New service, add it
                let idx = deduplicated.len();
                seen.insert(key, idx);
                deduplicated.push(service);
            }
        }
        
        Ok(deduplicated)
    }

    /// Detect services in a specific file
    fn detect_in_file(&self, file_path: &Path, file_type: &FileType) -> Result<Vec<DetectedService>> {
        let mut services = Vec::new();
        let content = std::fs::read_to_string(file_path)?;
        let file_name = file_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        match file_type {
            FileType::Infrastructure => {
                // Detect AWS services in Terraform/CloudFormation
                services.extend(self.detect_aws_services(&content, file_path)?);
                
                // Detect Vercel/Netlify in config files
                if file_name.contains("vercel") {
                    services.push(DetectedService {
                        provider: ServiceProvider::Vercel,
                        service_type: ServiceType::CloudProvider,
                        name: "Vercel".to_string(),
                        configuration: self.extract_vercel_config(&content),
                        file_path: file_path.to_string_lossy().to_string(),
                        line_number: None,
                        confidence: 0.9,
                    });
                }
                
                if file_name.contains("netlify") {
                    services.push(DetectedService {
                        provider: ServiceProvider::Netlify,
                        service_type: ServiceType::CloudProvider,
                        name: "Netlify".to_string(),
                        configuration: self.extract_netlify_config(&content),
                        file_path: file_path.to_string_lossy().to_string(),
                        line_number: None,
                        confidence: 0.9,
                    });
                }
            }
            FileType::Config => {
                // Detect services in environment/config files
                services.extend(self.detect_from_env_vars(&content, file_path)?);
                
                // Detect database connections
                services.extend(self.detect_databases(&content, file_path)?);
            }
            _ => {}
        }
        
        Ok(services)
    }

    /// Detect services in code files
    fn detect_in_code(&self, file_path: &Path, language: &Option<String>) -> Result<Vec<DetectedService>> {
        let mut services = Vec::new();
        let content = std::fs::read_to_string(file_path)?;
        
        // Detect service SDKs and API keys
        services.extend(self.detect_service_sdks(&content, file_path, language.as_deref())?);
        
        // Detect API endpoints
        services.extend(self.detect_api_endpoints(&content, file_path)?);
        
        Ok(services)
    }

    /// Detect AWS services from infrastructure code
    fn detect_aws_services(&self, content: &str, file_path: &Path) -> Result<Vec<DetectedService>> {
        let mut services = Vec::new();
        let content_lower = content.to_lowercase();
        
        // AWS service patterns
        let aws_patterns = vec![
            ("aws_s3_bucket", ServiceProvider::Aws, "S3"),
            ("aws_lambda", ServiceProvider::Aws, "Lambda"),
            ("aws_iam", ServiceProvider::Aws, "IAM"),
            ("aws_dynamodb", ServiceProvider::DynamoDB, "DynamoDB"),
            ("aws_rds", ServiceProvider::RDS, "RDS"),
            ("aws_ec2", ServiceProvider::Aws, "EC2"),
            ("aws_ecs", ServiceProvider::Aws, "ECS"),
            ("aws_cloudfront", ServiceProvider::CloudFront, "CloudFront"),
            ("aws_sns", ServiceProvider::Aws, "SNS"),
            ("aws_sqs", ServiceProvider::Aws, "SQS"),
            ("aws_api_gateway", ServiceProvider::Aws, "API Gateway"),
            ("aws_cognito", ServiceProvider::Aws, "Cognito"),
        ];
        
        for (pattern, provider, service_name) in aws_patterns {
            if content_lower.contains(pattern) {
                let mut config = HashMap::new();
                config.insert("service".to_string(), service_name.to_string());
                config.insert("provider".to_string(), "AWS".to_string());
                
                services.push(DetectedService {
                    provider: provider.clone(),
                    service_type: ServiceType::CloudProvider,
                    name: service_name.to_string(),
                    configuration: config,
                    file_path: file_path.to_string_lossy().to_string(),
                    line_number: self.find_line_number(content, pattern),
                    confidence: 0.8,
                });
            }
        }
        
        // Detect AWS CloudFormation resources
        if content_lower.contains("awstemplateformatversion") {
            services.push(DetectedService {
                provider: ServiceProvider::Aws,
                service_type: ServiceType::CloudProvider,
                name: "AWS CloudFormation".to_string(),
                configuration: HashMap::new(),
                file_path: file_path.to_string_lossy().to_string(),
                line_number: None,
                confidence: 0.9,
            });
        }
        
        Ok(services)
    }

    /// Detect services from environment variables
    fn detect_from_env_vars(&self, content: &str, file_path: &Path) -> Result<Vec<DetectedService>> {
        let mut services = Vec::new();
        
        // Common service environment variable patterns
        let env_patterns = vec![
            ("CLERK", ServiceProvider::Clerk, ServiceType::Auth),
            ("AUTH0", ServiceProvider::Auth0, ServiceType::Auth),
            ("STRIPE", ServiceProvider::Stripe, ServiceType::Payment),
            ("TWILIO", ServiceProvider::Twilio, ServiceType::Api),
            ("SENDGRID", ServiceProvider::SendGrid, ServiceType::Api),
            ("MAILGUN", ServiceProvider::Mailgun, ServiceType::Api),
            ("SLACK", ServiceProvider::Slack, ServiceType::Api),
            ("DISCORD", ServiceProvider::Discord, ServiceType::Api),
            ("DATADOG", ServiceProvider::Datadog, ServiceType::Monitoring),
            ("NEW_RELIC", ServiceProvider::NewRelic, ServiceType::Monitoring),
            ("SENTRY", ServiceProvider::Sentry, ServiceType::Monitoring),
            ("LOGROCKET", ServiceProvider::LogRocket, ServiceType::Monitoring),
            ("VERCEL", ServiceProvider::Vercel, ServiceType::CloudProvider),
            ("NETLIFY", ServiceProvider::Netlify, ServiceType::CloudProvider),
            ("CLOUDFLARE", ServiceProvider::Cloudflare, ServiceType::Cdn),
            // AI Services
            ("OPENAI", ServiceProvider::OpenAI, ServiceType::AI),
            ("ANTHROPIC", ServiceProvider::Anthropic, ServiceType::AI),
            ("GITHUB_COPILOT", ServiceProvider::GitHubCopilot, ServiceType::AI),
            ("GOOGLE_AI", ServiceProvider::GoogleAI, ServiceType::AI),
            ("GEMINI", ServiceProvider::GoogleAI, ServiceType::AI),
            ("COHERE", ServiceProvider::Cohere, ServiceType::AI),
            ("HUGGINGFACE", ServiceProvider::HuggingFace, ServiceType::AI),
            ("HF_", ServiceProvider::HuggingFace, ServiceType::AI),
            ("REPLICATE", ServiceProvider::Replicate, ServiceType::AI),
            ("TOGETHER", ServiceProvider::TogetherAI, ServiceType::AI),
            ("TOGETHER_AI", ServiceProvider::TogetherAI, ServiceType::AI),
            ("MISTRAL", ServiceProvider::MistralAI, ServiceType::AI),
            ("PERPLEXITY", ServiceProvider::Perplexity, ServiceType::AI),
        ];
        
        for line in content.lines() {
            let line_upper = line.to_uppercase();
            for (pattern, provider, service_type) in &env_patterns {
                if line_upper.contains(pattern) && (line_upper.contains("API_KEY") || 
                    line_upper.contains("SECRET") || line_upper.contains("TOKEN") ||
                    line_upper.contains("ID") || line_upper.contains("KEY")) {
                    let mut config = HashMap::new();
                    config.insert("env_var".to_string(), line.trim().to_string());
                    
                    services.push(DetectedService {
                        provider: provider.clone(),
                        service_type: service_type.clone(),
                        name: format!("{} Service", pattern),
                        configuration: config,
                        file_path: file_path.to_string_lossy().to_string(),
                        line_number: self.find_line_number(content, line),
                        confidence: 0.7,
                    });
                }
            }
        }
        
        Ok(services)
    }

    /// Detect database connections
    fn detect_databases(&self, content: &str, file_path: &Path) -> Result<Vec<DetectedService>> {
        let mut services = Vec::new();
        let content_lower = content.to_lowercase();
        
        let db_patterns = vec![
            ("postgresql://", ServiceProvider::Postgres),
            ("postgres://", ServiceProvider::Postgres),
            ("mysql://", ServiceProvider::MySQL),
            ("mongodb://", ServiceProvider::MongoDB),
            ("redis://", ServiceProvider::Redis),
            ("dynamodb", ServiceProvider::DynamoDB),
        ];
        
        for (pattern, provider) in db_patterns {
            if content_lower.contains(pattern) {
                services.push(DetectedService {
                    provider: provider.clone(),
                    service_type: ServiceType::Database,
                    name: format!("{} Database", pattern.replace("://", "")),
                    configuration: HashMap::new(),
                    file_path: file_path.to_string_lossy().to_string(),
                    line_number: self.find_line_number(content, pattern),
                    confidence: 0.9,
                });
            }
        }
        
        Ok(services)
    }

    /// Detect service SDKs in code
    fn detect_service_sdks(&self, content: &str, file_path: &Path, language: Option<&str>) -> Result<Vec<DetectedService>> {
        let mut services = Vec::new();
        let content_lower = content.to_lowercase();
        
        // Detect specific AWS SDK clients from @aws-sdk/client-* imports
        let aws_sdk_client_pattern = regex::Regex::new(r"@aws-sdk/client-([a-z0-9-]+)").unwrap();
        for cap in aws_sdk_client_pattern.captures_iter(&content_lower) {
            if let Some(service_name) = cap.get(1) {
                let service = service_name.as_str();
                let display_name = self.format_aws_service_name(service);
                
                services.push(DetectedService {
                    provider: ServiceProvider::Aws,
                    service_type: ServiceType::CloudProvider,
                    name: format!("AWS {}", display_name),
                    configuration: {
                        let mut config = HashMap::new();
                        config.insert("sdk_client".to_string(), format!("@aws-sdk/client-{}", service));
                        config
                    },
                    file_path: file_path.to_string_lossy().to_string(),
                    line_number: self.find_line_number(content, &format!("@aws-sdk/client-{}", service)),
                    confidence: 0.9,
                });
            }
        }
        
        // Detect AWS SDK v2 imports (aws-sdk package with specific service imports)
        if content_lower.contains("aws-sdk") || content_lower.contains("from 'aws-sdk'") || content_lower.contains("require('aws-sdk')") {
            // Try to detect which specific AWS services are being used
            let aws_service_patterns = vec![
                ("s3", "S3"),
                ("dynamodb", "DynamoDB"),
                ("lambda", "Lambda"),
                ("sns", "SNS"),
                ("sqs", "SQS"),
                ("ses", "SES"),
                ("ec2", "EC2"),
                ("rds", "RDS"),
                ("cloudfront", "CloudFront"),
                ("cognito", "Cognito"),
                ("iam", "IAM"),
                ("sts", "STS"),
                ("cloudwatch", "CloudWatch"),
                ("kms", "KMS"),
                ("secretsmanager", "Secrets Manager"),
                ("ssm", "Systems Manager"),
                ("apigateway", "API Gateway"),
                ("eventbridge", "EventBridge"),
                ("stepfunctions", "Step Functions"),
            ];
            
            for (pattern, display_name) in aws_service_patterns {
                // Look for patterns like: new AWS.S3(), AWS.S3(), s3 = new AWS.S3()
                let service_patterns = vec![
                    format!("aws.{}", pattern),
                    format!("aws['{}']", pattern),
                    format!("aws[\"{}\"]", pattern),
                    format!("new aws.{}", pattern),
                    format!("new aws['{}']", pattern),
                    format!("new aws[\"{}\"]", pattern),
                ];
                
                for service_pattern in &service_patterns {
                    if content_lower.contains(service_pattern) {
                        services.push(DetectedService {
                            provider: ServiceProvider::Aws,
                            service_type: ServiceType::CloudProvider,
                            name: format!("AWS {}", display_name),
                            configuration: {
                                let mut config = HashMap::new();
                                config.insert("sdk_version".to_string(), "v2".to_string());
                                config.insert("service".to_string(), pattern.to_string());
                                config
                            },
                            file_path: file_path.to_string_lossy().to_string(),
                            line_number: self.find_line_number(content, service_pattern),
                            confidence: 0.85,
                        });
                        break; // Only add once per service per file
                    }
                }
            }
            
            // If no specific services found, add a generic AWS SDK entry
            if !services.iter().any(|s| s.name.starts_with("AWS ") && s.configuration.contains_key("sdk_version")) {
                services.push(DetectedService {
                    provider: ServiceProvider::Aws,
                    service_type: ServiceType::CloudProvider,
                    name: "AWS SDK (v2)".to_string(),
                    configuration: {
                        let mut config = HashMap::new();
                        config.insert("sdk_version".to_string(), "v2".to_string());
                        config.insert("note".to_string(), "Generic AWS SDK detected - specific services not identified".to_string());
                        config
                    },
                    file_path: file_path.to_string_lossy().to_string(),
                    line_number: self.find_line_number(content, "aws-sdk"),
                    confidence: 0.7,
                });
            }
        }
        
        // Detect other service SDKs
        let sdk_patterns = vec![
            ("@clerk/", ServiceProvider::Clerk, ServiceType::Auth),
            ("@auth0/", ServiceProvider::Auth0, ServiceType::Auth),
            ("stripe", ServiceProvider::Stripe, ServiceType::Payment),
            ("twilio", ServiceProvider::Twilio, ServiceType::Api),
            ("@sendgrid/", ServiceProvider::SendGrid, ServiceType::Api),
            ("@slack/", ServiceProvider::Slack, ServiceType::Api),
            ("discord.js", ServiceProvider::Discord, ServiceType::Api),
            ("@datadog/", ServiceProvider::Datadog, ServiceType::Monitoring),
            ("@sentry/", ServiceProvider::Sentry, ServiceType::Monitoring),
            ("@vercel/", ServiceProvider::Vercel, ServiceType::CloudProvider),
            // AI SDKs
            ("openai", ServiceProvider::OpenAI, ServiceType::AI),
            ("@openai/", ServiceProvider::OpenAI, ServiceType::AI),
            ("anthropic", ServiceProvider::Anthropic, ServiceType::AI),
            ("@anthropic-ai/", ServiceProvider::Anthropic, ServiceType::AI),
            ("@anthropic-ai/sdk", ServiceProvider::Anthropic, ServiceType::AI),
            ("github-copilot", ServiceProvider::GitHubCopilot, ServiceType::AI),
            ("@copilot/", ServiceProvider::GitHubCopilot, ServiceType::AI),
            ("@google/generative-ai", ServiceProvider::GoogleAI, ServiceType::AI),
            ("@google-ai/generativelanguage", ServiceProvider::GoogleAI, ServiceType::AI),
            ("google-generativeai", ServiceProvider::GoogleAI, ServiceType::AI),
            ("cohere", ServiceProvider::Cohere, ServiceType::AI),
            ("@cohere-ai/", ServiceProvider::Cohere, ServiceType::AI),
            ("huggingface", ServiceProvider::HuggingFace, ServiceType::AI),
            ("@huggingface/", ServiceProvider::HuggingFace, ServiceType::AI),
            ("transformers", ServiceProvider::HuggingFace, ServiceType::AI),
            ("replicate", ServiceProvider::Replicate, ServiceType::AI),
            ("together", ServiceProvider::TogetherAI, ServiceType::AI),
            ("@together-ai/", ServiceProvider::TogetherAI, ServiceType::AI),
            ("mistralai", ServiceProvider::MistralAI, ServiceType::AI),
            ("@mistralai/", ServiceProvider::MistralAI, ServiceType::AI),
            ("perplexity", ServiceProvider::Perplexity, ServiceType::AI),
            ("@perplexity/", ServiceProvider::Perplexity, ServiceType::AI),
        ];
        
        for (pattern, provider, service_type) in sdk_patterns {
            if content_lower.contains(pattern) {
                services.push(DetectedService {
                    provider: provider.clone(),
                    service_type: service_type.clone(),
                    name: format!("{} SDK", pattern.replace("@", "").replace("/", "")),
                    configuration: HashMap::new(),
                    file_path: file_path.to_string_lossy().to_string(),
                    line_number: self.find_line_number(content, pattern),
                    confidence: 0.8,
                });
            }
        }
        
        Ok(services)
    }
    
    /// Format AWS service name from SDK client name
    fn format_aws_service_name(&self, service: &str) -> String {
        // Map common AWS SDK client names to display names
        let service_map: std::collections::HashMap<&str, &str> = [
            ("s3", "S3"),
            ("dynamodb", "DynamoDB"),
            ("lambda", "Lambda"),
            ("sns", "SNS"),
            ("sqs", "SQS"),
            ("ses", "SES"),
            ("ec2", "EC2"),
            ("rds", "RDS"),
            ("cloudfront", "CloudFront"),
            ("cognito", "Cognito"),
            ("cognito-identity-provider", "Cognito"),
            ("iam", "IAM"),
            ("sts", "STS"),
            ("cloudwatch", "CloudWatch"),
            ("kms", "KMS"),
            ("secrets-manager", "Secrets Manager"),
            ("ssm", "Systems Manager"),
            ("apigateway", "API Gateway"),
            ("apigatewayv2", "API Gateway v2"),
            ("eventbridge", "EventBridge"),
            ("stepfunctions", "Step Functions"),
            ("s3-control", "S3 Control"),
            ("s3-outposts", "S3 Outposts"),
            ("textract", "Textract"),
            ("comprehend", "Comprehend"),
            ("translate", "Translate"),
            ("polly", "Polly"),
            ("rekognition", "Rekognition"),
            ("transcribe", "Transcribe"),
            ("bedrock", "Bedrock"),
            ("bedrock-runtime", "Bedrock Runtime"),
        ]
        .iter()
        .cloned()
        .collect();
        
        // Check if we have a mapping
        if let Some(display_name) = service_map.get(service) {
            return display_name.to_string();
        }
        
        // Otherwise, format it nicely
        service.split('-')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Detect API endpoints
    fn detect_api_endpoints(&self, content: &str, file_path: &Path) -> Result<Vec<DetectedService>> {
        let mut services = Vec::new();
        
        // Common API endpoint patterns
        let api_patterns = vec![
            ("api.github.com", ServiceProvider::GitHub),
            ("api.gitlab.com", ServiceProvider::GitLab),
            ("api.linear.app", ServiceProvider::Linear),
            ("api.atlassian.com", ServiceProvider::Jira),
            // AI API endpoints
            ("api.openai.com", ServiceProvider::OpenAI),
            ("api.anthropic.com", ServiceProvider::Anthropic),
            ("api.cohere.ai", ServiceProvider::Cohere),
            ("api-inference.huggingface.co", ServiceProvider::HuggingFace),
            ("api.replicate.com", ServiceProvider::Replicate),
            ("api.together.xyz", ServiceProvider::TogetherAI),
            ("api.mistral.ai", ServiceProvider::MistralAI),
            ("api.perplexity.ai", ServiceProvider::Perplexity),
            ("generativelanguage.googleapis.com", ServiceProvider::GoogleAI),
            ("generativeai.googleapis.com", ServiceProvider::GoogleAI),
        ];
        
        for (pattern, provider) in api_patterns {
            if content.contains(pattern) {
                services.push(DetectedService {
                    provider: provider.clone(),
                    service_type: ServiceType::Api,
                    name: format!("{} API", pattern),
                    configuration: HashMap::new(),
                    file_path: file_path.to_string_lossy().to_string(),
                    line_number: self.find_line_number(content, pattern),
                    confidence: 0.7,
                });
            }
        }
        
        Ok(services)
    }

    /// Extract Vercel configuration
    fn extract_vercel_config(&self, content: &str) -> HashMap<String, String> {
        let mut config = HashMap::new();
        
        // Try to parse as JSON
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(content) {
            if let Some(project_id) = json.get("projectId").and_then(|v| v.as_str()) {
                config.insert("projectId".to_string(), project_id.to_string());
            }
        }
        
        config
    }

    /// Extract Netlify configuration
    fn extract_netlify_config(&self, content: &str) -> HashMap<String, String> {
        let mut config = HashMap::new();
        
        // Try to parse as TOML
        if let Ok(toml) = toml::from_str::<toml::Value>(content) {
            if let Some(build) = toml.get("build") {
                if let Some(command) = build.get("command").and_then(|v| v.as_str()) {
                    config.insert("build_command".to_string(), command.to_string());
                }
            }
        }
        
        config
    }

    /// Find line number of a pattern in content
    fn find_line_number(&self, content: &str, pattern: &str) -> Option<usize> {
        content.lines()
            .enumerate()
            .find(|(_, line)| line.contains(pattern))
            .map(|(idx, _)| idx + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_detect_aws_services() {
        let temp_dir = TempDir::new().unwrap();
        let tf_file = temp_dir.path().join("main.tf");
        
        fs::write(&tf_file, r#"
resource "aws_s3_bucket" "example" {
  bucket = "my-bucket"
}

resource "aws_lambda_function" "example" {
  function_name = "my-function"
}
        "#).unwrap();

        let detector = ServiceDetector::new();
        let services = detector.detect_aws_services(&fs::read_to_string(&tf_file).unwrap(), &tf_file).unwrap();
        
        assert!(services.iter().any(|s| s.name == "S3"));
        assert!(services.iter().any(|s| s.name == "Lambda"));
    }

    #[test]
    fn test_detect_env_vars() {
        let temp_dir = TempDir::new().unwrap();
        let env_file = temp_dir.path().join(".env");
        
        fs::write(&env_file, r#"
CLERK_SECRET_KEY=sk_test_123
STRIPE_API_KEY=sk_live_456
        "#).unwrap();

        let detector = ServiceDetector::new();
        let services = detector.detect_from_env_vars(&fs::read_to_string(&env_file).unwrap(), &env_file).unwrap();
        
        assert!(services.iter().any(|s| s.provider == ServiceProvider::Clerk));
        assert!(services.iter().any(|s| s.provider == ServiceProvider::Stripe));
    }
}

