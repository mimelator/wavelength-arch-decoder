use actix_web::{web, App, HttpServer};
use actix_files::Files;
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use crate::api::{ApiState, health, version};
use crate::graphql::GraphQLSchema;
use crate::api::repositories::{
    create_repository, list_repositories, get_repository,
    analyze_repository, get_dependencies, search_dependencies,
    delete_repository,
};
use crate::api::services::{get_services, search_services_by_provider};
use crate::api::tools::{get_tools, get_tool_scripts, search_tools};
use crate::api::graph::{get_graph, get_graph_statistics, get_node_neighbors};
use crate::api::code::{get_code_elements, get_code_calls, get_code_relationships};
use crate::api::security::{get_security_entities, get_security_relationships, get_security_vulnerabilities};
use crate::api::entity_details::get_entity_details;
use crate::api::jobs::{create_job, get_job_status, list_jobs, create_scheduled_job, batch_analyze};
use crate::api::progress::get_analysis_progress;
use crate::api::reports::generate_report;
use crate::crawler::webhooks::{handle_github_webhook, handle_gitlab_webhook};
use crate::config::Config;
use crate::storage::{Database, RepositoryRepository, DependencyRepository, ServiceRepository, CodeElementRepository, CodeRelationshipRepository, SecurityRepository, ToolRepository};
use crate::api::progress::ProgressTracker;
use std::sync::Arc;
use actix_web::HttpResponse;
use std::fs;

async fn index_handler() -> actix_web::Result<HttpResponse> {
    let content = fs::read_to_string("./static/index.html")
        .unwrap_or_else(|_| "<h1>UI not found</h1>".to_string());
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(content))
}

pub async fn start_server(config: Config) -> std::io::Result<()> {
    // Initialize database
    let db = Database::new(&config.database)
        .expect("Failed to initialize database");
    
    // Initialize repositories
    let repo_repo = RepositoryRepository::new(db.clone());
    let dep_repo = DependencyRepository::new(db.clone());
    let service_repo = ServiceRepository::new(db.clone());
    let code_repo = CodeElementRepository::new(db.clone());
    let code_relationship_repo = CodeRelationshipRepository::new(db.clone());
    let security_repo = SecurityRepository::new(db.clone());
    let tool_repo = ToolRepository::new(db.clone());
    
    // Initialize progress tracker
    let progress_tracker = Arc::new(ProgressTracker::new());
    
    // Create API state
    let api_state = web::Data::new(ApiState {
        repo_repo: repo_repo.clone(),
        dep_repo: dep_repo.clone(),
        service_repo: service_repo.clone(),
        code_repo: code_repo.clone(),
        code_relationship_repo: code_relationship_repo.clone(),
        security_repo: security_repo.clone(),
        tool_repo: tool_repo.clone(),
        progress_tracker: progress_tracker.clone(),
    });
    
    // Create progress tracker state for the progress endpoint
    let progress_state = web::Data::new(progress_tracker.clone());

    // Create GraphQL schema
    let schema = GraphQLSchema::build(
        crate::graphql::QueryRoot,
        crate::graphql::MutationRoot,
        async_graphql::EmptySubscription,
    )
    .data(api_state.clone())
    .finish();
    let schema = web::Data::new(schema);

    // GraphQL handler
    async fn graphql_handler(
        schema: web::Data<GraphQLSchema>,
        req: GraphQLRequest,
    ) -> GraphQLResponse {
        schema.execute(req.into_inner()).await.into()
    }
    
    // GraphiQL handler
    async fn graphiql_handler() -> impl actix_web::Responder {
        actix_web::HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(async_graphql::http::GraphiQLSource::build().endpoint("/graphql").finish())
    }

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(api_state.clone())
            .app_data(progress_state.clone())
            .app_data(schema.clone())
            .route("/health", web::get().to(health))
            .route("/graphql", web::post().to(graphql_handler))
            .route("/graphiql", web::get().to(graphiql_handler))
            // Serve static files for UI
            .service(Files::new("/static", "./static").show_files_listing())
            // Serve index.html for root path
            .service(web::resource("/").route(web::get().to(index_handler)))
            .service(
                web::scope("/api/v1")
                    // Health and version endpoints
                    .route("/version", web::get().to(version))
                    // Repository endpoints
                    .route("/repositories", web::post().to(create_repository))
                    .route("/repositories", web::get().to(list_repositories))
                    .route("/repositories/{id}", web::get().to(get_repository))
                    .route("/repositories/{id}", web::delete().to(delete_repository))
                    .route("/repositories/{id}/analyze", web::post().to(analyze_repository))
                    .route("/repositories/{id}/progress", web::get().to(get_analysis_progress))
                    .route("/repositories/{id}/dependencies", web::get().to(get_dependencies))
                    // Dependency search
                    .route("/dependencies/search", web::get().to(search_dependencies))
                    // Service endpoints
                    .route("/repositories/{id}/services", web::get().to(get_services))
                    .route("/services/search", web::get().to(search_services_by_provider))
                    // Tool endpoints
                    .route("/repositories/{id}/tools", web::get().to(get_tools))
                    .route("/repositories/{repo_id}/tools/{tool_id}/scripts", web::get().to(get_tool_scripts))
                    .route("/tools/search", web::get().to(search_tools))
                    // Graph endpoints
                    .route("/repositories/{id}/graph", web::get().to(get_graph))
                    .route("/repositories/{id}/graph/statistics", web::get().to(get_graph_statistics))
                    .route("/repositories/{id}/graph/nodes/{node_id}/neighbors", web::get().to(get_node_neighbors))
                    // Code structure endpoints
                    .route("/repositories/{id}/code/elements", web::get().to(get_code_elements))
                    .route("/repositories/{id}/code/calls", web::get().to(get_code_calls))
                    .route("/repositories/{id}/code/relationships", web::get().to(get_code_relationships))
                    // Security endpoints
                    .route("/repositories/{id}/security/entities", web::get().to(get_security_entities))
                    .route("/repositories/{id}/security/relationships", web::get().to(get_security_relationships))
                    .route("/repositories/{id}/security/vulnerabilities", web::get().to(get_security_vulnerabilities))
                    // Entity details endpoints
                    .route("/repositories/{repo_id}/entities/{entity_type}/{entity_id}", web::get().to(get_entity_details))
                    // Report endpoints
                    .route("/repositories/{id}/report", web::get().to(generate_report))
                    // Job endpoints (Phase 8)
                    .route("/jobs", web::post().to(create_job))
                    .route("/jobs", web::get().to(list_jobs))
                    .route("/jobs/{id}", web::get().to(get_job_status))
                    .route("/jobs/scheduled", web::post().to(create_scheduled_job))
                    .route("/jobs/batch", web::post().to(batch_analyze))
                    // Webhook endpoints (Phase 8)
                    .route("/webhooks/github", web::post().to(handle_github_webhook))
                    .route("/webhooks/gitlab", web::post().to(handle_gitlab_webhook))
            )
    })
    .bind(format!("{}:{}", config.server.host, config.server.port))?
    .run()
    .await
}

