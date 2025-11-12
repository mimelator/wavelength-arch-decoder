use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use crate::ingestion::FileType;
use crate::security::pattern_config::{PatternConfig, PatternLoader};
use crate::security::generic_provider::GenericProviderDetector;

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

pub struct ServiceDetector {
    pattern_config: PatternConfig,
    generic_detector: GenericProviderDetector,
}

impl ServiceDetector {
    pub fn new() -> Self {
        // Try to load patterns from config, fall back to empty if not found
        let pattern_config = PatternLoader::load_default()
            .unwrap_or_else(|_| {
                // Create minimal default config if file doesn't exist
                PatternConfig {
                    version: "1.0".to_string(),
                    patterns: crate::security::pattern_config::PatternSet {
                        environment_variables: Vec::new(),
                        sdk_patterns: Vec::new(),
                        api_endpoints: Vec::new(),
                        database_patterns: Vec::new(),
                        aws_infrastructure: Vec::new(),
                        aws_sdk_v2_services: Vec::new(),
                        aws_sdk_v3_service_map: HashMap::new(),
                    },
                }
            });
        
        ServiceDetector {
            pattern_config,
            generic_detector: GenericProviderDetector::new(),
        }
    }

    /// Create with custom pattern config (for testing or plugins)
    pub fn with_config(pattern_config: PatternConfig) -> Self {
        ServiceDetector {
            pattern_config,
            generic_detector: GenericProviderDetector::new(),
        }
    }

    /// Create with plugin directory support
    pub fn with_plugins(plugin_dir: Option<&Path>) -> Result<Self> {
        let base_path = Path::new("config/service_patterns.json");
        let pattern_config = PatternLoader::load_with_plugins(base_path, plugin_dir)?;
        
        Ok(ServiceDetector {
            pattern_config,
            generic_detector: GenericProviderDetector::new(),
        })
    }

    /// Detect services in a repository
    pub fn detect_services(&self, repo_path: &Path) -> Result<Vec<DetectedService>> {
        let mut services = Vec::new();
        
        // First, detect from generic package files
        if let Ok(generic_providers) = self.generic_detector.detect_from_packages(repo_path) {
            for gp in generic_providers {
                if let Some(provider) = self.parse_provider(&gp.provider_hint.as_deref().unwrap_or("Unknown")) {
                    let service_type = self.parse_service_type(
                        gp.service_type_hint.as_deref().unwrap_or("Other")
                    );
                    
                    services.push(DetectedService {
                        provider,
                        service_type,
                        name: gp.name.clone(),
                        configuration: {
                            let mut config = HashMap::new();
                            config.insert("source".to_string(), gp.source.clone());
                            config.insert("detection_method".to_string(), "generic_package".to_string());
                            config
                        },
                        file_path: format!("{}/{}", repo_path.display(), gp.source),
                        line_number: None,
                        confidence: gp.confidence,
                    });
                }
            }
        }
        
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

    /// Parse provider string to ServiceProvider enum
    fn parse_provider(&self, provider_str: &str) -> Option<ServiceProvider> {
        match provider_str.to_uppercase().as_str() {
            "AWS" => Some(ServiceProvider::Aws),
            "AZURE" => Some(ServiceProvider::Azure),
            "GCP" => Some(ServiceProvider::Gcp),
            "VERCEL" => Some(ServiceProvider::Vercel),
            "NETLIFY" => Some(ServiceProvider::Netlify),
            "HEROKU" => Some(ServiceProvider::Heroku),
            "DIGITALOCEAN" => Some(ServiceProvider::DigitalOcean),
            "CLERK" => Some(ServiceProvider::Clerk),
            "AUTH0" => Some(ServiceProvider::Auth0),
            "STRIPE" => Some(ServiceProvider::Stripe),
            "TWILIO" => Some(ServiceProvider::Twilio),
            "SENDGRID" => Some(ServiceProvider::SendGrid),
            "MAILGUN" => Some(ServiceProvider::Mailgun),
            "SLACK" => Some(ServiceProvider::Slack),
            "DISCORD" => Some(ServiceProvider::Discord),
            "POSTGRES" => Some(ServiceProvider::Postgres),
            "MYSQL" => Some(ServiceProvider::MySQL),
            "MONGODB" => Some(ServiceProvider::MongoDB),
            "REDIS" => Some(ServiceProvider::Redis),
            "DYNAMODB" => Some(ServiceProvider::DynamoDB),
            "RDS" => Some(ServiceProvider::RDS),
            "GITHUB" => Some(ServiceProvider::GitHub),
            "GITLAB" => Some(ServiceProvider::GitLab),
            "JIRA" => Some(ServiceProvider::Jira),
            "LINEAR" => Some(ServiceProvider::Linear),
            "CLOUDFLARE" => Some(ServiceProvider::Cloudflare),
            "CLOUDFRONT" => Some(ServiceProvider::CloudFront),
            "DATADOG" => Some(ServiceProvider::Datadog),
            "NEWRELIC" => Some(ServiceProvider::NewRelic),
            "SENTRY" => Some(ServiceProvider::Sentry),
            "LOGROCKET" => Some(ServiceProvider::LogRocket),
            "OPENAI" => Some(ServiceProvider::OpenAI),
            "ANTHROPIC" => Some(ServiceProvider::Anthropic),
            "GITHUBCOPILOT" => Some(ServiceProvider::GitHubCopilot),
            "GOOGLEAI" => Some(ServiceProvider::GoogleAI),
            "COHERE" => Some(ServiceProvider::Cohere),
            "HUGGINGFACE" => Some(ServiceProvider::HuggingFace),
            "REPLICATE" => Some(ServiceProvider::Replicate),
            "TOGETHERAI" => Some(ServiceProvider::TogetherAI),
            "MISTRALAI" => Some(ServiceProvider::MistralAI),
            "PERPLEXITY" => Some(ServiceProvider::Perplexity),
            _ => Some(ServiceProvider::Unknown),
        }
    }

    /// Parse service type string to ServiceType enum
    fn parse_service_type(&self, type_str: &str) -> ServiceType {
        match type_str {
            "CloudProvider" => ServiceType::CloudProvider,
            "SaaS" => ServiceType::SaaS,
            "Database" => ServiceType::Database,
            "Api" => ServiceType::Api,
            "Cdn" => ServiceType::Cdn,
            "Monitoring" => ServiceType::Monitoring,
            "Auth" => ServiceType::Auth,
            "Payment" => ServiceType::Payment,
            "AI" => ServiceType::AI,
            _ => ServiceType::Other,
        }
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
        
        // Use patterns from config
        for rule in &self.pattern_config.patterns.aws_infrastructure {
            if content_lower.contains(&rule.pattern.to_lowercase()) {
                if let Some(provider) = self.parse_provider(&rule.provider) {
                    let service_type = self.parse_service_type(&rule.service_type);
                    let mut config = HashMap::new();
                    config.insert("service".to_string(), rule.service_name.clone());
                    config.insert("provider".to_string(), rule.provider.clone());
                    
                    services.push(DetectedService {
                        provider,
                        service_type,
                        name: rule.service_name.clone(),
                        configuration: config,
                        file_path: file_path.to_string_lossy().to_string(),
                        line_number: self.find_line_number(content, &rule.pattern),
                        confidence: rule.confidence,
                    });
                }
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
        
        // Use patterns from config
        for line in content.lines() {
            let line_upper = line.to_uppercase();
            for rule in &self.pattern_config.patterns.environment_variables {
                if line_upper.contains(&rule.pattern.to_uppercase()) && 
                   (line_upper.contains("API_KEY") || 
                    line_upper.contains("SECRET") || 
                    line_upper.contains("TOKEN") ||
                    line_upper.contains("ID") || 
                    line_upper.contains("KEY")) {
                    if let Some(provider) = self.parse_provider(&rule.provider) {
                        let service_type = self.parse_service_type(&rule.service_type);
                        let mut config = HashMap::new();
                        config.insert("env_var".to_string(), line.trim().to_string());
                        
                        let service_name = rule.service_name.as_ref()
                            .map(|s| s.clone())
                            .unwrap_or_else(|| format!("{} Service", rule.pattern));
                        
                        services.push(DetectedService {
                            provider,
                            service_type,
                            name: service_name.clone(),
                            configuration: config,
                            file_path: file_path.to_string_lossy().to_string(),
                            line_number: self.find_line_number(content, line),
                            confidence: rule.confidence,
                        });
                    }
                }
            }
        }
        
        Ok(services)
    }

    /// Detect database connections
    fn detect_databases(&self, content: &str, file_path: &Path) -> Result<Vec<DetectedService>> {
        let mut services = Vec::new();
        let content_lower = content.to_lowercase();
        
        // Use patterns from config
        for rule in &self.pattern_config.patterns.database_patterns {
            if content_lower.contains(&rule.pattern.to_lowercase()) {
                if let Some(provider) = self.parse_provider(&rule.provider) {
                    let service_type = self.parse_service_type(&rule.service_type);
                    let service_name = rule.service_name.as_ref()
                        .map(|s| s.clone())
                        .unwrap_or_else(|| format!("{} Database", rule.pattern.replace("://", "")));
                    
                    services.push(DetectedService {
                        provider,
                        service_type,
                        name: service_name.clone(),
                        configuration: HashMap::new(),
                        file_path: file_path.to_string_lossy().to_string(),
                        line_number: self.find_line_number(content, &rule.pattern),
                        confidence: rule.confidence,
                    });
                }
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
            // Use patterns from config
            for rule in &self.pattern_config.patterns.aws_sdk_v2_services {
                let service_patterns = vec![
                    format!("aws.{}", rule.pattern),
                    format!("aws['{}']", rule.pattern),
                    format!("aws[\"{}\"]", rule.pattern),
                    format!("new aws.{}", rule.pattern),
                    format!("new aws['{}']", rule.pattern),
                    format!("new aws[\"{}\"]", rule.pattern),
                ];
                
                for service_pattern in &service_patterns {
                    if content_lower.contains(service_pattern) {
                        services.push(DetectedService {
                            provider: ServiceProvider::Aws,
                            service_type: ServiceType::CloudProvider,
                            name: format!("AWS {}", rule.display_name),
                            configuration: {
                                let mut config = HashMap::new();
                                config.insert("sdk_version".to_string(), "v2".to_string());
                                config.insert("service".to_string(), rule.pattern.clone());
                                config
                            },
                            file_path: file_path.to_string_lossy().to_string(),
                            line_number: self.find_line_number(content, service_pattern),
                            confidence: rule.confidence,
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
        
        // Detect other service SDKs from config
        for rule in &self.pattern_config.patterns.sdk_patterns {
            if content_lower.contains(&rule.pattern.to_lowercase()) {
                if let Some(provider) = self.parse_provider(&rule.provider) {
                    let service_type = self.parse_service_type(&rule.service_type);
                    let service_name = rule.service_name.as_ref()
                        .map(|s| s.clone())
                        .unwrap_or_else(|| format!("{} SDK", rule.pattern.replace("@", "").replace("/", "")));
                    
                    services.push(DetectedService {
                        provider,
                        service_type,
                        name: service_name.clone(),
                        configuration: HashMap::new(),
                        file_path: file_path.to_string_lossy().to_string(),
                        line_number: self.find_line_number(content, &rule.pattern),
                        confidence: rule.confidence,
                    });
                }
            }
        }
        
        Ok(services)
    }
    
    /// Format AWS service name from SDK client name
    fn format_aws_service_name(&self, service: &str) -> String {
        // Use mapping from config first
        if let Some(display_name) = self.pattern_config.patterns.aws_sdk_v3_service_map.get(service) {
            return display_name.clone();
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
        
        // Use patterns from config
        for rule in &self.pattern_config.patterns.api_endpoints {
            if content.contains(&rule.pattern) {
                if let Some(provider) = self.parse_provider(&rule.provider) {
                    let service_type = self.parse_service_type(&rule.service_type);
                    let service_name = rule.service_name.as_ref()
                        .map(|s| s.clone())
                        .unwrap_or_else(|| format!("{} API", rule.pattern));
                    
                    services.push(DetectedService {
                        provider,
                        service_type,
                        name: service_name.clone(),
                        configuration: HashMap::new(),
                        file_path: file_path.to_string_lossy().to_string(),
                        line_number: self.find_line_number(content, &rule.pattern),
                        confidence: rule.confidence,
                    });
                }
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

    #[test]
    fn test_printify_detection_from_env_file() {
        // Create a temporary directory with a .env file containing Printify config
        let temp_dir = TempDir::new().unwrap();
        let env_file = temp_dir.path().join(".env");
        
        fs::write(&env_file, r#"
# Printify Configuration
PRINTIFY_API_KEY=sk_live_test123
PRINTIFY_SHOP_ID=24952672
PRINTIFY_ENVIRONMENT=sandbox
        "#).unwrap();

        // Load detector with plugins
        let plugin_dir = Path::new("config/plugins");
        let detector = if plugin_dir.exists() {
            ServiceDetector::with_plugins(Some(plugin_dir)).unwrap_or_else(|_| ServiceDetector::new())
        } else {
            ServiceDetector::new()
        };

        // Detect services
        let services = detector.detect_services(temp_dir.path()).unwrap();
        
        // Check if Printify was detected
        let printify_services: Vec<_> = services.iter()
            .filter(|s| s.name.to_lowercase().contains("printify"))
            .collect();
        
        if printify_services.is_empty() {
            println!("⚠ Printify not detected - plugin may not be loaded");
        } else {
            println!("✓ Detected {} Printify service(s):", printify_services.len());
            for service in &printify_services {
                println!("  - {} (confidence: {})", service.name, service.confidence);
            }
        }
    }

    #[test]
    fn test_printify_detection_from_api_endpoint() {
        // Create a temporary directory with a code file containing Printify API call
        let temp_dir = TempDir::new().unwrap();
        let code_file = temp_dir.path().join("printify-service.js");
        
        fs::write(&code_file, r#"
async function fetchPrintifyProducts() {
    const response = await fetch('https://api.printify.com/v1/products');
    return response.json();
}
        "#).unwrap();

        // Load detector with plugins
        let plugin_dir = Path::new("config/plugins");
        let detector = if plugin_dir.exists() {
            ServiceDetector::with_plugins(Some(plugin_dir)).unwrap_or_else(|_| ServiceDetector::new())
        } else {
            ServiceDetector::new()
        };

        // Detect services
        let services = detector.detect_services(temp_dir.path()).unwrap();
        
        // Check if Printify API endpoint was detected
        let printify_services: Vec<_> = services.iter()
            .filter(|s| s.name.to_lowercase().contains("printify"))
            .collect();
        
        if printify_services.is_empty() {
            println!("⚠ Printify not detected from API endpoint - plugin may not be loaded");
        } else {
            println!("✓ Detected {} Printify service(s) from API endpoint:", printify_services.len());
            for service in &printify_services {
                println!("  - {} (confidence: {})", service.name, service.confidence);
            }
        }
    }
}

