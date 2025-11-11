use actix_web::{web, HttpResponse, Responder, HttpRequest};
use serde::{Deserialize, Serialize};
use crate::auth::{AuthService, RegisterRequest, LoginRequest, CreateApiKeyRequest};
use crate::storage::{RepositoryRepository, DependencyRepository, ServiceRepository, CodeElementRepository, SecurityRepository};

pub mod server;
pub mod repositories;
pub mod services;
pub mod graph;
pub mod code;
pub mod security;
pub mod jobs;

pub struct ApiState {
    pub auth_service: AuthService,
    pub repo_repo: RepositoryRepository,
    pub dep_repo: DependencyRepository,
    pub service_repo: ServiceRepository,
    pub code_repo: CodeElementRepository,
    pub security_repo: SecurityRepository,
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

// Auth endpoints
pub async fn register(
    state: web::Data<ApiState>,
    req: web::Json<RegisterRequest>,
) -> impl Responder {
    match state.auth_service.register(req.into_inner()) {
        Ok(api_key) => HttpResponse::Created().json(serde_json::json!({
            "api_key": api_key,
            "message": "User registered successfully"
        })),
        Err(e) => HttpResponse::BadRequest().json(ErrorResponse {
            error: e.to_string(),
        }),
    }
}

pub async fn login(
    state: web::Data<ApiState>,
    req: web::Json<LoginRequest>,
) -> impl Responder {
    match state.auth_service.login(req.into_inner()) {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::Unauthorized().json(ErrorResponse {
            error: e.to_string(),
        }),
    }
}

pub async fn create_api_key(
    state: web::Data<ApiState>,
    req: HttpRequest,
    body: web::Json<CreateApiKeyRequest>,
) -> impl Responder {
    // Extract API key from Authorization header
    let api_key = match extract_api_key(&req) {
        Ok(key) => key,
        Err(resp) => return resp,
    };
    
    // Validate API key
    let key_info = match state.auth_service.validate_api_key(&api_key) {
        Ok(info) => info,
        Err(e) => {
            return HttpResponse::Unauthorized().json(ErrorResponse {
                error: e.to_string(),
            });
        }
    };
    
    // Check if user has admin scope
    if !key_info.scopes.contains(&"admin".to_string()) {
        return HttpResponse::Forbidden().json(ErrorResponse {
            error: "Admin scope required".to_string(),
        });
    }

    match state.auth_service.create_api_key(&key_info.user_id, body.into_inner()) {
        Ok(new_key) => HttpResponse::Created().json(serde_json::json!({
            "api_key": new_key,
            "message": "API key created successfully"
        })),
        Err(e) => HttpResponse::BadRequest().json(ErrorResponse {
            error: e.to_string(),
        }),
    }
}

// Helper function to extract API key from request
pub fn extract_api_key(req: &HttpRequest) -> Result<String, HttpResponse> {
    req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| {
            if s.starts_with("Bearer ") {
                Some(s[7..].to_string())
            } else {
                None
            }
        })
        .ok_or_else(|| {
            HttpResponse::Unauthorized().json(ErrorResponse {
                error: "Missing or invalid Authorization header".to_string(),
            })
        })
}
