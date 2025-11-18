use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;
use crate::storage::Database;
use rusqlite::params;
use crate::analysis::{DetectedPort, PortType};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StoredPort {
    pub id: String,
    pub repository_id: String,
    pub port: u16,
    pub port_type: String,
    pub context: String,
    pub file_path: String,
    pub line_number: Option<usize>,
    pub framework: Option<String>,
    pub environment: Option<String>,
    pub is_config: bool,
    pub created_at: String,
}

#[derive(Clone)]
pub struct PortRepository {
    db: Database,
}

impl PortRepository {
    pub fn new(db: Database) -> Self {
        PortRepository { db }
    }

    pub fn store_ports(&self, repository_id: &str, ports: &[DetectedPort]) -> Result<()> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        // Delete existing ports for this repository
        conn.execute(
            "DELETE FROM ports WHERE repository_id = ?1",
            params![repository_id],
        )?;
        
        // Insert new ports
        let now = Utc::now();
        for port in ports {
            let id = Uuid::new_v4().to_string();
            let port_type_str = self.port_type_to_string(&port.port_type);
            
            conn.execute(
                "INSERT INTO ports 
                 (id, repository_id, port, port_type, context, file_path, line_number, framework, environment, is_config, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    id,
                    repository_id,
                    port.port as i32,
                    port_type_str,
                    port.context,
                    port.file_path,
                    port.line_number.map(|n| n as i32),
                    port.framework,
                    port.environment,
                    if port.is_config { 1 } else { 0 },
                    now.to_rfc3339()
                ],
            )?;
        }
        
        Ok(())
    }

    pub fn get_by_repository(&self, repository_id: &str) -> Result<Vec<StoredPort>> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, repository_id, port, port_type, context, file_path, line_number, framework, environment, is_config, created_at
             FROM ports WHERE repository_id = ?1 ORDER BY port"
        )?;
        
        let ports = stmt.query_map(params![repository_id], |row| {
            Ok(StoredPort {
                id: row.get(0)?,
                repository_id: row.get(1)?,
                port: row.get::<_, i32>(2)? as u16,
                port_type: row.get(3)?,
                context: row.get(4)?,
                file_path: row.get(5)?,
                line_number: row.get::<_, Option<i32>>(6)?.map(|n| n as usize),
                framework: row.get(7)?,
                environment: row.get(8)?,
                is_config: row.get::<_, i32>(9)? != 0,
                created_at: row.get(10)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(ports)
    }

    pub fn get_by_port(&self, port: u16, repository_id: Option<&str>) -> Result<Vec<StoredPort>> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        let ports: Vec<StoredPort> = if let Some(repo_id) = repository_id {
            let mut stmt = conn.prepare(
                "SELECT id, repository_id, port, port_type, context, file_path, line_number, framework, environment, is_config, created_at
                 FROM ports WHERE port = ?1 AND repository_id = ?2 ORDER BY repository_id"
            )?;
            let result: Result<Vec<_>, _> = stmt.query_map(params![port as i32, repo_id], |row| {
                Ok(StoredPort {
                    id: row.get(0)?,
                    repository_id: row.get(1)?,
                    port: row.get::<_, i32>(2)? as u16,
                    port_type: row.get(3)?,
                    context: row.get(4)?,
                    file_path: row.get(5)?,
                    line_number: row.get::<_, Option<i32>>(6)?.map(|n| n as usize),
                    framework: row.get(7)?,
                    environment: row.get(8)?,
                    is_config: row.get::<_, i32>(9)? != 0,
                    created_at: row.get(10)?,
                })
            })?.collect();
            result?
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, repository_id, port, port_type, context, file_path, line_number, framework, environment, is_config, created_at
                 FROM ports WHERE port = ?1 ORDER BY repository_id"
            )?;
            let result: Result<Vec<_>, _> = stmt.query_map(params![port as i32], |row| {
                Ok(StoredPort {
                    id: row.get(0)?,
                    repository_id: row.get(1)?,
                    port: row.get::<_, i32>(2)? as u16,
                    port_type: row.get(3)?,
                    context: row.get(4)?,
                    file_path: row.get(5)?,
                    line_number: row.get::<_, Option<i32>>(6)?.map(|n| n as usize),
                    framework: row.get(7)?,
                    environment: row.get(8)?,
                    is_config: row.get::<_, i32>(9)? != 0,
                    created_at: row.get(10)?,
                })
            })?.collect();
            result?
        };

        Ok(ports)
    }

    pub fn get_by_type(&self, port_type: &str, repository_id: Option<&str>) -> Result<Vec<StoredPort>> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        let ports: Vec<StoredPort> = if let Some(repo_id) = repository_id {
            let mut stmt = conn.prepare(
                "SELECT id, repository_id, port, port_type, context, file_path, line_number, framework, environment, is_config, created_at
                 FROM ports WHERE port_type = ?1 AND repository_id = ?2 ORDER BY port"
            )?;
            let result: Result<Vec<_>, _> = stmt.query_map(params![port_type, repo_id], |row| {
                Ok(StoredPort {
                    id: row.get(0)?,
                    repository_id: row.get(1)?,
                    port: row.get::<_, i32>(2)? as u16,
                    port_type: row.get(3)?,
                    context: row.get(4)?,
                    file_path: row.get(5)?,
                    line_number: row.get::<_, Option<i32>>(6)?.map(|n| n as usize),
                    framework: row.get(7)?,
                    environment: row.get(8)?,
                    is_config: row.get::<_, i32>(9)? != 0,
                    created_at: row.get(10)?,
                })
            })?.collect();
            result?
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, repository_id, port, port_type, context, file_path, line_number, framework, environment, is_config, created_at
                 FROM ports WHERE port_type = ?1 ORDER BY repository_id, port"
            )?;
            let result: Result<Vec<_>, _> = stmt.query_map(params![port_type], |row| {
                Ok(StoredPort {
                    id: row.get(0)?,
                    repository_id: row.get(1)?,
                    port: row.get::<_, i32>(2)? as u16,
                    port_type: row.get(3)?,
                    context: row.get(4)?,
                    file_path: row.get(5)?,
                    line_number: row.get::<_, Option<i32>>(6)?.map(|n| n as usize),
                    framework: row.get(7)?,
                    environment: row.get(8)?,
                    is_config: row.get::<_, i32>(9)? != 0,
                    created_at: row.get(10)?,
                })
            })?.collect();
            result?
        };

        Ok(ports)
    }

    fn port_type_to_string(&self, port_type: &PortType) -> String {
        match port_type {
            PortType::HttpServer => "http_server",
            PortType::Database => "database",
            PortType::MessageQueue => "message_queue",
            PortType::Cache => "cache",
            PortType::Other => "other",
        }.to_string()
    }
}

