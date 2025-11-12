use actix_web::{web, HttpResponse, Responder, HttpRequest};
use crate::api::{ApiState, ErrorResponse};

// Service endpoints
pub async fn get_services(
    state: web::Data<ApiState>,
    _req: HttpRequest,
    path: web::Path<String>,
) -> impl Responder {
    // API key validation removed for local tool simplicity
    match state.service_repo.get_by_repository(&path.into_inner()) {
        Ok(services) => HttpResponse::Ok().json(services),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: e.to_string(),
        }),
    }
}

pub async fn search_services_by_provider(
    state: web::Data<ApiState>,
    _req: HttpRequest,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    // API key validation removed for local tool simplicity
    if let Some(provider) = query.get("provider") {
        // Global search - no repository_id filter (intentional cross-repo search)
        match state.service_repo.get_by_provider(provider, None) {
            Ok(services) => HttpResponse::Ok().json(services),
            Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
                error: e.to_string(),
            }),
        }
    } else if let Some(service_type) = query.get("type") {
        // Global search - no repository_id filter (intentional cross-repo search)
        match state.service_repo.get_by_service_type(service_type, None) {
            Ok(services) => HttpResponse::Ok().json(services),
            Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
                error: e.to_string(),
            }),
        }
    } else {
        HttpResponse::BadRequest().json(ErrorResponse {
            error: "Missing 'provider' or 'type' query parameter".to_string(),
        })
    }
}

