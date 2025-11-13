use actix_web::{web, HttpResponse, Responder, HttpRequest};
use serde::Deserialize;
use crate::api::ApiState;
use log::info;

#[derive(Debug, Deserialize)]
pub struct GitHubWebhookPayload {
    pub action: Option<String>,
    pub repository: Option<GitHubRepository>,
    #[allow(dead_code)]
    pub pusher: Option<GitHubPusher>,
}

#[derive(Debug, Deserialize)]
pub struct GitHubRepository {
    #[allow(dead_code)]
    pub id: u64,
    #[allow(dead_code)]
    pub name: String,
    pub full_name: String,
    #[allow(dead_code)]
    pub html_url: String,
    #[allow(dead_code)]
    pub clone_url: String,
    #[allow(dead_code)]
    pub default_branch: String,
}

#[derive(Debug, Deserialize)]
pub struct GitHubPusher {
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct GitLabWebhookPayload {
    pub object_kind: String,
    pub project: Option<GitLabProject>,
    #[allow(dead_code)]
    pub user_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GitLabProject {
    #[allow(dead_code)]
    pub id: u64,
    #[allow(dead_code)]
    pub name: String,
    pub path_with_namespace: String,
    #[allow(dead_code)]
    pub web_url: String,
    #[allow(dead_code)]
    pub git_http_url: String,
    #[allow(dead_code)]
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

