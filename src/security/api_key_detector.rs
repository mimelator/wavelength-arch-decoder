use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use walkdir::WalkDir;
use serde_json::Value;
use uuid::Uuid;
use crate::security::{SecurityEntity, SecurityRelationship, SecurityVulnerability, SecurityEntityType, VulnerabilitySeverity};
use crate::analysis::{CodeElement, CodeStructure};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedApiKey {
    pub name: String,
    pub key_type: String, // "hardcoded", "env_var", "config"
    pub provider: String,  // "aws", "github", "firebase", "generic", etc.
    pub value_preview: Option<String>, // First few chars if hardcoded
    pub file_path: String,
    pub line_number: Option<usize>,
    pub context: String, // Surrounding code/context
    pub used_by: Vec<String>, // Code element IDs that use this key
    pub service_ids: Vec<String>, // Service IDs that this key authenticates
}

pub struct ApiKeyDetector;

impl ApiKeyDetector {
    pub fn new() -> Self {
        ApiKeyDetector
    }

    /// Detect API keys in a repository and link them to code elements and services
    pub fn detect_api_keys(
        &self, 
        repo_path: &Path,
        code_structure: Option<&CodeStructure>,
        services: Option<&[crate::security::DetectedService]>,
    ) -> Result<(Vec<SecurityEntity>, Vec<SecurityRelationship>, Vec<SecurityVulnerability>)> {
        log::info!("Starting API key detection in repository: {}", repo_path.display());
        let mut entities = Vec::new();
        let mut relationships = Vec::new();
        let mut vulnerabilities = Vec::new();
        let mut detected_keys: Vec<DetectedApiKey> = Vec::new();

        // Build code element map by file path (normalize paths relative to repo root)
        let mut code_elements_by_file: HashMap<String, Vec<&CodeElement>> = HashMap::new();
        if let Some(code_struct) = code_structure {
            log::info!("Building code element map from {} elements", code_struct.elements.len());
            for element in &code_struct.elements {
                // Normalize path: make it relative to repo_path if it's absolute
                let normalized_path = if let Ok(rel_path) = std::path::Path::new(&element.file_path).strip_prefix(repo_path) {
                    rel_path.to_string_lossy().to_string()
                } else {
                    element.file_path.clone()
                };
                code_elements_by_file
                    .entry(normalized_path)
                    .or_insert_with(Vec::new)
                    .push(element);
            }
            log::info!("Built code element map with {} unique files", code_elements_by_file.len());
        } else {
            log::warn!("No code structure provided for API key detection");
        }

        // Build service map by provider and name for matching API keys
        let mut services_by_provider: HashMap<String, Vec<String>> = HashMap::new();
        let mut services_by_name: HashMap<String, Vec<(String, String)>> = HashMap::new(); // provider -> [(service_name, service_id)]
        if let Some(svcs) = services {
            log::info!("Found {} services for API key matching", svcs.len());
            for service in svcs {
                let provider = match service.provider {
                    crate::security::ServiceProvider::Aws => "aws",
                    crate::security::ServiceProvider::Gcp => "gcp",
                    crate::security::ServiceProvider::Azure => "azure",
                    crate::security::ServiceProvider::GitHub => "github",
                    crate::security::ServiceProvider::Vercel => "vercel",
                    crate::security::ServiceProvider::Netlify => "netlify",
                    crate::security::ServiceProvider::Stripe => "stripe",
                    crate::security::ServiceProvider::Twilio => "twilio",
                    crate::security::ServiceProvider::SendGrid => "sendgrid",
                    crate::security::ServiceProvider::Mailgun => "mailgun",
                    // AI Providers
                    crate::security::ServiceProvider::OpenAI => "openai",
                    crate::security::ServiceProvider::Anthropic => "anthropic",
                    crate::security::ServiceProvider::GitHubCopilot => "github_copilot",
                    crate::security::ServiceProvider::GoogleAI => "google_ai",
                    crate::security::ServiceProvider::Cohere => "cohere",
                    crate::security::ServiceProvider::HuggingFace => "huggingface",
                    crate::security::ServiceProvider::Replicate => "replicate",
                    crate::security::ServiceProvider::TogetherAI => "together_ai",
                    crate::security::ServiceProvider::MistralAI => "mistral_ai",
                    crate::security::ServiceProvider::Perplexity => "perplexity",
                    crate::security::ServiceProvider::Unknown => "generic",
                    _ => "generic",
                };
                services_by_provider
                    .entry(provider.to_string())
                    .or_insert_with(Vec::new)
                    .push(service.name.clone());
                services_by_name
                    .entry(provider.to_string())
                    .or_insert_with(Vec::new)
                    .push((service.name.clone(), format!("service:{}", service.name.clone())));
            }
        } else {
            log::warn!("No services provided for API key detection");
        }

        let mut files_scanned = 0;
        let mut files_with_keys = 0;
        let mut files_skipped = 0;

        // First pass: Scan code files for API key patterns
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

            // Skip common ignore patterns
            if file_name.starts_with('.') || 
               path.to_string_lossy().contains("node_modules") ||
               path.to_string_lossy().contains("target") ||
               path.to_string_lossy().contains(".git") ||
               path.to_string_lossy().contains("vendor") ||
               file_name.ends_with(".min.js") ||
               file_name.ends_with(".bundle.js") {
                continue;
            }

            // Check file size - skip files larger than 1MB to avoid memory issues
            if let Ok(metadata) = std::fs::metadata(path) {
                if metadata.len() > 1_048_576 { // 1MB
                    files_skipped += 1;
                    if files_skipped % 100 == 0 {
                        log::debug!("Skipped {} large files so far...", files_skipped);
                    }
                    continue;
                }
            }

            // Only scan code files
            if file_name.ends_with(".js") || 
               file_name.ends_with(".ts") ||
               file_name.ends_with(".jsx") ||
               file_name.ends_with(".tsx") ||
               file_name.ends_with(".py") ||
               file_name.ends_with(".rs") ||
               file_name.ends_with(".go") ||
               file_name.ends_with(".java") ||
               file_name.ends_with(".rb") ||
               file_name.ends_with(".php") ||
               file_name.ends_with(".env") ||
               file_name.ends_with(".config.js") ||
               file_name.ends_with(".config.ts") ||
               file_name.contains("config") {
                
                files_scanned += 1;
                
                // Log progress every 100 files
                if files_scanned % 100 == 0 {
                    log::info!("API key detection progress: scanned {} files, found keys in {} files...", 
                        files_scanned, files_with_keys);
                }
                
                // Try to read file, but handle errors gracefully
                match std::fs::read_to_string(path) {
                    Ok(content) => {
                        // Normalize path relative to repo_path for matching and storage
                        let file_path_str = if let Ok(rel_path) = path.strip_prefix(repo_path) {
                            // Remove leading "./" if present
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
                        };
                        let file_elements = code_elements_by_file.get(&file_path_str);
                        match self.scan_file_for_keys(&content, path, file_elements, &file_path_str) {
                            Ok(keys) => {
                                if !keys.is_empty() {
                                    files_with_keys += 1;
                                    log::info!("Found {} API keys in {}", keys.len(), file_path_str);
                                }
                                detected_keys.extend(keys);
                            },
                            Err(e) => {
                                log::warn!("Error scanning file {}: {}", file_path_str, e);
                            }
                        }
                    },
                    Err(e) => {
                        // File might be binary or have encoding issues - skip silently
                        if files_scanned % 500 == 0 {
                            log::debug!("Skipped file {} (read error: {})", path.display(), e);
                        }
                    }
                }
            }
        }

        log::info!("API key detection complete: scanned {} files (skipped {} large files), found keys in {} files, total keys detected: {}", 
            files_scanned, files_skipped, files_with_keys, detected_keys.len());

        // Convert detected keys to security entities and create relationships
        for key in detected_keys {
            // Use UUID for unique ID, but include file path and line for reference
            let id = Uuid::new_v4().to_string();
            
            let mut config = HashMap::new();
            config.insert("key_name".to_string(), Value::String(key.name.clone()));
            config.insert("key_type".to_string(), Value::String(key.key_type.clone()));
            config.insert("provider".to_string(), Value::String(key.provider.clone()));
            config.insert("context".to_string(), Value::String(key.context.clone()));
            config.insert("file_path".to_string(), Value::String(key.file_path.clone()));
            if let Some(line_num) = key.line_number {
                config.insert("line_number".to_string(), Value::Number(serde_json::Number::from(line_num)));
            }
            if let Some(preview) = key.value_preview {
                config.insert("value_preview".to_string(), Value::String(preview));
            }
            config.insert("used_by_count".to_string(), Value::Number(serde_json::Number::from(key.used_by.len())));
            // Store code element IDs that use this key (for reference, even though we can't create DB relationships)
            if !key.used_by.is_empty() {
                config.insert("used_by_elements".to_string(), Value::Array(
                    key.used_by.iter().take(20).map(|id| Value::String(id.clone())).collect()
                ));
            }
            // Store which services this key might authenticate (by provider and name matching)
            let mut matched_services: Vec<String> = Vec::new();
            
            // Match by provider first (e.g., key.provider = "anthropic" matches Anthropic services)
            if let Some(service_names) = services_by_provider.get(&key.provider) {
                matched_services.extend(service_names.iter().cloned());
            }
            
            // Also try to match by key name patterns (e.g., ANTHROPIC_API_KEY -> Anthropic service)
            // This handles cases where the provider detection might have missed it
            let key_name_upper = key.name.to_uppercase();
            for (provider, service_list) in &services_by_name {
                // Check if key name contains provider name (with variations)
                let provider_variations = vec![
                    provider.to_uppercase(),
                    provider.to_uppercase().replace("_", ""),
                    provider.to_uppercase().replace("_", "_"),
                ];
                
                for provider_variant in provider_variations {
                    if key_name_upper.contains(&provider_variant) {
                        matched_services.extend(service_list.iter().map(|(name, _)| name.clone()));
                        break; // Found match, no need to check other variations
                    }
                }
            }
            
            // Remove duplicates and sort for consistent display
            matched_services.sort();
            matched_services.dedup();
            
            if !matched_services.is_empty() {
                config.insert("related_services".to_string(), Value::Array(
                    matched_services.iter().take(10).map(|n| Value::String(n.clone())).collect()
                ));
                config.insert("service_count".to_string(), Value::Number(serde_json::Number::from(matched_services.len())));
                log::info!("API key '{}' (provider: {}) matched to {} services: {:?}", 
                    key.name, key.provider, matched_services.len(), matched_services);
            } else {
                config.insert("service_count".to_string(), Value::Number(serde_json::Number::from(0)));
                log::debug!("API key '{}' (provider: {}) did not match any services", key.name, key.provider);
            }

            entities.push(SecurityEntity {
                id: id.clone(),
                entity_type: SecurityEntityType::ApiKey,
                name: format!("{} ({})", key.name, key.provider),
                provider: key.provider.clone(),
                configuration: config,
                file_path: key.file_path.clone(),
                line_number: key.line_number,
                arn: None,
                region: None,
            });

            // Note: We don't create relationships to code elements or services here because:
            // 1. Code elements are in the code_elements table, not security_entities
            // 2. Services are in the services table, not security_entities  
            // 3. security_relationships has foreign key constraints requiring both source and target
            //    to be in security_entities table
            // Instead, we store code element IDs and service names in the API key's configuration JSON
            // for reference. Relationships can be created later if needed via the knowledge graph.

            // Flag hardcoded keys as vulnerabilities
            if key.key_type == "hardcoded" {
                vulnerabilities.push(SecurityVulnerability {
                    id: format!("{}:vuln:hardcoded", Uuid::new_v4()),
                    entity_id: id.clone(),
                    vulnerability_type: "HardcodedApiKey".to_string(),
                    severity: VulnerabilitySeverity::Critical,
                    description: format!("Hardcoded API key detected: {}", key.name),
                    recommendation: "Move API key to environment variables or secure secret management system".to_string(),
                    file_path: key.file_path,
                    line_number: key.line_number,
                });
            }
        }

        Ok((entities, relationships, vulnerabilities))
    }

    /// Scan a file for API key patterns and link to code elements
    fn scan_file_for_keys(
        &self, 
        content: &str, 
        path: &Path,
        file_elements: Option<&Vec<&CodeElement>>,
        normalized_path: &str,
    ) -> Result<Vec<DetectedApiKey>> {
        let mut keys = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        // Common API key patterns (more specific to reduce false positives)
        let key_patterns = vec![
            // Variable assignments - require longer values and specific patterns
            (r#"(?i)(api[_-]?key|apikey)\s*[:=]\s*["']([^"']{16,})["']"#, "hardcoded"),
            (r#"(?i)(secret[_-]?key|secretkey|secret[_-]?access[_-]?key)\s*[:=]\s*["']([^"']{16,})["']"#, "hardcoded"),
            (r#"(?i)(access[_-]?token|accesstoken)\s*[:=]\s*["']([^"']{16,})["']"#, "hardcoded"),
            (r#"(?i)(auth[_-]?token|authtoken)\s*[:=]\s*["']([^"']{16,})["']"#, "hardcoded"),
            (r#"(?i)(bearer[_-]?token|bearertoken)\s*[:=]\s*["']([^"']{16,})["']"#, "hardcoded"),
            // AWS patterns - very specific
            (r#"(?i)(aws[_-]?access[_-]?key[_-]?id|aws_access_key_id)\s*[:=]\s*["'](AKIA[0-9A-Z]{16})["']"#, "hardcoded"),
            (r#"(?i)(aws[_-]?secret[_-]?access[_-]?key|aws_secret_access_key)\s*[:=]\s*["']([^"']{20,})["']"#, "hardcoded"),
            // GitHub patterns - specific token format
            (r#"(?i)(github[_-]?token)\s*[:=]\s*["'](ghp_[a-zA-Z0-9]{36})["']"#, "hardcoded"),
            (r#"(?i)(ghp_[a-zA-Z0-9]{36})"#, "hardcoded"),
            // Firebase patterns
            (r#"(?i)(firebase[_-]?api[_-]?key|firebase_api_key)\s*[:=]\s*["']([A-Za-z0-9_-]{20,})["']"#, "hardcoded"),
            // Stripe patterns
            (r#"(?i)(stripe[_-]?[a-z]*[_-]?key|stripe[_-]?secret)\s*[:=]\s*["'](sk_[a-z0-9_]{20,})["']"#, "hardcoded"),
            // Generic token patterns - require longer values and exclude common false positives
            (r#"(?i)(private[_-]?key|public[_-]?key|encryption[_-]?key)\s*[:=]\s*["']([^"']{20,})["']"#, "hardcoded"),
        ];

        // Environment variable patterns
        let env_patterns = vec![
            (r#"(?i)process\.env\.([A-Z_][A-Z0-9_]*)"#, "env_var"),
            (r#"(?i)os\.getenv\(["']([^"']+)["']"#, "env_var"),
            (r#"(?i)os\.environ\[["']([^"']+)["']"#, "env_var"),
            (r#"(?i)env\[["']([^"']+)["']"#, "env_var"),
            (r#"(?i)\$\{([A-Z_][A-Z0-9_]*)\}"#, "env_var"),
            (r#"(?i)System\.getenv\(["']([^"']+)["']"#, "env_var"),
        ];

        // Scan for hardcoded keys
        for (pattern, key_type) in key_patterns {
            if let Ok(re) = Regex::new(pattern) {
                for (line_num, line) in lines.iter().enumerate() {
                    for cap in re.captures_iter(line) {
                        let key_name = cap.get(1).map(|m| m.as_str().to_string())
                            .unwrap_or_else(|| "unknown".to_string());
                        let value = cap.get(2).map(|m| m.as_str().to_string());
                        
                        // Skip if this looks like a false positive (e.g., "token" in comments or strings)
                        if self.is_false_positive(line, &key_name, value.as_deref()) {
                            continue;
                        }
                        
                        // For generic "apiKey" patterns, require the value to look like an actual API key
                        let key_lower = key_name.to_lowercase();
                        if (key_lower == "apikey" || key_lower == "api_key") && value.is_some() {
                            if !self.looks_like_api_key(value.as_ref().unwrap()) {
                                continue; // Skip if value doesn't look like a real API key
                            }
                        }
                        
                        // Determine provider based on key name
                        let provider = self.determine_provider(&key_name, value.as_deref());
                        
                        // Get context (surrounding lines)
                        let context_start = if line_num > 2 { line_num - 2 } else { 0 };
                        let context_end = if line_num + 3 < lines.len() { line_num + 3 } else { lines.len() };
                        let context = lines[context_start..context_end].join("\n");

                        let value_preview = value.as_ref().map(|v| {
                            if v.len() > 20 {
                                format!("{}...", &v[..20])
                            } else {
                                v.clone()
                            }
                        });

                        // Find code elements in this file that might use this key
                        let used_by = self.find_related_code_elements(
                            line_num + 1,
                            file_elements,
                        );

                            keys.push(DetectedApiKey {
                                name: key_name.clone(),
                                key_type: key_type.to_string(),
                                provider: provider.clone(),
                                value_preview,
                                file_path: normalized_path.to_string(),
                                line_number: Some(line_num + 1),
                                context,
                                used_by,
                                service_ids: Vec::new(),
                            });
                    }
                }
            }
        }

        // Scan for environment variable references
        for (pattern, key_type) in env_patterns {
            if let Ok(re) = Regex::new(pattern) {
                for (line_num, line) in lines.iter().enumerate() {
                    for cap in re.captures_iter(line) {
                        let var_name = cap.get(1).map(|m| m.as_str().to_string())
                            .unwrap_or_else(|| "unknown".to_string());
                        
                        // Only track if it looks like an API key variable
                        if self.is_api_key_variable(&var_name) {
                            let provider = self.determine_provider(&var_name, None);
                            
                            // Get context
                            let context_start = if line_num > 2 { line_num - 2 } else { 0 };
                            let context_end = if line_num + 3 < lines.len() { line_num + 3 } else { lines.len() };
                            let context = lines[context_start..context_end].join("\n");

                            // Find code elements that use this env var
                            let used_by = self.find_related_code_elements(
                                line_num + 1,
                                file_elements,
                            );

                            keys.push(DetectedApiKey {
                                name: var_name.clone(),
                                key_type: key_type.to_string(),
                                provider: provider.clone(),
                                value_preview: None,
                                file_path: normalized_path.to_string(),
                                line_number: Some(line_num + 1),
                                context,
                                used_by,
                                service_ids: Vec::new(),
                            });
                        }
                    }
                }
            }
        }

        Ok(keys)
    }

    /// Find code elements that might use an API key (within 50 lines)
    fn find_related_code_elements(
        &self,
        key_line: usize,
        file_elements: Option<&Vec<&CodeElement>>,
    ) -> Vec<String> {
        let mut related = Vec::new();
        
        if let Some(elements) = file_elements {
            for element in elements {
                // If code element is within 50 lines of the key, consider it related
                if element.line_number >= key_line.saturating_sub(50) && 
                   element.line_number <= key_line + 50 {
                    related.push(element.id.clone());
                }
            }
        }
        
        related
    }

    /// Check if a value looks like an actual API key (not a config value, URL, etc.)
    fn looks_like_api_key(&self, value: &str) -> bool {
        let value_trimmed = value.trim().trim_matches(|c| c == '"' || c == '\'' || c == '`');
        
        // Skip if it's a template literal or variable reference (not a hardcoded value)
        if value_trimmed.contains("${") || 
           value_trimmed.contains("$(") ||
           value_trimmed.contains("{{") ||
           value_trimmed.starts_with("$") ||
           value_trimmed.contains("process.env") ||
           value_trimmed.contains("process['env']") ||
           value_trimmed.contains("process[\"env\"]") ||
           value_trimmed.contains("getenv") ||
           value_trimmed.contains("environ") ||
           value_trimmed.contains("config.") ||
           value_trimmed.contains("env.") ||
           value_trimmed.contains("process.") ||
           // Property access patterns (e.g., firebaseConfig.apiKey, config.apiKey)
           (value_trimmed.contains(".") && value_trimmed.matches(".").count() == 1 && 
            value_trimmed.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '_' || c == '$' || c == '{' || c == '}')) {
            return false; // This is a variable reference, not a hardcoded key
        }
        
        // API keys are typically:
        // - At least 20 characters (we already require 16+ in pattern, but be stricter here)
        if value_trimmed.len() < 20 {
            return false;
        }
        
        // - Don't contain URLs or paths
        if value_trimmed.contains("http://") || 
           value_trimmed.contains("https://") ||
           value_trimmed.contains("://") ||
           (value_trimmed.contains("/") && value_trimmed.matches("/").count() > 1) ||
           value_trimmed.contains("\\") {
            return false;
        }
        
        // - Don't contain spaces (unless it's a multi-part key with specific format)
        if value_trimmed.contains(" ") && !value_trimmed.starts_with("sk_") && !value_trimmed.starts_with("pk_") {
            return false;
        }
        
        // - Don't look like file paths (multiple dots, slashes)
        if value_trimmed.matches(".").count() > 2 && !value_trimmed.starts_with("sk_") {
            return false;
        }
        
        // - Contain mostly alphanumeric characters (some special chars OK like _, -, =)
        let alphanumeric_count = value_trimmed.chars().filter(|c| c.is_alphanumeric()).count();
        let total_chars = value_trimmed.chars().count();
        if total_chars > 0 && (alphanumeric_count as f64 / total_chars as f64) < 0.7 {
            return false; // Too many special characters, probably not an API key
        }
        
        true
    }

    /// Check if a match is likely a false positive
    fn is_false_positive(&self, line: &str, key_name: &str, value: Option<&str>) -> bool {
        let line_lower = line.to_lowercase();
        let key_lower = key_name.to_lowercase();
        
        // Skip if in comments
        if line_lower.trim_start().starts_with("//") ||
           line_lower.trim_start().starts_with("#") ||
           line_lower.contains("/*") ||
           line_lower.contains("*/") ||
           line_lower.contains("<!--") {
            return true;
        }
        
        // Skip common false positives
        if key_lower == "key" && !line_lower.contains("api") && !line_lower.contains("secret") {
            return true; // Just "key" without context is likely false positive
        }
        
        // Skip if it's a type definition or interface
        if line_lower.contains("interface") || 
           line_lower.contains("type ") && line_lower.contains("=") ||
           line_lower.contains("typedef") {
            return true;
        }
        
        // Skip if it's a function parameter without assignment
        if line_lower.contains("function") && line_lower.contains(&key_lower) && !line_lower.contains("=") {
            return true;
        }
        
        // Skip if "token" appears in common non-key contexts
        if key_lower.contains("token") {
            if line_lower.contains("jsonwebtoken") ||
               line_lower.contains("jwt") ||
               line_lower.contains("csrf") ||
               line_lower.contains("refresh") ||
               line_lower.contains("session") {
                return true;
            }
        }
        
        // Skip if it's clearly a documentation example
        if line_lower.contains("example") ||
           line_lower.contains("sample") ||
           line_lower.contains("placeholder") ||
           line_lower.contains("your_") ||
           line_lower.contains("replace_") {
            return true;
        }
        
        // Additional checks for generic "apiKey" patterns - now handled by looks_like_api_key above
        
        false
    }

    /// Determine provider from key name
    fn determine_provider(&self, key_name: &str, value: Option<&str>) -> String {
        let name_lower = key_name.to_lowercase();
        
        // AI Services (check first for specificity)
        if name_lower.contains("anthropic") || name_lower.contains("claude") {
            "anthropic".to_string()
        } else if name_lower.contains("openai") || name_lower.contains("gpt") {
            "openai".to_string()
        } else if name_lower.contains("copilot") {
            "github_copilot".to_string()
        } else if name_lower.contains("google_ai") || name_lower.contains("gemini") {
            "google_ai".to_string()
        } else if name_lower.contains("cohere") {
            "cohere".to_string()
        } else if name_lower.contains("huggingface") || name_lower.contains("hf_") {
            "huggingface".to_string()
        } else if name_lower.contains("replicate") {
            "replicate".to_string()
        } else if name_lower.contains("together") {
            "together_ai".to_string()
        } else if name_lower.contains("mistral") {
            "mistral_ai".to_string()
        } else if name_lower.contains("perplexity") {
            "perplexity".to_string()
        }
        // Cloud Providers
        else if name_lower.contains("aws") || name_lower.contains("amazon") {
            "aws".to_string()
        } else if name_lower.contains("github") || name_lower.contains("ghp_") {
            "github".to_string()
        } else if name_lower.contains("firebase") {
            "firebase".to_string()
        } else if name_lower.contains("google") || name_lower.contains("gcp") {
            "gcp".to_string()
        } else if name_lower.contains("azure") {
            "azure".to_string()
        } else if name_lower.contains("stripe") {
            "stripe".to_string()
        } else if name_lower.contains("twilio") {
            "twilio".to_string()
        } else if name_lower.contains("sendgrid") {
            "sendgrid".to_string()
        } else if name_lower.contains("mailgun") {
            "mailgun".to_string()
        } else if name_lower.contains("vercel") {
            "vercel".to_string()
        } else if name_lower.contains("netlify") {
            "netlify".to_string()
        } else if let Some(val) = value {
            // Check value patterns
            if val.starts_with("ghp_") {
                "github".to_string()
            } else if val.starts_with("sk_live_") || val.starts_with("sk_test_") {
                "stripe".to_string()
            } else if val.starts_with("AKIA") {
                "aws".to_string()
            } else {
                "generic".to_string()
            }
        } else {
            "generic".to_string()
        }
    }

    /// Check if a variable name looks like an API key variable
    fn is_api_key_variable(&self, var_name: &str) -> bool {
        let name_lower = var_name.to_lowercase();
        name_lower.contains("api") && name_lower.contains("key") ||
        name_lower.contains("secret") ||
        name_lower.contains("token") ||
        name_lower.contains("auth") ||
        name_lower.contains("credential") ||
        name_lower.contains("password") ||
        name_lower.contains("access")
    }
}


