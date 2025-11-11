mod api;
mod auth;
mod config;
mod storage;
mod ingestion;
mod parsers;

use api::server::start_server;
use config::Config;
use log::info;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    info!("Starting Wavelength Architecture Decoder...");

    // Load configuration
    let config = Config::from_env()
        .expect("Failed to load configuration. Please check your environment variables.");

    info!("Configuration loaded successfully");
    info!("Server will start on {}:{}", config.server.host, config.server.port);

    // Start server
    start_server(config).await
}
