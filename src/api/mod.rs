use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use crate::storage::{RepositoryRepository, DependencyRepository, ServiceRepository, CodeElementRepository, CodeRelationshipRepository, SecurityRepository, ToolRepository, DocumentationRepository, TestRepository};
use std::sync::Arc;

pub mod server;
pub mod repositories;
pub mod services;
pub mod tools;
pub mod graph;
pub mod code;
pub mod security;
pub mod jobs;
pub mod entity_details;
pub mod progress;
pub mod reports;
pub mod documentation;
pub mod tests;
pub mod version_check;
pub mod plugins;

pub struct ApiState {
    pub repo_repo: RepositoryRepository,
    pub dep_repo: DependencyRepository,
    pub service_repo: ServiceRepository,
    pub code_repo: CodeElementRepository,
    pub code_relationship_repo: CodeRelationshipRepository,
    pub security_repo: SecurityRepository,
    pub tool_repo: ToolRepository,
    pub documentation_repo: DocumentationRepository,
    pub test_repo: TestRepository,
    pub progress_tracker: Arc<progress::ProgressTracker>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

// Health check endpoint
pub async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "wavelength-arch-decoder"
    }))
}

// Version endpoint
pub async fn version(query: web::Query<std::collections::HashMap<String, String>>) -> impl Responder {
    use crate::api::version_check;
    
    // Get current version
    let current_version = version_check::get_current_version();
    
    // Get editor protocol from environment (default: vscode)
    let editor_protocol = std::env::var("EDITOR_PROTOCOL")
        .unwrap_or_else(|_| "vscode".to_string());
    
    // Check if force refresh is requested
    let force = query.get("force").and_then(|v| v.parse::<bool>().ok()).unwrap_or(false);
    
    // Check for updates (non-blocking, uses cache unless forced)
    let version_info = version_check::check_for_updates(force).await;
    
    // Get loaded plugins
    let plugin_dir = std::path::Path::new("config/plugins");
    let mut plugin_names = Vec::new();
    if plugin_dir.exists() && plugin_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(plugin_dir) {
            plugin_names = entries.filter_map(|e| e.ok())
                .filter(|e| {
                    e.path().is_file() && 
                    e.path().extension().and_then(|s| s.to_str()) == Some("json")
                })
                .filter_map(|e| {
                    e.path().file_stem()
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_string())
                })
                .collect();
            plugin_names.sort();
        }
    }
    
    log::debug!("Version endpoint returning: {}, editor_protocol: {}, force: {}, plugins: {:?}", current_version, editor_protocol, force, plugin_names);
    HttpResponse::Ok().json(serde_json::json!({
        "version": current_version,
        "editor_protocol": editor_protocol,
        "latest_version": version_info.latest,
        "update_available": version_info.update_available,
        "last_checked": version_info.last_checked,
        "plugins": plugin_names
    }))
}

