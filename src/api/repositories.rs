use actix_web::{web, HttpResponse, Responder, HttpRequest};
use serde::{Deserialize, Serialize};
use std::path::Path;
use base64::{Engine as _, engine::general_purpose};
use crate::api::{ApiState, ErrorResponse};
use crate::ingestion::{RepositoryCrawler, RepositoryCredentials, AuthType};
use crate::analysis::{DependencyExtractor, ToolDetector, TestDetector, CodeElement};
use crate::security::ServiceDetector;
use crate::security::analyzer::SecurityAnalyzer;
use crate::graph::GraphBuilder;
use crate::analysis::CodeAnalyzer;
use crate::config::StorageConfig;
use crate::config::Config;
use serde_json::Value;
use std::process::Command;

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

// Helper function to get repository local path
fn get_repo_local_path(url: &str) -> Option<String> {
    let config = Config::from_env().unwrap_or_else(|_| {
        // Use default config if env loading fails
        Config {
            server: crate::config::ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                environment: "development".to_string(),
                editor_protocol: "vscode".to_string(),
            },
            database: crate::config::DatabaseConfig {
                database_path: "./data/wavelength.db".to_string(),
                graph_db_path: "./data/graph.db".to_string(),
            },
            security: crate::config::SecurityConfig {},
            storage: crate::config::StorageConfig {
                repository_cache_path: "./cache/repos".to_string(),
                max_cache_size: "10GB".to_string(),
            },
            logging: crate::config::LoggingConfig {
                log_level: "info".to_string(),
                log_format: "json".to_string(),
            },
        }
    });
    
    let crawler = match RepositoryCrawler::new(&config.storage) {
        Ok(c) => c,
        Err(_) => return None,
    };
    
    let local_path = crawler.get_repo_path(url);
    Some(local_path.to_string_lossy().to_string())
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
        Ok(repos) => {
            // Add local_path to each repository
            let repos_with_paths: Vec<serde_json::Value> = repos.iter()
                .map(|repo| {
                    let mut repo_json = serde_json::to_value(repo).unwrap_or(serde_json::json!({}));
                    if let Some(obj) = repo_json.as_object_mut() {
                        if let Some(local_path) = get_repo_local_path(&repo.url) {
                            obj.insert("local_path".to_string(), serde_json::Value::String(local_path));
                        }
                    }
                    repo_json
                })
                .collect();
            
            HttpResponse::Ok().json(repos_with_paths)
        },
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
        Ok(Some(repo)) => {
            // Add local_path to the response
            let mut repo_json = serde_json::to_value(&repo).unwrap_or(serde_json::json!({}));
            if let Some(obj) = repo_json.as_object_mut() {
                if let Some(local_path) = get_repo_local_path(&repo.url) {
                    obj.insert("local_path".to_string(), serde_json::Value::String(local_path));
                }
            }
            
            HttpResponse::Ok().json(repo_json)
        },
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
    
    // Start progress tracking (11 steps including test detection and documentation indexing)
    state.progress_tracker.start_analysis(&repository_id, 11);
    
    // Clone state for the blocking task
    let state_clone = state.clone();
    let repository_id_clone = repository_id.clone();
    
    // Move the blocking analysis work to a blocking thread pool
    // This allows other API requests to continue being served
    let analysis_result = web::block(move || {
        perform_analysis(state_clone, &repository_id_clone)
    }).await;
    
    match analysis_result {
        Ok(Ok(result)) => HttpResponse::Ok().json(serde_json::json!({
            "message": result.message,
            "repository": result.repository,
            "results": result.results
        })),
        Ok(Err(e)) => {
            log::error!("Analysis failed: {}", e);
            state.progress_tracker.fail_analysis(&repository_id, &e.to_string());
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: e.to_string(),
            })
        }
        Err(e) => {
            log::error!("Blocking task error: {}", e);
            state.progress_tracker.fail_analysis(&repository_id, &e.to_string());
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: format!("Failed to execute analysis: {}", e),
            })
        }
    }
}

#[derive(Serialize)]
struct AnalysisResult {
    message: String,
    repository: serde_json::Value,
    results: serde_json::Value,
}

/// Perform the actual analysis work (runs in blocking thread pool)
fn perform_analysis(
    state: web::Data<ApiState>,
    repository_id: &str,
) -> Result<AnalysisResult, anyhow::Error> {
    
    // API key validation removed for local tool simplicity
    // Get repository
    state.progress_tracker.update_progress(&repository_id, 1, "Fetching repository information", "Loading repository details...", None);
    log::info!("Step 1/11: Fetching repository information...");
    let repo = match state.repo_repo.find_by_id(&repository_id) {
        Ok(Some(repo)) => {
            log::info!("Found repository: {} ({})", repo.name, repo.url);
            repo
        },
        Ok(None) => {
            log::error!("Repository not found: {}", repository_id);
            state.progress_tracker.fail_analysis(&repository_id, "Repository not found");
            return Err(anyhow::anyhow!("Repository not found"));
        }
        Err(e) => {
            log::error!("Database error fetching repository: {}", e);
            state.progress_tracker.fail_analysis(&repository_id, &format!("Database error: {}", e));
            return Err(anyhow::anyhow!("Database error: {}", e));
        }
    };

    // Clone/update repository
    state.progress_tracker.update_progress(&repository_id, 2, "Initializing crawler", "Setting up repository crawler...", None);
    log::info!("Step 2/11: Initializing repository crawler...");
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
            return Err(anyhow::anyhow!("Failed to initialize crawler: {}", e));
        }
    };

    state.progress_tracker.update_progress(&repository_id, 3, "Preparing repository", 
        if crate::ingestion::crawler::RepositoryCrawler::is_local_path(&repo.url) {
            format!("Using local repository at {}...", repo.url)
        } else {
            format!("Fetching repository from {}...", repo.url)
        }.as_str(), 
        Some(serde_json::json!({"url": repo.url, "branch": repo.branch, "is_local": crate::ingestion::crawler::RepositoryCrawler::is_local_path(&repo.url)})));
    log::info!("Step 3/11: Preparing repository from {} (branch: {})...", repo.url, repo.branch);
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
            return Err(anyhow::anyhow!("Failed to clone repository: {}", e));
        }
    };

    // Extract dependencies
    state.progress_tracker.update_progress(&repository_id, 4, "Extracting dependencies", "Scanning package.json, requirements.txt, Cargo.toml, and other manifest files...", None);
    log::info!("Step 4/11: Extracting dependencies from repository...");
    let extractor = DependencyExtractor::new();
    let manifests = match extractor.extract_from_repository(&repo_path) {
        Ok(m) => {
            let total_deps: usize = m.iter().map(|manifest| manifest.dependencies.len()).sum();
            log::info!("✓ Found {} manifest files with {} total dependencies", m.len(), total_deps);
            m
        },
        Err(e) => {
            log::error!("✗ Failed to extract dependencies: {}", e);
            return Err(anyhow::anyhow!("Failed to extract dependencies: {}", e));
        }
    };

    // Store dependencies
    let total_deps_to_store: usize = manifests.iter().map(|m| m.dependencies.len()).sum();
    log::info!("Storing {} dependencies from {} manifest file(s) in database...", total_deps_to_store, manifests.len());
    let mut stored_deps = 0;
    let mut manifest_count = 0;
    for manifest in &manifests {
        manifest_count += 1;
        log::info!("  Processing manifest {}/{}: {} ({} dependencies)", manifest_count, manifests.len(), manifest.file_path, manifest.dependencies.len());
        if let Err(e) = state.dep_repo.store_dependencies(
            &repo.id,
            &manifest.dependencies,
            &manifest.file_path,
        ) {
            log::error!("✗ Failed to store dependencies from {}: {}", manifest.file_path, e);
            return Err(anyhow::anyhow!("Failed to store dependencies from {}: {}", manifest.file_path, e));
        }
        stored_deps += manifest.dependencies.len();
    }
    log::info!("✓ Successfully stored {} dependencies from {} manifest file(s)", stored_deps, manifests.len());

    // Detect services
    state.progress_tracker.update_progress(&repository_id, 5, "Detecting external services", "Scanning for AWS, Firebase, Clerk, AI services, and other integrations...", None);
    log::info!("Step 5/11: Detecting external services...");
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
            if !s.is_empty() {
                let service_names: Vec<String> = s.iter().map(|svc| format!("{} ({:?})", svc.name, svc.provider)).collect();
                log::info!("✓ Detected {} service(s): {}", s.len(), service_names.join(", "));
            } else {
                log::info!("✓ No external services detected");
            }
            s
        },
        Err(e) => {
            log::error!("✗ Failed to detect services: {}", e);
            return Err(anyhow::anyhow!("Failed to detect services: {}", e));
        }
    };

    // Store services
    log::info!("Storing {} service(s) in database...", services.len());
    if let Err(e) = state.service_repo.store_services(&repo.id, &services) {
        log::error!("✗ Failed to store services: {}", e);
        return Err(anyhow::anyhow!("Failed to store services: {}", e));
    }
    log::info!("✓ Successfully stored {} service(s)", services.len());

    // Detect developer tools and scripts
    state.progress_tracker.update_progress(&repository_id, 6, "Detecting developer tools", "Scanning for build tools, test frameworks, linters, and scripts...", None);
    log::info!("Step 6/11: Detecting developer tools...");
    let tool_detector = ToolDetector::new();
    let tools = match tool_detector.detect_tools(&repo_path) {
        Ok(t) => {
            if !t.is_empty() {
                let tool_names: Vec<String> = t.iter().map(|tool| format!("{} ({:?})", tool.name, tool.category)).collect();
                log::info!("✓ Detected {} tool(s): {}", t.len(), tool_names.join(", "));
            } else {
                log::info!("✓ No developer tools detected");
            }
            t
        },
        Err(e) => {
            log::error!("✗ Failed to detect tools: {}", e);
            return Err(anyhow::anyhow!("Failed to detect tools: {}", e));
        }
    };

    // Store tools
    log::info!("Storing {} tool(s) in database...", tools.len());
    if let Err(e) = state.tool_repo.store_tools(&repo.id, &tools) {
        log::error!("✗ Failed to store tools: {}", e);
        return Err(anyhow::anyhow!("Failed to store tools: {}", e));
    }
    log::info!("✓ Successfully stored {} tool(s)", tools.len());

    // Build and store knowledge graph
    state.progress_tracker.update_progress(&repository_id, 7, "Building knowledge graph", "Creating relationships between repositories, dependencies, services, and code elements...", None);
    log::info!("Step 7/11: Building knowledge graph...");
    let graph_builder = GraphBuilder::new(
        state.repo_repo.db.clone(),
        state.repo_repo.clone(),
        state.dep_repo.clone(),
        state.service_repo.clone(),
        state.tool_repo.clone(),
        state.code_relationship_repo.clone(),
        state.test_repo.clone(),
    );
    
    log::info!("Building knowledge graph from stored data (dependencies, services, code elements)...");
    match graph_builder.build_for_repository(&repo.id) {
        Ok(graph) => {
            // Count node types for better diagnostics
            use std::collections::HashMap;
            let mut node_type_counts: HashMap<String, usize> = HashMap::new();
            for node in &graph.nodes {
                let node_type_str = format!("{:?}", node.node_type);
                *node_type_counts.entry(node_type_str).or_insert(0) += 1;
            }
            let node_type_summary: Vec<String> = node_type_counts.iter()
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect();
            log::info!("✓ Knowledge graph built: {} nodes ({}), {} edges", 
                graph.nodes.len(), node_type_summary.join(", "), graph.edges.len());
            
            log::info!("Storing knowledge graph in database...");
            {
                let progress_tracker_nodes = state.progress_tracker.clone();
                let progress_tracker_edges = state.progress_tracker.clone();
                let repository_id_clone = repository_id.clone();
                let repository_id_clone2 = repository_id.clone();
                if let Err(e) = graph_builder.store_graph(
                    &repo.id, 
                    &graph,
                    Some(move |stored, total| {
                        let percent = (stored as f64 / total as f64 * 100.0) as u32;
                        progress_tracker_nodes.update_status_message(
                            &repository_id_clone,
                            &format!("Storing graph nodes: {}/{} ({}%)...", stored, total, percent)
                        );
                    }),
                    Some(move |stored, total| {
                        let percent = (stored as f64 / total as f64 * 100.0) as u32;
                        progress_tracker_edges.update_status_message(
                            &repository_id_clone2,
                            &format!("Storing graph edges: {}/{} ({}%)...", stored, total, percent)
                        );
                    })
                ) {
                    log::error!("✗ Failed to store graph: {}", e);
                    return Err(anyhow::anyhow!("Failed to store graph: {}", e));
                }
            }
            log::info!("✓ Successfully stored knowledge graph");
        }
        Err(e) => {
            log::error!("✗ Failed to build graph: {}", e);
            return Err(anyhow::anyhow!("Failed to build graph: {}", e));
        }
    }

    /// Extract plugin assets and relationships via Python plugin (generic)
    /// Plugin outputs decoder-compatible format, so we can deserialize generically
    /// Currently calls the webMethods plugin, but this is generic and can be extended
    fn extract_webmethods_assets_and_relationships(repo_path: &Path) -> Result<(Vec<CodeElement>, Vec<crate::analysis::CodeRelationship>), anyhow::Error> {
        // Call plugin and get decoder-compatible JSON output
        // Note: Function name kept for backward compatibility, but implementation is generic
        // Pass webMethods-specific plugin parameters
        let (elements, relationships) = call_plugin_for_decoder_format(
            repo_path,
            "webm_asset_plugin",
            "WebMethodsAssetDetector",
        )?;
        Ok((elements, relationships))
    }
    
    /// Generic plugin caller - calls Python plugin and deserializes decoder format
    /// Works with any plugin that follows the standard pattern
    fn call_plugin_for_decoder_format(
        repo_path: &Path,
        plugin_module: &str,
        detector_class: &str,
    ) -> Result<(Vec<CodeElement>, Vec<crate::analysis::CodeRelationship>), anyhow::Error> {
        // Generic plugin discovery - try common locations
        let current_dir = std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
        let plugin_name = plugin_module.replace("_", "-");
        let plugin_paths = vec![
            current_dir.join(format!("../wavelength-arch-decoder-{}-plugin", plugin_name)),
            current_dir.join(format!("../../wavelength-arch-decoder-{}-plugin", plugin_name)),
            current_dir.join(format!("wavelength-arch-decoder-{}-plugin", plugin_name)),
            Path::new(&format!("../wavelength-arch-decoder-{}-plugin", plugin_name)).to_path_buf(),
            Path::new(&format!("../../wavelength-arch-decoder-{}-plugin", plugin_name)).to_path_buf(),
            Path::new(&format!("wavelength-arch-decoder-{}-plugin", plugin_name)).to_path_buf(),
        ];
        
        let mut plugin_dir: Option<std::path::PathBuf> = None;
        for path in &plugin_paths {
            if path.exists() && path.join(plugin_module).exists() {
                plugin_dir = Some(path.clone());
                break;
            }
        }
        
        let plugin_dir_path = match plugin_dir {
            Some(path) => path,
            None => {
                log::debug!("Plugin not found: {}", plugin_module);
                return Ok((Vec::new(), Vec::new()));
            }
        };
        
        let plugin_dir_absolute = plugin_dir_path.canonicalize()
            .unwrap_or_else(|_| plugin_dir_path.clone());
        log::info!("Using plugin '{}' at: {}", plugin_module, plugin_dir_absolute.display());
        
        let repo_path_str = repo_path.to_string_lossy();
        let plugin_path_str = plugin_dir_absolute.to_string_lossy();
        
        // Generic Python script that works with any plugin following the pattern
        let python_script = format!(r#"
import sys
import json
from pathlib import Path

plugin_path = Path(r'{}')
if plugin_path.exists() and (plugin_path / '{}').exists():
    sys.path.insert(0, str(plugin_path))
else:
    print("ERROR: Plugin path not found", file=sys.stderr)
    sys.exit(1)

try:
    from {}.detector import {}
    
    detector = {}()
    result = detector.detect_assets(r'{}')
    data = result.to_dict()
    
    # Output decoder_format section (plugin adapts to decoder's data model)
    decoder_format = data.get('decoder_format', {{}})
    print(json.dumps(decoder_format))
except Exception as e:
    import traceback
    print("ERROR: " + str(e), file=sys.stderr)
    traceback.print_exc(file=sys.stderr)
    sys.exit(1)
"#, plugin_path_str, plugin_module, plugin_module, detector_class, detector_class, repo_path_str);
        
        let output = Command::new("python3")
            .arg("-c")
            .arg(&python_script)
            .output()?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            log::warn!("Plugin execution failed: {}", stderr);
            return Ok((Vec::new(), Vec::new()));
        }
        
        let json_str = String::from_utf8(output.stdout)?;
        if json_str.trim().is_empty() {
            log::warn!("Plugin returned empty output");
            return Ok((Vec::new(), Vec::new()));
        }
        
        let decoder_format: Value = match serde_json::from_str(&json_str) {
            Ok(v) => v,
            Err(e) => {
                log::warn!("Failed to parse plugin decoder format: {} (output: {})", e, json_str.chars().take(200).collect::<String>());
                return Ok((Vec::new(), Vec::new()));
            }
        };
        
        // Deserialize code elements generically (no webMethods-specific code)
        let elements: Vec<CodeElement> = decoder_format
            .get("code_elements")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();
        
        // Deserialize relationships generically
        let relationships: Vec<crate::analysis::CodeRelationship> = decoder_format
            .get("code_relationships")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();
        
        log::debug!("Deserialized {} code elements and {} relationships from plugin", elements.len(), relationships.len());
        
        Ok((elements, relationships))
    }

    /// Extract plugin assets via Python plugin and convert to code elements (legacy function for compatibility)
    /// Note: Function name kept for backward compatibility, but implementation is generic
    fn extract_webmethods_assets_as_code_elements(repo_path: &Path) -> Result<Vec<CodeElement>, anyhow::Error> {
        let (elements, _) = extract_webmethods_assets_and_relationships(repo_path)?;
        Ok(elements)
    }

    // Analyze code structure
    state.progress_tracker.update_progress(&repository_id, 8, "Analyzing code structure", "Scanning source files and extracting functions, classes, modules, and their relationships...", None);
    log::info!("Step 8/11: Analyzing code structure...");
    log::info!("Scanning repository for source code files (this may take a while for large repositories)...");
    let code_analyzer = CodeAnalyzer::new();
    let code_structure = match code_analyzer.analyze_repository(&repo_path) {
        Ok(structure) => {
            // Count element types for better diagnostics
            use std::collections::HashMap;
            let mut element_type_counts: HashMap<String, usize> = HashMap::new();
            let mut language_counts: HashMap<String, usize> = HashMap::new();
            for element in &structure.elements {
                *element_type_counts.entry(format!("{:?}", element.element_type)).or_insert(0) += 1;
                *language_counts.entry(element.language.clone()).or_insert(0) += 1;
            }
            let element_summary: Vec<String> = element_type_counts.iter()
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect();
            let language_summary: Vec<String> = language_counts.iter()
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect();
            log::info!("✓ Code analysis complete: {} elements ({}), {} calls", 
                structure.elements.len(), element_summary.join(", "), structure.calls.len());
            if !language_summary.is_empty() {
                log::info!("  Languages detected: {}", language_summary.join(", "));
            }
            structure
        },
        Err(e) => {
            log::error!("✗ Failed to analyze code structure: {}", e);
            return Err(anyhow::anyhow!("Failed to analyze code structure: {}", e));
        }
    };

    // Extract plugin assets and merge with code elements
    log::info!("Extracting plugin assets...");
    let mut all_code_elements = code_structure.elements.clone();
    let mut plugin_relationships = Vec::new();
    match extract_webmethods_assets_and_relationships(&repo_path) {
        Ok((plugin_elements, rels)) => {
            plugin_relationships = rels;
            if !plugin_elements.is_empty() {
                log::info!("✓ Found {} plugin assets", plugin_elements.len());
                if !plugin_relationships.is_empty() {
                    log::info!("✓ Found {} plugin relationships", plugin_relationships.len());
                }
                all_code_elements.extend(plugin_elements);
            } else {
                log::info!("✓ No plugin assets found");
            }
        },
        Err(e) => {
            log::warn!("⚠ Failed to extract plugin assets (plugin may not be available): {}", e);
        }
    }

    // Store code elements and calls
    log::info!("Storing {} code elements in database...", all_code_elements.len());
    {
        let progress_tracker = state.progress_tracker.clone();
        let repository_id_clone = repository_id.clone();
        if let Err(e) = state.code_repo.store_elements(
            &repo.id, 
            &all_code_elements,
            Some(move |stored, total| {
                let percent = (stored as f64 / total as f64 * 100.0) as u32;
                progress_tracker.update_status_message(
                    &repository_id_clone,
                    &format!("Storing code elements: {}/{} ({}%)...", stored, total, percent)
                );
            })
        ) {
            log::error!("✗ Failed to store code elements: {}", e);
            state.progress_tracker.fail_analysis(&repository_id, &format!("Failed to store code elements: {}", e));
            return Err(anyhow::anyhow!("Failed to store code elements: {}", e));
        }
    }
    log::info!("✓ Stored {} code elements", all_code_elements.len());
    
    log::info!("Storing {} code calls in database...", code_structure.calls.len());
    {
        let progress_tracker = state.progress_tracker.clone();
        let repository_id_clone = repository_id.clone();
        if let Err(e) = state.code_repo.store_calls(
            &repo.id, 
            &code_structure.calls,
            Some(move |stored, total| {
                let percent = (stored as f64 / total as f64 * 100.0) as u32;
                progress_tracker.update_status_message(
                    &repository_id_clone,
                    &format!("Storing code calls: {}/{} ({}%)...", stored, total, percent)
                );
            })
        ) {
            log::error!("✗ Failed to store code calls: {}", e);
            state.progress_tracker.fail_analysis(&repository_id, &format!("Failed to store code calls: {}", e));
            return Err(anyhow::anyhow!("Failed to store code calls: {}", e));
        }
    }
    log::info!("✓ Stored {} code calls", code_structure.calls.len());

    // Detect relationships between code elements and services/dependencies (part of step 8)
    state.progress_tracker.update_status_message(&repository_id, "Detecting relationships between code elements and services/dependencies...");
    log::info!("Detecting relationships between code elements and services/dependencies...");
    use crate::analysis::CodeRelationshipDetector;
    let relationship_detector = CodeRelationshipDetector::new(&repo_path);
    
    // Get stored services and dependencies for relationship detection
    log::info!("  Loading {} service(s) and dependencies for relationship detection...", services.len());
    let stored_services = match state.service_repo.get_by_repository(&repo.id) {
        Ok(s) => {
            log::info!("  Loaded {} service(s) from database", s.len());
            s
        },
        Err(e) => {
            log::warn!("⚠ Failed to get services for relationship detection: {}", e);
            Vec::new()
        }
    };
    
    let stored_deps_vec = match state.dep_repo.get_by_repository(&repo.id) {
        Ok(d) => {
            log::info!("  Loaded {} dependencies from database", d.len());
            d
        },
        Err(e) => {
            log::warn!("⚠ Failed to get dependencies for relationship detection: {}", e);
            Vec::new()
        }
    };
    
    log::info!("  Analyzing {} code element(s) for relationships to {} service(s) and {} dependencies...", 
        all_code_elements.len(), stored_services.len(), stored_deps_vec.len());
    let code_relationships = match relationship_detector.detect_relationships(&code_structure, &stored_services, &stored_deps_vec) {
        Ok(rels) => {
            if !rels.is_empty() {
                log::info!("✓ Detected {} code-to-service/dependency relationship(s)", rels.len());
            } else {
                log::info!("✓ No code relationships detected");
            }
            rels
        },
        Err(e) => {
            log::error!("✗ Failed to detect code relationships: {}", e);
            Vec::new() // Continue even if relationship detection fails
        }
    };
    
    // Combine regular code relationships with plugin relationships
    let mut all_code_relationships = code_relationships;
    all_code_relationships.extend(plugin_relationships);
    
    // Store code relationships (still part of step 8)
    if !all_code_relationships.is_empty() {
        log::info!("Storing {} code relationship(s) in database...", all_code_relationships.len());
        {
            let progress_tracker = state.progress_tracker.clone();
            let repository_id_clone = repository_id.clone();
            if let Err(e) = state.code_relationship_repo.store_relationships(
                &repo.id, 
                &all_code_relationships,
                Some(move |stored, total| {
                    let percent = (stored as f64 / total as f64 * 100.0) as u32;
                    progress_tracker.update_status_message(
                        &repository_id_clone,
                        &format!("Storing code relationships: {}/{} ({}%)...", stored, total, percent)
                    );
                })
            ) {
                log::error!("✗ Failed to store code relationships: {}", e);
            } else {
                log::info!("✓ Successfully stored {} code relationship(s)", all_code_relationships.len());
            }
        }
    }

    // Detect tests
    state.progress_tracker.update_progress(&repository_id, 9, "Detecting tests", "Scanning for test files and test functions...", None);
    log::info!("Step 9/11: Detecting tests...");
    log::info!("Scanning repository for test files (this may take a while for large repositories)...");
    let test_detector = TestDetector::new();
    let tests = match test_detector.detect_tests(&repo_path) {
        Ok(t) => {
            // Count test frameworks for better diagnostics
            use std::collections::HashMap;
            let mut framework_counts: HashMap<String, usize> = HashMap::new();
            let mut language_counts: HashMap<String, usize> = HashMap::new();
            for test in &t {
                *framework_counts.entry(format!("{:?}", test.test_framework)).or_insert(0) += 1;
                *language_counts.entry(test.language.clone()).or_insert(0) += 1;
            }
            let framework_summary: Vec<String> = framework_counts.iter()
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect();
            let language_summary: Vec<String> = language_counts.iter()
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect();
            log::info!("✓ Test detection complete: {} test(s) ({}), languages: {}", 
                t.len(), framework_summary.join(", "), language_summary.join(", "));
            state.progress_tracker.update_progress(&repository_id, 9, "Detecting tests", 
                format!("Found {} test(s) using {}", t.len(), framework_summary.join(", ")).as_str(),
                Some(serde_json::json!({
                    "tests": t.len(),
                    "frameworks": framework_counts.len()
                })));
            t
        },
        Err(e) => {
            log::error!("✗ Failed to detect tests: {}", e);
            // Don't fail the entire analysis if test detection fails
            log::warn!("⚠ Continuing analysis without test detection");
            Vec::new()
        }
    };

    // Store tests
    if !tests.is_empty() {
        log::info!("Storing {} test(s) in database...", tests.len());
        if let Err(e) = state.test_repo.store_tests(&repo.id, &tests) {
            log::warn!("⚠ Failed to store tests: {}", e);
            // Don't fail the entire analysis if test storage fails
        } else {
            log::info!("✓ Stored {} test(s)", tests.len());
        }
    } else {
        log::info!("✓ No tests detected");
    }

    // Analyze security configuration
    state.progress_tracker.update_progress(&repository_id, 10, "Analyzing security configuration", "Scanning configuration files and source code for security entities, API keys, and vulnerabilities...", None);
    log::info!("Step 10/11: Analyzing security configuration...");
    log::info!("Scanning repository for security entities (API keys, secrets, IAM roles, etc.)...");
    let security_analyzer = SecurityAnalyzer::new();
    let security_analysis = match security_analyzer.analyze_repository(&repo_path, Some(&code_structure), Some(&services)) {
        Ok(analysis) => {
            // Count entity types for better diagnostics
            use std::collections::HashMap;
            let mut entity_type_counts: HashMap<String, usize> = HashMap::new();
            for entity in &analysis.entities {
                *entity_type_counts.entry(format!("{:?}", entity.entity_type)).or_insert(0) += 1;
            }
            let entity_summary: Vec<String> = entity_type_counts.iter()
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect();
            log::info!("✓ Security analysis complete: {} entities ({}), {} relationships, {} vulnerabilities", 
                analysis.entities.len(), entity_summary.join(", "), analysis.relationships.len(), analysis.vulnerabilities.len());
            state.progress_tracker.update_progress(&repository_id, 10, "Analyzing security configuration", 
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
            return Err(anyhow::anyhow!("Failed to analyze security: {}", e));
        }
    };

    // Store security entities, relationships, and vulnerabilities
    // IMPORTANT: Delete in reverse dependency order to avoid foreign key constraint issues
    // Delete vulnerabilities and relationships first (they reference entities), then entities
    log::info!("Storing security data: {} entities, {} relationships, {} vulnerabilities...", 
        security_analysis.entities.len(), security_analysis.relationships.len(), security_analysis.vulnerabilities.len());
    
    // First, delete old vulnerabilities and relationships (they reference entities)
    log::info!("Clearing existing security data...");
    if let Err(e) = state.security_repo.store_vulnerabilities(&repo.id, &[]) {
        log::warn!("Failed to clear old vulnerabilities: {}", e);
    }
    if let Err(e) = state.security_repo.store_relationships(&repo.id, &[]) {
        log::warn!("Failed to clear old relationships: {}", e);
    }
    
    // Now store entities (they can be deleted safely)
    log::info!("Storing {} security entities...", security_analysis.entities.len());
    if let Err(e) = state.security_repo.store_entities(&repo.id, &security_analysis.entities) {
        log::error!("✗ Failed to store security entities: {}", e);
        return Err(anyhow::anyhow!("Failed to store security entities: {}", e));
    }
    log::info!("✓ Stored {} security entities", security_analysis.entities.len());

    // Now store relationships (entities exist now)
    log::info!("Storing {} security relationships...", security_analysis.relationships.len());
    if let Err(e) = state.security_repo.store_relationships(&repo.id, &security_analysis.relationships) {
        log::error!("✗ Failed to store security relationships: {}", e);
        return Err(anyhow::anyhow!("Failed to store security relationships: {}", e));
    }
    log::info!("✓ Stored {} security relationships", security_analysis.relationships.len());

    // Finally store vulnerabilities (entities exist now)
    log::info!("Storing {} security vulnerabilities...", security_analysis.vulnerabilities.len());
    if let Err(e) = state.security_repo.store_vulnerabilities(&repo.id, &security_analysis.vulnerabilities) {
        log::error!("✗ Failed to store security vulnerabilities: {}", e);
        return Err(anyhow::anyhow!("Failed to store security vulnerabilities: {}", e));
    }
    log::info!("✓ Stored {} security vulnerabilities", security_analysis.vulnerabilities.len());

    // Index documentation files (experimental - may be removed)
    state.progress_tracker.update_progress(&repository_id, 11, "Indexing developer documentation", "Scanning for README, API docs, and other documentation files...", None);
    log::info!("Step 11/11: Indexing developer documentation...");
    use crate::analysis::DocumentationIndexer;
    let doc_indexer = DocumentationIndexer::new();
    match doc_indexer.index_repository(&repo_path, &repo.id) {
        Ok(docs) => {
            log::info!("✓ Indexed {} documentation files", docs.len());
            state.progress_tracker.update_progress(&repository_id, 11, "Indexing developer documentation", 
                format!("Indexed {} documentation files", docs.len()).as_str(),
                Some(serde_json::json!({
                    "documentation_files": docs.len()
                })));
            
            // Store documentation
            if let Err(e) = state.documentation_repo.store_documentation(&docs) {
                log::warn!("⚠ Failed to store documentation: {}", e);
                // Don't fail the entire analysis if documentation storage fails
            } else {
                log::info!("✓ Stored {} documentation files", docs.len());
            }
        },
        Err(e) => {
            log::warn!("⚠ Failed to index documentation: {}", e);
            // Don't fail the entire analysis if documentation indexing fails
        }
    }

    // Update last analyzed timestamp
    log::info!("Updating repository timestamp...");
    if let Err(e) = state.repo_repo.update_last_analyzed(&repo.id) {
        log::error!("✗ Failed to update repository timestamp: {}", e);
        return Err(anyhow::anyhow!("Failed to update repository: {}", e));
    }

    // Mark analysis as complete
    state.progress_tracker.complete_analysis(&repository_id);
    
    log::info!("✓ Analysis complete for repository: {}", repo.name);
    Ok(AnalysisResult {
        message: "Repository analyzed successfully".to_string(),
        repository: serde_json::json!({
            "id": repo.id,
            "name": repo.name,
            "url": repo.url,
            "branch": repo.branch
        }),
        results: serde_json::json!({
            "manifests_found": manifests.len(),
            "total_dependencies": stored_deps,
            "services_found": services.len(),
            "graph_built": true,
            "code_elements_found": code_structure.elements.len(),
            "code_calls_found": code_structure.calls.len(),
            "security_entities_found": security_analysis.entities.len(),
            "security_relationships_found": security_analysis.relationships.len(),
            "security_vulnerabilities_found": security_analysis.vulnerabilities.len(),
            "tests_found": tests.len(),
            "documentation_indexed": true
        }),
    })
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
        // Global search - no repository_id filter (intentional cross-repo search)
        match state.dep_repo.get_by_package_name(package_name, None) {
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

