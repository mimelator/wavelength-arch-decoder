use actix_web::{web, HttpResponse, Responder, HttpRequest};
use crate::api::{ApiState, ErrorResponse};

/// Get security entities for a repository
pub async fn get_security_entities(
    state: web::Data<ApiState>,
    _req: HttpRequest,
    path: web::Path<String>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    // API key validation removed for local tool simplicity
    let repository_id = path.into_inner();
    
    // Check if filtering by type
    if let Some(entity_type) = query.get("type") {
        match state.security_repo.get_by_type(&repository_id, entity_type) {
            Ok(entities) => HttpResponse::Ok().json(entities),
            Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
                error: e.to_string(),
            }),
        }
    } else {
        match state.security_repo.get_entities(&repository_id) {
            Ok(entities) => HttpResponse::Ok().json(entities),
            Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
                error: e.to_string(),
            }),
        }
    }
}

/// Get security relationships for a repository
pub async fn get_security_relationships(
    state: web::Data<ApiState>,
    _req: HttpRequest,
    path: web::Path<String>,
) -> impl Responder {
    // API key validation removed for local tool simplicity
    let repository_id = path.into_inner();
    
    match state.security_repo.get_relationships(&repository_id) {
        Ok(relationships) => HttpResponse::Ok().json(relationships),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: e.to_string(),
        }),
    }
}

/// Get security vulnerabilities for a repository
pub async fn get_security_vulnerabilities(
    state: web::Data<ApiState>,
    _req: HttpRequest,
    path: web::Path<String>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    // API key validation removed for local tool simplicity
    let repository_id = path.into_inner();
    
    // Check if filtering by severity
    if let Some(severity) = query.get("severity") {
        match state.security_repo.get_vulnerabilities_by_severity(&repository_id, severity) {
            Ok(vulnerabilities) => HttpResponse::Ok().json(vulnerabilities),
            Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
                error: e.to_string(),
            }),
        }
    } else {
        match state.security_repo.get_vulnerabilities(&repository_id) {
            Ok(vulnerabilities) => HttpResponse::Ok().json(vulnerabilities),
            Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
                error: e.to_string(),
            }),
        }
    }
}

