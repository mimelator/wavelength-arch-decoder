use anyhow::Result;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};
use crate::crawler::{JobQueue, AnalysisJob, JobType, JobStatus, Scheduler, ScheduledJob};
use crate::api::ApiState;
use log::{info, warn};
use chrono::Utc;

pub struct JobProcessor {
    job_queue: Arc<Mutex<JobQueue>>,
    scheduler: Arc<Mutex<Scheduler>>,
    running: Arc<Mutex<bool>>,
}

impl JobProcessor {
    pub fn new() -> Self {
        JobProcessor {
            job_queue: Arc::new(Mutex::new(JobQueue::new())),
            scheduler: Arc::new(Mutex::new(Scheduler::new())),
            running: Arc::new(Mutex::new(false)),
        }
    }

    pub fn enqueue_job(&self, job: AnalysisJob) -> String {
        let job_id = job.id.clone();
        let mut queue = self.job_queue.lock().unwrap();
        queue.enqueue(job);
        info!("Job {} enqueued", job_id);
        job_id
    }

    pub fn get_job(&self, job_id: &str) -> Option<AnalysisJob> {
        let queue = self.job_queue.lock().unwrap();
        queue.get_job(job_id).cloned()
    }

    pub fn list_jobs(&self, status: Option<JobStatus>) -> Vec<AnalysisJob> {
        let queue = self.job_queue.lock().unwrap();
        queue.list_jobs(status).into_iter().cloned().collect()
    }

    pub fn add_scheduled_job(&self, job: ScheduledJob) {
        let mut scheduler = self.scheduler.lock().unwrap();
        scheduler.add_job(job);
    }

    pub fn get_scheduled_jobs(&self) -> Vec<ScheduledJob> {
        let scheduler = self.scheduler.lock().unwrap();
        scheduler.get_jobs().to_vec()
    }

    pub async fn start_processor(&self, _api_state: Arc<ApiState>) {
        let mut running = self.running.lock().unwrap();
        if *running {
            warn!("Job processor already running");
            return;
        }
        *running = true;
        drop(running);

        info!("Starting job processor...");

        // Start job worker
        let job_queue = self.job_queue.clone();
        let api_state_clone = _api_state.clone();
        tokio::spawn(async move {
            Self::job_worker_loop(job_queue, api_state_clone).await;
        });

        // Start scheduler
        let scheduler = self.scheduler.clone();
        let job_queue_clone = self.job_queue.clone();
        let api_state_clone = _api_state.clone();
        tokio::spawn(async move {
            Self::scheduler_loop(scheduler, job_queue_clone, api_state_clone).await;
        });
    }

    pub fn stop(&self) {
        let mut running = self.running.lock().unwrap();
        *running = false;
        info!("Job processor stopped");
    }

    async fn job_worker_loop(job_queue: Arc<Mutex<JobQueue>>, _api_state: Arc<ApiState>) {
        loop {
            // Check for pending jobs
            let job = {
                let mut queue = job_queue.lock().unwrap();
                queue.dequeue()
            };

            if let Some(mut job) = job {
                info!("Processing job {}: {:?}", job.id, job.job_type);
                job.start();

                // Update job in queue
                {
                    let mut queue = job_queue.lock().unwrap();
                    if let Some(j) = queue.get_job_mut(&job.id) {
                        *j = job.clone();
                    }
                }

                // Process the job
                let result = Self::process_job(job.clone(), _api_state.clone()).await;

                // Update job status
                {
                    let mut queue = job_queue.lock().unwrap();
                    if let Some(j) = queue.get_job_mut(&job.id) {
                        match result {
                            Ok(_) => j.complete(),
                            Err(e) => j.fail(e.to_string()),
                        }
                    }
                }
            } else {
                // No jobs, wait a bit
                sleep(Duration::from_secs(1)).await;
            }
        }
    }

    async fn scheduler_loop(
        scheduler: Arc<Mutex<Scheduler>>,
        job_queue: Arc<Mutex<JobQueue>>,
        _api_state: Arc<ApiState>,
    ) {
        loop {
            sleep(Duration::from_secs(60)).await; // Check every minute

            let due_jobs = {
                let sched = scheduler.lock().unwrap();
                sched.get_due_jobs().into_iter().cloned().collect::<Vec<_>>()
            };

            for scheduled_job in due_jobs {
                info!("Scheduled job {} is due", scheduled_job.name);

                // Create analysis job from scheduled job
                let analysis_job = AnalysisJob::new(
                    scheduled_job.job_type.clone(),
                    scheduled_job.repository_id.clone(),
                    None,
                );

                // Enqueue the job
                {
                    let mut queue = job_queue.lock().unwrap();
                    queue.enqueue(analysis_job);
                }

                // Update scheduled job's last_run and next_run
                {
                    let mut sched = scheduler.lock().unwrap();
                    if let Some(job) = sched.get_job_mut(&scheduled_job.id) {
                        job.last_run = Some(Utc::now());
                        // TODO: Calculate next_run based on cron expression
                        // For now, just set it to 24 hours from now
                        job.next_run = Some(Utc::now() + chrono::Duration::hours(24));
                    }
                }
            }
        }
    }

    async fn process_job(_job: AnalysisJob, _api_state: Arc<ApiState>) -> Result<()> {
        match _job.job_type {
            JobType::AnalyzeRepository => {
                if let Some(repo_id) = &_job.repository_id {
                    // Trigger repository analysis
                    info!("Analyzing repository {}", repo_id);
                    // TODO: Actually trigger the analysis
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("Repository ID not provided"))
                }
            }
            JobType::BatchAnalyze => {
                // Process batch analysis
                info!("Processing batch analysis");
                Ok(())
            }
            JobType::ScheduledReanalyze => {
                if let Some(repo_id) = &_job.repository_id {
                    info!("Re-analyzing repository {}", repo_id);
                    // TODO: Actually trigger the analysis
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("Repository ID not provided"))
                }
            }
        }
    }
}

impl Default for JobProcessor {
    fn default() -> Self {
        Self::new()
    }
}
