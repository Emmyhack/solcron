//! Example: Keeper node implementation
//!
//! This example demonstrates how to build a keeper node that monitors
//! and executes automation jobs using the SolCron Rust SDK.

use solcron_sdk::{
    SolCronClient, TriggerEvaluation, ExecutionResult,
    types::{TriggerType, AutomationJob, RegistryState},
    utils::{Utils, TimeUtils},
    error::{SolCronError, SolCronResult},
};
use solana_sdk::{
    signature::{Keypair, Signer},
    pubkey::Pubkey,
};
use tokio::{time::{sleep, Duration}, select};
use std::{collections::HashMap, sync::Arc};

/// Configuration for the keeper node
#[derive(Debug, Clone)]
pub struct KeeperConfig {
    /// Solana cluster RPC URL
    pub cluster_url: String,
    /// Keeper keypair for signing transactions
    pub keeper_keypair: Keypair,
    /// Monitoring interval in seconds
    pub monitoring_interval: u64,
    /// Maximum gas price willing to pay
    pub max_gas_price: u64,
    /// Minimum profit margin required for execution (in lamports)
    pub min_profit_margin: u64,
    /// Maximum number of jobs to execute concurrently
    pub max_concurrent_executions: usize,
}

/// Keeper node that monitors and executes automation jobs
pub struct KeeperNode {
    /// SolCron client
    client: SolCronClient,
    /// Keeper configuration
    config: KeeperConfig,
    /// Job execution statistics
    stats: KeeperStats,
    /// Currently monitored jobs
    monitored_jobs: HashMap<u64, JobMonitorInfo>,
}

/// Statistics for keeper performance
#[derive(Debug, Default)]
pub struct KeeperStats {
    pub total_executions_attempted: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub total_rewards_earned: u64,
    pub total_gas_used: u64,
    pub uptime_seconds: u64,
}

/// Information for monitoring a specific job
#[derive(Debug, Clone)]
pub struct JobMonitorInfo {
    pub job: AutomationJob,
    pub last_check: u64,
    pub next_execution_time: Option<u64>,
    pub consecutive_failures: u32,
}

impl KeeperNode {
    /// Create a new keeper node
    pub async fn new(config: KeeperConfig) -> SolCronResult<Self> {
        println!("🚀 Initializing SolCron Keeper Node");
        println!("====================================");
        
        let client = SolCronClient::new_with_payer(
            &config.cluster_url,
            config.keeper_keypair.insecure_clone(),
            None,
        ).await?;

        let keeper_node = Self {
            client,
            config,
            stats: KeeperStats::default(),
            monitored_jobs: HashMap::new(),
        };

        println!("✅ Keeper node initialized");
        println!("🔑 Keeper pubkey: {}", keeper_node.config.keeper_keypair.pubkey());
        
        Ok(keeper_node)
    }

    /// Start the keeper node
    pub async fn start(&mut self) -> SolCronResult<()> {
        println!("\n🎯 Starting keeper node operations...");
        
        // Register as keeper if not already registered
        self.ensure_keeper_registered().await?;
        
        // Start monitoring loop
        let start_time = Utils::current_timestamp();
        let mut last_stats_update = start_time;
        
        loop {
            let loop_start = std::time::Instant::now();
            
            select! {
                // Main monitoring and execution loop
                _ = self.monitoring_cycle() => {
                    // Continue loop
                }
                
                // Periodic stats update
                _ = sleep(Duration::from_secs(60)) => {
                    let current_time = Utils::current_timestamp();
                    self.stats.uptime_seconds = current_time - start_time;
                    
                    if current_time - last_stats_update >= 300 { // Every 5 minutes
                        self.print_stats().await;
                        last_stats_update = current_time;
                    }
                }
            }
            
            // Ensure minimum loop interval
            let elapsed = loop_start.elapsed();
            if elapsed < Duration::from_secs(self.config.monitoring_interval) {
                let sleep_duration = Duration::from_secs(self.config.monitoring_interval) - elapsed;
                sleep(sleep_duration).await;
            }
        }
    }

    /// Ensure the keeper is registered
    async fn ensure_keeper_registered(&self) -> SolCronResult<()> {
        let keeper_address = self.config.keeper_keypair.pubkey();
        
        match self.client.get_keeper(&keeper_address).await {
            Ok(keeper) => {
                if keeper.is_active {
                    println!("✅ Keeper already registered and active");
                    println!("   - Stake: {} SOL", Utils::lamports_to_sol_string(keeper.stake_amount, 3));
                    println!("   - Reputation: {}/10000", keeper.reputation_score);
                    println!("   - Success rate: {:.1}%", keeper.success_rate());
                } else {
                    println!("⚠️  Keeper registered but inactive");
                    return Err(SolCronError::KeeperNotActive {
                        keeper: keeper_address.to_string(),
                    });
                }
            }
            Err(SolCronError::KeeperNotFound { .. }) => {
                println!("📝 Registering new keeper...");
                
                // Get registry state to check minimum stake
                let registry = self.client.get_registry_state().await?;
                let stake_amount = registry.min_stake * 2; // Stake 2x minimum for better selection
                
                println!("💰 Staking {} SOL", Utils::lamports_to_sol_string(stake_amount, 3));
                
                let signature = self.client.register_keeper(
                    stake_amount,
                    &self.config.keeper_keypair,
                ).await?;
                
                println!("✅ Keeper registered! Signature: {}", signature);
            }
            Err(e) => return Err(e),
        }
        
        Ok(())
    }

    /// Main monitoring cycle
    async fn monitoring_cycle(&mut self) -> SolCronResult<()> {
        // 1. Discover new jobs
        self.discover_jobs().await?;
        
        // 2. Evaluate job triggers
        let executable_jobs = self.evaluate_triggers().await?;
        
        // 3. Execute jobs that are ready
        if !executable_jobs.is_empty() {
            println!("🎯 Found {} job(s) ready for execution", executable_jobs.len());
            
            for job_id in executable_jobs.into_iter().take(self.config.max_concurrent_executions) {
                if let Err(e) = self.execute_job(job_id).await {
                    println!("❌ Failed to execute job {}: {}", job_id, e);
                    self.stats.failed_executions += 1;
                }
            }
        }
        
        // 4. Claim any pending rewards
        if self.stats.successful_executions % 10 == 0 && self.stats.successful_executions > 0 {
            if let Err(e) = self.claim_pending_rewards().await {
                println!("⚠️  Failed to claim rewards: {}", e);
            }
        }
        
        Ok(())
    }

    /// Discover new automation jobs to monitor
    async fn discover_jobs(&mut self) -> SolCronResult<()> {
        // Note: In a real implementation, you would scan all job accounts
        // For this example, we'll simulate job discovery
        
        // Get registry state to see total jobs
        let registry = self.client.get_registry_state().await?;
        
        // Check if there are new jobs since our last scan
        let current_job_count = self.monitored_jobs.len() as u64;
        if registry.active_jobs > current_job_count {
            println!("🔍 Discovered {} new jobs to monitor", registry.active_jobs - current_job_count);
            
            // In practice, you would scan accounts here
            // For demo purposes, we'll create a mock job
            self.add_mock_job_for_demo(registry.next_job_id - 1).await?;
        }
        
        Ok(())
    }

    /// Add a mock job for demonstration (replace with actual job scanning)
    async fn add_mock_job_for_demo(&mut self, job_id: u64) -> SolCronResult<()> {
        if self.monitored_jobs.contains_key(&job_id) {
            return Ok(()); // Already monitoring
        }

        // Create a mock job for demonstration
        let mock_job = AutomationJob {
            job_id,
            owner: Pubkey::new_unique(),
            target_program: Pubkey::new_unique(),
            target_instruction: "harvest_demo".to_string(),
            trigger_type: TriggerType::TimeBased { interval: 300 }, // 5 minutes for demo
            trigger_params: vec![],
            gas_limit: 200_000,
            balance: Utils::sol_to_lamports(0.1),
            min_balance: Utils::sol_to_lamports(0.001),
            is_active: true,
            execution_count: 0,
            last_execution: 0,
            created_at: Utils::current_timestamp(),
        };

        let monitor_info = JobMonitorInfo {
            job: mock_job.clone(),
            last_check: Utils::current_timestamp(),
            next_execution_time: Some(Utils::current_timestamp() + 60), // 1 minute from now for demo
            consecutive_failures: 0,
        };

        self.monitored_jobs.insert(job_id, monitor_info);
        println!("➕ Added job {} to monitoring list", job_id);
        
        Ok(())
    }

    /// Evaluate triggers for all monitored jobs
    async fn evaluate_triggers(&mut self) -> SolCronResult<Vec<u64>> {
        let mut executable_jobs = Vec::new();
        let current_time = Utils::current_timestamp();
        
        for (job_id, monitor_info) in &mut self.monitored_jobs {
            let evaluation = self.evaluate_job_trigger(&monitor_info.job, current_time)?;
            
            if evaluation.should_execute {
                // Check profitability
                if self.is_execution_profitable(&monitor_info.job).await? {
                    executable_jobs.push(*job_id);
                    println!("✅ Job {} ready: {}", job_id, evaluation.reason);
                } else {
                    println!("💰 Job {} ready but not profitable", job_id);
                }
            }
            
            // Update next check time
            monitor_info.last_check = current_time;
            if let Some(next_time) = evaluation.next_evaluation {
                monitor_info.next_execution_time = Some(next_time);
            }
        }
        
        Ok(executable_jobs)
    }

    /// Evaluate a specific job's trigger conditions
    fn evaluate_job_trigger(&self, job: &AutomationJob, current_time: u64) -> SolCronResult<TriggerEvaluation> {
        match &job.trigger_type {
            TriggerType::TimeBased { interval } => {
                let should_execute = Utils::should_execute_time_trigger(
                    *interval,
                    job.last_execution,
                    current_time,
                );
                
                let next_evaluation = if should_execute {
                    None
                } else {
                    Some(TimeUtils::next_execution_time(*interval, job.last_execution))
                };
                
                Ok(TriggerEvaluation {
                    should_execute,
                    reason: if should_execute {
                        format!("Time interval ({} seconds) has elapsed", interval)
                    } else {
                        format!("Waiting for time interval ({} seconds)", interval)
                    },
                    next_evaluation,
                })
            }
            
            TriggerType::Conditional { .. } => {
                // For conditional triggers, you would implement specific logic
                // based on the condition stored in trigger_params
                Ok(TriggerEvaluation {
                    should_execute: false, // Implement actual condition checking
                    reason: "Conditional trigger evaluation not implemented in demo".to_string(),
                    next_evaluation: Some(current_time + 60), // Check again in 1 minute
                })
            }
            
            TriggerType::LogBased { .. } => {
                // For log-based triggers, you would monitor blockchain events
                Ok(TriggerEvaluation {
                    should_execute: false, // Implement actual log monitoring
                    reason: "Log-based trigger evaluation not implemented in demo".to_string(),
                    next_evaluation: Some(current_time + 30), // Check again in 30 seconds
                })
            }
            
            TriggerType::Hybrid { .. } => {
                // For hybrid triggers, combine multiple conditions
                Ok(TriggerEvaluation {
                    should_execute: false, // Implement hybrid logic
                    reason: "Hybrid trigger evaluation not implemented in demo".to_string(),
                    next_evaluation: Some(current_time + 60),
                })
            }
        }
    }

    /// Check if executing a job would be profitable
    async fn is_execution_profitable(&self, job: &AutomationJob) -> SolCronResult<bool> {
        let registry = self.client.get_registry_state().await?;
        
        // Estimate execution cost
        let estimated_gas = Utils::estimate_gas_usage(&job.target_instruction);
        let execution_fee = Utils::calculate_execution_fee(
            registry.base_fee,
            estimated_gas,
            self.config.max_gas_price,
        );
        
        // Calculate potential reward
        let keeper_reward = Utils::calculate_keeper_reward(execution_fee, registry.protocol_fee_bps);
        
        // Check if job has sufficient balance
        if job.balance < execution_fee {
            return Ok(false);
        }
        
        // Check minimum profit margin
        Ok(keeper_reward >= self.config.min_profit_margin)
    }

    /// Execute a specific job
    async fn execute_job(&mut self, job_id: u64) -> SolCronResult<ExecutionResult> {
        self.stats.total_executions_attempted += 1;
        
        println!("🚀 Executing job {}...", job_id);
        
        match self.client.execute_job(job_id, &self.config.keeper_keypair).await {
            Ok(result) => {
                self.stats.successful_executions += 1;
                self.stats.total_rewards_earned += result.fee_charged;
                self.stats.total_gas_used += result.gas_used;
                
                // Update job info
                if let Some(monitor_info) = self.monitored_jobs.get_mut(&job_id) {
                    monitor_info.job.execution_count += 1;
                    monitor_info.job.last_execution = result.execution_time;
                    monitor_info.consecutive_failures = 0;
                }
                
                println!("✅ Job {} executed successfully! Reward: {} lamports", 
                    job_id, result.fee_charged);
                
                Ok(result)
            }
            Err(e) => {
                self.stats.failed_executions += 1;
                
                // Update failure count
                if let Some(monitor_info) = self.monitored_jobs.get_mut(&job_id) {
                    monitor_info.consecutive_failures += 1;
                    
                    // Remove jobs that fail too many times
                    if monitor_info.consecutive_failures >= 5 {
                        println!("❌ Removing job {} after {} consecutive failures", 
                            job_id, monitor_info.consecutive_failures);
                        self.monitored_jobs.remove(&job_id);
                    }
                }
                
                Err(e)
            }
        }
    }

    /// Claim pending rewards
    async fn claim_pending_rewards(&self) -> SolCronResult<()> {
        let keeper_address = self.config.keeper_keypair.pubkey();
        let keeper = self.client.get_keeper(&keeper_address).await?;
        
        if keeper.pending_rewards > 0 {
            println!("💰 Claiming {} lamports in rewards...", keeper.pending_rewards);
            
            let signature = self.client.claim_rewards(&self.config.keeper_keypair).await?;
            println!("✅ Rewards claimed! Signature: {}", signature);
        }
        
        Ok(())
    }

    /// Print keeper statistics
    async fn print_stats(&self) {
        println!("\n📊 Keeper Statistics");
        println!("====================");
        println!("⏱️  Uptime: {}", Utils::format_duration(self.stats.uptime_seconds));
        println!("🎯 Jobs monitored: {}", self.monitored_jobs.len());
        println!("🚀 Executions attempted: {}", self.stats.total_executions_attempted);
        println!("✅ Successful executions: {}", self.stats.successful_executions);
        println!("❌ Failed executions: {}", self.stats.failed_executions);
        
        if self.stats.total_executions_attempted > 0 {
            let success_rate = (self.stats.successful_executions as f64 / 
                self.stats.total_executions_attempted as f64) * 100.0;
            println!("📈 Success rate: {:.1}%", success_rate);
        }
        
        println!("💰 Total rewards earned: {} SOL", 
            Utils::lamports_to_sol_string(self.stats.total_rewards_earned, 6));
        println!("⛽ Total gas used: {}", self.stats.total_gas_used);
        
        // Get current keeper info
        if let Ok(keeper) = self.client.get_keeper(&self.config.keeper_keypair.pubkey()).await {
            println!("🏆 Reputation score: {}/10000", keeper.reputation_score);
            println!("💎 Pending rewards: {} SOL", 
                Utils::lamports_to_sol_string(keeper.pending_rewards, 6));
        }
        
        println!();
    }
}

/// Example main function to run the keeper node
#[tokio::main]
async fn main() -> SolCronResult<()> {
    env_logger::init();
    
    // Create keeper configuration
    let config = KeeperConfig {
        cluster_url: "https://api.devnet.solana.com".to_string(),
        keeper_keypair: Keypair::new(), // In practice, load from file
        monitoring_interval: 10, // Check every 10 seconds
        max_gas_price: 1,
        min_profit_margin: 1000, // 0.000001 SOL minimum profit
        max_concurrent_executions: 3,
    };
    
    // Create and start keeper node
    let mut keeper = KeeperNode::new(config).await?;
    
    println!("\n🎉 Keeper node ready! Press Ctrl+C to stop.");
    
    // Handle graceful shutdown
    tokio::select! {
        result = keeper.start() => {
            if let Err(e) = result {
                println!("❌ Keeper node error: {}", e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            println!("\n👋 Shutting down keeper node...");
            keeper.print_stats().await;
        }
    }
    
    Ok(())
}

/// Additional utility functions for keeper operations
impl KeeperNode {
    /// Get detailed job information for monitoring
    pub async fn get_job_details(&self, job_id: u64) -> SolCronResult<AutomationJob> {
        self.client.get_job(job_id).await
    }
    
    /// Check if keeper needs more stake
    pub async fn check_stake_health(&self) -> SolCronResult<bool> {
        let keeper_address = self.config.keeper_keypair.pubkey();
        let keeper = self.client.get_keeper(&keeper_address).await?;
        let registry = self.client.get_registry_state().await?;
        
        // Recommend more stake if below 2x minimum
        Ok(keeper.stake_amount >= registry.min_stake * 2)
    }
    
    /// Estimate potential earnings for the next period
    pub async fn estimate_earnings(&self, period_hours: u64) -> SolCronResult<u64> {
        let registry = self.client.get_registry_state().await?;
        
        // Simple estimation based on current job count and execution frequency
        let jobs_per_hour = self.monitored_jobs.len() as u64;
        let avg_execution_fee = registry.base_fee + 100_000; // Estimate
        let avg_keeper_reward = Utils::calculate_keeper_reward(avg_execution_fee, registry.protocol_fee_bps);
        
        Ok(avg_keeper_reward * jobs_per_hour * period_hours)
    }
}