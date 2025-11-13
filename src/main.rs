mod api;
mod config;
mod storage;
mod ingestion;
mod parsers;
mod analysis;
mod security;
mod graph;
mod graphql;
mod crawler;
mod report;

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
    
    // Check and log plugins at startup
    let plugin_dir = std::path::Path::new("config/plugins");
    if plugin_dir.exists() && plugin_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(plugin_dir) {
            let plugins: Vec<String> = entries.filter_map(|e| e.ok())
                .filter(|e| {
                    e.path().is_file() && 
                    e.path().extension().and_then(|s| s.to_str()) == Some("json")
                })
                .filter_map(|e| {
                    e.path().file_stem()
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_string())
                })
                .collect();
            
            if !plugins.is_empty() {
                info!("Found {} plugin(s) in config/plugins/: {}", plugins.len(), plugins.join(", "));
            }
        }
    }
    
    info!("Server will start on {}:{}", config.server.host, config.server.port);

    // Start server
    start_server(config).await
}
