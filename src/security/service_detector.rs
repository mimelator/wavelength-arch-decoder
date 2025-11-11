use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use crate::ingestion::{FileIndexer, FileType};

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
        
        let indexer = FileIndexer::new(
            crate::ingestion::RepositoryCrawler::new(&crate::config::StorageConfig {
                repository_cache_path: "./cache".to_string(),
                max_cache_size: "10GB".to_string(),
            }).unwrap()
        );
        
        let files = indexer.index_repository(repo_path.to_str().unwrap())?;
        
        for file in files {
            // Detect services in configuration files
            if file.file_type == FileType::Config || file.file_type == FileType::Infrastructure {
                let detected = self.detect_in_file(&file.path, &file.file_type)?;
                services.extend(detected);
            }
            
            // Detect services in code files
            if file.file_type == FileType::Code {
                let detected = self.detect_in_code(&file.path, &file.language)?;
                services.extend(detected);
            }
        }
        
        Ok(services)
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
        
        // Detect npm packages / imports
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
            ("aws-sdk", ServiceProvider::Aws, ServiceType::CloudProvider),
            ("@aws-sdk/", ServiceProvider::Aws, ServiceType::CloudProvider),
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

    /// Detect API endpoints
    fn detect_api_endpoints(&self, content: &str, file_path: &Path) -> Result<Vec<DetectedService>> {
        let mut services = Vec::new();
        
        // Common API endpoint patterns
        let api_patterns = vec![
            ("api.github.com", ServiceProvider::GitHub),
            ("api.gitlab.com", ServiceProvider::GitLab),
            ("api.linear.app", ServiceProvider::Linear),
            ("api.atlassian.com", ServiceProvider::Jira),
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

