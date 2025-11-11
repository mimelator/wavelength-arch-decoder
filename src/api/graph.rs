use actix_web::{web, HttpResponse, Responder, HttpRequest};
use crate::api::{ApiState, ErrorResponse};
use crate::api::extract_api_key;
use crate::graph::{GraphBuilder, KnowledgeGraph};

/// Get knowledge graph for a repository
pub async fn get_graph(
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
    let graph_builder = GraphBuilder::new(
        state.repo_repo.db.clone(),
        state.repo_repo.clone(),
        state.dep_repo.clone(),
        state.service_repo.clone(),
    );

    match graph_builder.get_graph(&repository_id) {
        Ok(graph) => HttpResponse::Ok().json(graph),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: e.to_string(),
        }),
    }
}

/// Get graph statistics for a repository
pub async fn get_graph_statistics(
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
    let graph_builder = GraphBuilder::new(
        state.repo_repo.db.clone(),
        state.repo_repo.clone(),
        state.dep_repo.clone(),
        state.service_repo.clone(),
    );

    match graph_builder.get_graph(&repository_id) {
        Ok(graph) => HttpResponse::Ok().json(graph.get_statistics()),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: e.to_string(),
        }),
    }
}

/// Get neighbors of a specific node
pub async fn get_node_neighbors(
    state: web::Data<ApiState>,
    req: HttpRequest,
    path: web::Path<(String, String)>,
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

    let (repository_id, node_id) = path.into_inner();
    let graph_builder = GraphBuilder::new(
        state.repo_repo.db.clone(),
        state.repo_repo.clone(),
        state.dep_repo.clone(),
        state.service_repo.clone(),
    );

    match graph_builder.get_graph(&repository_id) {
        Ok(graph) => {
            let neighbors = graph.get_neighbors(&node_id);
            HttpResponse::Ok().json(neighbors)
        }
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: e.to_string(),
        }),
    }
}

