use actix_web::{web, App, HttpServer};
use crate::api::{ApiState, health, register, login, create_api_key};
use crate::auth::AuthService;
use crate::config::Config;
use crate::storage::{Database, UserRepository, ApiKeyRepository};

pub async fn start_server(config: Config) -> std::io::Result<()> {
    // Initialize database
    let db = Database::new(&config.database)
        .expect("Failed to initialize database");
    
    // Initialize repositories
    let user_repo = UserRepository::new(db.clone());
    let api_key_repo = ApiKeyRepository::new(db);
    
    // Initialize auth service
    let auth_service = AuthService::new(
        user_repo,
        api_key_repo,
        config.security.api_key_encryption_key.clone(),
    );
    
    // Create API state
    let api_state = web::Data::new(ApiState {
        auth_service,
    });

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(api_state.clone())
            .route("/health", web::get().to(health))
            .service(
                web::scope("/api/v1")
                    .route("/auth/register", web::post().to(register))
                    .route("/auth/login", web::post().to(login))
                    .route("/auth/keys", web::post().to(create_api_key))
            )
    })
    .bind(format!("{}:{}", config.server.host, config.server.port))?
    .run()
    .await
}

