use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::storage::Database;
use rusqlite::params;
use crate::analysis::{PackageDependency, PackageManager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub id: String,
    pub name: String,
    pub url: String,
    pub branch: String,
    pub last_analyzed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredDependency {
    pub id: String,
    pub repository_id: String,
    pub name: String,
    pub version: String,
    pub package_manager: String,
    pub is_dev: bool,
    pub is_optional: bool,
    pub file_path: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct RepositoryRepository {
    pub db: Database,
}

impl RepositoryRepository {
    pub fn new(db: Database) -> Self {
        RepositoryRepository { db }
    }

    pub fn create(&self, name: &str, url: &str, branch: Option<&str>) -> Result<Repository> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let branch = branch.unwrap_or("main");
        
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        conn.execute(
            "INSERT INTO repositories (id, name, url, branch, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, name, url, branch, now.to_rfc3339(), now.to_rfc3339()],
        )?;

        Ok(Repository {
            id,
            name: name.to_string(),
            url: url.to_string(),
            branch: branch.to_string(),
            last_analyzed_at: None,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn find_by_id(&self, id: &str) -> Result<Option<Repository>> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, name, url, branch, last_analyzed_at, created_at, updated_at
             FROM repositories WHERE id = ?1"
        )?;
        
        let repo_result = stmt.query_row(params![id], |row| {
            Ok(Repository {
                id: row.get(0)?,
                name: row.get(1)?,
                url: row.get(2)?,
                branch: row.get(3)?,
                last_analyzed_at: row.get::<_, Option<String>>(4)?
                    .map(|s| DateTime::parse_from_rfc3339(&s).unwrap().with_timezone(&Utc)),
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                    .unwrap()
                    .with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                    .unwrap()
                    .with_timezone(&Utc),
            })
        });

        match repo_result {
            Ok(repo) => Ok(Some(repo)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(anyhow::anyhow!("Database error: {}", e)),
        }
    }

    pub fn list_all(&self) -> Result<Vec<Repository>> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, name, url, branch, last_analyzed_at, created_at, updated_at
             FROM repositories ORDER BY created_at DESC"
        )?;
        
        let repos = stmt.query_map([], |row| {
            Ok(Repository {
                id: row.get(0)?,
                name: row.get(1)?,
                url: row.get(2)?,
                branch: row.get(3)?,
                last_analyzed_at: row.get::<_, Option<String>>(4)?
                    .map(|s| DateTime::parse_from_rfc3339(&s).unwrap().with_timezone(&Utc)),
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
                    .unwrap()
                    .with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                    .unwrap()
                    .with_timezone(&Utc),
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(repos)
    }

    pub fn update_last_analyzed(&self, id: &str) -> Result<()> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        conn.execute(
            "UPDATE repositories SET last_analyzed_at = ?1, updated_at = ?2 WHERE id = ?3",
            params![Utc::now().to_rfc3339(), Utc::now().to_rfc3339(), id],
        )?;
        
        Ok(())
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        conn.execute("DELETE FROM repositories WHERE id = ?1", params![id])?;
        
        Ok(())
    }
}

#[derive(Clone)]
pub struct DependencyRepository {
    db: Database,
}

impl DependencyRepository {
    pub fn new(db: Database) -> Self {
        DependencyRepository { db }
    }

    pub fn store_dependencies(&self, repository_id: &str, dependencies: &[PackageDependency], file_path: &str) -> Result<()> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        // Delete existing dependencies for this repository
        conn.execute(
            "DELETE FROM dependencies WHERE repository_id = ?1",
            params![repository_id],
        )?;
        
        // Insert new dependencies
        let now = Utc::now();
        for dep in dependencies {
            let id = Uuid::new_v4().to_string();
            let package_manager_str = match dep.package_manager {
                PackageManager::Npm => "npm",
                PackageManager::Pip => "pip",
                PackageManager::Cargo => "cargo",
                PackageManager::Maven => "maven",
                PackageManager::Gradle => "gradle",
                PackageManager::Go => "go",
                PackageManager::Composer => "composer",
                PackageManager::NuGet => "nuget",
            };
            
            conn.execute(
                "INSERT INTO dependencies 
                 (id, repository_id, name, version, package_manager, is_dev, is_optional, file_path, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    id,
                    repository_id,
                    dep.name,
                    dep.version,
                    package_manager_str,
                    dep.is_dev as i32,
                    dep.is_optional as i32,
                    file_path,
                    now.to_rfc3339()
                ],
            )?;
        }
        
        Ok(())
    }

    pub fn get_by_repository(&self, repository_id: &str) -> Result<Vec<StoredDependency>> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, repository_id, name, version, package_manager, is_dev, is_optional, file_path, created_at
             FROM dependencies WHERE repository_id = ?1 ORDER BY package_manager, name"
        )?;
        
        let deps = stmt.query_map(params![repository_id], |row| {
            Ok(StoredDependency {
                id: row.get(0)?,
                repository_id: row.get(1)?,
                name: row.get(2)?,
                version: row.get(3)?,
                package_manager: row.get(4)?,
                is_dev: row.get::<_, i32>(5)? != 0,
                is_optional: row.get::<_, i32>(6)? != 0,
                file_path: row.get(7)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                    .unwrap()
                    .with_timezone(&Utc),
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(deps)
    }

    pub fn get_by_package_name(&self, name: &str) -> Result<Vec<StoredDependency>> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, repository_id, name, version, package_manager, is_dev, is_optional, file_path, created_at
             FROM dependencies WHERE name = ?1 ORDER BY repository_id"
        )?;
        
        let deps = stmt.query_map(params![name], |row| {
            Ok(StoredDependency {
                id: row.get(0)?,
                repository_id: row.get(1)?,
                name: row.get(2)?,
                version: row.get(3)?,
                package_manager: row.get(4)?,
                is_dev: row.get::<_, i32>(5)? != 0,
                is_optional: row.get::<_, i32>(6)? != 0,
                file_path: row.get(7)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                    .unwrap()
                    .with_timezone(&Utc),
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(deps)
    }
}

