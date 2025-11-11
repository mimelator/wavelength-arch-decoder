use actix_web::{web, App, HttpServer};
use crate::api::{ApiState, health, register, login, create_api_key};
use crate::api::repositories::{
    create_repository, list_repositories, get_repository,
    analyze_repository, get_dependencies, search_dependencies,
};
use crate::api::services::{get_services, search_services_by_provider};
use crate::api::graph::{get_graph, get_graph_statistics, get_node_neighbors};
use crate::api::code::{get_code_elements, get_code_calls};
use crate::auth::AuthService;
use crate::config::Config;
use crate::storage::{Database, UserRepository, ApiKeyRepository, RepositoryRepository, DependencyRepository, ServiceRepository, CodeElementRepository};

pub async fn start_server(config: Config) -> std::io::Result<()> {
    // Initialize database
    let db = Database::new(&config.database)
        .expect("Failed to initialize database");
    
    // Initialize repositories
    let user_repo = UserRepository::new(db.clone());
    let api_key_repo = ApiKeyRepository::new(db.clone());
    let repo_repo = RepositoryRepository::new(db.clone());
    let dep_repo = DependencyRepository::new(db.clone());
    let service_repo = ServiceRepository::new(db.clone());
    let code_repo = CodeElementRepository::new(db);
    
    // Initialize auth service
    let auth_service = AuthService::new(
        user_repo,
        api_key_repo,
        config.security.api_key_encryption_key.clone(),
    );
    
    // Create API state
    let api_state = web::Data::new(ApiState {
        auth_service,
        repo_repo,
        dep_repo,
        service_repo,
        code_repo,
    });

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(api_state.clone())
            .route("/health", web::get().to(health))
            .service(
                web::scope("/api/v1")
                    // Auth endpoints
                    .route("/auth/register", web::post().to(register))
                    .route("/auth/login", web::post().to(login))
                    .route("/auth/keys", web::post().to(create_api_key))
                    // Repository endpoints
                    .route("/repositories", web::post().to(create_repository))
                    .route("/repositories", web::get().to(list_repositories))
                    .route("/repositories/{id}", web::get().to(get_repository))
                    .route("/repositories/{id}/analyze", web::post().to(analyze_repository))
                    .route("/repositories/{id}/dependencies", web::get().to(get_dependencies))
                    // Dependency search
                    .route("/dependencies/search", web::get().to(search_dependencies))
                    // Service endpoints
                    .route("/repositories/{id}/services", web::get().to(get_services))
                    .route("/services/search", web::get().to(search_services_by_provider))
                    // Graph endpoints
                    .route("/repositories/{id}/graph", web::get().to(get_graph))
                    .route("/repositories/{id}/graph/statistics", web::get().to(get_graph_statistics))
                    .route("/repositories/{id}/graph/nodes/{node_id}/neighbors", web::get().to(get_node_neighbors))
                    // Code structure endpoints
                    .route("/repositories/{id}/code/elements", web::get().to(get_code_elements))
                    .route("/repositories/{id}/code/calls", web::get().to(get_code_calls))
            )
    })
    .bind(format!("{}:{}", config.server.host, config.server.port))?
    .run()
    .await
}

