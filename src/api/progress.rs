use actix_web::{web, HttpResponse, Responder, HttpRequest};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisProgress {
    pub repository_id: String,
    pub current_step: u32,
    pub total_steps: u32,
    pub step_name: String,
    pub progress_percent: f64,
    pub status_message: String,
    pub details: Option<serde_json::Value>,
    pub started_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
}

pub struct ProgressTracker {
    progress: Arc<Mutex<HashMap<String, AnalysisProgress>>>,
}

impl ProgressTracker {
    pub fn new() -> Self {
        ProgressTracker {
            progress: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn start_analysis(&self, repository_id: &str, total_steps: u32) {
        let mut progress_map = self.progress.lock().unwrap();
        progress_map.insert(
            repository_id.to_string(),
            AnalysisProgress {
                repository_id: repository_id.to_string(),
                current_step: 0,
                total_steps,
                step_name: "Starting...".to_string(),
                progress_percent: 0.0,
                status_message: "Initializing analysis".to_string(),
                details: None,
                started_at: Utc::now(),
                last_updated: Utc::now(),
            },
        );
    }

    pub fn update_progress(
        &self,
        repository_id: &str,
        current_step: u32,
        step_name: &str,
        status_message: &str,
        details: Option<serde_json::Value>,
    ) {
        let mut progress_map = self.progress.lock().unwrap();
        if let Some(progress) = progress_map.get_mut(repository_id) {
            progress.current_step = current_step;
            progress.step_name = step_name.to_string();
            progress.status_message = status_message.to_string();
            progress.details = details;
            progress.progress_percent = (current_step as f64 / progress.total_steps as f64) * 100.0;
            progress.last_updated = Utc::now();
        }
    }

    pub fn get_progress(&self, repository_id: &str) -> Option<AnalysisProgress> {
        let progress_map = self.progress.lock().unwrap();
        progress_map.get(repository_id).cloned()
    }

    pub fn complete_analysis(&self, repository_id: &str) {
        let mut progress_map = self.progress.lock().unwrap();
        if let Some(progress) = progress_map.get_mut(repository_id) {
            progress.current_step = progress.total_steps;
            progress.step_name = "Complete".to_string();
            progress.status_message = "Analysis completed successfully".to_string();
            progress.progress_percent = 100.0;
            progress.last_updated = Utc::now();
        }
    }

    pub fn fail_analysis(&self, repository_id: &str, error: &str) {
        let mut progress_map = self.progress.lock().unwrap();
        if let Some(progress) = progress_map.get_mut(repository_id) {
            progress.step_name = "Failed".to_string();
            progress.status_message = format!("Analysis failed: {}", error);
            progress.last_updated = Utc::now();
        }
    }

    pub fn clear_progress(&self, repository_id: &str) {
        let mut progress_map = self.progress.lock().unwrap();
        progress_map.remove(repository_id);
    }
}

impl Default for ProgressTracker {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn get_analysis_progress(
    state: web::Data<ProgressTracker>,
    _req: HttpRequest,
    path: web::Path<String>,
) -> impl Responder {
    let repository_id = path.into_inner();
    match state.get_progress(&repository_id) {
        Some(progress) => HttpResponse::Ok().json(progress),
        None => HttpResponse::NotFound().json(serde_json::json!({
            "error": "No analysis in progress for this repository"
        })),
    }
}

