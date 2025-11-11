use actix_web::{web, HttpResponse, Responder, HttpRequest};
use crate::api::{ApiState, ErrorResponse};
use crate::api::extract_api_key;

// Service endpoints
pub async fn get_services(
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

    match state.service_repo.get_by_repository(&path.into_inner()) {
        Ok(services) => HttpResponse::Ok().json(services),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: e.to_string(),
        }),
    }
}

pub async fn search_services_by_provider(
    state: web::Data<ApiState>,
    req: HttpRequest,
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

    if let Some(provider) = query.get("provider") {
        match state.service_repo.get_by_provider(provider) {
            Ok(services) => HttpResponse::Ok().json(services),
            Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
                error: e.to_string(),
            }),
        }
    } else if let Some(service_type) = query.get("type") {
        match state.service_repo.get_by_service_type(service_type) {
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

