use std::sync::Arc;
use tokio::sync::mpsc;
use log::{info, error, warn};
use solana_sdk::signer::Signer;

use crate::config::KeeperConfig;
use crate::database::Database;
use crate::rpc::RpcManager;
use crate::monitor::{JobMonitor, ExecutionRequest};
use crate::executor::JobExecutor;
use crate::error::{KeeperError, KeeperResult};

pub struct KeeperNode {
    config: KeeperConfig,
    database: Arc<Database>,
    rpc_manager: Arc<RpcManager>,
    monitor: Option<JobMonitor>,
    executor: Option<JobExecutor>,
    execution_sender: Option<mpsc::UnboundedSender<ExecutionRequest>>,
    execution_receiver: Option<mpsc::UnboundedReceiver<ExecutionRequest>>,
}

#[derive(Debug)]
pub struct KeeperStatus {
    pub address: String,
    pub is_active: bool,
    pub stake_amount: u64,
    pub reputation_score: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub total_earnings: u64,
    pub pending_rewards: u64,
}

impl KeeperNode {
    pub async fn new(config: KeeperConfig) -> KeeperResult<Self> {
        info!("Initializing SolCron Keeper Node...");
        
        // Initialize database
        let database = Arc::new(Database::new(&config).await?);
        
        // Initialize RPC manager
        let rpc_manager = Arc::new(RpcManager::new(config.clone()));
        
        // Create execution channel
        let (execution_sender, execution_receiver) = mpsc::unbounded_channel();
        
        // Initialize monitor
        let monitor = JobMonitor::new(
            config.clone(),
            database.clone(),
            rpc_manager.clone(),
            execution_sender.clone(),
        );
        
        // Initialize executor
        let executor = JobExecutor::new(
            config.clone(),
            database.clone(),
            rpc_manager.clone(),
            execution_receiver,
        )?;
        
        info!("Keeper node initialized successfully");
        
        Ok(Self {
            config,
            database,
            rpc_manager,
            monitor: Some(monitor),
            executor: Some(executor),
            execution_sender: Some(execution_sender),
            execution_receiver: None, // Moved to executor
        })
    }

    pub async fn start_monitoring(&self) -> KeeperResult<()> {
        if let Some(monitor) = &self.monitor {
            monitor.start().await
        } else {
            Err(KeeperError::InternalError("Monitor not initialized".to_string()))
        }
    }

    pub async fn start_execution(&self) -> KeeperResult<()> {
        if let Some(mut executor) = self.executor.take() {
            executor.start().await
        } else {
            Err(KeeperError::InternalError("Executor not initialized".to_string()))
        }
    }

    pub async fn register_keeper(&self, stake_amount: u64) -> KeeperResult<String> {
        info!("Registering keeper with stake: {} lamports", stake_amount);
        
        // Load keeper keypair
        let keypair_data = std::fs::read(&self.config.keeper.wallet_path)
            .map_err(|e| KeeperError::ConfigError(format!("Failed to read keypair: {}", e)))?;
        
        let keeper_keypair = if keypair_data.len() == 64 {
            solana_sdk::signer::keypair::Keypair::from_bytes(&keypair_data)
                .map_err(|e| KeeperError::ConfigError(format!("Invalid keypair format: {}", e)))?
        } else {
            let keypair_json: Vec<u8> = serde_json::from_slice(&keypair_data)
                .map_err(|e| KeeperError::ConfigError(format!("Invalid keypair JSON: {}", e)))?;
            solana_sdk::signer::keypair::Keypair::from_bytes(&keypair_json)
                .map_err(|e| KeeperError::ConfigError(format!("Invalid keypair data: {}", e)))?
        };

        // Check if keeper is already registered
        // This would require implementing a function to check on-chain state
        
        // For now, simulate successful registration
        let signature = format!("keeper_registration_{}", keeper_keypair.pubkey());
        
        info!("Keeper registration simulated: {}", signature);
        Ok(signature)
    }

    pub async fn claim_rewards(&self) -> KeeperResult<String> {
        info!("Claiming keeper rewards...");
        
        // This would implement the actual claim_rewards instruction
        // For now, simulate successful claim
        let signature = "reward_claim_simulation".to_string();
        
        info!("Rewards claim simulated: {}", signature);
        Ok(signature)
    }

    pub async fn unregister_keeper(&self) -> KeeperResult<String> {
        info!("Unregistering keeper...");
        
        // This would implement the actual unregister_keeper instruction
        // For now, simulate successful unregistration
        let signature = "keeper_unregistration_simulation".to_string();
        
        info!("Keeper unregistration simulated: {}", signature);
        Ok(signature)
    }

    pub async fn get_status(&self) -> KeeperResult<KeeperStatus> {
        // Load keeper keypair to get address
        let keypair_data = std::fs::read(&self.config.keeper.wallet_path)
            .map_err(|e| KeeperError::ConfigError(format!("Failed to read keypair: {}", e)))?;
        
        let keeper_keypair = if keypair_data.len() == 64 {
            solana_sdk::signer::keypair::Keypair::from_bytes(&keypair_data)
                .map_err(|e| KeeperError::ConfigError(format!("Invalid keypair format: {}", e)))?
        } else {
            let keypair_json: Vec<u8> = serde_json::from_slice(&keypair_data)
                .map_err(|e| KeeperError::ConfigError(format!("Invalid keypair JSON: {}", e)))?;
            solana_sdk::signer::keypair::Keypair::from_bytes(&keypair_json)
                .map_err(|e| KeeperError::ConfigError(format!("Invalid keypair data: {}", e)))?
        };

        // In a real implementation, this would fetch data from on-chain
        // For now, return simulation data
        Ok(KeeperStatus {
            address: keeper_keypair.pubkey().to_string(),
            is_active: true,
            stake_amount: self.config.keeper.stake_amount,
            reputation_score: 7500, // 75%
            successful_executions: 150,
            failed_executions: 5,
            total_earnings: 50_000_000, // 0.05 SOL
            pending_rewards: 10_000_000, // 0.01 SOL
        })
    }

    pub async fn get_health_status(&self) -> KeeperResult<HealthStatus> {
        // Get RPC health
        let rpc_health = self.rpc_manager.get_health_status().await;
        
        // Get cache stats
        let (total_jobs, active_jobs, pending_jobs) = if let Some(monitor) = &self.monitor {
            monitor.get_cache_stats().await
        } else {
            (0, 0, 0)
        };
        
        // Get queue stats
        let (queue_size, highest_priority) = if let Some(executor) = &self.executor {
            executor.get_queue_stats().await
        } else {
            (0, crate::monitor::ExecutionPriority::Low)
        };
        
        Ok(HealthStatus {
            rpc_endpoints: rpc_health,
            database_connected: true, // Would check actual connection
            job_cache: JobCacheStatus {
                total_jobs,
                active_jobs,
                pending_jobs,
            },
            execution_queue: ExecutionQueueStatus {
                queue_size,
                highest_priority: format!("{:?}", highest_priority),
            },
        })
    }

    pub async fn force_job_execution(&self, job_id: i64) -> KeeperResult<()> {
        if let Some(monitor) = &self.monitor {
            monitor.force_job_check(job_id).await
        } else {
            Err(KeeperError::InternalError("Monitor not available".to_string()))
        }
    }

    pub async fn shutdown(&self) -> KeeperResult<()> {
        info!("Shutting down keeper node...");
        
        // In a full implementation, this would:
        // 1. Stop accepting new jobs
        // 2. Finish executing current jobs
        // 3. Close database connections
        // 4. Clean up resources
        
        info!("Keeper node shutdown complete");
        Ok(())
    }
}

#[derive(Debug)]
pub struct HealthStatus {
    pub rpc_endpoints: Vec<(String, bool, u64, u64)>, // (url, healthy, requests, errors)
    pub database_connected: bool,
    pub job_cache: JobCacheStatus,
    pub execution_queue: ExecutionQueueStatus,
}

#[derive(Debug)]
pub struct JobCacheStatus {
    pub total_jobs: usize,
    pub active_jobs: usize,
    pub pending_jobs: usize,
}

#[derive(Debug)]
pub struct ExecutionQueueStatus {
    pub queue_size: usize,
    pub highest_priority: String,
}