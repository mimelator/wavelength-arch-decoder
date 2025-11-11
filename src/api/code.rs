use actix_web::{web, HttpResponse, Responder, HttpRequest};
use crate::api::{ApiState, ErrorResponse};
use crate::api::extract_api_key;

/// Get code elements for a repository
pub async fn get_code_elements(
    state: web::Data<ApiState>,
    req: HttpRequest,
    path: web::Path<String>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    // Validate API key
    let _api_key = match extract_api_key(&req) {
        Ok(key) => key,
        Err(resp) => return resp,
    };

    match state.auth_service.validate_api_key(&_api_key) {
        Ok(_) => {},
        Err(e) => {
            return HttpResponse::Unauthorized().json(ErrorResponse {
                error: e.to_string(),
            });
        }
    }

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
    req: HttpRequest,
    path: web::Path<String>,
) -> impl Responder {
    // Validate API key
    let _api_key = match extract_api_key(&req) {
        Ok(key) => key,
        Err(resp) => return resp,
    };

    match state.auth_service.validate_api_key(&_api_key) {
        Ok(_) => {},
        Err(e) => {
            return HttpResponse::Unauthorized().json(ErrorResponse {
                error: e.to_string(),
            });
        }
    }

    let repository_id = path.into_inner();
    
    match state.code_repo.get_calls(&repository_id) {
        Ok(calls) => HttpResponse::Ok().json(calls),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: e.to_string(),
        }),
    }
}

