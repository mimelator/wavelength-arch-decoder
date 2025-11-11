use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::storage::Database;
use rusqlite::params;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub scopes: Vec<String>,
    pub rate_limit: u32,
    pub requests_count: u32,
    pub last_reset_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

pub struct UserRepository {
    db: Database,
}

impl UserRepository {
    pub fn new(db: Database) -> Self {
        UserRepository { db }
    }

    pub fn create_user(&self, email: &str, password_hash: &str) -> Result<User> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        conn.execute(
            "INSERT INTO users (id, email, password_hash, created_at, updated_at) 
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, email, password_hash, now.to_rfc3339(), now.to_rfc3339()],
        )?;

        Ok(User {
            id,
            email: email.to_string(),
            created_at: now,
            updated_at: now,
        })
    }

    pub fn find_by_email(&self, email: &str) -> Result<Option<(User, String)>> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, email, password_hash, created_at, updated_at FROM users WHERE email = ?1"
        )?;
        
        let user_result = stmt.query_row(params![email], |row| {
            Ok((
                User {
                    id: row.get(0)?,
                    email: row.get(1)?,
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                        .unwrap()
                        .with_timezone(&Utc),
                    updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                        .unwrap()
                        .with_timezone(&Utc),
                },
                row.get::<_, String>(2)?,
            ))
        });

        match user_result {
            Ok((user, password_hash)) => Ok(Some((user, password_hash))),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(anyhow::anyhow!("Database error: {}", e)),
        }
    }
}

pub struct ApiKeyRepository {
    db: Database,
}

impl ApiKeyRepository {
    pub fn new(db: Database) -> Self {
        ApiKeyRepository { db }
    }

    pub fn create_api_key(
        &self,
        user_id: &str,
        key_hash: &str,
        name: &str,
        scopes: Vec<String>,
        rate_limit: u32,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<ApiKey> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let scopes_json = serde_json::to_string(&scopes)?;
        
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        conn.execute(
            "INSERT INTO api_keys 
             (id, user_id, key_hash, name, scopes, rate_limit, requests_count, last_reset_at, expires_at, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, ?7, ?8, ?9)",
            params![
                id,
                user_id,
                key_hash,
                name,
                scopes_json,
                rate_limit,
                now.to_rfc3339(),
                expires_at.map(|d| d.to_rfc3339()),
                now.to_rfc3339()
            ],
        )?;

        Ok(ApiKey {
            id,
            user_id: user_id.to_string(),
            name: name.to_string(),
            scopes,
            rate_limit,
            requests_count: 0,
            last_reset_at: now,
            expires_at,
            created_at: now,
        })
    }

    pub fn find_by_key_hash(&self, key_hash: &str) -> Result<Option<ApiKey>> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, user_id, name, scopes, rate_limit, requests_count, last_reset_at, expires_at, created_at
             FROM api_keys WHERE key_hash = ?1"
        )?;
        
        let key_result = stmt.query_row(params![key_hash], |row| {
            let scopes_json: String = row.get(3)?;
            let scopes: Vec<String> = serde_json::from_str(&scopes_json).unwrap_or_default();
            
            Ok(ApiKey {
                id: row.get(0)?,
                user_id: row.get(1)?,
                name: row.get(2)?,
                scopes,
                rate_limit: row.get(4)?,
                requests_count: row.get(5)?,
                last_reset_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                    .unwrap()
                    .with_timezone(&Utc),
                expires_at: row.get::<_, Option<String>>(7)?
                    .map(|s| DateTime::parse_from_rfc3339(&s).unwrap().with_timezone(&Utc)),
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                    .unwrap()
                    .with_timezone(&Utc),
            })
        });

        match key_result {
            Ok(key) => Ok(Some(key)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(anyhow::anyhow!("Database error: {}", e)),
        }
    }

    pub fn increment_request_count(&self, key_id: &str) -> Result<()> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        conn.execute(
            "UPDATE api_keys SET requests_count = requests_count + 1 WHERE id = ?1",
            params![key_id],
        )?;
        
        Ok(())
    }

    pub fn reset_rate_limit_if_needed(&self, key_id: &str) -> Result<()> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        // Reset if last reset was more than an hour ago
        let now = Utc::now();
        conn.execute(
            "UPDATE api_keys 
             SET requests_count = 0, last_reset_at = ?1 
             WHERE id = ?2 AND datetime(last_reset_at) < datetime(?1, '-1 hour')",
            params![now.to_rfc3339(), key_id],
        )?;
        
        Ok(())
    }
}

