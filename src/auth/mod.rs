use anyhow::Result;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::fmt;
use uuid::Uuid;
use crate::storage::{ApiKeyRepository, UserRepository};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub api_key: String,
    pub refresh_token: String,
    pub expires_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub scopes: Vec<String>,
    pub expires_in_days: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct ApiKeyInfo {
    pub user_id: String,
    pub scopes: Vec<String>,
    pub rate_limit: u32,
}

pub struct AuthService {
    user_repo: UserRepository,
    api_key_repo: ApiKeyRepository,
    encryption_key: String,
}

impl AuthService {
    pub fn new(
        user_repo: UserRepository,
        api_key_repo: ApiKeyRepository,
        encryption_key: String,
    ) -> Self {
        AuthService {
            user_repo,
            api_key_repo,
            encryption_key,
        }
    }

    pub fn register(&self, req: RegisterRequest) -> Result<String> {
        // Hash password
        let password_hash = hash(&req.password, DEFAULT_COST)?;

        // Create user
        let user = self.user_repo.create_user(&req.email, &password_hash)?;

        // Generate initial API key
        let api_key = self.generate_api_key(&user.id, "default", vec!["read".to_string(), "write".to_string()], None)?;

        Ok(api_key)
    }

    pub fn login(&self, req: LoginRequest) -> Result<LoginResponse> {
        // Find user
        let (user, password_hash) = self.user_repo
            .find_by_email(&req.email)?
            .ok_or_else(|| anyhow::anyhow!("Invalid email or password"))?;

        // Verify password
        if !verify(&req.password, &password_hash)? {
            return Err(anyhow::anyhow!("Invalid email or password"));
        }

        // Generate API key
        let api_key = self.generate_api_key(&user.id, "login", vec!["read".to_string(), "write".to_string()], Some(Utc::now() + Duration::days(90)))?;
        let expires_at = Utc::now() + Duration::days(90);

        Ok(LoginResponse {
            api_key,
            refresh_token: Uuid::new_v4().to_string(), // TODO: Implement proper refresh tokens
            expires_at: expires_at.to_rfc3339(),
        })
    }

    pub fn create_api_key(
        &self,
        user_id: &str,
        req: CreateApiKeyRequest,
    ) -> Result<String> {
        let expires_at = req.expires_in_days.map(|days| Utc::now() + Duration::days(days as i64));
        self.generate_api_key(&user_id, &req.name, req.scopes, expires_at)
    }

    fn generate_api_key(
        &self,
        user_id: &str,
        name: &str,
        scopes: Vec<String>,
        expires_at: Option<chrono::DateTime<Utc>>,
    ) -> Result<String> {
        // Generate API key
        let key_value = format!("wl_live_{}", Uuid::new_v4().to_string().replace("-", ""));
        
        // Hash the key for storage
        let mut hasher = Sha256::new();
        hasher.update(&key_value);
        hasher.update(&self.encryption_key);
        let key_hash = format!("{:x}", hasher.finalize());

        // Store in database
        let rate_limit = if scopes.contains(&"admin".to_string()) {
            10000
        } else if scopes.contains(&"read".to_string()) && !scopes.contains(&"write".to_string()) {
            5000
        } else {
            1000
        };

        self.api_key_repo.create_api_key(
            user_id,
            &key_hash,
            name,
            scopes,
            rate_limit,
            expires_at,
        )?;

        Ok(key_value)
    }

    pub fn validate_api_key(&self, api_key: &str) -> Result<ApiKeyInfo> {
        // Hash the provided key
        let mut hasher = Sha256::new();
        hasher.update(api_key);
        hasher.update(&self.encryption_key);
        let key_hash = format!("{:x}", hasher.finalize());

        // Find API key
        let api_key_record = self.api_key_repo
            .find_by_key_hash(&key_hash)?
            .ok_or_else(|| anyhow::anyhow!("Invalid API key"))?;

        // Check expiration
        if let Some(expires_at) = api_key_record.expires_at {
            if Utc::now() > expires_at {
                return Err(anyhow::anyhow!("API key has expired"));
            }
        }

        // Check rate limit
        self.api_key_repo.reset_rate_limit_if_needed(&api_key_record.id)?;
        
        if api_key_record.requests_count >= api_key_record.rate_limit {
            return Err(anyhow::anyhow!("Rate limit exceeded"));
        }

        // Increment request count
        self.api_key_repo.increment_request_count(&api_key_record.id)?;

        Ok(ApiKeyInfo {
            user_id: api_key_record.user_id,
            scopes: api_key_record.scopes,
            rate_limit: api_key_record.rate_limit,
        })
    }
}

#[derive(Debug)]
pub enum AuthError {
    InvalidCredentials,
    InvalidApiKey,
    ExpiredApiKey,
    RateLimitExceeded,
    DatabaseError(String),
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AuthError::InvalidCredentials => write!(f, "Invalid email or password"),
            AuthError::InvalidApiKey => write!(f, "Invalid API key"),
            AuthError::ExpiredApiKey => write!(f, "API key has expired"),
            AuthError::RateLimitExceeded => write!(f, "Rate limit exceeded"),
            AuthError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
        }
    }
}

impl std::error::Error for AuthError {}

