use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};
use crate::config::DatabaseConfig;

pub mod repositories;
pub mod repository_repo;
pub mod service_repo;
pub mod code_repo;
pub mod security_repo;
pub use repositories::{UserRepository, ApiKeyRepository};
pub use repository_repo::{RepositoryRepository, DependencyRepository, Repository, StoredDependency};
pub use service_repo::{ServiceRepository, StoredService};
pub use code_repo::CodeElementRepository;
pub use security_repo::SecurityRepository;

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
                auth_type TEXT,
                auth_value TEXT,
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

        // Services table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS services (
                id TEXT PRIMARY KEY,
                repository_id TEXT NOT NULL,
                provider TEXT NOT NULL,
                service_type TEXT NOT NULL,
                name TEXT NOT NULL,
                configuration TEXT,
                file_path TEXT NOT NULL,
                line_number INTEGER,
                confidence REAL NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (repository_id) REFERENCES repositories(id)
            )",
            [],
        )?;

        // Code elements table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS code_elements (
                id TEXT PRIMARY KEY,
                repository_id TEXT NOT NULL,
                name TEXT NOT NULL,
                element_type TEXT NOT NULL,
                file_path TEXT NOT NULL,
                line_number INTEGER NOT NULL,
                language TEXT NOT NULL,
                signature TEXT,
                doc_comment TEXT,
                visibility TEXT,
                parameters TEXT,
                return_type TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY (repository_id) REFERENCES repositories(id)
            )",
            [],
        )?;

        // Code calls table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS code_calls (
                id TEXT PRIMARY KEY,
                repository_id TEXT NOT NULL,
                caller_id TEXT NOT NULL,
                callee_id TEXT NOT NULL,
                call_type TEXT NOT NULL,
                line_number INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (repository_id) REFERENCES repositories(id),
                FOREIGN KEY (caller_id) REFERENCES code_elements(id),
                FOREIGN KEY (callee_id) REFERENCES code_elements(id)
            )",
            [],
        )?;

        // Security entities table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS security_entities (
                id TEXT PRIMARY KEY,
                repository_id TEXT NOT NULL,
                entity_type TEXT NOT NULL,
                name TEXT NOT NULL,
                provider TEXT NOT NULL,
                configuration TEXT NOT NULL,
                file_path TEXT NOT NULL,
                line_number INTEGER,
                arn TEXT,
                region TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY (repository_id) REFERENCES repositories(id)
            )",
            [],
        )?;

        // Security relationships table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS security_relationships (
                id TEXT PRIMARY KEY,
                repository_id TEXT NOT NULL,
                source_entity_id TEXT NOT NULL,
                target_entity_id TEXT NOT NULL,
                relationship_type TEXT NOT NULL,
                permissions TEXT NOT NULL,
                condition TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY (repository_id) REFERENCES repositories(id),
                FOREIGN KEY (source_entity_id) REFERENCES security_entities(id),
                FOREIGN KEY (target_entity_id) REFERENCES security_entities(id)
            )",
            [],
        )?;

        // Security vulnerabilities table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS security_vulnerabilities (
                id TEXT PRIMARY KEY,
                repository_id TEXT NOT NULL,
                entity_id TEXT NOT NULL,
                vulnerability_type TEXT NOT NULL,
                severity TEXT NOT NULL,
                description TEXT NOT NULL,
                recommendation TEXT NOT NULL,
                file_path TEXT NOT NULL,
                line_number INTEGER,
                created_at TEXT NOT NULL,
                FOREIGN KEY (repository_id) REFERENCES repositories(id),
                FOREIGN KEY (entity_id) REFERENCES security_entities(id)
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
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_services_repository ON services(repository_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_services_provider ON services(provider)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_services_type ON services(service_type)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_code_elements_repository ON code_elements(repository_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_code_elements_type ON code_elements(element_type)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_code_elements_language ON code_elements(language)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_code_calls_repository ON code_calls(repository_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_code_calls_caller ON code_calls(caller_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_code_calls_callee ON code_calls(callee_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_security_entities_repository ON security_entities(repository_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_security_entities_type ON security_entities(entity_type)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_security_entities_provider ON security_entities(provider)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_security_relationships_repository ON security_relationships(repository_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_security_vulnerabilities_repository ON security_vulnerabilities(repository_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_security_vulnerabilities_severity ON security_vulnerabilities(severity)",
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

