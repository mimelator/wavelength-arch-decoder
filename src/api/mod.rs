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
pub async fn version() -> impl Responder {
    use crate::api::version_check;
    
    // Get current version
    let current_version = version_check::get_current_version();
    
    // Get editor protocol from environment (default: vscode)
    let editor_protocol = std::env::var("EDITOR_PROTOCOL")
        .unwrap_or_else(|_| "vscode".to_string());
    
    // Check for updates (non-blocking, uses cache)
    let version_info = version_check::check_for_updates(false).await;
    
    log::debug!("Version endpoint returning: {}, editor_protocol: {}", current_version, editor_protocol);
    HttpResponse::Ok().json(serde_json::json!({
        "version": current_version,
        "editor_protocol": editor_protocol,
        "latest_version": version_info.latest,
        "update_available": version_info.update_available,
        "last_checked": version_info.last_checked
    }))
}

