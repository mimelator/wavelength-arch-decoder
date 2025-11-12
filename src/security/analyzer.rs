use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use walkdir::WalkDir;
use crate::security::api_key_detector::ApiKeyDetector;
use crate::security::types::SecurityAnalysis;
use crate::security::helpers::{normalize_path, is_cloudformation, is_sam_template};
use crate::security::terraform::analyze_terraform;
use crate::security::cloudformation::analyze_cloudformation;
use crate::security::serverless::{analyze_serverless, analyze_sam};
use crate::security::firebase::analyze_firebase_rules;
use crate::security::env_config::analyze_env_template;
use crate::security::security_config::analyze_security_config;

pub struct SecurityAnalyzer;

impl SecurityAnalyzer {
    pub fn new() -> Self {
        SecurityAnalyzer
    }

    /// Analyze security configuration in a repository
    pub fn analyze_repository(
        &self, 
        repo_path: &Path,
        code_structure: Option<&crate::analysis::CodeStructure>,
        services: Option<&[crate::security::DetectedService]>,
    ) -> Result<SecurityAnalysis> {
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
            let normalized_path = normalize_path(path, repo_path);
            let file_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_lowercase();

            // Skip hidden files and common ignore patterns
            let path_str = path.to_string_lossy().to_lowercase();
            if file_name.starts_with('.') || 
               path_str.contains("node_modules") ||
               path_str.contains("target") ||
               path_str.contains(".git") ||
               path_str.contains("/dist/") ||
               path_str.contains("/build/") ||
               path_str.contains("/.next/") ||
               path_str.contains("\\.next\\") ||  // Windows path separator
               path_str.contains("/out/") ||
               path_str.contains("/.nuxt/") ||
               path_str.contains("/.cache/") ||
               path_str.contains("/coverage/") ||
               path_str.contains("/.next/static/") ||  // Next.js build artifacts
               path_str.contains("/.next/server/") ||  // Next.js server chunks
               file_name.ends_with(".min.js") ||
               file_name.ends_with(".min.css") ||
               file_name.ends_with(".bundle.js") ||
               file_name.ends_with(".chunk.js") ||
               file_name.ends_with(".class") ||
               file_name.ends_with(".pyc") ||
               file_name.ends_with(".pyo") ||
               file_name.ends_with(".so") ||
               file_name.ends_with(".dll") ||
               file_name.ends_with(".dylib") ||
               file_name.ends_with(".a") ||
               file_name.ends_with(".o") ||
               file_name.ends_with(".rlib") {
                continue;
            }

            // Analyze Terraform files
            if file_name.ends_with(".tf") || file_name.ends_with(".tfvars") {
                if let Ok(content) = std::fs::read_to_string(path) {
                    let (tf_entities, tf_relationships, tf_vulns) = 
                        analyze_terraform(&content, path, &normalized_path, &mut entity_map)?;
                    entities.extend(tf_entities);
                    relationships.extend(tf_relationships);
                    vulnerabilities.extend(tf_vulns);
                }
            }

            // Analyze CloudFormation files
            if file_name.ends_with(".yaml") || file_name.ends_with(".yml") {
                if let Ok(content) = std::fs::read_to_string(path) {
                    if is_cloudformation(&content) {
                        let (cf_entities, cf_relationships, cf_vulns) = 
                            analyze_cloudformation(&content, path, &normalized_path, &mut entity_map)?;
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
                        analyze_serverless(&content, path, &normalized_path, &mut entity_map)?;
                    entities.extend(sls_entities);
                    relationships.extend(sls_relationships);
                    vulnerabilities.extend(sls_vulns);
                }
            }

            // Analyze AWS SAM templates
            if file_name.contains("template") && (file_name.ends_with(".yaml") || file_name.ends_with(".yml")) {
                if let Ok(content) = std::fs::read_to_string(path) {
                    if is_sam_template(&content) {
                        let (sam_entities, sam_relationships, sam_vulns) = 
                            analyze_sam(&content, path, &normalized_path, &mut entity_map)?;
                        entities.extend(sam_entities);
                        relationships.extend(sam_relationships);
                        vulnerabilities.extend(sam_vulns);
                    }
                }
            }

            // Analyze Firebase rules files
            if file_name == "firestore.rules" || 
               file_name == "storage.rules" || 
               file_name == "database.rules.json" ||
               file_name.ends_with(".rules") ||
               (file_name.contains("firebase") && file_name.ends_with(".json")) {
                if let Ok(content) = std::fs::read_to_string(path) {
                    let (fb_entities, fb_vulns) = analyze_firebase_rules(&content, path, &normalized_path, &mut entity_map)?;
                    entities.extend(fb_entities);
                    vulnerabilities.extend(fb_vulns);
                }
            }

            // Analyze environment template files
            if file_name == ".env.example" || 
               file_name == ".env.template" || 
               file_name == ".env.sample" ||
               file_name == "env.example" ||
               file_name == "env.template" ||
               file_name == "env.sample" ||
               (file_name.starts_with(".env.") && !file_name.ends_with(".local") && !file_name.ends_with(".secret")) {
                if let Ok(content) = std::fs::read_to_string(path) {
                    let env_entities = analyze_env_template(&content, path, &normalized_path, &mut entity_map)?;
                    entities.extend(env_entities);
                }
            }

            // Analyze other security config files
            if file_name == "security.json" ||
               file_name == "security.yml" ||
               file_name == "security.yaml" ||
               file_name == ".security" ||
               (file_name.contains("security") && (file_name.ends_with(".json") || file_name.ends_with(".yml") || file_name.ends_with(".yaml"))) {
                if let Ok(content) = std::fs::read_to_string(path) {
                    let sec_entities = analyze_security_config(&content, path, &normalized_path, &mut entity_map)?;
                    entities.extend(sec_entities);
                }
            }
        }

        // Detect API keys in code files
        log::info!("Detecting API keys in code files...");
        let api_key_detector = ApiKeyDetector::new();
        let (api_key_entities, api_key_relationships, api_key_vulns) = match api_key_detector.detect_api_keys(repo_path, code_structure, services) {
            Ok(result) => {
                log::info!("✓ API key detection complete: {} keys, {} relationships, {} vulnerabilities", 
                    result.0.len(), result.1.len(), result.2.len());
                result
            },
            Err(e) => {
                log::error!("✗ API key detection failed: {}", e);
                (Vec::new(), Vec::new(), Vec::new())
            }
        };
        entities.extend(api_key_entities);
        relationships.extend(api_key_relationships);
        vulnerabilities.extend(api_key_vulns);

        Ok(SecurityAnalysis {
            entities,
            relationships,
            vulnerabilities,
        })
    }
}
