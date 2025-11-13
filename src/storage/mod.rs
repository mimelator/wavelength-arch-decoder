use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};
use crate::config::DatabaseConfig;

pub mod repositories;
pub mod repository_repo;
pub mod service_repo;
pub mod code_repo;
pub mod code_relationship_repo;
pub mod security_repo;
pub mod tool_repo;
pub mod documentation_repo;
pub mod test_repo;
// UserRepository and ApiKeyRepository kept for database schema but not exported (auth removed)
pub use repository_repo::{RepositoryRepository, DependencyRepository, Repository, StoredDependency};
pub use service_repo::{ServiceRepository, StoredService};
pub use code_repo::CodeElementRepository;
pub use code_relationship_repo::CodeRelationshipRepository;
pub use security_repo::SecurityRepository;
pub use tool_repo::ToolRepository;
pub use documentation_repo::DocumentationRepository;
pub use test_repo::TestRepository;

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
        
        // Migration: Drop auth tables if they exist (auth functionality removed)
        // Disable foreign keys temporarily to allow dropping tables
        conn.execute("PRAGMA foreign_keys = OFF", [])?;
        
        // Drop indexes first (if they exist)
        let _ = conn.execute("DROP INDEX IF EXISTS idx_api_keys_user_id", []);
        let _ = conn.execute("DROP INDEX IF EXISTS idx_api_keys_key_hash", []);
        
        // Drop tables (if they exist)
        let _ = conn.execute("DROP TABLE IF EXISTS api_keys", []);
        let _ = conn.execute("DROP TABLE IF EXISTS users", []);
        
        // Re-enable foreign keys
        conn.execute("PRAGMA foreign_keys = ON", [])?;

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

        // Migration: Add auth_type and auth_value columns if they don't exist
        // SQLite doesn't support ALTER TABLE ADD COLUMN IF NOT EXISTS, so we check first
        let table_info: Result<Vec<String>, _> = conn.prepare("PRAGMA table_info(repositories)")?
            .query_map([], |row| {
                Ok(row.get::<_, String>(1)?) // Column name is at index 1
            })?
            .collect();
        
        if let Ok(columns) = table_info {
            if !columns.iter().any(|c| c == "auth_type") {
                conn.execute("ALTER TABLE repositories ADD COLUMN auth_type TEXT", [])?;
            }
            if !columns.iter().any(|c| c == "auth_value") {
                conn.execute("ALTER TABLE repositories ADD COLUMN auth_value TEXT", [])?;
            }
        }

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

        // Tools table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tools (
                id TEXT PRIMARY KEY,
                repository_id TEXT NOT NULL,
                name TEXT NOT NULL,
                tool_type TEXT NOT NULL,
                category TEXT NOT NULL,
                version TEXT,
                file_path TEXT NOT NULL,
                line_number INTEGER,
                detection_method TEXT NOT NULL,
                configuration TEXT NOT NULL,
                confidence REAL NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (repository_id) REFERENCES repositories(id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Tool scripts table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tool_scripts (
                id TEXT PRIMARY KEY,
                tool_id TEXT NOT NULL,
                name TEXT NOT NULL,
                command TEXT NOT NULL,
                description TEXT,
                file_path TEXT NOT NULL,
                line_number INTEGER,
                created_at TEXT NOT NULL,
                FOREIGN KEY (tool_id) REFERENCES tools(id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Tool relationships table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tool_relationships (
                id TEXT PRIMARY KEY,
                tool_id TEXT NOT NULL,
                target_type TEXT NOT NULL,
                target_id TEXT NOT NULL,
                relationship_type TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (tool_id) REFERENCES tools(id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Code relationships table (links code elements to services and dependencies)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS code_relationships (
                id TEXT PRIMARY KEY,
                repository_id TEXT NOT NULL,
                code_element_id TEXT NOT NULL,
                target_type TEXT NOT NULL,
                target_id TEXT NOT NULL,
                relationship_type TEXT NOT NULL,
                confidence REAL NOT NULL,
                evidence TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (repository_id) REFERENCES repositories(id) ON DELETE CASCADE,
                FOREIGN KEY (code_element_id) REFERENCES code_elements(id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Documentation table (experimental - may be removed)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS documentation (
                id TEXT PRIMARY KEY,
                repository_id TEXT NOT NULL,
                file_path TEXT NOT NULL,
                file_name TEXT NOT NULL,
                doc_type TEXT NOT NULL,
                title TEXT,
                description TEXT,
                content_preview TEXT NOT NULL,
                word_count INTEGER NOT NULL,
                line_count INTEGER NOT NULL,
                has_code_examples INTEGER NOT NULL,
                has_api_references INTEGER NOT NULL,
                has_diagrams INTEGER NOT NULL,
                metadata TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (repository_id) REFERENCES repositories(id) ON DELETE CASCADE,
                UNIQUE(repository_id, file_path)
            )",
            [],
        )?;

        // Tests table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tests (
                id TEXT PRIMARY KEY,
                repository_id TEXT NOT NULL,
                name TEXT NOT NULL,
                test_framework TEXT NOT NULL,
                file_path TEXT NOT NULL,
                line_number INTEGER NOT NULL,
                language TEXT NOT NULL,
                test_type TEXT NOT NULL,
                suite_name TEXT,
                assertions TEXT NOT NULL,
                setup_methods TEXT NOT NULL,
                teardown_methods TEXT NOT NULL,
                signature TEXT,
                doc_comment TEXT,
                parameters TEXT NOT NULL,
                return_type TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY (repository_id) REFERENCES repositories(id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Create indexes
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
            "CREATE INDEX IF NOT EXISTS idx_tools_repository ON tools(repository_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tools_category ON tools(category)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tool_scripts_tool ON tool_scripts(tool_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tool_relationships_tool ON tool_relationships(tool_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tool_relationships_target ON tool_relationships(target_type, target_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_code_relationships_repository ON code_relationships(repository_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_code_relationships_element ON code_relationships(code_element_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_code_relationships_target ON code_relationships(target_type, target_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_documentation_repository ON documentation(repository_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_documentation_type ON documentation(doc_type)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tests_repository ON tests(repository_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tests_framework ON tests(test_framework)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tests_language ON tests(language)",
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

