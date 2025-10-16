//! Advanced simulation and testing utilities for SolCron development
//!
//! This module provides tools for simulating various scenarios, load testing,
//! and advanced development workflows with the SolCron platform.

use crate::{
    client::SolCronClient,
    types::{AutomationJob, Keeper, TriggerType, RegistryState},
    error::{SolCronError, SolCronResult},
    utils::{Utils, TimeUtils},
    batch::{BatchOperations, BatchJobParams, BatchConfig},
    monitoring::{Monitor, MonitoringConfig, SystemMetrics},
};
use solana_sdk::{
    signature::{Keypair, Signer},
    pubkey::Pubkey,
};
use std::{
    collections::HashMap,
    sync::{Arc, atomic::{AtomicU64, Ordering}},
    time::{Duration, Instant},
};
use tokio::{time::sleep, sync::RwLock};
use rand::{Rng, seq::SliceRandom};

/// Configuration for simulation scenarios
#[derive(Debug, Clone)]
pub struct SimulationConfig {
    /// Duration of the simulation (seconds)
    pub duration_seconds: u64,
    /// Number of jobs to simulate
    pub job_count: u64,
    /// Number of keepers to simulate
    pub keeper_count: u64,
    /// Base execution interval for time-based jobs (seconds)
    pub base_interval: u64,
    /// Percentage of jobs that should fail (0-100)
    pub failure_rate: f64,
    /// Network latency simulation (milliseconds)
    pub network_latency_ms: u64,
    /// Whether to enable detailed logging
    pub verbose: bool,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            duration_seconds: 3600, // 1 hour
            job_count: 100,
            keeper_count: 10,
            base_interval: 300, // 5 minutes
            failure_rate: 5.0, // 5% failure rate
            network_latency_ms: 100,
            verbose: false,
        }
    }
}

/// Results from a simulation run
#[derive(Debug)]
pub struct SimulationResults {
    pub duration_seconds: u64,
    pub total_jobs_created: u64,
    pub total_keepers_registered: u64,
    pub total_executions_attempted: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub total_gas_used: u64,
    pub total_fees_paid: u64,
    pub avg_execution_time_ms: f64,
    pub peak_tps: f64,
    pub keeper_performance: Vec<KeeperSimResults>,
    pub system_metrics_history: Vec<SystemMetrics>,
}

impl SimulationResults {
    pub fn success_rate(&self) -> f64 {
        if self.total_executions_attempted == 0 {
            100.0
        } else {
            (self.successful_executions as f64 / self.total_executions_attempted as f64) * 100.0
        }
    }
    
    pub fn print_summary(&self) {
        println!("\n🎯 Simulation Results Summary");
        println!("═══════════════════════════════");
        println!("⏱️  Duration: {} minutes", self.duration_seconds / 60);
        println!("🎯 Jobs Created: {}", self.total_jobs_created);
        println!("🛡️  Keepers Registered: {}", self.total_keepers_registered);
        println!("🚀 Total Executions: {}", self.total_executions_attempted);
        println!("✅ Successful: {} ({:.1}%)", self.successful_executions, self.success_rate());
        println!("❌ Failed: {}", self.failed_executions);
        println!("⛽ Total Gas Used: {}", self.total_gas_used);
        println!("💰 Total Fees: {} SOL", Utils::lamports_to_sol_string(self.total_fees_paid, 6));
        println!("⚡ Avg Execution Time: {:.1}ms", self.avg_execution_time_ms);
        println!("📈 Peak TPS: {:.2}", self.peak_tps);
        
        if !self.keeper_performance.is_empty() {
            println!("\n🏆 Top Performing Keepers:");
            let mut sorted_keepers = self.keeper_performance.clone();
            sorted_keepers.sort_by(|a, b| b.success_rate().partial_cmp(&a.success_rate()).unwrap());
            
            for (i, keeper) in sorted_keepers.iter().take(5).enumerate() {
                println!("  {}. {} - {:.1}% success, {} executions, {} SOL earned",
                    i + 1,
                    &keeper.address[..8],
                    keeper.success_rate(),
                    keeper.total_executions,
                    Utils::lamports_to_sol_string(keeper.total_earnings, 4)
                );
            }
        }
        
        println!();
    }
}

#[derive(Debug, Clone)]
pub struct KeeperSimResults {
    pub address: String,
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub total_earnings: u64,
    pub avg_response_time_ms: f64,
}

impl KeeperSimResults {
    pub fn success_rate(&self) -> f64 {
        if self.total_executions == 0 {
            100.0
        } else {
            (self.successful_executions as f64 / self.total_executions as f64) * 100.0
        }
    }
}

/// Load testing configuration
#[derive(Debug, Clone)]
pub struct LoadTestConfig {
    /// Target transactions per second
    pub target_tps: f64,
    /// Test duration (seconds)
    pub duration_seconds: u64,
    /// Ramp up period (seconds)
    pub ramp_up_seconds: u64,
    /// Maximum concurrent operations
    pub max_concurrent: usize,
    /// Types of operations to test
    pub operation_mix: OperationMix,
}

#[derive(Debug, Clone)]
pub struct OperationMix {
    pub job_registrations: u32, // Percentage
    pub job_executions: u32,    // Percentage
    pub balance_top_ups: u32,   // Percentage
    pub status_queries: u32,    // Percentage
}

impl Default for OperationMix {
    fn default() -> Self {
        Self {
            job_registrations: 10,
            job_executions: 60,
            balance_top_ups: 15,
            status_queries: 15,
        }
    }
}

/// Advanced simulation and testing framework
pub struct Simulator {
    client: SolCronClient,
    config: SimulationConfig,
    // Shared state for concurrent operations
    execution_counter: Arc<AtomicU64>,
    success_counter: Arc<AtomicU64>,
    failure_counter: Arc<AtomicU64>,
    gas_used_counter: Arc<AtomicU64>,
    fees_paid_counter: Arc<AtomicU64>,
    
    // Simulated entities
    simulated_jobs: Arc<RwLock<HashMap<u64, AutomationJob>>>,
    simulated_keepers: Arc<RwLock<HashMap<Pubkey, Keeper>>>,
    execution_times: Arc<RwLock<Vec<f64>>>,
}

impl Simulator {
    /// Create a new simulation framework
    pub fn new(client: SolCronClient, config: SimulationConfig) -> Self {
        Self {
            client,
            config,
            execution_counter: Arc::new(AtomicU64::new(0)),
            success_counter: Arc::new(AtomicU64::new(0)),
            failure_counter: Arc::new(AtomicU64::new(0)),
            gas_used_counter: Arc::new(AtomicU64::new(0)),
            fees_paid_counter: Arc::new(AtomicU64::new(0)),
            simulated_jobs: Arc::new(RwLock::new(HashMap::new())),
            simulated_keepers: Arc::new(RwLock::new(HashMap::new())),
            execution_times: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Run a comprehensive simulation
    pub async fn run_simulation(&self) -> SolCronResult<SimulationResults> {
        println!("🎬 Starting SolCron simulation...");
        println!("📋 Config: {} jobs, {} keepers, {} seconds", 
            self.config.job_count, self.config.keeper_count, self.config.duration_seconds);
        
        let start_time = Instant::now();
        
        // Phase 1: Setup - Create jobs and register keepers
        self.setup_simulation().await?;
        
        // Phase 2: Execution - Run the simulation
        let monitor = self.run_execution_phase().await?;
        
        // Phase 3: Cleanup and results
        let results = self.collect_results(start_time.elapsed(), monitor).await?;
        
        println!("✅ Simulation completed successfully!");
        results.print_summary();
        
        Ok(results)
    }

    async fn setup_simulation(&self) -> SolCronResult<()> {
        println!("🔧 Setting up simulation environment...");
        
        // Register simulated keepers
        self.register_simulated_keepers().await?;
        
        // Create simulated jobs
        self.create_simulated_jobs().await?;
        
        println!("✅ Simulation setup complete");
        Ok(())
    }

    async fn register_simulated_keepers(&self) -> SolCronResult<()> {
        let mut keepers = self.simulated_keepers.write().await;
        let mut rng = rand::thread_rng();
        
        for i in 0..self.config.keeper_count {
            let keeper_keypair = Keypair::new();
            let address = keeper_keypair.pubkey();
            
            // Simulate keeper with realistic stats
            let keeper = Keeper {
                keeper: address,
                stake_amount: Utils::sol_to_lamports(rng.gen_range(1.0..10.0)),
                reputation_score: rng.gen_range(7000..9500), // 70-95% reputation
                is_active: true,
                total_executions: rng.gen_range(0..1000),
                successful_executions: rng.gen_range(0..1000),
                total_earnings: Utils::sol_to_lamports(rng.gen_range(0.1..5.0)),
                pending_rewards: 0,
                last_execution_time: Utils::current_timestamp() - rng.gen_range(0..3600),
                registered_at: Utils::current_timestamp() - rng.gen_range(0..86400 * 30),
            };
            
            keepers.insert(address, keeper);
            
            if self.config.verbose {
                println!("🛡️  Registered keeper {} ({}/{})", 
                    &address.to_string()[..8], i + 1, self.config.keeper_count);
            }
        }
        
        Ok(())
    }

    async fn create_simulated_jobs(&self) -> SolCronResult<()> {
        let mut jobs = self.simulated_jobs.write().await;
        let mut rng = rand::thread_rng();
        
        let trigger_types = vec![
            TriggerType::TimeBased { interval: self.config.base_interval },
            TriggerType::TimeBased { interval: self.config.base_interval / 2 },
            TriggerType::TimeBased { interval: self.config.base_interval * 2 },
            TriggerType::Conditional { condition: "price > threshold".to_string() },
            TriggerType::LogBased { event_signature: "Transfer(address,address,uint256)".to_string() },
        ];
        
        for i in 0..self.config.job_count {
            let job_id = i;
            let trigger_type = trigger_types.choose(&mut rng).unwrap().clone();
            
            let job = AutomationJob {
                job_id,
                owner: Keypair::new().pubkey(),
                target_program: Keypair::new().pubkey(),
                target_instruction: format!("instruction_{}", rng.gen_range(1..10)),
                trigger_type,
                trigger_params: vec![],
                gas_limit: rng.gen_range(100_000..500_000),
                balance: Utils::sol_to_lamports(rng.gen_range(0.01..0.1)),
                min_balance: Utils::sol_to_lamports(0.001),
                is_active: true,
                execution_count: 0,
                last_execution: 0,
                created_at: Utils::current_timestamp() - rng.gen_range(0..86400),
            };
            
            jobs.insert(job_id, job);
            
            if self.config.verbose && (i + 1) % 10 == 0 {
                println!("🎯 Created jobs {}/{}", i + 1, self.config.job_count);
            }
        }
        
        Ok(())
    }

    async fn run_execution_phase(&self) -> SolCronResult<Monitor> {
        println!("⚡ Starting execution phase...");
        
        // Set up monitoring
        let monitoring_config = MonitoringConfig {
            collection_interval: 30, // Collect every 30 seconds during simulation
            history_retention: 200,
            ..Default::default()
        };
        let mut monitor = Monitor::new(self.client.clone(), Some(monitoring_config));
        
        // Start execution simulation
        let simulation_tasks = self.spawn_execution_tasks().await?;
        
        // Run for specified duration
        let mut elapsed = 0u64;
        let report_interval = 60; // Report every minute
        
        while elapsed < self.config.duration_seconds {
            sleep(Duration::from_secs(report_interval)).await;
            elapsed += report_interval;
            
            // Collect metrics
            if let Ok(metrics) = monitor.collect_metrics().await {
                if self.config.verbose {
                    println!("📊 [{}/{}s] Executions: {}, Success Rate: {:.1}%",
                        elapsed, self.config.duration_seconds,
                        self.execution_counter.load(Ordering::Relaxed),
                        if self.execution_counter.load(Ordering::Relaxed) > 0 {
                            (self.success_counter.load(Ordering::Relaxed) as f64 / 
                             self.execution_counter.load(Ordering::Relaxed) as f64) * 100.0
                        } else { 100.0 }
                    );
                }
            }
        }
        
        println!("🏁 Execution phase completed");
        Ok(monitor)
    }

    async fn spawn_execution_tasks(&self) -> SolCronResult<Vec<tokio::task::JoinHandle<()>>> {
        let mut tasks = Vec::new();
        
        // Spawn keeper simulation tasks
        let keepers: Vec<Pubkey> = {
            let keeper_map = self.simulated_keepers.read().await;
            keeper_map.keys().cloned().collect()
        };
        
        for keeper_address in keepers {
            let task = self.spawn_keeper_task(keeper_address).await;
            tasks.push(task);
        }
        
        Ok(tasks)
    }

    async fn spawn_keeper_task(&self, keeper_address: Pubkey) -> tokio::task::JoinHandle<()> {
        let jobs = Arc::clone(&self.simulated_jobs);
        let execution_counter = Arc::clone(&self.execution_counter);
        let success_counter = Arc::clone(&self.success_counter);
        let failure_counter = Arc::clone(&self.failure_counter);
        let gas_used_counter = Arc::clone(&self.gas_used_counter);
        let fees_paid_counter = Arc::clone(&self.fees_paid_counter);
        let execution_times = Arc::clone(&self.execution_times);
        let config = self.config.clone();
        
        tokio::spawn(async move {
            let mut rng = rand::thread_rng();
            
            loop {
                // Simulate keeper checking for executable jobs
                let jobs_snapshot = {
                    let jobs_read = jobs.read().await;
                    jobs_read.clone()
                };
                
                for (job_id, job) in jobs_snapshot {
                    if !job.is_active {
                        continue;
                    }
                    
                    // Simulate trigger evaluation
                    let should_execute = match &job.trigger_type {
                        TriggerType::TimeBased { interval } => {
                            Utils::should_execute_time_trigger(
                                *interval,
                                job.last_execution,
                                Utils::current_timestamp(),
                            )
                        }
                        _ => rng.gen_bool(0.1), // 10% chance for other trigger types
                    };
                    
                    if should_execute {
                        // Simulate execution
                        let execution_start = Instant::now();
                        execution_counter.fetch_add(1, Ordering::Relaxed);
                        
                        // Simulate network latency
                        if config.network_latency_ms > 0 {
                            sleep(Duration::from_millis(config.network_latency_ms)).await;
                        }
                        
                        // Determine if execution succeeds
                        let success = rng.gen_range(0.0..100.0) > config.failure_rate;
                        
                        if success {
                            success_counter.fetch_add(1, Ordering::Relaxed);
                            
                            // Simulate gas usage and fees
                            let gas_used = rng.gen_range(job.gas_limit / 2..job.gas_limit);
                            let fee = gas_used * rng.gen_range(1..5); // 1-5 lamports per gas
                            
                            gas_used_counter.fetch_add(gas_used, Ordering::Relaxed);
                            fees_paid_counter.fetch_add(fee, Ordering::Relaxed);
                            
                            // Update job state
                            {
                                let mut jobs_write = jobs.write().await;
                                if let Some(job) = jobs_write.get_mut(&job_id) {
                                    job.execution_count += 1;
                                    job.last_execution = Utils::current_timestamp();
                                    job.balance = job.balance.saturating_sub(fee);
                                }
                            }
                        } else {
                            failure_counter.fetch_add(1, Ordering::Relaxed);
                        }
                        
                        // Record execution time
                        let execution_time = execution_start.elapsed().as_millis() as f64;
                        {
                            let mut times = execution_times.write().await;
                            times.push(execution_time);
                        }
                    }
                }
                
                // Keeper checks every few seconds
                sleep(Duration::from_secs(rng.gen_range(2..8))).await;
            }
        })
    }

    async fn collect_results(&self, duration: Duration, monitor: Monitor) -> SolCronResult<SimulationResults> {
        let total_executions = self.execution_counter.load(Ordering::Relaxed);
        let successful = self.success_counter.load(Ordering::Relaxed);
        let failed = self.failure_counter.load(Ordering::Relaxed);
        
        // Calculate average execution time
        let execution_times = self.execution_times.read().await;
        let avg_execution_time = if execution_times.is_empty() {
            0.0
        } else {
            execution_times.iter().sum::<f64>() / execution_times.len() as f64
        };
        
        // Calculate peak TPS (simplified - based on total executions)
        let peak_tps = total_executions as f64 / duration.as_secs() as f64;
        
        // Collect keeper performance
        let keeper_performance = self.collect_keeper_performance().await;
        
        Ok(SimulationResults {
            duration_seconds: duration.as_secs(),
            total_jobs_created: self.config.job_count,
            total_keepers_registered: self.config.keeper_count,
            total_executions_attempted: total_executions,
            successful_executions: successful,
            failed_executions: failed,
            total_gas_used: self.gas_used_counter.load(Ordering::Relaxed),
            total_fees_paid: self.fees_paid_counter.load(Ordering::Relaxed),
            avg_execution_time_ms: avg_execution_time,
            peak_tps,
            keeper_performance,
            system_metrics_history: vec![], // Would contain monitor history in real implementation
        })
    }

    async fn collect_keeper_performance(&self) -> Vec<KeeperSimResults> {
        let keepers = self.simulated_keepers.read().await;
        let mut performance = Vec::new();
        
        for (address, keeper) in keepers.iter() {
            performance.push(KeeperSimResults {
                address: address.to_string(),
                total_executions: keeper.total_executions,
                successful_executions: keeper.successful_executions,
                failed_executions: keeper.total_executions - keeper.successful_executions,
                total_earnings: keeper.total_earnings,
                avg_response_time_ms: 150.0, // Simulated average
            });
        }
        
        performance
    }

    /// Run a focused load test
    pub async fn run_load_test(&self, config: LoadTestConfig) -> SolCronResult<LoadTestResults> {
        println!("🔥 Starting load test - Target: {:.1} TPS for {} seconds", 
            config.target_tps, config.duration_seconds);
        
        let start_time = Instant::now();
        let mut operations_completed = 0u64;
        let mut successful_ops = 0u64;
        let mut failed_ops = 0u64;
        
        // Calculate operation interval
        let interval = Duration::from_secs_f64(1.0 / config.target_tps);
        
        while start_time.elapsed().as_secs() < config.duration_seconds {
            let op_start = Instant::now();
            
            // Simulate operation based on mix
            match self.select_operation(&config.operation_mix).await {
                Ok(_) => successful_ops += 1,
                Err(_) => failed_ops += 1,
            }
            
            operations_completed += 1;
            
            // Maintain target TPS
            let elapsed = op_start.elapsed();
            if elapsed < interval {
                sleep(interval - elapsed).await;
            }
        }
        
        let actual_duration = start_time.elapsed();
        let actual_tps = operations_completed as f64 / actual_duration.as_secs_f64();
        
        let results = LoadTestResults {
            target_tps: config.target_tps,
            actual_tps,
            duration_seconds: actual_duration.as_secs(),
            total_operations: operations_completed,
            successful_operations: successful_ops,
            failed_operations: failed_ops,
            success_rate: (successful_ops as f64 / operations_completed as f64) * 100.0,
        };
        
        results.print_summary();
        Ok(results)
    }

    async fn select_operation(&self, mix: &OperationMix) -> SolCronResult<()> {
        let mut rng = rand::thread_rng();
        let roll = rng.gen_range(1..=100);
        
        match roll {
            r if r <= mix.job_registrations => {
                // Simulate job registration
                sleep(Duration::from_millis(50)).await;
                Ok(())
            }
            r if r <= mix.job_registrations + mix.job_executions => {
                // Simulate job execution
                sleep(Duration::from_millis(rng.gen_range(100..300))).await;
                if rng.gen_bool(0.95) { Ok(()) } else { Err(SolCronError::ExecutionFailed { job_id: 1, reason: "Simulated failure".to_string() }) }
            }
            r if r <= mix.job_registrations + mix.job_executions + mix.balance_top_ups => {
                // Simulate balance top-up
                sleep(Duration::from_millis(30)).await;
                Ok(())
            }
            _ => {
                // Simulate status query
                sleep(Duration::from_millis(10)).await;
                Ok(())
            }
        }
    }
}

/// Results from a load test
#[derive(Debug)]
pub struct LoadTestResults {
    pub target_tps: f64,
    pub actual_tps: f64,
    pub duration_seconds: u64,
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub success_rate: f64,
}

impl LoadTestResults {
    pub fn print_summary(&self) {
        println!("\n🔥 Load Test Results");
        println!("==================");
        println!("🎯 Target TPS: {:.1}", self.target_tps);
        println!("📈 Actual TPS: {:.1}", self.actual_tps);
        println!("⏱️  Duration: {} seconds", self.duration_seconds);
        println!("🚀 Total Operations: {}", self.total_operations);
        println!("✅ Successful: {} ({:.1}%)", self.successful_operations, self.success_rate);
        println!("❌ Failed: {}", self.failed_operations);
        println!("📊 TPS Achievement: {:.1}%", (self.actual_tps / self.target_tps) * 100.0);
        println!();
    }
}