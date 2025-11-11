use actix_web::{web, HttpResponse, Responder, HttpRequest};
use serde::{Deserialize, Serialize};
use crate::api::{ApiState, ErrorResponse};
use log::info;

#[derive(Debug, Deserialize)]
pub struct GitHubWebhookPayload {
    pub action: Option<String>,
    pub repository: Option<GitHubRepository>,
    pub pusher: Option<GitHubPusher>,
}

#[derive(Debug, Deserialize)]
pub struct GitHubRepository {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub html_url: String,
    pub clone_url: String,
    pub default_branch: String,
}

#[derive(Debug, Deserialize)]
pub struct GitHubPusher {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct GitLabWebhookPayload {
    pub object_kind: String,
    pub project: Option<GitLabProject>,
    pub user_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GitLabProject {
    pub id: u64,
    pub name: String,
    pub path_with_namespace: String,
    pub web_url: String,
    pub git_http_url: String,
    pub default_branch: String,
}

/// Handle GitHub webhook
pub async fn handle_github_webhook(
    _state: web::Data<ApiState>,
    _req: HttpRequest,
    body: web::Json<GitHubWebhookPayload>,
) -> impl Responder {
    // TODO: Verify webhook signature
    
    info!("Received GitHub webhook: {:?}", body.action);
    
    if let Some(repo) = &body.repository {
        if let Some(action) = &body.action {
            if action == "push" {
                info!("Push event for repository: {}", repo.full_name);
                // TODO: Trigger repository analysis
                return HttpResponse::Ok().json(serde_json::json!({
                    "message": "Webhook received",
                    "repository": repo.full_name
                }));
            }
        }
    }
    
    HttpResponse::Ok().json(serde_json::json!({
        "message": "Webhook received"
    }))
}

/// Handle GitLab webhook
pub async fn handle_gitlab_webhook(
    _state: web::Data<ApiState>,
    _req: HttpRequest,
    body: web::Json<GitLabWebhookPayload>,
) -> impl Responder {
    // TODO: Verify webhook signature
    
    info!("Received GitLab webhook: {}", body.object_kind);
    
    if let Some(project) = &body.project {
        if body.object_kind == "push" {
            info!("Push event for project: {}", project.path_with_namespace);
            // TODO: Trigger repository analysis
            return HttpResponse::Ok().json(serde_json::json!({
                "message": "Webhook received",
                "project": project.path_with_namespace
            }));
        }
    }
    
    HttpResponse::Ok().json(serde_json::json!({
        "message": "Webhook received"
    }))
}

