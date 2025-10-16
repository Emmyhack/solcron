use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{interval, Duration, Instant};
use chrono::Utc;
use log::{info, warn, error, debug};

use crate::config::KeeperConfig;
use crate::database::{Database, JobRecord};
use crate::rpc::RpcManager;
use crate::evaluator::{TriggerEvaluator, EvaluationResult};
use crate::error::{KeeperError, KeeperResult};

pub struct JobMonitor {
    config: KeeperConfig,
    database: Arc<Database>,
    rpc_manager: Arc<RpcManager>,
    evaluator: Arc<TriggerEvaluator>,
    job_cache: Arc<RwLock<HashMap<i64, CachedJob>>>,
    execution_sender: mpsc::UnboundedSender<ExecutionRequest>,
}

#[derive(Clone, Debug)]
struct CachedJob {
    job: JobRecord,
    last_evaluation: Instant,
    next_check_time: Option<chrono::DateTime<Utc>>,
    evaluation_count: u64,
    consecutive_failures: u32,
}

#[derive(Debug, Clone)]
pub struct ExecutionRequest {
    pub job: JobRecord,
    pub reason: String,
    pub priority: ExecutionPriority,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExecutionPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

impl JobMonitor {
    pub fn new(
        config: KeeperConfig,
        database: Arc<Database>,
        rpc_manager: Arc<RpcManager>,
        execution_sender: mpsc::UnboundedSender<ExecutionRequest>,
    ) -> Self {
        let evaluator = Arc::new(TriggerEvaluator::new(rpc_manager.as_ref().clone()));
        
        Self {
            config,
            database,
            rpc_manager,
            evaluator,
            job_cache: Arc::new(RwLock::new(HashMap::new())),
            execution_sender,
        }
    }

    pub async fn start(&self) -> KeeperResult<()> {
        info!("Starting job monitor...");
        
        let poll_interval = self.config.get_poll_interval();
        let mut monitoring_interval = interval(poll_interval);
        
        // Cache refresh interval (every 5 minutes)
        let mut cache_refresh_interval = interval(Duration::from_secs(300));
        
        // Cleanup interval (every hour)
        let mut cleanup_interval = interval(Duration::from_secs(3600));

        loop {
            tokio::select! {
                _ = monitoring_interval.tick() => {
                    if let Err(e) = self.monitor_jobs().await {
                        error!("Error in job monitoring cycle: {:?}", e);
                    }
                }
                _ = cache_refresh_interval.tick() => {
                    if let Err(e) = self.refresh_job_cache().await {
                        error!("Error refreshing job cache: {:?}", e);
                    }
                }
                _ = cleanup_interval.tick() => {
                    if let Err(e) = self.cleanup_cache().await {
                        error!("Error cleaning up cache: {:?}", e);
                    }
                }
            }
        }
    }

    async fn monitor_jobs(&self) -> KeeperResult<()> {
        debug!("Starting monitoring cycle");
        
        // Get eligible jobs from database
        let eligible_jobs = self.get_jobs_to_check().await?;
        debug!("Found {} jobs to check", eligible_jobs.len());
        
        if eligible_jobs.is_empty() {
            return Ok(());
        }
        
        // Process jobs concurrently but with limited concurrency
        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.config.monitoring.max_concurrent_jobs));
        let mut handles = Vec::new();
        
        for job in eligible_jobs {
            let semaphore = semaphore.clone();
            let evaluator = self.evaluator.clone();
            let database = self.database.clone();
            let job_cache = self.job_cache.clone();
            let execution_sender = self.execution_sender.clone();
            
            let handle = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.expect("Semaphore acquisition failed");
                
                if let Err(e) = Self::process_job(
                    job,
                    evaluator,
                    database,
                    job_cache,
                    execution_sender,
                ).await {
                    warn!("Error processing job: {:?}", e);
                }
            });
            
            handles.push(handle);
        }
        
        // Wait for all job processing to complete
        for handle in handles {
            if let Err(e) = handle.await {
                warn!("Job processing task failed: {:?}", e);
            }
        }
        
        debug!("Monitoring cycle completed");
        Ok(())
    }

    async fn get_jobs_to_check(&self) -> KeeperResult<Vec<JobRecord>> {
        // First, get jobs from cache that are ready to be checked
        let mut jobs_from_cache = Vec::new();
        {
            let cache = self.job_cache.read().await;
            let now = Instant::now();
            let now_utc = Utc::now();
            
            for cached_job in cache.values() {
                // Check if it's time to evaluate this job
                let should_check = if let Some(next_check) = cached_job.next_check_time {
                    now_utc >= next_check
                } else {
                    // If no next check time, use last evaluation + cache TTL
                    now.duration_since(cached_job.last_evaluation) >= 
                        Duration::from_secs(self.config.monitoring.job_cache_ttl_seconds)
                };
                
                if should_check {
                    jobs_from_cache.push(cached_job.job.clone());
                }
            }
        }
        
        // If we have cached jobs to check, return those
        if !jobs_from_cache.is_empty() {
            debug!("Using {} jobs from cache", jobs_from_cache.len());
            return Ok(jobs_from_cache);
        }
        
        // Otherwise, get fresh jobs from database
        let db_jobs = self.database.get_eligible_jobs("keeper_address").await?;
        debug!("Loaded {} jobs from database", db_jobs.len());
        
        Ok(db_jobs)
    }

    async fn process_job(
        job: JobRecord,
        evaluator: Arc<TriggerEvaluator>,
        database: Arc<Database>,
        job_cache: Arc<RwLock<HashMap<i64, CachedJob>>>,
        execution_sender: mpsc::UnboundedSender<ExecutionRequest>,
    ) -> KeeperResult<()> {
        debug!("Processing job {}", job.job_id);
        
        // Update last checked time in database
        if let Err(e) = database.update_job_checked(job.job_id).await {
            warn!("Failed to update job check time for job {}: {:?}", job.job_id, e);
        }
        
        // Evaluate the job
        let evaluation = evaluator.evaluate_job(&job).await?;
        
        // Update cache
        {
            let mut cache = job_cache.write().await;
            let cached_job = cache.entry(job.job_id).or_insert_with(|| CachedJob {
                job: job.clone(),
                last_evaluation: Instant::now(),
                next_check_time: None,
                evaluation_count: 0,
                consecutive_failures: 0,
            });
            
            cached_job.job = job.clone();
            cached_job.last_evaluation = Instant::now();
            cached_job.next_check_time = evaluation.next_check_time;
            cached_job.evaluation_count += 1;
        }
        
        // If job should be executed, send to execution queue
        if evaluation.should_execute {
            let priority = Self::determine_execution_priority(&job, &evaluation);
            
            let execution_request = ExecutionRequest {
                job,
                reason: evaluation.reason,
                priority,
            };
            
            if let Err(e) = execution_sender.send(execution_request) {
                error!("Failed to send execution request: {:?}", e);
                return Err(KeeperError::InternalError("Execution queue full".to_string()));
            }
            
            debug!("Job {} queued for execution: {}", job.job_id, evaluation.reason);
        } else {
            debug!("Job {} not ready: {}", job.job_id, evaluation.reason);
        }
        
        Ok(())
    }

    fn determine_execution_priority(job: &JobRecord, evaluation: &EvaluationResult) -> ExecutionPriority {
        // Determine priority based on various factors
        
        // High priority for jobs that haven't been executed in a long time
        if let Some(last_executed) = job.last_executed {
            let elapsed = Utc::now().signed_duration_since(last_executed);
            if elapsed.num_hours() > 24 {
                return ExecutionPriority::High;
            }
        } else {
            // Never executed jobs get high priority
            return ExecutionPriority::High;
        }
        
        // Critical priority for jobs with high failure rates that finally succeed
        if job.failed_count > 5 && job.failed_count > job.execution_count / 2 {
            return ExecutionPriority::Critical;
        }
        
        // Low priority for frequently executed jobs
        if job.execution_count > 100 {
            return ExecutionPriority::Low;
        }
        
        // Check if the job has time-sensitive conditions
        if evaluation.reason.contains("Time interval elapsed") {
            return ExecutionPriority::Normal;
        }
        
        ExecutionPriority::Normal
    }

    async fn refresh_job_cache(&self) -> KeeperResult<()> {
        debug!("Refreshing job cache from database");
        
        let active_jobs = self.database.get_active_jobs().await?;
        let mut cache = self.job_cache.write().await;
        
        // Update existing jobs and add new ones
        for job in active_jobs {
            match cache.get_mut(&job.job_id) {
                Some(cached_job) => {
                    // Update existing cached job
                    cached_job.job = job;
                }
                None => {
                    // Add new job to cache
                    cache.insert(job.job_id, CachedJob {
                        job,
                        last_evaluation: Instant::now(),
                        next_check_time: None,
                        evaluation_count: 0,
                        consecutive_failures: 0,
                    });
                }
            }
        }
        
        // Remove inactive jobs from cache
        let active_job_ids: std::collections::HashSet<i64> = 
            cache.values().filter(|j| j.job.is_active).map(|j| j.job.job_id).collect();
        
        cache.retain(|&job_id, cached_job| {
            cached_job.job.is_active || active_job_ids.contains(&job_id)
        });
        
        info!("Job cache refreshed: {} jobs", cache.len());
        Ok(())
    }

    async fn cleanup_cache(&self) -> KeeperResult<()> {
        debug!("Cleaning up job cache");
        
        let mut cache = self.job_cache.write().await;
        let cache_ttl = Duration::from_secs(self.config.monitoring.job_cache_ttl_seconds * 10); // 10x TTL for cleanup
        let now = Instant::now();
        
        let initial_size = cache.len();
        
        cache.retain(|_, cached_job| {
            // Keep if job is active and recently evaluated
            cached_job.job.is_active && 
            now.duration_since(cached_job.last_evaluation) < cache_ttl
        });
        
        let removed = initial_size - cache.len();
        if removed > 0 {
            info!("Cleaned up {} stale jobs from cache", removed);
        }
        
        Ok(())
    }

    pub async fn get_cache_stats(&self) -> (usize, usize, usize) {
        let cache = self.job_cache.read().await;
        let total_jobs = cache.len();
        let active_jobs = cache.values().filter(|j| j.job.is_active).count();
        let pending_jobs = cache.values().filter(|j| {
            j.job.is_active && j.next_check_time.map_or(true, |t| Utc::now() >= t)
        }).count();
        
        (total_jobs, active_jobs, pending_jobs)
    }

    pub async fn force_job_check(&self, job_id: i64) -> KeeperResult<()> {
        // Force immediate check of a specific job
        let job = {
            let cache = self.job_cache.read().await;
            cache.get(&job_id).map(|cached_job| cached_job.job.clone())
        };

        if let Some(job) = job {
            Self::process_job(
                job,
                self.evaluator.clone(),
                self.database.clone(),
                self.job_cache.clone(),
                self.execution_sender.clone(),
            ).await?;
        } else {
            return Err(KeeperError::InvalidJobError(format!("Job {} not found in cache", job_id)));
        }

        Ok(())
    }
}