use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::fs;
use uuid::Uuid;
use crate::analysis::{CodeElement, CodeStructure};
use crate::storage::{StoredDependency, StoredService};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeRelationship {
    pub id: String,
    pub code_element_id: String,
    pub target_type: RelationshipTargetType,
    pub target_id: String,
    pub relationship_type: String, // "uses", "imports", "calls", "depends_on"
    pub confidence: f64,
    pub evidence: String, // What in the code indicates this relationship
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RelationshipTargetType {
    Service,
    Dependency,
}

pub struct CodeRelationshipDetector {
    repo_path: Box<Path>,
}

impl CodeRelationshipDetector {
    pub fn new(repo_path: &Path) -> Self {
        CodeRelationshipDetector {
            repo_path: repo_path.to_path_buf().into_boxed_path(),
        }
    }

    /// Detect relationships between code elements and services/dependencies
    pub fn detect_relationships(
        &self,
        code_structure: &CodeStructure,
        services: &[StoredService],
        dependencies: &[StoredDependency],
    ) -> Result<Vec<CodeRelationship>> {
        let mut relationships = Vec::new();
        
        // Group code elements by file for efficient analysis
        let mut elements_by_file: HashMap<String, Vec<&CodeElement>> = HashMap::new();
        for element in &code_structure.elements {
            elements_by_file
                .entry(element.file_path.clone())
                .or_insert_with(Vec::new)
                .push(element);
        }

        // Analyze each file to find service/dependency usage
        for (file_path, elements) in &elements_by_file {
            let full_path = self.repo_path.join(file_path);
            if !full_path.exists() {
                continue;
            }

            let content = match fs::read_to_string(&full_path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            // Detect relationships for each code element in this file
            for element in elements {
                // Find service relationships
                let service_rels = self.detect_service_relationships(
                    element,
                    &content,
                    services,
                )?;
                relationships.extend(service_rels);

                // Find dependency relationships
                let dep_rels = self.detect_dependency_relationships(
                    element,
                    &content,
                    dependencies,
                )?;
                relationships.extend(dep_rels);
            }
        }

        Ok(relationships)
    }

    /// Detect relationships between a code element and services
    fn detect_service_relationships(
        &self,
        element: &CodeElement,
        file_content: &str,
        services: &[StoredService],
    ) -> Result<Vec<CodeRelationship>> {
        let mut relationships = Vec::new();
        let _content_lower = file_content.to_lowercase();
        
        // Get the function/class body (approximate by looking around the element's line)
        let element_body = self.extract_element_body(file_content, element.line_number);

        for service in services {
            // Check if service is used in this element's context
            let mut evidence = Vec::new();
            let mut confidence: f64 = 0.0;

            // Check service name in code
            let service_name_lower = service.name.to_lowercase();
            if element_body.contains(&service_name_lower) {
                evidence.push(format!("Service name '{}' found in code", service.name));
                confidence += 0.3;
            }

            // Check provider name
            let provider_lower = service.provider.to_lowercase();
            if element_body.contains(&provider_lower) {
                evidence.push(format!("Provider '{}' found in code", service.provider));
                confidence += 0.2;
            }

            // Check service-specific SDK patterns
            let sdk_patterns = self.get_service_sdk_patterns(service);
            for pattern in &sdk_patterns {
                if self.is_word_boundary_match(&element_body, pattern) {
                    evidence.push(format!("SDK pattern '{}' detected", pattern));
                    confidence += 0.4;
                }
            }

            // Check API endpoints
            if let Some(endpoint) = self.extract_api_endpoint(service) {
                if element_body.contains(&endpoint) {
                    evidence.push(format!("API endpoint '{}' detected", endpoint));
                    confidence += 0.5;
                }
            }

            // Check configuration for service-specific identifiers
            if let Ok(config) = serde_json::from_str::<serde_json::Value>(&service.configuration) {
                if let Some(env_var) = config.get("env_var").and_then(|v| v.as_str()) {
                    if element_body.contains(env_var) {
                        evidence.push(format!("Environment variable '{}' used", env_var));
                        confidence += 0.3;
                    }
                }
            }

            // Only create relationship if we have sufficient evidence
            if confidence >= 0.3 && !evidence.is_empty() {
                relationships.push(CodeRelationship {
                    id: Uuid::new_v4().to_string(),
                    code_element_id: element.id.clone(),
                    target_type: RelationshipTargetType::Service,
                    target_id: service.id.clone(),
                    relationship_type: "uses".to_string(),
                    confidence: confidence.min(1.0_f64),
                    evidence: evidence.join("; "),
                });
            }
        }

        Ok(relationships)
    }

    /// Detect relationships between a code element and dependencies
    fn detect_dependency_relationships(
        &self,
        element: &CodeElement,
        file_content: &str,
        dependencies: &[StoredDependency],
    ) -> Result<Vec<CodeRelationship>> {
        let mut relationships = Vec::new();
        let content_lower = file_content.to_lowercase();
        
        // Get the function/class body
        let element_body = self.extract_element_body(file_content, element.line_number);

        for dep in dependencies {
            let mut evidence = Vec::new();
            let mut confidence: f64 = 0.0;

            // Check for import statements matching dependency name
            let dep_name_lower = dep.name.to_lowercase();
            
            // Match import patterns
            let import_patterns = vec![
                format!("import {}", dep_name_lower),
                format!("from '{}", dep_name_lower),
                format!("from \"{}\"", dep_name_lower),
                format!("require('{}", dep_name_lower),
                format!("require(\"{}\"", dep_name_lower),
                format!("use {}", dep_name_lower),
                format!("extern crate {}", dep_name_lower),
            ];

            for pattern in &import_patterns {
                if content_lower.contains(pattern) {
                    evidence.push(format!("Import statement found: {}", pattern));
                    confidence += 0.6;
                    break;
                }
            }

            // Check for package name in code (word boundary match)
            if self.is_word_boundary_match(&element_body, &dep_name_lower) {
                evidence.push(format!("Package '{}' used in code", dep.name));
                confidence += 0.4;
            }

            // Check for scoped packages (e.g., @firebase/app)
            if dep.name.contains('/') {
                let parts: Vec<&str> = dep.name.split('/').collect();
                for part in &parts {
                    if self.is_word_boundary_match(&element_body, &part.to_lowercase()) {
                        evidence.push(format!("Scoped package part '{}' found", part));
                        confidence += 0.3;
                    }
                }
            }

            // Only create relationship if we have sufficient evidence
            if confidence >= 0.3 && !evidence.is_empty() {
                relationships.push(CodeRelationship {
                    id: Uuid::new_v4().to_string(),
                    code_element_id: element.id.clone(),
                    target_type: RelationshipTargetType::Dependency,
                    target_id: dep.id.clone(),
                    relationship_type: "imports".to_string(),
                    confidence: confidence.min(1.0_f64),
                    evidence: evidence.join("; "),
                });
            }
        }

        Ok(relationships)
    }

    /// Extract the body of a code element (function/class) for analysis
    fn extract_element_body(&self, content: &str, start_line: usize) -> String {
        let lines: Vec<&str> = content.lines().collect();
        if start_line == 0 || start_line > lines.len() {
            return String::new();
        }

        // Get a window around the element (50 lines before and 200 lines after)
        let start = if start_line > 50 {
            start_line - 50
        } else {
            0
        };
        let end = (start_line + 200).min(lines.len());

        lines[start..end].join("\n")
    }

    /// Get SDK patterns for a service
    fn get_service_sdk_patterns(&self, service: &StoredService) -> Vec<String> {
        let mut patterns = Vec::new();
        
        // Common SDK patterns based on provider
        match service.provider.to_lowercase().as_str() {
            "firebase" => {
                patterns.push("firebase".to_string());
                patterns.push("@firebase/".to_string());
                patterns.push("firebase/app".to_string());
                patterns.push("firebase/auth".to_string());
                patterns.push("firebase/firestore".to_string());
                patterns.push("firebase/storage".to_string());
            }
            "aws" => {
                patterns.push("aws-sdk".to_string());
                patterns.push("@aws-sdk/".to_string());
            }
            "stripe" => {
                patterns.push("stripe".to_string());
            }
            "clerk" => {
                patterns.push("@clerk/".to_string());
                patterns.push("clerk".to_string());
            }
            "vercel" => {
                patterns.push("@vercel/".to_string());
            }
            "openai" => {
                patterns.push("openai".to_string());
                patterns.push("@openai/".to_string());
            }
            "anthropic" => {
                patterns.push("anthropic".to_string());
                patterns.push("@anthropic-ai/".to_string());
            }
            _ => {
                // Generic pattern from service name
                patterns.push(service.name.to_lowercase());
            }
        }

        patterns
    }

    /// Extract API endpoint from service configuration
    fn extract_api_endpoint(&self, service: &StoredService) -> Option<String> {
        if let Ok(config) = serde_json::from_str::<serde_json::Value>(&service.configuration) {
            // Check common endpoint fields
            for field in &["endpoint", "api_url", "url", "base_url"] {
                if let Some(url) = config.get(field).and_then(|v| v.as_str()) {
                    return Some(url.to_lowercase());
                }
            }
        }
        None
    }

    /// Check if pattern matches at word boundaries
    fn is_word_boundary_match(&self, content: &str, pattern: &str) -> bool {
        let pattern_lower = pattern.to_lowercase();
        let mut start = 0;
        
        while let Some(pos) = content[start..].to_lowercase().find(&pattern_lower) {
            let actual_pos = start + pos;
            let before = if actual_pos > 0 {
                content.chars().nth(actual_pos - 1)
            } else {
                None
            };
            let after_pos = actual_pos + pattern.len();
            let after = if after_pos < content.len() {
                content.chars().nth(after_pos)
            } else {
                None
            };
            
            let is_boundary = match (before, after) {
                (None, _) | (_, None) => true,
                (Some(b), Some(a)) => {
                    (!b.is_alphanumeric() || b == '_' || b == '-' || b == '/' || b == '.') &&
                    (!a.is_alphanumeric() || a == '_' || a == '-' || a == '/' || a == '.')
                }
            };
            
            if is_boundary {
                return true;
            }
            
            start = actual_pos + 1;
        }
        
        false
    }
}

