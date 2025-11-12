use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use walkdir::WalkDir;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationFile {
    pub id: String,
    pub repository_id: String,
    pub file_path: String,
    pub file_name: String,
    pub doc_type: DocumentationType,
    pub title: Option<String>,
    pub description: Option<String>,
    pub content_preview: String, // First 500 chars
    pub word_count: usize,
    pub line_count: usize,
    pub has_code_examples: bool,
    pub has_api_references: bool,
    pub has_diagrams: bool,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DocumentationType {
    Readme,
    Contributing,
    Changelog,
    License,
    ApiDocs,
    Architecture,
    Setup,
    Tutorial,
    Reference,
    Other,
}

impl DocumentationType {
    pub fn from_path(path: &Path) -> Self {
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        let path_str = path.to_string_lossy().to_lowercase();
        
        // Check file name patterns
        if file_name == "readme.md" || file_name == "readme.txt" || file_name == "readme.rst" {
            return DocumentationType::Readme;
        }
        if file_name.contains("contributing") {
            return DocumentationType::Contributing;
        }
        if file_name.contains("changelog") || file_name.contains("changes") {
            return DocumentationType::Changelog;
        }
        if file_name == "license" || file_name == "licence" {
            return DocumentationType::License;
        }
        // Only mark as ApiDocs if it's explicitly an API documentation file
        // (e.g., api.md, api-docs.txt, not api.js or files in /api/ directories)
        if (file_name == "api" || file_name.starts_with("api.") || file_name.contains("api-docs") || file_name.contains("api_docs")) 
            && (file_name.ends_with(".md") || file_name.ends_with(".txt") || file_name.ends_with(".rst") || !file_name.contains(".")) {
            return DocumentationType::ApiDocs;
        }
        if file_name.contains("architecture") || file_name.contains("arch") {
            return DocumentationType::Architecture;
        }
        if file_name.contains("setup") || file_name.contains("install") || file_name.contains("getting-started") {
            return DocumentationType::Setup;
        }
        if file_name.contains("tutorial") || file_name.contains("guide") || file_name.contains("howto") {
            return DocumentationType::Tutorial;
        }
        if path_str.contains("/docs/") || path_str.contains("/documentation/") {
            return DocumentationType::Reference;
        }
        
        DocumentationType::Other
    }
}

pub struct DocumentationIndexer;

impl DocumentationIndexer {
    pub fn new() -> Self {
        DocumentationIndexer
    }

    pub fn index_repository(&self, repo_path: &Path, repository_id: &str) -> Result<Vec<DocumentationFile>> {
        let mut docs = Vec::new();
        let mut seen_paths = std::collections::HashSet::new(); // Track seen file paths to prevent duplicates
        
        // Documentation file extensions
        let doc_extensions = ["md", "txt", "rst", "adoc", "org", "wiki"];
        
        // Common documentation file names
        let doc_names = [
            "readme", "readme.md", "readme.txt", "readme.rst",
            "contributing", "changelog", "changes", "license", "licence",
            "api", "architecture", "arch", "setup", "install", "getting-started",
            "tutorial", "guide", "howto", "docs"
        ];
        
        log::info!("Indexing documentation files in repository...");
        
        for entry in WalkDir::new(repo_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            let file_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            
            let file_name_lower = file_name.to_lowercase();
            let extension = path.extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase());
            
            // Skip hidden files and common ignore patterns
            let path_str = path.to_string_lossy().to_lowercase();
            if file_name.starts_with('.') ||
               path_str.contains("node_modules") ||
               path_str.contains("target") ||
               path_str.contains(".git") ||
               path_str.contains("venv") ||
               path_str.contains("__pycache__") ||
               path_str.contains("/dist/") ||
               path_str.contains("/build/") ||
               path_str.contains("/.next/") ||
               path_str.contains("/out/") ||
               path_str.contains("/.nuxt/") ||
               path_str.contains("/.cache/") ||
               path_str.contains("/coverage/") ||
               file_name.ends_with(".min.js") ||
               file_name.ends_with(".bundle.js") ||
               file_name.ends_with(".chunk.js") {
                continue;
            }
            
            // STRICT: Only index files with documentation extensions OR exact doc names without extensions
            // This ensures we never index code files, even if they have "api" or "readme" in the name
            
            let has_doc_extension = extension.as_ref().map(|e| doc_extensions.contains(&e.as_str())).unwrap_or(false);
            
            // Check for exact doc names - ONLY if file has NO extension
            let is_exact_doc_name = extension.is_none() && doc_names.iter().any(|name| file_name_lower == *name);
            
            // Only proceed if it has a doc extension OR is an exact doc name without extension
            if !has_doc_extension && !is_exact_doc_name {
                continue;
            }
            
            // Final safety: if file has ANY extension that's not a doc extension, skip it
            if let Some(ext) = &extension {
                if !doc_extensions.contains(&ext.as_str()) {
                    continue;
                }
            }
            
            // Read and analyze the file
            if let Ok(content) = std::fs::read_to_string(path) {
                // Check if content appears to be minified/compiled code
                if is_minified_or_compiled(&content) {
                    continue;
                }
                let normalized_path = normalize_path(path, repo_path);
                
                // Deduplicate by normalized path (case-insensitive)
                let path_key = normalized_path.to_lowercase();
                if seen_paths.contains(&path_key) {
                    log::debug!("Skipping duplicate documentation file: {}", normalized_path);
                    continue;
                }
                seen_paths.insert(path_key);
                
                let doc = self.analyze_documentation_file(
                    path,
                    &normalized_path,
                    &content,
                    repository_id,
                )?;
                docs.push(doc);
            }
        }
        
        log::info!("✓ Indexed {} unique documentation files", docs.len());
        Ok(docs)
    }

    fn analyze_documentation_file(
        &self,
        path: &Path,
        normalized_path: &str,
        content: &str,
        repository_id: &str,
    ) -> Result<DocumentationFile> {
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        let doc_type = DocumentationType::from_path(path);
        
        // Extract title (first heading or first line)
        let title = extract_title(content);
        
        // Extract description (first paragraph or first few lines)
        let description = extract_description(content);
        
        // Create preview (first 500 chars)
        let content_preview = content.chars().take(500).collect::<String>();
        
        // Count words and lines
        let word_count = content.split_whitespace().count();
        let line_count = content.lines().count();
        
        // Detect features
        let has_code_examples = detect_code_examples(content);
        let has_api_references = detect_api_references(content);
        let has_diagrams = detect_diagrams(content);
        
        // Extract metadata
        let metadata = extract_metadata(content, &doc_type);
        
        Ok(DocumentationFile {
            id: Uuid::new_v4().to_string(),
            repository_id: repository_id.to_string(),
            file_path: normalized_path.to_string(),
            file_name,
            doc_type,
            title,
            description,
            content_preview,
            word_count,
            line_count,
            has_code_examples,
            has_api_references,
            has_diagrams,
            metadata,
        })
    }
}

fn normalize_path(path: &Path, repo_path: &Path) -> String {
    path.strip_prefix(repo_path)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn extract_title(content: &str) -> Option<String> {
    // Try to find first markdown heading
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("# ") {
            return Some(trimmed[2..].trim().to_string());
        }
        if trimmed.starts_with("## ") {
            return Some(trimmed[3..].trim().to_string());
        }
    }
    
    // Fall back to first non-empty line
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() && trimmed.len() < 100 {
            return Some(trimmed.to_string());
        }
    }
    
    None
}

fn extract_description(content: &str) -> Option<String> {
    // Find first paragraph (non-heading, non-code block)
    let mut in_code_block = false;
    for line in content.lines() {
        let trimmed = line.trim();
        
        // Track code blocks
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        
        if in_code_block {
            continue;
        }
        
        // Skip headings and empty lines
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }
        
        // Found a paragraph
        if trimmed.len() > 20 && trimmed.len() < 300 {
            return Some(trimmed.to_string());
        }
    }
    
    None
}

fn detect_code_examples(content: &str) -> bool {
    // Check for code blocks
    content.contains("```") || 
    content.contains("```") ||
    content.contains("code::") ||
    content.contains(".. code-block::")
}

fn detect_api_references(content: &str) -> bool {
    // Check for API-related keywords
    let api_keywords = [
        "api", "endpoint", "route", "method", "request", "response",
        "curl", "http", "rest", "graphql", "openapi", "swagger"
    ];
    
    let content_lower = content.to_lowercase();
    api_keywords.iter().any(|keyword| content_lower.contains(keyword))
}

fn detect_diagrams(content: &str) -> bool {
    // Check for diagram syntax
    content.contains("```mermaid") ||
    content.contains("```plantuml") ||
    content.contains("```graphviz") ||
    content.contains("```dot") ||
    content.contains(".. graphviz::") ||
    content.contains(".. mermaid::") ||
    content.contains("![") && (content.contains("diagram") || content.contains("arch"))
}

fn extract_metadata(content: &str, doc_type: &DocumentationType) -> serde_json::Value {
    let mut metadata = serde_json::json!({});
    
    // Extract frontmatter if present (YAML frontmatter)
    if content.starts_with("---") {
        if let Some(end) = content[3..].find("---") {
            let frontmatter = &content[3..end + 3];
            // Simple extraction - could be enhanced
            if frontmatter.contains("title:") || frontmatter.contains("description:") {
                metadata["has_frontmatter"] = serde_json::Value::Bool(true);
            }
        }
    }
    
    // Count sections (headings)
    let heading_count = content.lines()
        .filter(|l| l.trim().starts_with('#'))
        .count();
    metadata["heading_count"] = serde_json::Value::Number(serde_json::Number::from(heading_count));
    
    // Detect language if README
    if *doc_type == DocumentationType::Readme {
        let content_lower = content.to_lowercase();
        if content_lower.contains("rust") || content_lower.contains("cargo") {
            metadata["mentions_rust"] = serde_json::Value::Bool(true);
        }
        if content_lower.contains("python") || content_lower.contains("pip") {
            metadata["mentions_python"] = serde_json::Value::Bool(true);
        }
        if content_lower.contains("javascript") || content_lower.contains("node") || content_lower.contains("npm") {
            metadata["mentions_javascript"] = serde_json::Value::Bool(true);
        }
    }
    
    metadata
}

/// Check if content appears to be minified or compiled code
fn is_minified_or_compiled(content: &str) -> bool {
    if content.is_empty() {
        return false;
    }
    
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return false;
    }
    
    // Check for very long average line length (minified code has long lines)
    let total_chars: usize = lines.iter().map(|l| l.len()).sum();
    let avg_line_length = total_chars as f64 / lines.len() as f64;
    if avg_line_length > 200.0 {
        return true;
    }
    
    // Check for very few line breaks relative to content size
    let line_break_ratio = lines.len() as f64 / content.len() as f64;
    if line_break_ratio < 0.01 && content.len() > 1000 {
        return true;
    }
    
    // Check for high ratio of non-whitespace characters (minified code has no spaces)
    let non_whitespace: usize = content.chars().filter(|c| !c.is_whitespace()).count();
    let whitespace_ratio = 1.0 - (non_whitespace as f64 / content.len() as f64);
    if whitespace_ratio < 0.1 && content.len() > 500 {
        return true;
    }
    
    // Check for common minified patterns
    if content.contains("function(") && content.contains("=>") && whitespace_ratio < 0.15 {
        return true;
    }
    
    // Check for source map comments (indicates compiled/minified code)
    if content.contains("//# sourceMappingURL=") || content.contains("//@ sourceMappingURL=") {
        return true;
    }
    
    false
}

