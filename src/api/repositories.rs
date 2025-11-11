use actix_web::{web, HttpResponse, Responder, HttpRequest};
use serde::{Deserialize, Serialize};
use crate::api::{ApiState, ErrorResponse};
use crate::api::extract_api_key;
use crate::storage::{RepositoryRepository, DependencyRepository};
use crate::ingestion::RepositoryCrawler;
use crate::analysis::DependencyExtractor;
use crate::security::ServiceDetector;
use crate::graph::GraphBuilder;
use crate::analysis::CodeAnalyzer;
use crate::config::StorageConfig;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRepositoryRequest {
    pub name: String,
    pub url: String,
    pub branch: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyzeRepositoryRequest {
    pub repository_id: String,
}

// Repository endpoints
pub async fn create_repository(
    state: web::Data<ApiState>,
    req: HttpRequest,
    body: web::Json<CreateRepositoryRequest>,
) -> impl Responder {
    // Validate API key
    let _api_key = match extract_api_key(&req) {
        Ok(key) => key,
        Err(resp) => return resp,
    };

    let key_info = match state.auth_service.validate_api_key(&_api_key) {
        Ok(info) => info,
        Err(e) => {
            return HttpResponse::Unauthorized().json(ErrorResponse {
                error: e.to_string(),
            });
        }
    };

    // Check write scope
    if !key_info.scopes.contains(&"write".to_string()) && !key_info.scopes.contains(&"admin".to_string()) {
        return HttpResponse::Forbidden().json(ErrorResponse {
            error: "Write scope required".to_string(),
        });
    }

    match state.repo_repo.create(&body.name, &body.url, body.branch.as_deref()) {
        Ok(repo) => HttpResponse::Created().json(repo),
        Err(e) => HttpResponse::BadRequest().json(ErrorResponse {
            error: e.to_string(),
        }),
    }
}

pub async fn list_repositories(
    state: web::Data<ApiState>,
    req: HttpRequest,
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

    match state.repo_repo.list_all() {
        Ok(repos) => HttpResponse::Ok().json(repos),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: e.to_string(),
        }),
    }
}

pub async fn get_repository(
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

    match state.repo_repo.find_by_id(&path.into_inner()) {
        Ok(Some(repo)) => HttpResponse::Ok().json(repo),
        Ok(None) => HttpResponse::NotFound().json(ErrorResponse {
            error: "Repository not found".to_string(),
        }),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: e.to_string(),
        }),
    }
}

pub async fn analyze_repository(
    state: web::Data<ApiState>,
    req: HttpRequest,
    body: web::Json<AnalyzeRepositoryRequest>,
) -> impl Responder {
    // Validate API key
    let _api_key = match extract_api_key(&req) {
        Ok(key) => key,
        Err(resp) => return resp,
    };

    let key_info = match state.auth_service.validate_api_key(&_api_key) {
        Ok(info) => info,
        Err(e) => {
            return HttpResponse::Unauthorized().json(ErrorResponse {
                error: e.to_string(),
            });
        }
    };

    // Check write scope
    if !key_info.scopes.contains(&"write".to_string()) && !key_info.scopes.contains(&"admin".to_string()) {
        return HttpResponse::Forbidden().json(ErrorResponse {
            error: "Write scope required".to_string(),
        });
    }

    // Get repository
    let repo = match state.repo_repo.find_by_id(&body.repository_id) {
        Ok(Some(repo)) => repo,
        Ok(None) => {
            return HttpResponse::NotFound().json(ErrorResponse {
                error: "Repository not found".to_string(),
            });
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: e.to_string(),
            });
        }
    };

    // Clone/update repository
    let storage_config = StorageConfig {
        repository_cache_path: "./cache/repos".to_string(),
        max_cache_size: "10GB".to_string(),
    };
    
    let crawler = match RepositoryCrawler::new(&storage_config) {
        Ok(c) => c,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to initialize crawler: {}", e),
            });
        }
    };

    let repo_path = match crawler.clone_or_update(&repo.url, Some(&repo.branch)) {
        Ok(path) => path,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to clone repository: {}", e),
            });
        }
    };

    // Extract dependencies
    let extractor = DependencyExtractor::new();
    let manifests = match extractor.extract_from_repository(&repo_path) {
        Ok(m) => m,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to extract dependencies: {}", e),
            });
        }
    };

    // Store dependencies
    for manifest in &manifests {
        if let Err(e) = state.dep_repo.store_dependencies(
            &repo.id,
            &manifest.dependencies,
            &manifest.file_path,
        ) {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to store dependencies: {}", e),
            });
        }
    }

    // Detect services
    let detector = ServiceDetector::new();
    let services = match detector.detect_services(&repo_path) {
        Ok(s) => s,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to detect services: {}", e),
            });
        }
    };

    // Store services
    if let Err(e) = state.service_repo.store_services(&repo.id, &services) {
        return HttpResponse::InternalServerError().json(ErrorResponse {
            error: format!("Failed to store services: {}", e),
        });
    }

    // Build and store knowledge graph
    let graph_builder = GraphBuilder::new(
        state.repo_repo.db.clone(),
        state.repo_repo.clone(),
        state.dep_repo.clone(),
        state.service_repo.clone(),
    );
    
    match graph_builder.build_for_repository(&repo.id) {
        Ok(graph) => {
            if let Err(e) = graph_builder.store_graph(&repo.id, &graph) {
                return HttpResponse::InternalServerError().json(ErrorResponse {
                    error: format!("Failed to store graph: {}", e),
                });
            }
        }
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to build graph: {}", e),
            });
        }
    }

    // Analyze code structure
    let code_analyzer = CodeAnalyzer::new();
    let code_structure = match code_analyzer.analyze_repository(&repo_path) {
        Ok(structure) => structure,
        Err(e) => {
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to analyze code structure: {}", e),
            });
        }
    };

    // Store code elements and calls
    if let Err(e) = state.code_repo.store_elements(&repo.id, &code_structure.elements) {
        return HttpResponse::InternalServerError().json(ErrorResponse {
            error: format!("Failed to store code elements: {}", e),
        });
    }

    if let Err(e) = state.code_repo.store_calls(&repo.id, &code_structure.calls) {
        return HttpResponse::InternalServerError().json(ErrorResponse {
            error: format!("Failed to store code calls: {}", e),
        });
    }

    // Update last analyzed timestamp
    if let Err(e) = state.repo_repo.update_last_analyzed(&repo.id) {
        return HttpResponse::InternalServerError().json(ErrorResponse {
            error: format!("Failed to update repository: {}", e),
        });
    }

    HttpResponse::Ok().json(serde_json::json!({
        "message": "Repository analyzed successfully",
        "manifests_found": manifests.len(),
        "total_dependencies": manifests.iter().map(|m| m.dependencies.len()).sum::<usize>(),
        "services_found": services.len(),
        "graph_built": true,
        "code_elements_found": code_structure.elements.len(),
        "code_calls_found": code_structure.calls.len()
    }))
}

pub async fn get_dependencies(
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

    match state.dep_repo.get_by_repository(&path.into_inner()) {
        Ok(deps) => HttpResponse::Ok().json(deps),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: e.to_string(),
        }),
    }
}

pub async fn search_dependencies(
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

    if let Some(package_name) = query.get("name") {
        match state.dep_repo.get_by_package_name(package_name) {
            Ok(deps) => HttpResponse::Ok().json(deps),
            Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
                error: e.to_string(),
            }),
        }
    } else {
        HttpResponse::BadRequest().json(ErrorResponse {
            error: "Missing 'name' query parameter".to_string(),
        })
    }
}

