use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub security: SecurityConfig,
    pub storage: StorageConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub environment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub database_path: String,
    pub graph_db_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub api_key_encryption_key: String,
    pub jwt_secret: String,
    pub rate_limit_per_key: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub repository_cache_path: String,
    pub max_cache_size: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub log_level: String,
    pub log_format: String,
}

impl Config {
    pub fn from_env() -> Result<Self, anyhow::Error> {
        // Load .env.local first (local overrides), then .env
        dotenv::from_filename(".env.local").ok();
        dotenv::dotenv().ok();

        Ok(Config {
            server: ServerConfig {
                host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("PORT")
                    .unwrap_or_else(|_| "8080".to_string())
                    .parse()
                    .unwrap_or(8080),
                environment: env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()),
            },
            database: DatabaseConfig {
                database_path: env::var("DATABASE_PATH")
                    .unwrap_or_else(|_| "./data/wavelength.db".to_string()),
                graph_db_path: env::var("GRAPH_DB_PATH")
                    .unwrap_or_else(|_| "./data/graph.db".to_string()),
            },
            security: SecurityConfig {
                api_key_encryption_key: env::var("API_KEY_ENCRYPTION_KEY")
                    .expect("API_KEY_ENCRYPTION_KEY must be set"),
                jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
                rate_limit_per_key: env::var("RATE_LIMIT_PER_KEY")
                    .unwrap_or_else(|_| "1000".to_string())
                    .parse()
                    .unwrap_or(1000),
            },
            storage: StorageConfig {
                repository_cache_path: env::var("REPOSITORY_CACHE_PATH")
                    .unwrap_or_else(|_| "./cache/repos".to_string()),
                max_cache_size: env::var("MAX_CACHE_SIZE")
                    .unwrap_or_else(|_| "10GB".to_string()),
            },
            logging: LoggingConfig {
                log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
                log_format: env::var("LOG_FORMAT").unwrap_or_else(|_| "json".to_string()),
            },
        })
    }
}

