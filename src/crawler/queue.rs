use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobType {
    AnalyzeRepository,
    BatchAnalyze,
    ScheduledReanalyze,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisJob {
    pub id: String,
    pub job_type: JobType,
    pub repository_id: Option<String>,
    pub repository_url: Option<String>,
    pub status: JobStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub progress: f64, // 0.0 to 1.0
    pub metadata: serde_json::Value,
}

impl AnalysisJob {
    pub fn new(job_type: JobType, repository_id: Option<String>, repository_url: Option<String>) -> Self {
        AnalysisJob {
            id: Uuid::new_v4().to_string(),
            job_type,
            repository_id,
            repository_url,
            status: JobStatus::Pending,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            error_message: None,
            progress: 0.0,
            metadata: serde_json::json!({}),
        }
    }

    pub fn start(&mut self) {
        self.status = JobStatus::Running;
        self.started_at = Some(Utc::now());
    }

    pub fn complete(&mut self) {
        self.status = JobStatus::Completed;
        self.completed_at = Some(Utc::now());
        self.progress = 1.0;
    }

    pub fn fail(&mut self, error: String) {
        self.status = JobStatus::Failed;
        self.completed_at = Some(Utc::now());
        self.error_message = Some(error);
    }

    pub fn update_progress(&mut self, progress: f64) {
        self.progress = progress.min(1.0).max(0.0);
    }
}

pub struct JobQueue {
    jobs: Vec<AnalysisJob>,
}

impl JobQueue {
    pub fn new() -> Self {
        JobQueue {
            jobs: Vec::new(),
        }
    }

    pub fn enqueue(&mut self, job: AnalysisJob) -> String {
        let job_id = job.id.clone();
        self.jobs.push(job);
        job_id
    }

    pub fn dequeue(&mut self) -> Option<AnalysisJob> {
        self.jobs.iter()
            .position(|j| j.status == JobStatus::Pending)
            .map(|idx| self.jobs.remove(idx))
    }

    pub fn get_job(&self, job_id: &str) -> Option<&AnalysisJob> {
        self.jobs.iter().find(|j| j.id == job_id)
    }

    pub fn get_job_mut(&mut self, job_id: &str) -> Option<&mut AnalysisJob> {
        self.jobs.iter_mut().find(|j| j.id == job_id)
    }

    pub fn list_jobs(&self, status: Option<JobStatus>) -> Vec<&AnalysisJob> {
        if let Some(s) = status {
            self.jobs.iter().filter(|j| j.status == s).collect()
        } else {
            self.jobs.iter().collect()
        }
    }

    pub fn get_pending_count(&self) -> usize {
        self.jobs.iter().filter(|j| j.status == JobStatus::Pending).count()
    }

    pub fn get_running_count(&self) -> usize {
        self.jobs.iter().filter(|j| j.status == JobStatus::Running).count()
    }
}

impl Default for JobQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledJob {
    pub id: String,
    pub name: String,
    pub schedule: String, // Cron expression
    pub job_type: JobType,
    pub repository_id: Option<String>,
    pub enabled: bool,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
}

impl ScheduledJob {
    pub fn new(name: String, schedule: String, job_type: JobType, repository_id: Option<String>) -> Self {
        ScheduledJob {
            id: Uuid::new_v4().to_string(),
            name,
            schedule,
            job_type,
            repository_id,
            enabled: true,
            last_run: None,
            next_run: None,
            metadata: serde_json::json!({}),
        }
    }
}

pub struct Scheduler {
    scheduled_jobs: Vec<ScheduledJob>,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            scheduled_jobs: Vec::new(),
        }
    }

    pub fn add_job(&mut self, job: ScheduledJob) {
        self.scheduled_jobs.push(job);
    }

    pub fn remove_job(&mut self, job_id: &str) -> bool {
        if let Some(pos) = self.scheduled_jobs.iter().position(|j| j.id == job_id) {
            self.scheduled_jobs.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn get_jobs(&self) -> &[ScheduledJob] {
        &self.scheduled_jobs
    }

    pub fn get_job(&self, job_id: &str) -> Option<&ScheduledJob> {
        self.scheduled_jobs.iter().find(|j| j.id == job_id)
    }

    pub fn get_job_mut(&mut self, job_id: &str) -> Option<&mut ScheduledJob> {
        self.scheduled_jobs.iter_mut().find(|j| j.id == job_id)
    }

    pub fn get_due_jobs(&self) -> Vec<&ScheduledJob> {
        let now = Utc::now();
        self.scheduled_jobs.iter()
            .filter(|j| j.enabled)
            .filter(|j| {
                if let Some(next_run) = j.next_run {
                    next_run <= now
                } else {
                    true // Never run before
                }
            })
            .collect()
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

