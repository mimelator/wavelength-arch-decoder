use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use crate::analysis::dependencies::DependencyExtractor;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericProvider {
    pub name: String,
    pub provider_hint: Option<String>,
    pub service_type_hint: Option<String>,
    pub confidence: f64,
    pub source: String, // "package.json", "requirements.txt", etc.
}

pub struct GenericProviderDetector;

impl GenericProviderDetector {
    pub fn new() -> Self {
        GenericProviderDetector
    }

    /// Detect providers from package files
    pub fn detect_from_packages(&self, repo_path: &Path) -> Result<Vec<GenericProvider>> {
        let mut providers = Vec::new();
        let extractor = DependencyExtractor::new();
        let manifests = extractor.extract_from_repository(repo_path)?;

        for manifest in manifests {
            match manifest.package_manager {
                crate::analysis::dependencies::PackageManager::Npm => {
                    providers.extend(self.detect_from_npm_packages(&manifest.dependencies)?);
                }
                crate::analysis::dependencies::PackageManager::Pip => {
                    providers.extend(self.detect_from_pip_packages(&manifest.dependencies)?);
                }
                crate::analysis::dependencies::PackageManager::Cargo => {
                    providers.extend(self.detect_from_cargo_packages(&manifest.dependencies)?);
                }
                crate::analysis::dependencies::PackageManager::Go => {
                    providers.extend(self.detect_from_go_packages(&manifest.dependencies)?);
                }
                _ => {}
            }
        }

        Ok(providers)
    }

    fn detect_from_npm_packages(&self, deps: &[crate::analysis::dependencies::PackageDependency]) -> Result<Vec<GenericProvider>> {
        let mut providers = Vec::new();
        
        // Common patterns for npm packages that indicate services
        let service_patterns = vec![
            // Cloud providers
            ("@vercel/", "Vercel", Some("CloudProvider")),
            ("@netlify/", "Netlify", Some("CloudProvider")),
            ("@aws/", "AWS", Some("CloudProvider")),
            ("@azure/", "Azure", Some("CloudProvider")),
            ("@google-cloud/", "GCP", Some("CloudProvider")),
            
            // Auth
            ("@clerk/", "Clerk", Some("Auth")),
            ("@auth0/", "Auth0", Some("Auth")),
            ("passport-", "Auth", Some("Auth")),
            
            // Payment
            ("stripe", "Stripe", Some("Payment")),
            ("paypal", "PayPal", Some("Payment")),
            
            // Monitoring
            ("@datadog/", "Datadog", Some("Monitoring")),
            ("@sentry/", "Sentry", Some("Monitoring")),
            ("@newrelic/", "NewRelic", Some("Monitoring")),
            
            // AI
            ("openai", "OpenAI", Some("AI")),
            ("@openai/", "OpenAI", Some("AI")),
            ("anthropic", "Anthropic", Some("AI")),
            ("@anthropic-ai/", "Anthropic", Some("AI")),
            ("@google/generative-ai", "GoogleAI", Some("AI")),
            ("cohere", "Cohere", Some("AI")),
            ("@huggingface/", "HuggingFace", Some("AI")),
            
            // APIs
            ("twilio", "Twilio", Some("Api")),
            ("@sendgrid/", "SendGrid", Some("Api")),
            ("@slack/", "Slack", Some("Api")),
            ("discord.js", "Discord", Some("Api")),
        ];

        for dep in deps {
            let name_lower = dep.name.to_lowercase();
            
            // Check against known patterns
            for (pattern, provider, service_type) in &service_patterns {
                if name_lower.contains(pattern) {
                    providers.push(GenericProvider {
                        name: dep.name.clone(),
                        provider_hint: Some(provider.to_string()),
                        service_type_hint: service_type.map(|s| s.to_string()),
                        confidence: 0.6,
                        source: "package.json".to_string(),
                    });
                    break;
                }
            }
            
            // Generic detection: if package name suggests a service
            if self.looks_like_service_package(&name_lower) {
                // Extract potential provider name
                let provider_hint = self.extract_provider_name(&name_lower);
                providers.push(GenericProvider {
                    name: dep.name.clone(),
                    provider_hint,
                    service_type_hint: None,
                    confidence: 0.4,
                    source: "package.json".to_string(),
                });
            }
        }

        Ok(providers)
    }

    fn detect_from_pip_packages(&self, deps: &[crate::analysis::dependencies::PackageDependency]) -> Result<Vec<GenericProvider>> {
        let mut providers = Vec::new();
        
        let service_patterns = vec![
            ("boto3", "AWS", Some("CloudProvider")),
            ("google-cloud-", "GCP", Some("CloudProvider")),
            ("azure-", "Azure", Some("CloudProvider")),
            ("stripe", "Stripe", Some("Payment")),
            ("twilio", "Twilio", Some("Api")),
            ("sendgrid", "SendGrid", Some("Api")),
            ("slack-sdk", "Slack", Some("Api")),
            ("discord.py", "Discord", Some("Api")),
            ("openai", "OpenAI", Some("AI")),
            ("anthropic", "Anthropic", Some("AI")),
            ("cohere", "Cohere", Some("AI")),
            ("datadog", "Datadog", Some("Monitoring")),
            ("sentry-sdk", "Sentry", Some("Monitoring")),
        ];

        for dep in deps {
            let name_lower = dep.name.to_lowercase();
            
            for (pattern, provider, service_type) in &service_patterns {
                if name_lower.contains(pattern) {
                    providers.push(GenericProvider {
                        name: dep.name.clone(),
                        provider_hint: Some(provider.to_string()),
                        service_type_hint: service_type.map(|s| s.to_string()),
                        confidence: 0.6,
                        source: "requirements.txt".to_string(),
                    });
                    break;
                }
            }
        }

        Ok(providers)
    }

    fn detect_from_cargo_packages(&self, deps: &[crate::analysis::dependencies::PackageDependency]) -> Result<Vec<GenericProvider>> {
        let mut providers = Vec::new();
        
        let service_patterns = vec![
            ("aws-sdk", "AWS", Some("CloudProvider")),
            ("rusoto", "AWS", Some("CloudProvider")),
            ("google-cloud", "GCP", Some("CloudProvider")),
            ("azure", "Azure", Some("CloudProvider")),
            ("stripe", "Stripe", Some("Payment")),
            ("twilio", "Twilio", Some("Api")),
            ("sentry", "Sentry", Some("Monitoring")),
        ];

        for dep in deps {
            let name_lower = dep.name.to_lowercase();
            
            for (pattern, provider, service_type) in &service_patterns {
                if name_lower.contains(pattern) {
                    providers.push(GenericProvider {
                        name: dep.name.clone(),
                        provider_hint: Some(provider.to_string()),
                        service_type_hint: service_type.map(|s| s.to_string()),
                        confidence: 0.6,
                        source: "Cargo.toml".to_string(),
                    });
                    break;
                }
            }
        }

        Ok(providers)
    }

    fn detect_from_go_packages(&self, deps: &[crate::analysis::dependencies::PackageDependency]) -> Result<Vec<GenericProvider>> {
        let mut providers = Vec::new();
        
        let service_patterns = vec![
            ("github.com/aws/aws-sdk-go", "AWS", Some("CloudProvider")),
            ("cloud.google.com/go", "GCP", Some("CloudProvider")),
            ("github.com/stripe/", "Stripe", Some("Payment")),
            ("github.com/twilio/", "Twilio", Some("Api")),
            ("github.com/slack-go/", "Slack", Some("Api")),
            ("github.com/bwmarrin/discordgo", "Discord", Some("Api")),
        ];

        for dep in deps {
            let name_lower = dep.name.to_lowercase();
            
            for (pattern, provider, service_type) in &service_patterns {
                if name_lower.contains(pattern) {
                    providers.push(GenericProvider {
                        name: dep.name.clone(),
                        provider_hint: Some(provider.to_string()),
                        service_type_hint: service_type.map(|s| s.to_string()),
                        confidence: 0.6,
                        source: "go.mod".to_string(),
                    });
                    break;
                }
            }
        }

        Ok(providers)
    }

    /// Check if a package name looks like it might be a service SDK
    fn looks_like_service_package(&self, name: &str) -> bool {
        // Common suffixes/prefixes that suggest services
        let indicators = vec![
            "-sdk", "-client", "-api", "-service",
            "sdk-", "client-", "api-", "service-",
        ];
        
        indicators.iter().any(|indicator| name.contains(indicator))
    }

    /// Extract a potential provider name from a package name
    fn extract_provider_name(&self, name: &str) -> Option<String> {
        // Try to extract provider name from common patterns
        // e.g., "@company/service-sdk" -> "Company"
        // e.g., "company-service-client" -> "Company"
        
        // Remove common prefixes/suffixes
        let cleaned = name
            .replace("-sdk", "")
            .replace("-client", "")
            .replace("-api", "")
            .replace("-service", "")
            .replace("sdk-", "")
            .replace("client-", "")
            .replace("api-", "")
            .replace("service-", "");
        
        // Extract first meaningful word (after @ or /)
        let parts: Vec<&str> = cleaned.split(&['@', '/', '-', '_'][..]).collect();
        if let Some(first_part) = parts.first() {
            if !first_part.is_empty() && first_part.len() > 2 {
                // Capitalize first letter
                let mut chars = first_part.chars();
                if let Some(first) = chars.next() {
                    return Some(first.to_uppercase().collect::<String>() + chars.as_str());
                }
            }
        }
        
        None
    }
}

