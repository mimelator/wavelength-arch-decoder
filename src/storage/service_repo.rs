use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;
use crate::storage::Database;
use rusqlite::params;
use crate::security::{DetectedService, ServiceProvider, ServiceType};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StoredService {
    pub id: String,
    pub repository_id: String,
    pub provider: String,
    pub service_type: String,
    pub name: String,
    pub configuration: String,
    pub file_path: String,
    pub line_number: Option<usize>,
    pub confidence: f64,
    pub created_at: String,
}

#[derive(Clone)]
pub struct ServiceRepository {
    db: Database,
}

impl ServiceRepository {
    pub fn new(db: Database) -> Self {
        ServiceRepository { db }
    }

    pub fn store_services(&self, repository_id: &str, services: &[DetectedService]) -> Result<()> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        // Delete existing services for this repository
        conn.execute(
            "DELETE FROM services WHERE repository_id = ?1",
            params![repository_id],
        )?;
        
        // Insert new services
        let now = Utc::now();
        for service in services {
            let id = Uuid::new_v4().to_string();
            let provider_str = self.provider_to_string(&service.provider);
            let service_type_str = self.service_type_to_string(&service.service_type);
            let config_json = serde_json::to_string(&service.configuration)?;
            
            conn.execute(
                "INSERT INTO services 
                 (id, repository_id, provider, service_type, name, configuration, file_path, line_number, confidence, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    id,
                    repository_id,
                    provider_str,
                    service_type_str,
                    service.name,
                    config_json,
                    service.file_path,
                    service.line_number.map(|n| n as i32),
                    service.confidence,
                    now.to_rfc3339()
                ],
            )?;
        }
        
        Ok(())
    }

    pub fn get_by_repository(&self, repository_id: &str) -> Result<Vec<StoredService>> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        let mut stmt = conn.prepare(
            "SELECT id, repository_id, provider, service_type, name, configuration, file_path, line_number, confidence, created_at
             FROM services WHERE repository_id = ?1 ORDER BY provider, name"
        )?;
        
        let services = stmt.query_map(params![repository_id], |row| {
            Ok(StoredService {
                id: row.get(0)?,
                repository_id: row.get(1)?,
                provider: row.get(2)?,
                service_type: row.get(3)?,
                name: row.get(4)?,
                configuration: row.get(5)?,
                file_path: row.get(6)?,
                line_number: row.get::<_, Option<i32>>(7)?.map(|n| n as usize),
                confidence: row.get(8)?,
                created_at: row.get(9)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(services)
    }

    pub fn get_by_provider(&self, provider: &str, repository_id: Option<&str>) -> Result<Vec<StoredService>> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        let services: Vec<StoredService> = if let Some(repo_id) = repository_id {
            let mut stmt = conn.prepare(
                "SELECT id, repository_id, provider, service_type, name, configuration, file_path, line_number, confidence, created_at
                 FROM services WHERE provider = ?1 AND repository_id = ?2 ORDER BY repository_id"
            )?;
            let result: Result<Vec<_>, _> = stmt.query_map(params![provider, repo_id], |row| {
                Ok(StoredService {
                    id: row.get(0)?,
                    repository_id: row.get(1)?,
                    provider: row.get(2)?,
                    service_type: row.get(3)?,
                    name: row.get(4)?,
                    configuration: row.get(5)?,
                    file_path: row.get(6)?,
                    line_number: row.get::<_, Option<i32>>(7)?.map(|n| n as usize),
                    confidence: row.get(8)?,
                    created_at: row.get(9)?,
                })
            })?.collect();
            result?
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, repository_id, provider, service_type, name, configuration, file_path, line_number, confidence, created_at
                 FROM services WHERE provider = ?1 ORDER BY repository_id"
            )?;
            let result: Result<Vec<_>, _> = stmt.query_map(params![provider], |row| {
                Ok(StoredService {
                    id: row.get(0)?,
                    repository_id: row.get(1)?,
                    provider: row.get(2)?,
                    service_type: row.get(3)?,
                    name: row.get(4)?,
                    configuration: row.get(5)?,
                    file_path: row.get(6)?,
                    line_number: row.get::<_, Option<i32>>(7)?.map(|n| n as usize),
                    confidence: row.get(8)?,
                    created_at: row.get(9)?,
                })
            })?.collect();
            result?
        };

        Ok(services)
    }

    pub fn get_by_service_type(&self, service_type: &str, repository_id: Option<&str>) -> Result<Vec<StoredService>> {
        let conn = self.db.get_connection();
        let conn = conn.lock().unwrap();
        
        let services: Vec<StoredService> = if let Some(repo_id) = repository_id {
            let mut stmt = conn.prepare(
                "SELECT id, repository_id, provider, service_type, name, configuration, file_path, line_number, confidence, created_at
                 FROM services WHERE service_type = ?1 AND repository_id = ?2 ORDER BY provider"
            )?;
            let result: Result<Vec<_>, _> = stmt.query_map(params![service_type, repo_id], |row| {
                Ok(StoredService {
                    id: row.get(0)?,
                    repository_id: row.get(1)?,
                    provider: row.get(2)?,
                    service_type: row.get(3)?,
                    name: row.get(4)?,
                    configuration: row.get(5)?,
                    file_path: row.get(6)?,
                    line_number: row.get::<_, Option<i32>>(7)?.map(|n| n as usize),
                    confidence: row.get(8)?,
                    created_at: row.get(9)?,
                })
            })?.collect();
            result?
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, repository_id, provider, service_type, name, configuration, file_path, line_number, confidence, created_at
                 FROM services WHERE service_type = ?1 ORDER BY provider"
            )?;
            let result: Result<Vec<_>, _> = stmt.query_map(params![service_type], |row| {
                Ok(StoredService {
                    id: row.get(0)?,
                    repository_id: row.get(1)?,
                    provider: row.get(2)?,
                    service_type: row.get(3)?,
                    name: row.get(4)?,
                    configuration: row.get(5)?,
                    file_path: row.get(6)?,
                    line_number: row.get::<_, Option<i32>>(7)?.map(|n| n as usize),
                    confidence: row.get(8)?,
                    created_at: row.get(9)?,
                })
            })?.collect();
            result?
        };

        Ok(services)
    }

    fn provider_to_string(&self, provider: &ServiceProvider) -> String {
        match provider {
            ServiceProvider::Aws => "aws",
            ServiceProvider::Azure => "azure",
            ServiceProvider::Gcp => "gcp",
            ServiceProvider::Vercel => "vercel",
            ServiceProvider::Netlify => "netlify",
            ServiceProvider::Heroku => "heroku",
            ServiceProvider::DigitalOcean => "digitalocean",
            ServiceProvider::Clerk => "clerk",
            ServiceProvider::Auth0 => "auth0",
            ServiceProvider::Stripe => "stripe",
            ServiceProvider::Twilio => "twilio",
            ServiceProvider::SendGrid => "sendgrid",
            ServiceProvider::Mailgun => "mailgun",
            ServiceProvider::Slack => "slack",
            ServiceProvider::Discord => "discord",
            ServiceProvider::Postgres => "postgres",
            ServiceProvider::MySQL => "mysql",
            ServiceProvider::MongoDB => "mongodb",
            ServiceProvider::Redis => "redis",
            ServiceProvider::DynamoDB => "dynamodb",
            ServiceProvider::RDS => "rds",
            ServiceProvider::GitHub => "github",
            ServiceProvider::GitLab => "gitlab",
            ServiceProvider::Jira => "jira",
            ServiceProvider::Linear => "linear",
            ServiceProvider::Cloudflare => "cloudflare",
            ServiceProvider::CloudFront => "cloudfront",
            ServiceProvider::Datadog => "datadog",
            ServiceProvider::NewRelic => "newrelic",
            ServiceProvider::Sentry => "sentry",
            ServiceProvider::LogRocket => "logrocket",
            // AI Providers
            ServiceProvider::OpenAI => "openai",
            ServiceProvider::Anthropic => "anthropic",
            ServiceProvider::GitHubCopilot => "github_copilot",
            ServiceProvider::GoogleAI => "google_ai",
            ServiceProvider::Cohere => "cohere",
            ServiceProvider::HuggingFace => "huggingface",
            ServiceProvider::Replicate => "replicate",
            ServiceProvider::TogetherAI => "together_ai",
            ServiceProvider::MistralAI => "mistral_ai",
            ServiceProvider::Perplexity => "perplexity",
            ServiceProvider::WebMethods => "webmethods",
            ServiceProvider::Unknown => "unknown",
        }.to_string()
    }

    fn service_type_to_string(&self, service_type: &ServiceType) -> String {
        match service_type {
            ServiceType::CloudProvider => "cloud_provider",
            ServiceType::SaaS => "saas",
            ServiceType::Database => "database",
            ServiceType::Api => "api",
            ServiceType::Cdn => "cdn",
            ServiceType::Monitoring => "monitoring",
            ServiceType::Auth => "auth",
            ServiceType::Payment => "payment",
            ServiceType::AI => "ai",
            ServiceType::Other => "other",
        }.to_string()
    }
}

