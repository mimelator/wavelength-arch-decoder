use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use crate::api::{ApiState, ErrorResponse};

// Get tools for a repository
pub async fn get_tools(
    state: web::Data<ApiState>,
    path: web::Path<String>,
) -> impl Responder {
    // API key validation removed for local tool simplicity
    let repository_id = path.into_inner();
    
    match state.tool_repo.get_tools_by_repository(&repository_id) {
        Ok(tools) => HttpResponse::Ok().json(tools),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: e.to_string(),
        }),
    }
}

// Get scripts for a specific tool
pub async fn get_tool_scripts(
    state: web::Data<ApiState>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    // API key validation removed for local tool simplicity
    let (repository_id, tool_id) = path.into_inner();
    
    match state.tool_repo.get_tool_scripts(&repository_id, &tool_id) {
        Ok(scripts) => HttpResponse::Ok().json(scripts),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: e.to_string(),
        }),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolSearchQuery {
    pub category: Option<String>,
    pub tool_type: Option<String>,
    pub name: Option<String>,
}

// Search tools across repositories
pub async fn search_tools(
    state: web::Data<ApiState>,
    query: web::Query<ToolSearchQuery>,
) -> impl Responder {
    // API key validation removed for local tool simplicity
    // For now, return all tools (can be enhanced with filtering)
    // This would require a new method in ToolRepository to search across repositories
    HttpResponse::Ok().json(serde_json::json!({
        "message": "Tool search not yet implemented",
        "query": query.into_inner()
    }))
}

