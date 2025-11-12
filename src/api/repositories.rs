use actix_web::{web, HttpResponse, Responder, HttpRequest};
use serde::{Deserialize, Serialize};
use std::path::Path;
use base64::{Engine as _, engine::general_purpose};
use crate::api::{ApiState, ErrorResponse};
use crate::storage::{RepositoryRepository, DependencyRepository, ToolRepository};
use crate::ingestion::{RepositoryCrawler, RepositoryCredentials, AuthType};
use crate::analysis::{DependencyExtractor, ToolDetector};
use crate::security::ServiceDetector;
use crate::security::analyzer::SecurityAnalyzer;
use crate::graph::GraphBuilder;
use crate::analysis::CodeAnalyzer;
use crate::config::StorageConfig;
use crate::config::Config;

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
    let repository_id = body.repository_id.clone();
    log::info!("Starting analysis for repository ID: {}", repository_id);
    
    // Start progress tracking
    state.progress_tracker.start_analysis(&repository_id, 8);
    
    // API key validation removed for local tool simplicity
    // Get repository
    state.progress_tracker.update_progress(&repository_id, 1, "Fetching repository information", "Loading repository details...", None);
    log::info!("Step 1/8: Fetching repository information...");
    let repo = match state.repo_repo.find_by_id(&repository_id) {
        Ok(Some(repo)) => {
            log::info!("Found repository: {} ({})", repo.name, repo.url);
            repo
        },
        Ok(None) => {
            log::error!("Repository not found: {}", repository_id);
            state.progress_tracker.fail_analysis(&repository_id, "Repository not found");
            return HttpResponse::NotFound().json(ErrorResponse {
                error: "Repository not found".to_string(),
            });
        }
        Err(e) => {
            log::error!("Database error fetching repository: {}", e);
            state.progress_tracker.fail_analysis(&repository_id, &format!("Database error: {}", e));
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: e.to_string(),
            });
        }
    };

    // Clone/update repository
    state.progress_tracker.update_progress(&repository_id, 2, "Initializing crawler", "Setting up repository crawler...", None);
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
            state.progress_tracker.fail_analysis(&repository_id, &format!("Failed to initialize crawler: {}", e));
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to initialize crawler: {}", e),
            });
        }
    };

    state.progress_tracker.update_progress(&repository_id, 3, "Preparing repository", 
        if crate::ingestion::crawler::RepositoryCrawler::is_local_path(&repo.url) {
            format!("Using local repository at {}...", repo.url)
        } else {
            format!("Fetching repository from {}...", repo.url)
        }.as_str(), 
        Some(serde_json::json!({"url": repo.url, "branch": repo.branch, "is_local": crate::ingestion::crawler::RepositoryCrawler::is_local_path(&repo.url)})));
    log::info!("Step 3/8: Preparing repository from {} (branch: {})...", repo.url, repo.branch);
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
    // Load plugins from config/plugins directory if it exists
    let plugin_dir = Path::new("config/plugins");
    let detector = if plugin_dir.exists() && plugin_dir.is_dir() {
        match ServiceDetector::with_plugins(Some(plugin_dir)) {
            Ok(d) => {
                log::info!("✓ Loaded service detection patterns with plugins");
                d
            }
            Err(e) => {
                log::warn!("⚠ Failed to load plugins, using default patterns: {}", e);
                ServiceDetector::new()
            }
        }
    } else {
        ServiceDetector::new()
    };
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

    // Detect developer tools and scripts
    state.progress_tracker.update_progress(&repository_id, 6, "Detecting developer tools", "Scanning for build tools, test frameworks, linters, and scripts...", None);
    log::info!("Step 6/9: Detecting developer tools...");
    let tool_detector = ToolDetector::new();
    let tools = match tool_detector.detect_tools(&repo_path) {
        Ok(t) => {
            log::info!("✓ Detected {} tools", t.len());
            t
        },
        Err(e) => {
            log::error!("✗ Failed to detect tools: {}", e);
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to detect tools: {}", e),
            });
        }
    };

    // Store tools
    log::info!("Storing tools in database...");
    if let Err(e) = state.tool_repo.store_tools(&repo.id, &tools) {
        log::error!("✗ Failed to store tools: {}", e);
        return HttpResponse::InternalServerError().json(ErrorResponse {
            error: format!("Failed to store tools: {}", e),
        });
    }
    log::info!("✓ Stored {} tools", tools.len());

    // Build and store knowledge graph
    log::info!("Step 7/9: Building knowledge graph...");
    let graph_builder = GraphBuilder::new(
        state.repo_repo.db.clone(),
        state.repo_repo.clone(),
        state.dep_repo.clone(),
        state.service_repo.clone(),
        state.tool_repo.clone(),
        state.code_relationship_repo.clone(),
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
    log::info!("Step 8/9: Analyzing code structure...");
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
        state.progress_tracker.fail_analysis(&repository_id, &format!("Failed to store code elements: {}", e));
        return HttpResponse::InternalServerError().json(ErrorResponse {
            error: format!("Failed to store code elements: {}", e),
        });
    }
    log::info!("✓ Stored {} code elements", code_structure.elements.len());
    
    log::info!("Storing code calls...");
    if let Err(e) = state.code_repo.store_calls(&repo.id, &code_structure.calls) {
        log::error!("✗ Failed to store code calls: {}", e);
        state.progress_tracker.fail_analysis(&repository_id, &format!("Failed to store code calls: {}", e));
        return HttpResponse::InternalServerError().json(ErrorResponse {
            error: format!("Failed to store code calls: {}", e),
        });
    }
    log::info!("✓ Stored {} code calls", code_structure.calls.len());

    // Detect relationships between code elements and services/dependencies
    log::info!("Detecting code-to-service/dependency relationships...");
    use crate::analysis::CodeRelationshipDetector;
    let relationship_detector = CodeRelationshipDetector::new(&repo_path);
    
    // Get stored services and dependencies for relationship detection
    let stored_services = match state.service_repo.get_by_repository(&repo.id) {
        Ok(s) => s,
        Err(e) => {
            log::warn!("Failed to get services for relationship detection: {}", e);
            Vec::new()
        }
    };
    
    let stored_deps_vec = match state.dep_repo.get_by_repository(&repo.id) {
        Ok(d) => d,
        Err(e) => {
            log::warn!("Failed to get dependencies for relationship detection: {}", e);
            Vec::new()
        }
    };
    
    let code_relationships = match relationship_detector.detect_relationships(&code_structure, &stored_services, &stored_deps_vec) {
        Ok(rels) => {
            log::info!("✓ Detected {} code relationships", rels.len());
            rels
        },
        Err(e) => {
            log::error!("✗ Failed to detect code relationships: {}", e);
            Vec::new() // Continue even if relationship detection fails
        }
    };
    
    // Store code relationships
    if !code_relationships.is_empty() {
        if let Err(e) = state.code_relationship_repo.store_relationships(&repo.id, &code_relationships) {
            log::error!("✗ Failed to store code relationships: {}", e);
        } else {
            log::info!("✓ Stored {} code relationships", code_relationships.len());
        }
    }

    // Analyze security configuration
    state.progress_tracker.update_progress(&repository_id, 9, "Analyzing security configuration", "Scanning for security entities, API keys, and vulnerabilities...", None);
    log::info!("Step 9/9: Analyzing security configuration...");
    let security_analyzer = SecurityAnalyzer::new();
    let security_analysis = match security_analyzer.analyze_repository(&repo_path, Some(&code_structure), Some(&services)) {
        Ok(analysis) => {
            log::info!("✓ Security analysis complete: {} entities, {} relationships, {} vulnerabilities", 
                analysis.entities.len(), analysis.relationships.len(), analysis.vulnerabilities.len());
            state.progress_tracker.update_progress(&repository_id, 8, "Analyzing security configuration", 
                format!("Found {} security entities, {} relationships, {} vulnerabilities", 
                    analysis.entities.len(), analysis.relationships.len(), analysis.vulnerabilities.len()).as_str(),
                Some(serde_json::json!({
                    "entities": analysis.entities.len(),
                    "relationships": analysis.relationships.len(),
                    "vulnerabilities": analysis.vulnerabilities.len()
                })));
            analysis
        },
        Err(e) => {
            log::error!("✗ Failed to analyze security: {}", e);
            state.progress_tracker.fail_analysis(&repository_id, &format!("Failed to analyze security: {}", e));
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to analyze security: {}", e),
            });
        }
    };

    // Store security entities, relationships, and vulnerabilities
    // IMPORTANT: Delete in reverse dependency order to avoid foreign key constraint issues
    // Delete vulnerabilities and relationships first (they reference entities), then entities
    log::info!("Storing security data...");
    
    // First, delete old vulnerabilities and relationships (they reference entities)
    if let Err(e) = state.security_repo.store_vulnerabilities(&repo.id, &[]) {
        log::warn!("Failed to clear old vulnerabilities: {}", e);
    }
    if let Err(e) = state.security_repo.store_relationships(&repo.id, &[]) {
        log::warn!("Failed to clear old relationships: {}", e);
    }
    
    // Now store entities (they can be deleted safely)
    if let Err(e) = state.security_repo.store_entities(&repo.id, &security_analysis.entities) {
        log::error!("✗ Failed to store security entities: {}", e);
        return HttpResponse::InternalServerError().json(ErrorResponse {
            error: format!("Failed to store security entities: {}", e),
        });
    }
    log::info!("✓ Stored {} security entities", security_analysis.entities.len());

    // Now store relationships (entities exist now)
    if let Err(e) = state.security_repo.store_relationships(&repo.id, &security_analysis.relationships) {
        log::error!("✗ Failed to store security relationships: {}", e);
        return HttpResponse::InternalServerError().json(ErrorResponse {
            error: format!("Failed to store security relationships: {}", e),
        });
    }
    log::info!("✓ Stored {} security relationships", security_analysis.relationships.len());

    // Finally store vulnerabilities (entities exist now)
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

pub async fn delete_repository(
    state: web::Data<ApiState>,
    _req: HttpRequest,
    path: web::Path<String>,
) -> impl Responder {
    let repository_id = path.into_inner();
    
    log::info!("Deleting repository: {}", repository_id);
    
    // Get repository info before deletion (for cache cleanup)
    let repo = match state.repo_repo.find_by_id(&repository_id) {
        Ok(Some(repo)) => repo,
        Ok(None) => {
            return HttpResponse::NotFound().json(ErrorResponse {
                error: "Repository not found".to_string(),
            });
        }
        Err(e) => {
            log::error!("Failed to find repository: {}", e);
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to find repository: {}", e),
            });
        }
    };
    
    // Delete all repository data from database
    if let Err(e) = state.repo_repo.delete(&repository_id) {
        log::error!("Failed to delete repository data: {}", e);
        return HttpResponse::InternalServerError().json(ErrorResponse {
            error: format!("Failed to delete repository: {}", e),
        });
    }
    
    // Remove cached repository files
    let config = match Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            log::warn!("Failed to load config for cache cleanup: {}", e);
            // Continue even if we can't clean cache
            return HttpResponse::Ok().json(serde_json::json!({
                "message": "Repository deleted successfully",
                "repository_id": repository_id,
                "warning": "Cache cleanup failed, but repository data was deleted"
            }));
        }
    };
    
    let crawler = match RepositoryCrawler::new(&config.storage) {
        Ok(c) => c,
        Err(e) => {
            log::warn!("Failed to create crawler for cache cleanup: {}", e);
            // Continue even if we can't clean cache
            return HttpResponse::Ok().json(serde_json::json!({
                "message": "Repository deleted successfully",
                "repository_id": repository_id,
                "warning": "Cache cleanup failed, but repository data was deleted"
            }));
        }
    };
    
    if let Err(e) = crawler.remove_repository(&repo.url) {
        log::warn!("Failed to remove repository cache: {}", e);
        // Continue even if cache cleanup fails
    } else {
        log::info!("Removed repository cache for: {}", repo.url);
    }
    
    log::info!("✓ Successfully deleted repository: {}", repository_id);
    HttpResponse::Ok().json(serde_json::json!({
        "message": "Repository deleted successfully",
        "repository_id": repository_id
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

