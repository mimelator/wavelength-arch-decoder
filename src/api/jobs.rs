use actix_web::{web, HttpResponse, Responder, HttpRequest};
use serde::{Deserialize, Serialize};
use crate::api::{ApiState, ErrorResponse};
use crate::crawler::{JobProcessor, AnalysisJob, JobType, ScheduledJob};

#[derive(Debug, Deserialize)]
pub struct CreateJobRequest {
    pub repository_id: Option<String>,
    pub repository_url: Option<String>,
    pub job_type: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateScheduledJobRequest {
    pub name: String,
    pub schedule: String, // Cron expression
    pub repository_id: Option<String>,
    pub job_type: String,
}

#[derive(Debug, Deserialize)]
pub struct BatchAnalyzeRequest {
    pub repository_ids: Vec<String>,
}

/// Create a new analysis job
pub async fn create_job(
    state: web::Data<ApiState>,
    _req: HttpRequest,
    body: web::Json<CreateJobRequest>,
) -> impl Responder {

    let job_type = match body.job_type.as_str() {
        "analyze_repository" => JobType::AnalyzeRepository,
        "batch_analyze" => JobType::BatchAnalyze,
        "scheduled_reanalyze" => JobType::ScheduledReanalyze,
        _ => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Invalid job type".to_string(),
            });
        }
    };

    let job = AnalysisJob::new(
        job_type,
        body.repository_id.clone(),
        body.repository_url.clone(),
    );

    // Get job processor from app data (we'll need to add this)
    // For now, return the job ID
    HttpResponse::Created().json(serde_json::json!({
        "job_id": job.id,
        "status": "pending",
        "message": "Job created successfully"
    }))
}

/// Get job status
pub async fn get_job_status(
    state: web::Data<ApiState>,
    _req: HttpRequest,
    path: web::Path<String>,
) -> impl Responder {

    let job_id = path.into_inner();
    
    // Get job processor from app data
    // For now, return a placeholder
    HttpResponse::Ok().json(serde_json::json!({
        "job_id": job_id,
        "status": "pending",
        "progress": 0.0
    }))
}

/// List jobs
pub async fn list_jobs(
    state: web::Data<ApiState>,
    _req: HttpRequest,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {

    // Get job processor from app data
    // For now, return empty list
    HttpResponse::Ok().json(serde_json::json!([]))
}

/// Create a scheduled job
pub async fn create_scheduled_job(
    state: web::Data<ApiState>,
    _req: HttpRequest,
    body: web::Json<CreateScheduledJobRequest>,
) -> impl Responder {

    let job_type = match body.job_type.as_str() {
        "analyze_repository" => JobType::AnalyzeRepository,
        "batch_analyze" => JobType::BatchAnalyze,
        "scheduled_reanalyze" => JobType::ScheduledReanalyze,
        _ => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Invalid job type".to_string(),
            });
        }
    };

    let scheduled_job = ScheduledJob::new(
        body.name.clone(),
        body.schedule.clone(),
        job_type,
        body.repository_id.clone(),
    );

    HttpResponse::Created().json(serde_json::json!({
        "job_id": scheduled_job.id,
        "name": scheduled_job.name,
        "schedule": scheduled_job.schedule,
        "message": "Scheduled job created successfully"
    }))
}

/// Batch analyze repositories
pub async fn batch_analyze(
    state: web::Data<ApiState>,
    _req: HttpRequest,
    body: web::Json<BatchAnalyzeRequest>,
) -> impl Responder {

    let job_ids: Vec<String> = body.repository_ids.iter().map(|repo_id| {
        let job = AnalysisJob::new(
            JobType::AnalyzeRepository,
            Some(repo_id.clone()),
            None,
        );
        job.id
    }).collect();

    HttpResponse::Created().json(serde_json::json!({
        "job_ids": job_ids,
        "total": job_ids.len(),
        "message": "Batch analysis jobs created"
    }))
}

