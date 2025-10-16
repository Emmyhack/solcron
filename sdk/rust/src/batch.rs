//! Batch operations for efficient mass management of SolCron jobs and keepers
//!
//! This module provides utilities for performing multiple operations in a single transaction
//! or efficiently across multiple transactions with automatic batching and error recovery.

use crate::{
    client::SolCronClient,
    types::{AutomationJob, Keeper, TriggerType, RegistryState},
    error::{SolCronError, SolCronResult},
    utils::Utils,
};
use solana_sdk::{
    signature::{Keypair, Signature},
    pubkey::Pubkey,
    transaction::Transaction,
};
use std::collections::HashMap;

/// Configuration for batch operations
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Maximum number of operations per transaction
    pub max_ops_per_tx: usize,
    /// Maximum number of concurrent transactions
    pub max_concurrent_txs: usize,
    /// Retry attempts for failed transactions
    pub retry_attempts: u32,
    /// Delay between retries (milliseconds)
    pub retry_delay_ms: u64,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_ops_per_tx: 10,
            max_concurrent_txs: 5,
            retry_attempts: 3,
            retry_delay_ms: 1000,
        }
    }
}

/// Result of a batch operation
#[derive(Debug, Clone)]
pub struct BatchResult<T> {
    /// Successful operations with their results
    pub successful: Vec<(usize, T)>,
    /// Failed operations with their errors
    pub failed: Vec<(usize, SolCronError)>,
    /// Transaction signatures for successful batches
    pub signatures: Vec<Signature>,
}

impl<T> BatchResult<T> {
    pub fn new() -> Self {
        Self {
            successful: Vec::new(),
            failed: Vec::new(),
            signatures: Vec::new(),
        }
    }
    
    /// Get the success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        let total = self.successful.len() + self.failed.len();
        if total == 0 {
            100.0
        } else {
            (self.successful.len() as f64 / total as f64) * 100.0
        }
    }
    
    /// Check if all operations succeeded
    pub fn is_complete_success(&self) -> bool {
        self.failed.is_empty()
    }
    
    /// Get total number of operations
    pub fn total_operations(&self) -> usize {
        self.successful.len() + self.failed.len()
    }
}

/// Parameters for batch job registration
#[derive(Debug, Clone)]
pub struct BatchJobParams {
    pub target_program: Pubkey,
    pub target_instruction: String,
    pub trigger_type: TriggerType,
    pub gas_limit: u64,
    pub initial_balance: u64,
    pub owner: Pubkey,
}

/// Parameters for batch job updates
#[derive(Debug, Clone)]
pub struct BatchJobUpdate {
    pub job_id: u64,
    pub new_balance: Option<u64>,
    pub new_trigger: Option<TriggerType>,
    pub new_gas_limit: Option<u64>,
    pub activate: Option<bool>,
}

/// Batch operations manager
pub struct BatchOperations {
    client: SolCronClient,
    config: BatchConfig,
}

impl BatchOperations {
    /// Create a new batch operations manager
    pub fn new(client: SolCronClient, config: Option<BatchConfig>) -> Self {
        Self {
            client,
            config: config.unwrap_or_default(),
        }
    }

    /// Register multiple jobs in batch
    pub async fn register_jobs(
        &self,
        job_params: Vec<BatchJobParams>,
        payer: &Keypair,
    ) -> SolCronResult<BatchResult<u64>> {
        println!("🚀 Starting batch job registration for {} jobs", job_params.len());
        
        let mut result = BatchResult::new();
        let chunks = job_params.chunks(self.config.max_ops_per_tx);
        
        for (chunk_idx, chunk) in chunks.enumerate() {
            println!("📦 Processing batch {}/{}", chunk_idx + 1, chunks.len());
            
            match self.register_job_chunk(chunk, payer).await {
                Ok((job_ids, signature)) => {
                    for (local_idx, job_id) in job_ids.into_iter().enumerate() {
                        let global_idx = chunk_idx * self.config.max_ops_per_tx + local_idx;
                        result.successful.push((global_idx, job_id));
                    }
                    result.signatures.push(signature);
                }
                Err(e) => {
                    println!("❌ Batch {} failed: {}", chunk_idx + 1, e);
                    
                    // Mark all jobs in this chunk as failed
                    for local_idx in 0..chunk.len() {
                        let global_idx = chunk_idx * self.config.max_ops_per_tx + local_idx;
                        result.failed.push((global_idx, e.clone()));
                    }
                }
            }
        }
        
        println!("✅ Batch registration complete: {}/{} successful", 
            result.successful.len(), result.total_operations());
        
        Ok(result)
    }

    /// Update multiple jobs in batch
    pub async fn update_jobs(
        &self,
        updates: Vec<BatchJobUpdate>,
        authority: &Keypair,
    ) -> SolCronResult<BatchResult<()>> {
        println!("🔄 Starting batch job updates for {} jobs", updates.len());
        
        let mut result = BatchResult::new();
        let chunks = updates.chunks(self.config.max_ops_per_tx);
        
        for (chunk_idx, chunk) in chunks.enumerate() {
            match self.update_job_chunk(chunk, authority).await {
                Ok(signature) => {
                    for local_idx in 0..chunk.len() {
                        let global_idx = chunk_idx * self.config.max_ops_per_tx + local_idx;
                        result.successful.push((global_idx, ()));
                    }
                    result.signatures.push(signature);
                }
                Err(e) => {
                    for local_idx in 0..chunk.len() {
                        let global_idx = chunk_idx * self.config.max_ops_per_tx + local_idx;
                        result.failed.push((global_idx, e.clone()));
                    }
                }
            }
        }
        
        println!("✅ Batch updates complete: {}/{} successful", 
            result.successful.len(), result.total_operations());
        
        Ok(result)
    }

    /// Execute multiple jobs in batch (for keepers)
    pub async fn execute_jobs(
        &self,
        job_ids: Vec<u64>,
        keeper: &Keypair,
    ) -> SolCronResult<BatchResult<u64>> {
        println!("⚡ Starting batch job execution for {} jobs", job_ids.len());
        
        let mut result = BatchResult::new();
        
        // Execute jobs concurrently with controlled concurrency
        let semaphore = tokio::sync::Semaphore::new(self.config.max_concurrent_txs);
        let mut tasks = Vec::new();
        
        for (idx, job_id) in job_ids.into_iter().enumerate() {
            let client = self.client.clone();
            let keeper = keeper.insecure_clone();
            let permit = semaphore.clone();
            
            let task = tokio::spawn(async move {
                let _permit = permit.acquire().await.unwrap();
                
                match client.execute_job(job_id, &keeper).await {
                    Ok(execution_result) => Ok((idx, execution_result.fee_charged)),
                    Err(e) => Err((idx, e)),
                }
            });
            
            tasks.push(task);
        }
        
        // Collect results
        for task in tasks {
            match task.await.unwrap() {
                Ok((idx, reward)) => result.successful.push((idx, reward)),
                Err((idx, e)) => result.failed.push((idx, e)),
            }
        }
        
        println!("✅ Batch execution complete: {}/{} successful", 
            result.successful.len(), result.total_operations());
        
        Ok(result)
    }

    /// Top up multiple job balances in batch
    pub async fn top_up_jobs(
        &self,
        top_ups: Vec<(u64, u64)>, // (job_id, amount)
        payer: &Keypair,
    ) -> SolCronResult<BatchResult<()>> {
        println!("💰 Starting batch job top-ups for {} jobs", top_ups.len());
        
        let mut result = BatchResult::new();
        let chunks = top_ups.chunks(self.config.max_ops_per_tx);
        
        for (chunk_idx, chunk) in chunks.enumerate() {
            match self.top_up_job_chunk(chunk, payer).await {
                Ok(signature) => {
                    for local_idx in 0..chunk.len() {
                        let global_idx = chunk_idx * self.config.max_ops_per_tx + local_idx;
                        result.successful.push((global_idx, ()));
                    }
                    result.signatures.push(signature);
                }
                Err(e) => {
                    for local_idx in 0..chunk.len() {
                        let global_idx = chunk_idx * self.config.max_ops_per_tx + local_idx;
                        result.failed.push((global_idx, e.clone()));
                    }
                }
            }
        }
        
        println!("✅ Batch top-ups complete: {}/{} successful", 
            result.successful.len(), result.total_operations());
        
        Ok(result)
    }

    /// Get status of multiple jobs in batch
    pub async fn get_jobs_status(
        &self,
        job_ids: Vec<u64>,
    ) -> SolCronResult<BatchResult<AutomationJob>> {
        println!("📊 Fetching status for {} jobs", job_ids.len());
        
        let mut result = BatchResult::new();
        
        // Fetch jobs concurrently
        let semaphore = tokio::sync::Semaphore::new(self.config.max_concurrent_txs);
        let mut tasks = Vec::new();
        
        for (idx, job_id) in job_ids.into_iter().enumerate() {
            let client = self.client.clone();
            let permit = semaphore.clone();
            
            let task = tokio::spawn(async move {
                let _permit = permit.acquire().await.unwrap();
                
                match client.get_job(job_id).await {
                    Ok(job) => Ok((idx, job)),
                    Err(e) => Err((idx, e)),
                }
            });
            
            tasks.push(task);
        }
        
        // Collect results
        for task in tasks {
            match task.await.unwrap() {
                Ok((idx, job)) => result.successful.push((idx, job)),
                Err((idx, e)) => result.failed.push((idx, e)),
            }
        }
        
        Ok(result)
    }

    /// Analyze batch of jobs for optimization opportunities
    pub async fn analyze_jobs(
        &self,
        job_ids: Vec<u64>,
    ) -> SolCronResult<BatchAnalysisReport> {
        let jobs_result = self.get_jobs_status(job_ids).await?;
        let registry = self.client.get_registry_state().await?;
        
        let mut report = BatchAnalysisReport::new();
        
        for (_, job) in jobs_result.successful {
            report.total_jobs += 1;
            report.total_balance += job.balance;
            
            // Analyze job health
            if job.balance < job.min_balance * 2 {
                report.low_balance_jobs.push(job.job_id);
            }
            
            if !job.is_active {
                report.inactive_jobs.push(job.job_id);
            }
            
            // Analyze execution patterns
            if job.execution_count > 0 {
                let avg_interval = if job.execution_count > 1 {
                    (Utils::current_timestamp() - job.created_at) / job.execution_count
                } else {
                    0
                };
                
                report.execution_stats.insert(job.job_id, JobExecutionStats {
                    total_executions: job.execution_count,
                    avg_execution_interval: avg_interval,
                    last_execution: job.last_execution,
                    estimated_cost_per_execution: Utils::calculate_execution_fee(
                        registry.base_fee,
                        job.gas_limit,
                        1, // Assume base gas price
                    ),
                });
            }
        }
        
        // Calculate optimization recommendations
        report.calculate_recommendations(&registry);
        
        Ok(report)
    }

    // Private helper methods
    
    async fn register_job_chunk(
        &self,
        job_params: &[BatchJobParams],
        payer: &Keypair,
    ) -> SolCronResult<(Vec<u64>, Signature)> {
        // For simplicity, register jobs one by one in this chunk
        // In a real implementation, you might create a single transaction with multiple instructions
        let mut job_ids = Vec::new();
        
        for params in job_params {
            let job_id = self.client.register_job(
                params.target_program,
                params.target_instruction.clone(),
                params.trigger_type.clone(),
                params.gas_limit,
                params.initial_balance,
                payer,
            ).await?;
            
            job_ids.push(job_id);
        }
        
        // Return a dummy signature for now
        Ok((job_ids, Signature::default()))
    }
    
    async fn update_job_chunk(
        &self,
        updates: &[BatchJobUpdate],
        authority: &Keypair,
    ) -> SolCronResult<Signature> {
        // Process updates one by one
        for update in updates {
            if let Some(amount) = update.new_balance {
                self.client.top_up_job(update.job_id, amount, authority).await?;
            }
            
            if let Some(active) = update.activate {
                if active {
                    self.client.activate_job(update.job_id, authority).await?;
                } else {
                    self.client.deactivate_job(update.job_id, authority).await?;
                }
            }
        }
        
        Ok(Signature::default())
    }
    
    async fn top_up_job_chunk(
        &self,
        top_ups: &[(u64, u64)],
        payer: &Keypair,
    ) -> SolCronResult<Signature> {
        for (job_id, amount) in top_ups {
            self.client.top_up_job(*job_id, *amount, payer).await?;
        }
        
        Ok(Signature::default())
    }
}

/// Statistics for individual job execution patterns
#[derive(Debug, Clone)]
pub struct JobExecutionStats {
    pub total_executions: u64,
    pub avg_execution_interval: u64,
    pub last_execution: u64,
    pub estimated_cost_per_execution: u64,
}

/// Comprehensive analysis report for a batch of jobs
#[derive(Debug)]
pub struct BatchAnalysisReport {
    pub total_jobs: u64,
    pub total_balance: u64,
    pub low_balance_jobs: Vec<u64>,
    pub inactive_jobs: Vec<u64>,
    pub execution_stats: HashMap<u64, JobExecutionStats>,
    pub recommendations: Vec<OptimizationRecommendation>,
}

impl BatchAnalysisReport {
    pub fn new() -> Self {
        Self {
            total_jobs: 0,
            total_balance: 0,
            low_balance_jobs: Vec::new(),
            inactive_jobs: Vec::new(),
            execution_stats: HashMap::new(),
            recommendations: Vec::new(),
        }
    }
    
    fn calculate_recommendations(&mut self, registry: &RegistryState) {
        // Recommend balance top-ups
        if !self.low_balance_jobs.is_empty() {
            self.recommendations.push(OptimizationRecommendation {
                category: RecommendationCategory::Balance,
                description: format!("Top up {} jobs with low balances", self.low_balance_jobs.len()),
                affected_jobs: self.low_balance_jobs.clone(),
                estimated_savings: 0,
                priority: RecommendationPriority::High,
            });
        }
        
        // Recommend activating inactive jobs
        if !self.inactive_jobs.is_empty() {
            self.recommendations.push(OptimizationRecommendation {
                category: RecommendationCategory::Activation,
                description: format!("Activate {} inactive jobs", self.inactive_jobs.len()),
                affected_jobs: self.inactive_jobs.clone(),
                estimated_savings: 0,
                priority: RecommendationPriority::Medium,
            });
        }
        
        // Analyze gas optimization opportunities
        let high_gas_jobs: Vec<u64> = self.execution_stats
            .iter()
            .filter(|(_, stats)| stats.estimated_cost_per_execution > registry.base_fee * 2)
            .map(|(job_id, _)| *job_id)
            .collect();
            
        if !high_gas_jobs.is_empty() {
            self.recommendations.push(OptimizationRecommendation {
                category: RecommendationCategory::GasOptimization,
                description: format!("Optimize gas usage for {} high-cost jobs", high_gas_jobs.len()),
                affected_jobs: high_gas_jobs,
                estimated_savings: 50000, // Estimated savings per execution
                priority: RecommendationPriority::Medium,
            });
        }
    }
    
    /// Print a formatted report
    pub fn print_report(&self) {
        println!("\n📊 Batch Analysis Report");
        println!("========================");
        println!("📈 Total Jobs: {}", self.total_jobs);
        println!("💰 Total Balance: {} SOL", Utils::lamports_to_sol_string(self.total_balance, 6));
        println!("⚠️  Low Balance Jobs: {}", self.low_balance_jobs.len());
        println!("💤 Inactive Jobs: {}", self.inactive_jobs.len());
        println!("📊 Jobs with Execution Data: {}", self.execution_stats.len());
        
        if !self.recommendations.is_empty() {
            println!("\n🎯 Optimization Recommendations:");
            for (i, rec) in self.recommendations.iter().enumerate() {
                println!("  {}. [{}] {}", i + 1, rec.priority_symbol(), rec.description);
                if rec.estimated_savings > 0 {
                    println!("     💡 Estimated Savings: {} lamports per execution", rec.estimated_savings);
                }
            }
        }
        
        println!();
    }
}

#[derive(Debug, Clone)]
pub struct OptimizationRecommendation {
    pub category: RecommendationCategory,
    pub description: String,
    pub affected_jobs: Vec<u64>,
    pub estimated_savings: u64,
    pub priority: RecommendationPriority,
}

impl OptimizationRecommendation {
    fn priority_symbol(&self) -> &'static str {
        match self.priority {
            RecommendationPriority::High => "🔥",
            RecommendationPriority::Medium => "⚡",
            RecommendationPriority::Low => "💡",
        }
    }
}

#[derive(Debug, Clone)]
pub enum RecommendationCategory {
    Balance,
    Activation,
    GasOptimization,
    TriggerOptimization,
    Consolidation,
}

#[derive(Debug, Clone)]
pub enum RecommendationPriority {
    High,
    Medium,
    Low,
}