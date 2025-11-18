use actix_web::{web, HttpResponse, Responder, HttpRequest};
use crate::api::{ApiState, ErrorResponse};

// Endpoint endpoints
pub async fn get_endpoints(
    state: web::Data<ApiState>,
    _req: HttpRequest,
    path: web::Path<String>,
) -> impl Responder {
    match state.endpoint_repo.get_by_repository(&path.into_inner()) {
        Ok(endpoints) => HttpResponse::Ok().json(endpoints),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: e.to_string(),
        }),
    }
}

pub async fn search_endpoints(
    state: web::Data<ApiState>,
    _req: HttpRequest,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    if let Some(path_pattern) = query.get("path") {
        let repository_id = query.get("repository_id");
        match state.endpoint_repo.get_by_path(path_pattern, repository_id.map(|s| s.as_str())) {
            Ok(endpoints) => HttpResponse::Ok().json(endpoints),
            Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
                error: e.to_string(),
            }),
        }
    } else if let Some(method) = query.get("method") {
        let repository_id = query.get("repository_id");
        match state.endpoint_repo.get_by_method(method, repository_id.map(|s| s.as_str())) {
            Ok(endpoints) => HttpResponse::Ok().json(endpoints),
            Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
                error: e.to_string(),
            }),
        }
    } else if let Some(framework) = query.get("framework") {
        let repository_id = query.get("repository_id");
        match state.endpoint_repo.get_by_framework(framework, repository_id.map(|s| s.as_str())) {
            Ok(endpoints) => HttpResponse::Ok().json(endpoints),
            Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
                error: e.to_string(),
            }),
        }
    } else {
        HttpResponse::BadRequest().json(ErrorResponse {
            error: "Missing 'path', 'method', or 'framework' query parameter".to_string(),
        })
    }
}

