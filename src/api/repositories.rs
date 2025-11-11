use actix_web::{web, HttpResponse, Responder, HttpRequest};
use serde::{Deserialize, Serialize};
use base64::{Engine as _, engine::general_purpose};
use crate::api::{ApiState, ErrorResponse};
use crate::storage::{RepositoryRepository, DependencyRepository};
use crate::ingestion::{RepositoryCrawler, RepositoryCredentials, AuthType};
use crate::analysis::DependencyExtractor;
use crate::security::ServiceDetector;
use crate::security::analyzer::SecurityAnalyzer;
use crate::graph::GraphBuilder;
use crate::analysis::CodeAnalyzer;
use crate::config::StorageConfig;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRepositoryRequest {
    pub name: String,
    pub url: String,
    pub branch: Option<String>,
    pub auth_type: Option<String>,  // "ssh_key", "token", "username_password"
    pub auth_value: Option<String>,  // SSH key path, token, or base64(username:password)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyzeRepositoryRequest {
    pub repository_id: String,
}

// Repository endpoints
pub async fn create_repository(
    state: web::Data<ApiState>,
    _req: HttpRequest,
    body: web::Json<CreateRepositoryRequest>,
) -> impl Responder {
    // API key validation removed for local tool simplicity
    match state.repo_repo.create(
        &body.name,
        &body.url,
        body.branch.as_deref(),
        body.auth_type.as_deref(),
        body.auth_value.as_deref(),
    ) {
        Ok(repo) => HttpResponse::Created().json(repo),
        Err(e) => HttpResponse::BadRequest().json(ErrorResponse {
            error: e.to_string(),
        }),
    }
}

pub async fn list_repositories(
    state: web::Data<ApiState>,
    _req: HttpRequest,
) -> impl Responder {
    // API key validation removed for local tool simplicity
    match state.repo_repo.list_all() {
        Ok(repos) => HttpResponse::Ok().json(repos),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: e.to_string(),
        }),
    }
}

pub async fn get_repository(
    state: web::Data<ApiState>,
    _req: HttpRequest,
    path: web::Path<String>,
) -> impl Responder {
    // API key validation removed for local tool simplicity
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
    _req: HttpRequest,
    body: web::Json<AnalyzeRepositoryRequest>,
) -> impl Responder {
    log::info!("Starting analysis for repository ID: {}", body.repository_id);
    
    // API key validation removed for local tool simplicity
    // Get repository
    log::info!("Step 1/8: Fetching repository information...");
    let repo = match state.repo_repo.find_by_id(&body.repository_id) {
        Ok(Some(repo)) => {
            log::info!("Found repository: {} ({})", repo.name, repo.url);
            repo
        },
        Ok(None) => {
            log::error!("Repository not found: {}", body.repository_id);
            return HttpResponse::NotFound().json(ErrorResponse {
                error: "Repository not found".to_string(),
            });
        }
        Err(e) => {
            log::error!("Database error fetching repository: {}", e);
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: e.to_string(),
            });
        }
    };

    // Clone/update repository
    log::info!("Step 2/8: Initializing repository crawler...");
    let storage_config = StorageConfig {
        repository_cache_path: "./cache/repos".to_string(),
        max_cache_size: "10GB".to_string(),
    };
    
    let crawler = match RepositoryCrawler::new(&storage_config) {
        Ok(c) => {
            log::info!("Crawler initialized successfully");
            c
        },
        Err(e) => {
            log::error!("Failed to initialize crawler: {}", e);
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to initialize crawler: {}", e),
            });
        }
    };

    log::info!("Step 3/8: Cloning/updating repository from {} (branch: {})...", repo.url, repo.branch);
    let credentials = repo.auth_type.as_ref().and_then(|auth_type| {
        repo.auth_value.as_ref().map(|auth_value| {
            match auth_type.as_str() {
                "ssh_key" => {
                    log::info!("Using SSH key authentication");
                    RepositoryCredentials {
                        auth_type: AuthType::SshKey(auth_value.clone()),
                    }
                },
                "token" => {
                    log::info!("Using token authentication");
                    RepositoryCredentials {
                        auth_type: AuthType::Token(auth_value.clone()),
                    }
                },
                "username_password" => {
                    log::info!("Using username/password authentication");
                    // Decode base64(username:password)
                    let decoded = general_purpose::STANDARD.decode(auth_value).unwrap_or_default();
                    let creds_str = String::from_utf8(decoded).unwrap_or_default();
                    let parts: Vec<&str> = creds_str.splitn(2, ':').collect();
                    RepositoryCredentials {
                        auth_type: AuthType::UsernamePassword(
                            parts[0].to_string(),
                            parts.get(1).unwrap_or(&"").to_string(),
                        ),
                    }
                }
                _ => {
                    log::info!("Using default token authentication");
                    RepositoryCredentials {
                        auth_type: AuthType::Token(auth_value.clone()),
                    }
                },
            }
        })
    });
    
    let repo_path = match crawler.clone_or_update(
        &repo.url,
        Some(&repo.branch),
        credentials.as_ref(),
    ) {
        Ok(path) => {
            log::info!("✓ Repository cloned/updated successfully to: {}", path.display());
            path
        },
        Err(e) => {
            log::error!("✗ Failed to clone repository: {}", e);
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to clone repository: {}", e),
            });
        }
    };

    // Extract dependencies
    log::info!("Step 4/8: Extracting dependencies from repository...");
    let extractor = DependencyExtractor::new();
    let manifests = match extractor.extract_from_repository(&repo_path) {
        Ok(m) => {
            let total_deps: usize = m.iter().map(|manifest| manifest.dependencies.len()).sum();
            log::info!("✓ Found {} manifest files with {} total dependencies", m.len(), total_deps);
            m
        },
        Err(e) => {
            log::error!("✗ Failed to extract dependencies: {}", e);
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to extract dependencies: {}", e),
            });
        }
    };

    // Store dependencies
    log::info!("Storing dependencies in database...");
    let mut stored_deps = 0;
    for manifest in &manifests {
        if let Err(e) = state.dep_repo.store_dependencies(
            &repo.id,
            &manifest.dependencies,
            &manifest.file_path,
        ) {
            log::error!("✗ Failed to store dependencies from {}: {}", manifest.file_path, e);
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to store dependencies: {}", e),
            });
        }
        stored_deps += manifest.dependencies.len();
    }
    log::info!("✓ Stored {} dependencies", stored_deps);

    // Detect services
    log::info!("Step 5/8: Detecting external services...");
    let detector = ServiceDetector::new();
    let services = match detector.detect_services(&repo_path) {
        Ok(s) => {
            log::info!("✓ Detected {} services", s.len());
            s
        },
        Err(e) => {
            log::error!("✗ Failed to detect services: {}", e);
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to detect services: {}", e),
            });
        }
    };

    // Store services
    log::info!("Storing services in database...");
    if let Err(e) = state.service_repo.store_services(&repo.id, &services) {
        log::error!("✗ Failed to store services: {}", e);
        return HttpResponse::InternalServerError().json(ErrorResponse {
            error: format!("Failed to store services: {}", e),
        });
    }
    log::info!("✓ Stored {} services", services.len());

    // Build and store knowledge graph
    log::info!("Step 6/8: Building knowledge graph...");
    let graph_builder = GraphBuilder::new(
        state.repo_repo.db.clone(),
        state.repo_repo.clone(),
        state.dep_repo.clone(),
        state.service_repo.clone(),
    );
    
    match graph_builder.build_for_repository(&repo.id) {
        Ok(graph) => {
            log::info!("✓ Knowledge graph built: {} nodes, {} edges", graph.nodes.len(), graph.edges.len());
            if let Err(e) = graph_builder.store_graph(&repo.id, &graph) {
                log::error!("✗ Failed to store graph: {}", e);
                return HttpResponse::InternalServerError().json(ErrorResponse {
                    error: format!("Failed to store graph: {}", e),
                });
            }
            log::info!("✓ Graph stored successfully");
        }
        Err(e) => {
            log::error!("✗ Failed to build graph: {}", e);
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to build graph: {}", e),
            });
        }
    }

    // Analyze code structure
    log::info!("Step 7/8: Analyzing code structure...");
    let code_analyzer = CodeAnalyzer::new();
    let code_structure = match code_analyzer.analyze_repository(&repo_path) {
        Ok(structure) => {
            log::info!("✓ Code analysis complete: {} elements, {} calls", structure.elements.len(), structure.calls.len());
            structure
        },
        Err(e) => {
            log::error!("✗ Failed to analyze code structure: {}", e);
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to analyze code structure: {}", e),
            });
        }
    };

    // Store code elements and calls
    log::info!("Storing code elements...");
    if let Err(e) = state.code_repo.store_elements(&repo.id, &code_structure.elements) {
        log::error!("✗ Failed to store code elements: {}", e);
        return HttpResponse::InternalServerError().json(ErrorResponse {
            error: format!("Failed to store code elements: {}", e),
        });
    }
    log::info!("✓ Stored {} code elements", code_structure.elements.len());
    
    log::info!("Storing code calls...");
    if let Err(e) = state.code_repo.store_calls(&repo.id, &code_structure.calls) {
        log::error!("✗ Failed to store code calls: {}", e);
        return HttpResponse::InternalServerError().json(ErrorResponse {
            error: format!("Failed to store code calls: {}", e),
        });
    }
    log::info!("✓ Stored {} code calls", code_structure.calls.len());

    // Analyze security configuration
    log::info!("Step 8/8: Analyzing security configuration...");
    let security_analyzer = SecurityAnalyzer::new();
    let security_analysis = match security_analyzer.analyze_repository(&repo_path) {
        Ok(analysis) => {
            log::info!("✓ Security analysis complete: {} entities, {} relationships, {} vulnerabilities", 
                analysis.entities.len(), analysis.relationships.len(), analysis.vulnerabilities.len());
            analysis
        },
        Err(e) => {
            log::error!("✗ Failed to analyze security: {}", e);
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to analyze security: {}", e),
            });
        }
    };

    // Store security entities, relationships, and vulnerabilities
    log::info!("Storing security data...");
    if let Err(e) = state.security_repo.store_entities(&repo.id, &security_analysis.entities) {
        log::error!("✗ Failed to store security entities: {}", e);
        return HttpResponse::InternalServerError().json(ErrorResponse {
            error: format!("Failed to store security entities: {}", e),
        });
    }
    log::info!("✓ Stored {} security entities", security_analysis.entities.len());

    if let Err(e) = state.security_repo.store_relationships(&repo.id, &security_analysis.relationships) {
        log::error!("✗ Failed to store security relationships: {}", e);
        return HttpResponse::InternalServerError().json(ErrorResponse {
            error: format!("Failed to store security relationships: {}", e),
        });
    }
    log::info!("✓ Stored {} security relationships", security_analysis.relationships.len());

    if let Err(e) = state.security_repo.store_vulnerabilities(&repo.id, &security_analysis.vulnerabilities) {
        log::error!("✗ Failed to store security vulnerabilities: {}", e);
        return HttpResponse::InternalServerError().json(ErrorResponse {
            error: format!("Failed to store security vulnerabilities: {}", e),
        });
    }
    log::info!("✓ Stored {} security vulnerabilities", security_analysis.vulnerabilities.len());

    // Update last analyzed timestamp
    log::info!("Updating repository timestamp...");
    if let Err(e) = state.repo_repo.update_last_analyzed(&repo.id) {
        log::error!("✗ Failed to update repository timestamp: {}", e);
        return HttpResponse::InternalServerError().json(ErrorResponse {
            error: format!("Failed to update repository: {}", e),
        });
    }

    log::info!("✓ Analysis complete for repository: {}", repo.name);
    HttpResponse::Ok().json(serde_json::json!({
        "message": "Repository analyzed successfully",
        "repository": {
            "id": repo.id,
            "name": repo.name,
            "url": repo.url,
            "branch": repo.branch
        },
        "results": {
            "manifests_found": manifests.len(),
            "total_dependencies": stored_deps,
            "services_found": services.len(),
            "graph_built": true,
            "code_elements_found": code_structure.elements.len(),
            "code_calls_found": code_structure.calls.len(),
            "security_entities_found": security_analysis.entities.len(),
            "security_relationships_found": security_analysis.relationships.len(),
            "security_vulnerabilities_found": security_analysis.vulnerabilities.len()
        }
    }))
}

pub async fn get_dependencies(
    state: web::Data<ApiState>,
    _req: HttpRequest,
    path: web::Path<String>,
) -> impl Responder {
    // API key validation removed for local tool simplicity
    match state.dep_repo.get_by_repository(&path.into_inner()) {
        Ok(deps) => HttpResponse::Ok().json(deps),
        Err(e) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: e.to_string(),
        }),
    }
}

pub async fn search_dependencies(
    state: web::Data<ApiState>,
    _req: HttpRequest,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    // API key validation removed for local tool simplicity
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

