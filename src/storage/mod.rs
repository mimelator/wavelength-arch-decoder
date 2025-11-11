use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};
use crate::config::DatabaseConfig;

pub mod repositories;
pub mod repository_repo;
pub use repositories::{UserRepository, ApiKeyRepository};
pub use repository_repo::{RepositoryRepository, DependencyRepository, Repository, StoredDependency};

#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn new(config: &DatabaseConfig) -> Result<Self> {
        // Ensure data directory exists
        if let Some(parent) = Path::new(&config.database_path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&config.database_path)?;
        let db = Database {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        // Users table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                email TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        // API keys table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS api_keys (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                key_hash TEXT UNIQUE NOT NULL,
                name TEXT NOT NULL,
                scopes TEXT NOT NULL,
                rate_limit INTEGER NOT NULL,
                requests_count INTEGER DEFAULT 0,
                last_reset_at TEXT NOT NULL,
                expires_at TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users(id)
            )",
            [],
        )?;

        // Repositories table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS repositories (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                url TEXT NOT NULL,
                branch TEXT DEFAULT 'main',
                last_analyzed_at TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        // Graph nodes table (for knowledge graph)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS graph_nodes (
                id TEXT PRIMARY KEY,
                repository_id TEXT,
                node_type TEXT NOT NULL,
                name TEXT NOT NULL,
                properties TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY (repository_id) REFERENCES repositories(id)
            )",
            [],
        )?;

        // Graph edges table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS graph_edges (
                id TEXT PRIMARY KEY,
                source_node_id TEXT NOT NULL,
                target_node_id TEXT NOT NULL,
                edge_type TEXT NOT NULL,
                properties TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY (source_node_id) REFERENCES graph_nodes(id),
                FOREIGN KEY (target_node_id) REFERENCES graph_nodes(id)
            )",
            [],
        )?;

        // Dependencies table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS dependencies (
                id TEXT PRIMARY KEY,
                repository_id TEXT NOT NULL,
                name TEXT NOT NULL,
                version TEXT NOT NULL,
                package_manager TEXT NOT NULL,
                is_dev INTEGER NOT NULL DEFAULT 0,
                is_optional INTEGER NOT NULL DEFAULT 0,
                file_path TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (repository_id) REFERENCES repositories(id)
            )",
            [],
        )?;

        // Create indexes
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_api_keys_user_id ON api_keys(user_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_api_keys_key_hash ON api_keys(key_hash)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_graph_nodes_repository ON graph_nodes(repository_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_graph_edges_source ON graph_edges(source_node_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_graph_edges_target ON graph_edges(target_node_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_dependencies_repository ON dependencies(repository_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_dependencies_name ON dependencies(name)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_dependencies_package_manager ON dependencies(package_manager)",
            [],
        )?;

        Ok(())
    }

    pub fn get_connection(&self) -> Arc<Mutex<Connection>> {
        self.conn.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_database_initialization() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let config = DatabaseConfig {
            database_path: db_path.to_str().unwrap().to_string(),
            graph_db_path: temp_dir.path().join("graph.db").to_str().unwrap().to_string(),
        };
        
        let db = Database::new(&config).unwrap();
        assert!(db_path.exists());
    }
}

