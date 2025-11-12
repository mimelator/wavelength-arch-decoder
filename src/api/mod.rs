use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use crate::storage::{RepositoryRepository, DependencyRepository, ServiceRepository, CodeElementRepository, CodeRelationshipRepository, SecurityRepository, ToolRepository};
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

pub struct ApiState {
    pub repo_repo: RepositoryRepository,
    pub dep_repo: DependencyRepository,
    pub service_repo: ServiceRepository,
    pub code_repo: CodeElementRepository,
    pub code_relationship_repo: CodeRelationshipRepository,
    pub security_repo: SecurityRepository,
    pub tool_repo: ToolRepository,
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
    let version = match std::fs::read_to_string("VERSION") {
        Ok(v) => v.trim().to_string(),
        Err(_) => "0.1.0".to_string(),
    };
    HttpResponse::Ok().json(serde_json::json!({
        "version": version
    }))
}

