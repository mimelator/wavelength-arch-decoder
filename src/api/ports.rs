use actix_web::{web, HttpResponse, Responder, HttpRequest};
use crate::api::{ApiState, ErrorResponse};

// Port endpoints
pub async fn get_ports(
    state: web::Data<ApiState>,
    _req: HttpRequest,
    path: web::Path<String>,
) -> impl Responder {
    match state.port_repo.get_by_repository(&path.into_inner()) {
        Ok(ports) => HttpResponse::Ok().json(ports),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: e.to_string(),
        }),
    }
}

pub async fn search_ports_by_port(
    state: web::Data<ApiState>,
    _req: HttpRequest,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    if let Some(port_str) = query.get("port") {
        if let Ok(port) = port_str.parse::<u16>() {
            let repository_id = query.get("repository_id");
            match state.port_repo.get_by_port(port, repository_id.map(|s| s.as_str())) {
                Ok(ports) => HttpResponse::Ok().json(ports),
                Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
                    error: e.to_string(),
                }),
            }
        } else {
            HttpResponse::BadRequest().json(ErrorResponse {
                error: "Invalid port number".to_string(),
            })
        }
    } else if let Some(port_type) = query.get("type") {
        let repository_id = query.get("repository_id");
        match state.port_repo.get_by_type(port_type, repository_id.map(|s| s.as_str())) {
            Ok(ports) => HttpResponse::Ok().json(ports),
            Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
                error: e.to_string(),
            }),
        }
    } else {
        HttpResponse::BadRequest().json(ErrorResponse {
            error: "Missing 'port' or 'type' query parameter".to_string(),
        })
    }
}

