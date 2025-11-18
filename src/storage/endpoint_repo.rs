use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;
use crate::storage::Database;
use rusqlite::params;
use crate::analysis::{DetectedEndpoint, HttpMethod};
use serde_json;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StoredEndpoint {
    pub id: String,
    pub repository_id: String,
    pub path: String,
    pub method: String,
    pub handler: Option<String>,
    pub file_path: String,
    pub line_number: Option<usize>,
    pub framework: Option<String>,
    pub middleware: Vec<String>,
    pub parameters: Vec<String>,
    pub created_at: String,
}

#[derive(Clone)]
pub struct EndpointRepository {
    db: Database,
}

impl EndpointRepository {
    pub fn new(db: Database) -> Self {
        EndpointRepository { db }
    }

    pub fn store_endpoints(&self, repository_id: &str, endpoints: &[DetectedEndpoint]) -> Result<()> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        // Delete existing endpoints for this repository
        conn.execute(
            "DELETE FROM endpoints WHERE repository_id = ?1",
            params![repository_id],
        )?;
        
        // Insert new endpoints
        let now = Utc::now();
        for endpoint in endpoints {
            let id = Uuid::new_v4().to_string();
            let method_str = self.method_to_string(&endpoint.method);
            let middleware_json = serde_json::to_string(&endpoint.middleware)?;
            let parameters_json = serde_json::to_string(&endpoint.parameters)?;
            
            conn.execute(
                "INSERT INTO endpoints 
                 (id, repository_id, path, method, handler, file_path, line_number, framework, middleware, parameters, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                params![
                    id,
                    repository_id,
                    endpoint.path,
                    method_str,
                    endpoint.handler,
                    endpoint.file_path,
                    endpoint.line_number.map(|n| n as i32),
                    endpoint.framework,
                    middleware_json,
                    parameters_json,
                    now.to_rfc3339()
                ],
            )?;
        }
        
        Ok(())
    }

    pub fn get_by_repository(&self, repository_id: &str) -> Result<Vec<StoredEndpoint>> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, repository_id, path, method, handler, file_path, line_number, framework, middleware, parameters, created_at
             FROM endpoints WHERE repository_id = ?1 ORDER BY path, method"
        )?;
        
        let endpoints = stmt.query_map(params![repository_id], |row| {
            let middleware_json: String = row.get(8)?;
            let parameters_json: String = row.get(9)?;
            
            Ok(StoredEndpoint {
                id: row.get(0)?,
                repository_id: row.get(1)?,
                path: row.get(2)?,
                method: row.get(3)?,
                handler: row.get(4)?,
                file_path: row.get(5)?,
                line_number: row.get::<_, Option<i32>>(6)?.map(|n| n as usize),
                framework: row.get(7)?,
                middleware: serde_json::from_str(&middleware_json).unwrap_or_default(),
                parameters: serde_json::from_str(&parameters_json).unwrap_or_default(),
                created_at: row.get(10)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(endpoints)
    }

    pub fn get_by_path(&self, path: &str, repository_id: Option<&str>) -> Result<Vec<StoredEndpoint>> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        let endpoints: Vec<StoredEndpoint> = if let Some(repo_id) = repository_id {
            let mut stmt = conn.prepare(
                "SELECT id, repository_id, path, method, handler, file_path, line_number, framework, middleware, parameters, created_at
                 FROM endpoints WHERE path LIKE ?1 AND repository_id = ?2 ORDER BY method"
            )?;
            let result: Result<Vec<_>, _> = stmt.query_map(params![format!("%{}%", path), repo_id], |row| {
                let middleware_json: String = row.get(8)?;
                let parameters_json: String = row.get(9)?;
                
                Ok(StoredEndpoint {
                    id: row.get(0)?,
                    repository_id: row.get(1)?,
                    path: row.get(2)?,
                    method: row.get(3)?,
                    handler: row.get(4)?,
                    file_path: row.get(5)?,
                    line_number: row.get::<_, Option<i32>>(6)?.map(|n| n as usize),
                    framework: row.get(7)?,
                    middleware: serde_json::from_str(&middleware_json).unwrap_or_default(),
                    parameters: serde_json::from_str(&parameters_json).unwrap_or_default(),
                    created_at: row.get(10)?,
                })
            })?.collect();
            result?
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, repository_id, path, method, handler, file_path, line_number, framework, middleware, parameters, created_at
                 FROM endpoints WHERE path LIKE ?1 ORDER BY repository_id, method"
            )?;
            let result: Result<Vec<_>, _> = stmt.query_map(params![format!("%{}%", path)], |row| {
                let middleware_json: String = row.get(8)?;
                let parameters_json: String = row.get(9)?;
                
                Ok(StoredEndpoint {
                    id: row.get(0)?,
                    repository_id: row.get(1)?,
                    path: row.get(2)?,
                    method: row.get(3)?,
                    handler: row.get(4)?,
                    file_path: row.get(5)?,
                    line_number: row.get::<_, Option<i32>>(6)?.map(|n| n as usize),
                    framework: row.get(7)?,
                    middleware: serde_json::from_str(&middleware_json).unwrap_or_default(),
                    parameters: serde_json::from_str(&parameters_json).unwrap_or_default(),
                    created_at: row.get(10)?,
                })
            })?.collect();
            result?
        };

        Ok(endpoints)
    }

    pub fn get_by_method(&self, method: &str, repository_id: Option<&str>) -> Result<Vec<StoredEndpoint>> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        let endpoints: Vec<StoredEndpoint> = if let Some(repo_id) = repository_id {
            let mut stmt = conn.prepare(
                "SELECT id, repository_id, path, method, handler, file_path, line_number, framework, middleware, parameters, created_at
                 FROM endpoints WHERE method = ?1 AND repository_id = ?2 ORDER BY path"
            )?;
            let result: Result<Vec<_>, _> = stmt.query_map(params![method, repo_id], |row| {
                let middleware_json: String = row.get(8)?;
                let parameters_json: String = row.get(9)?;
                
                Ok(StoredEndpoint {
                    id: row.get(0)?,
                    repository_id: row.get(1)?,
                    path: row.get(2)?,
                    method: row.get(3)?,
                    handler: row.get(4)?,
                    file_path: row.get(5)?,
                    line_number: row.get::<_, Option<i32>>(6)?.map(|n| n as usize),
                    framework: row.get(7)?,
                    middleware: serde_json::from_str(&middleware_json).unwrap_or_default(),
                    parameters: serde_json::from_str(&parameters_json).unwrap_or_default(),
                    created_at: row.get(10)?,
                })
            })?.collect();
            result?
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, repository_id, path, method, handler, file_path, line_number, framework, middleware, parameters, created_at
                 FROM endpoints WHERE method = ?1 ORDER BY repository_id, path"
            )?;
            let result: Result<Vec<_>, _> = stmt.query_map(params![method], |row| {
                let middleware_json: String = row.get(8)?;
                let parameters_json: String = row.get(9)?;
                
                Ok(StoredEndpoint {
                    id: row.get(0)?,
                    repository_id: row.get(1)?,
                    path: row.get(2)?,
                    method: row.get(3)?,
                    handler: row.get(4)?,
                    file_path: row.get(5)?,
                    line_number: row.get::<_, Option<i32>>(6)?.map(|n| n as usize),
                    framework: row.get(7)?,
                    middleware: serde_json::from_str(&middleware_json).unwrap_or_default(),
                    parameters: serde_json::from_str(&parameters_json).unwrap_or_default(),
                    created_at: row.get(10)?,
                })
            })?.collect();
            result?
        };

        Ok(endpoints)
    }

    pub fn get_by_framework(&self, framework: &str, repository_id: Option<&str>) -> Result<Vec<StoredEndpoint>> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        let endpoints: Vec<StoredEndpoint> = if let Some(repo_id) = repository_id {
            let mut stmt = conn.prepare(
                "SELECT id, repository_id, path, method, handler, file_path, line_number, framework, middleware, parameters, created_at
                 FROM endpoints WHERE framework = ?1 AND repository_id = ?2 ORDER BY path"
            )?;
            let result: Result<Vec<_>, _> = stmt.query_map(params![framework, repo_id], |row| {
                let middleware_json: String = row.get(8)?;
                let parameters_json: String = row.get(9)?;
                
                Ok(StoredEndpoint {
                    id: row.get(0)?,
                    repository_id: row.get(1)?,
                    path: row.get(2)?,
                    method: row.get(3)?,
                    handler: row.get(4)?,
                    file_path: row.get(5)?,
                    line_number: row.get::<_, Option<i32>>(6)?.map(|n| n as usize),
                    framework: row.get(7)?,
                    middleware: serde_json::from_str(&middleware_json).unwrap_or_default(),
                    parameters: serde_json::from_str(&parameters_json).unwrap_or_default(),
                    created_at: row.get(10)?,
                })
            })?.collect();
            result?
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, repository_id, path, method, handler, file_path, line_number, framework, middleware, parameters, created_at
                 FROM endpoints WHERE framework = ?1 ORDER BY repository_id, path"
            )?;
            let result: Result<Vec<_>, _> = stmt.query_map(params![framework], |row| {
                let middleware_json: String = row.get(8)?;
                let parameters_json: String = row.get(9)?;
                
                Ok(StoredEndpoint {
                    id: row.get(0)?,
                    repository_id: row.get(1)?,
                    path: row.get(2)?,
                    method: row.get(3)?,
                    handler: row.get(4)?,
                    file_path: row.get(5)?,
                    line_number: row.get::<_, Option<i32>>(6)?.map(|n| n as usize),
                    framework: row.get(7)?,
                    middleware: serde_json::from_str(&middleware_json).unwrap_or_default(),
                    parameters: serde_json::from_str(&parameters_json).unwrap_or_default(),
                    created_at: row.get(10)?,
                })
            })?.collect();
            result?
        };

        Ok(endpoints)
    }

    fn method_to_string(&self, method: &HttpMethod) -> String {
        match method {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Options => "OPTIONS",
            HttpMethod::Head => "HEAD",
            HttpMethod::Any => "ANY",
        }.to_string()
    }
}

