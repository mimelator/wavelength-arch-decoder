use actix_web::{web, HttpResponse, Responder, HttpRequest};
use crate::api::{ApiState, ErrorResponse};

/// Get code elements for a repository
pub async fn get_code_elements(
    state: web::Data<ApiState>,
    _req: HttpRequest,
    path: web::Path<String>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    // API key validation removed for local tool simplicity
    let repository_id = path.into_inner();
    
    // Check if filtering by type
    if let Some(element_type) = query.get("type") {
        match state.code_repo.get_by_type(&repository_id, element_type) {
            Ok(elements) => HttpResponse::Ok().json(elements),
            Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
                error: e.to_string(),
            }),
        }
    } else {
        match state.code_repo.get_by_repository(&repository_id) {
            Ok(elements) => HttpResponse::Ok().json(elements),
            Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
                error: e.to_string(),
            }),
        }
    }
}

/// Get code calls for a repository
pub async fn get_code_calls(
    state: web::Data<ApiState>,
    _req: HttpRequest,
    path: web::Path<String>,
) -> impl Responder {
    // API key validation removed for local tool simplicity
    let repository_id = path.into_inner();
    
    match state.code_repo.get_calls(&repository_id) {
        Ok(calls) => HttpResponse::Ok().json(calls),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: e.to_string(),
        }),
    }
}

/// Get code relationships for a repository
pub async fn get_code_relationships(
    state: web::Data<ApiState>,
    _req: HttpRequest,
    path: web::Path<String>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    // API key validation removed for local tool simplicity
    let repository_id = path.into_inner();
    
    // Check if filtering by code element
    if let Some(code_element_id) = query.get("code_element_id") {
        match state.code_relationship_repo.get_by_code_element(code_element_id) {
            Ok(relationships) => HttpResponse::Ok().json(relationships),
            Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
                error: e.to_string(),
            }),
        }
    } else if let Some(target_type) = query.get("target_type") {
        if let Some(target_id) = query.get("target_id") {
            match state.code_relationship_repo.get_by_target(target_type, target_id) {
                Ok(relationships) => HttpResponse::Ok().json(relationships),
                Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
                    error: e.to_string(),
                }),
            }
        } else {
            HttpResponse::BadRequest().json(ErrorResponse {
                error: "target_id required when target_type is specified".to_string(),
            })
        }
    } else {
        HttpResponse::BadRequest().json(ErrorResponse {
            error: "Must specify code_element_id or target_type/target_id".to_string(),
        })
    }
}

