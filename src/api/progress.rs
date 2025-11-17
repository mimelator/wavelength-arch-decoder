use actix_web::{web, HttpResponse, Responder, HttpRequest};
use serde::{Deserialize, Serialize, Serializer, Deserializer};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
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

// Custom serialization to handle DateTime properly
impl Serialize for AnalysisProgress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("AnalysisProgress", 9)?;
        state.serialize_field("repository_id", &self.repository_id)?;
        state.serialize_field("current_step", &self.current_step)?;
        state.serialize_field("total_steps", &self.total_steps)?;
        state.serialize_field("step_name", &self.step_name)?;
        state.serialize_field("progress_percent", &self.progress_percent)?;
        state.serialize_field("status_message", &self.status_message)?;
        state.serialize_field("details", &self.details)?;
        state.serialize_field("started_at", &self.started_at.to_rfc3339())?;
        state.serialize_field("last_updated", &self.last_updated.to_rfc3339())?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for AnalysisProgress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        struct AnalysisProgressVisitor;

        impl<'de> Visitor<'de> for AnalysisProgressVisitor {
            type Value = AnalysisProgress;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct AnalysisProgress")
            }

            fn visit_map<V>(self, mut map: V) -> Result<AnalysisProgress, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut repository_id = None;
                let mut current_step = None;
                let mut total_steps = None;
                let mut step_name = None;
                let mut progress_percent = None;
                let mut status_message = None;
                let mut details = None;
                let mut started_at = None;
                let mut last_updated = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        "repository_id" => {
                            if repository_id.is_some() {
                                return Err(de::Error::duplicate_field("repository_id"));
                            }
                            repository_id = Some(map.next_value()?);
                        }
                        "current_step" => {
                            if current_step.is_some() {
                                return Err(de::Error::duplicate_field("current_step"));
                            }
                            current_step = Some(map.next_value()?);
                        }
                        "total_steps" => {
                            if total_steps.is_some() {
                                return Err(de::Error::duplicate_field("total_steps"));
                            }
                            total_steps = Some(map.next_value()?);
                        }
                        "step_name" => {
                            if step_name.is_some() {
                                return Err(de::Error::duplicate_field("step_name"));
                            }
                            step_name = Some(map.next_value()?);
                        }
                        "progress_percent" => {
                            if progress_percent.is_some() {
                                return Err(de::Error::duplicate_field("progress_percent"));
                            }
                            progress_percent = Some(map.next_value()?);
                        }
                        "status_message" => {
                            if status_message.is_some() {
                                return Err(de::Error::duplicate_field("status_message"));
                            }
                            status_message = Some(map.next_value()?);
                        }
                        "details" => {
                            if details.is_some() {
                                return Err(de::Error::duplicate_field("details"));
                            }
                            details = Some(map.next_value()?);
                        }
                        "started_at" => {
                            if started_at.is_some() {
                                return Err(de::Error::duplicate_field("started_at"));
                            }
                            let s: String = map.next_value()?;
                            started_at = Some(
                                DateTime::parse_from_rfc3339(&s)
                                    .map(|dt| dt.with_timezone(&Utc))
                                    .map_err(de::Error::custom)?
                            );
                        }
                        "last_updated" => {
                            if last_updated.is_some() {
                                return Err(de::Error::duplicate_field("last_updated"));
                            }
                            let s: String = map.next_value()?;
                            last_updated = Some(
                                DateTime::parse_from_rfc3339(&s)
                                    .map(|dt| dt.with_timezone(&Utc))
                                    .map_err(de::Error::custom)?
                            );
                        }
                        _ => {
                            let _ = map.next_value::<de::IgnoredAny>()?;
                        }
                    }
                }

                Ok(AnalysisProgress {
                    repository_id: repository_id.ok_or_else(|| de::Error::missing_field("repository_id"))?,
                    current_step: current_step.ok_or_else(|| de::Error::missing_field("current_step"))?,
                    total_steps: total_steps.ok_or_else(|| de::Error::missing_field("total_steps"))?,
                    step_name: step_name.ok_or_else(|| de::Error::missing_field("step_name"))?,
                    progress_percent: progress_percent.ok_or_else(|| de::Error::missing_field("progress_percent"))?,
                    status_message: status_message.ok_or_else(|| de::Error::missing_field("status_message"))?,
                    details,
                    started_at: started_at.ok_or_else(|| de::Error::missing_field("started_at"))?,
                    last_updated: last_updated.ok_or_else(|| de::Error::missing_field("last_updated"))?,
                })
            }
        }

        deserializer.deserialize_map(AnalysisProgressVisitor)
    }
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

    /// Update only the status message without changing the step number
    /// Useful for batch operations that want to show progress within a step
    pub fn update_status_message(
        &self,
        repository_id: &str,
        status_message: &str,
    ) {
        let mut progress_map = self.progress.lock().unwrap();
        if let Some(progress) = progress_map.get_mut(repository_id) {
            progress.status_message = status_message.to_string();
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
    
    /// Keep completed analyses for a period of time (5 minutes) before clearing
    pub fn cleanup_old_progress(&self, max_age_seconds: i64) {
        let mut progress_map = self.progress.lock().unwrap();
        let now = Utc::now();
        let mut to_remove = Vec::new();
        
        for (repo_id, progress) in progress_map.iter() {
            let age = now.signed_duration_since(progress.last_updated);
            // Only remove completed or failed analyses that are old
            if (progress.step_name == "Complete" || progress.step_name == "Failed") 
                && age.num_seconds() > max_age_seconds {
                to_remove.push(repo_id.clone());
            }
        }
        
        for repo_id in to_remove {
            progress_map.remove(&repo_id);
        }
    }
}

impl Default for ProgressTracker {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn get_analysis_progress(
    api_state: web::Data<crate::api::ApiState>,
    _req: HttpRequest,
    path: web::Path<String>,
) -> impl Responder {
    let state = &api_state.progress_tracker;
    let repository_id = path.into_inner();
    
    if repository_id.is_empty() {
        log::error!("Empty repository ID provided");
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Repository ID is required"
        }));
    }
    
    log::info!("Getting progress for repository: {}", repository_id);
    
    match state.get_progress(&repository_id) {
        Some(progress) => {
            log::info!("Found progress: step {}/{} - {} ({}%)", 
                progress.current_step, 
                progress.total_steps, 
                progress.step_name,
                progress.progress_percent
            );
            log::debug!("Progress details: started_at={:?}, last_updated={:?}", 
                progress.started_at, 
                progress.last_updated
            );
            
            // Serialize using our custom Serialize implementation
            match serde_json::to_value(&progress) {
                Ok(json_value) => {
                    log::info!("Successfully serialized progress for repository: {}", repository_id);
                    HttpResponse::Ok().json(json_value)
                },
                Err(e) => {
                    log::error!("Failed to serialize progress for repository {}: {}", repository_id, e);
                    // Fallback: return a simplified version
                    HttpResponse::Ok().json(serde_json::json!({
                        "repository_id": progress.repository_id,
                        "current_step": progress.current_step,
                        "total_steps": progress.total_steps,
                        "step_name": progress.step_name,
                        "progress_percent": progress.progress_percent,
                        "status_message": progress.status_message,
                        "details": progress.details,
                        "started_at": progress.started_at.to_rfc3339(),
                        "last_updated": progress.last_updated.to_rfc3339()
                    }))
                }
            }
        },
        None => {
            log::info!("No progress found for repository: {}", repository_id);
            HttpResponse::NotFound().json(serde_json::json!({
                "error": "No analysis in progress for this repository",
                "repository_id": repository_id
            }))
        },
    }
}

