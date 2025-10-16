use std::sync::Arc;
use std::collections::BinaryHeap;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{sleep, Duration, Instant};
use solana_sdk::{
    pubkey::Pubkey, 
    signer::{keypair::Keypair, Signer},
    transaction::Transaction,
    instruction::{Instruction, AccountMeta},
    system_program,
};
use anchor_client::{Client, Program, Cluster};
use log::{info, warn, error, debug};
use chrono::Utc;

use crate::config::KeeperConfig;
use crate::database::{Database, ExecutionRecord};
use crate::rpc::RpcManager;
use crate::monitor::{ExecutionRequest, ExecutionPriority};
use crate::error::{KeeperError, KeeperResult};

pub struct JobExecutor {
    config: KeeperConfig,
    database: Arc<Database>,
    rpc_manager: Arc<RpcManager>,
    keeper_keypair: Arc<Keypair>,
    execution_queue: Arc<Mutex<BinaryHeap<PrioritizedExecution>>>,
    execution_receiver: mpsc::UnboundedReceiver<ExecutionRequest>,
}

#[derive(Debug, Clone)]
struct PrioritizedExecution {
    request: ExecutionRequest,
    queued_at: Instant,
}

impl PartialEq for PrioritizedExecution {
    fn eq(&self, other: &Self) -> bool {
        self.request.priority == other.request.priority
    }
}

impl Eq for PrioritizedExecution {}

impl PartialOrd for PrioritizedExecution {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PrioritizedExecution {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Higher priority first, then older requests first
        self.request.priority.cmp(&other.request.priority)
            .then_with(|| other.queued_at.cmp(&self.queued_at))
    }
}

pub struct ExecutionResult {
    pub success: bool,
    pub signature: Option<String>,
    pub error: Option<String>,
    pub gas_used: u64,
    pub fee_paid: u64,
}

impl JobExecutor {
    pub fn new(
        config: KeeperConfig,
        database: Arc<Database>,
        rpc_manager: Arc<RpcManager>,
        execution_receiver: mpsc::UnboundedReceiver<ExecutionRequest>,
    ) -> KeeperResult<Self> {
        // Load keeper keypair
        let keypair_data = std::fs::read(&config.keeper.wallet_path)
            .map_err(|e| KeeperError::ConfigError(format!("Failed to read keypair: {}", e)))?;
        
        let keeper_keypair = if keypair_data.len() == 64 {
            // Raw bytes format
            Keypair::from_bytes(&keypair_data)
                .map_err(|e| KeeperError::ConfigError(format!("Invalid keypair format: {}", e)))?
        } else {
            // JSON format
            let keypair_json: Vec<u8> = serde_json::from_slice(&keypair_data)
                .map_err(|e| KeeperError::ConfigError(format!("Invalid keypair JSON: {}", e)))?;
            Keypair::from_bytes(&keypair_json)
                .map_err(|e| KeeperError::ConfigError(format!("Invalid keypair data: {}", e)))?
        };

        Ok(Self {
            config,
            database,
            rpc_manager,
            keeper_keypair: Arc::new(keeper_keypair),
            execution_queue: Arc::new(Mutex::new(BinaryHeap::new())),
            execution_receiver,
        })
    }

    pub async fn start(&mut self) -> KeeperResult<()> {
        info!("Starting job executor with keypair: {}", self.keeper_keypair.pubkey());
        
        // Start queue processor
        let queue = self.execution_queue.clone();
        let database = self.database.clone();
        let rpc_manager = self.rpc_manager.clone();
        let keeper_keypair = self.keeper_keypair.clone();
        let config = self.config.clone();
        
        let processor_handle = tokio::spawn(async move {
            Self::process_execution_queue(
                queue,
                database,
                rpc_manager,
                keeper_keypair,
                config,
            ).await
        });
        
        // Receive and queue execution requests
        let queue = self.execution_queue.clone();
        loop {
            tokio::select! {
                request = self.execution_receiver.recv() => {
                    match request {
                        Some(request) => {
                            let prioritized = PrioritizedExecution {
                                request,
                                queued_at: Instant::now(),
                            };
                            
                            let mut queue_guard = queue.lock().await;
                            queue_guard.push(prioritized);
                            
                            debug!("Queued execution request for job {}", 
                                   queue_guard.peek().unwrap().request.job.job_id);
                        }
                        None => {
                            warn!("Execution receiver closed");
                            break;
                        }
                    }
                }
                result = &mut processor_handle => {
                    if let Err(e) = result {
                        error!("Execution processor failed: {:?}", e);
                    }
                    break;
                }
            }
        }
        
        Ok(())
    }

    async fn process_execution_queue(
        queue: Arc<Mutex<BinaryHeap<PrioritizedExecution>>>,
        database: Arc<Database>,
        rpc_manager: Arc<RpcManager>,
        keeper_keypair: Arc<Keypair>,
        config: KeeperConfig,
    ) -> KeeperResult<()> {
        loop {
            // Get next execution request
            let execution = {
                let mut queue_guard = queue.lock().await;
                queue_guard.pop()
            };
            
            if let Some(execution) = execution {
                info!("Executing job {} (priority: {:?})", 
                      execution.request.job.job_id, execution.request.priority);
                
                let result = Self::execute_job(
                    &execution.request,
                    &database,
                    &rpc_manager,
                    &keeper_keypair,
                    &config,
                ).await;
                
                // Record execution result
                if let Err(e) = Self::record_execution_result(
                    &execution.request,
                    result,
                    &database,
                    &keeper_keypair.pubkey(),
                ).await {
                    error!("Failed to record execution result: {:?}", e);
                }
                
                // Small delay between executions
                sleep(Duration::from_millis(100)).await;
            } else {
                // No executions in queue, wait a bit
                sleep(Duration::from_millis(500)).await;
            }
        }
    }

    async fn execute_job(
        request: &ExecutionRequest,
        database: &Database,
        rpc_manager: &RpcManager,
        keeper_keypair: &Keypair,
        config: &KeeperConfig,
    ) -> ExecutionResult {
        let job = &request.job;
        
        debug!("Executing job {}: {}", job.job_id, job.target_instruction);
        
        // Build the execution instruction
        let instruction_result = Self::build_execution_instruction(
            job,
            keeper_keypair,
        ).await;
        
        let instruction = match instruction_result {
            Ok(ix) => ix,
            Err(e) => {
                error!("Failed to build execution instruction: {:?}", e);
                return ExecutionResult {
                    success: false,
                    signature: None,
                    error: Some(format!("Failed to build instruction: {}", e)),
                    gas_used: 0,
                    fee_paid: 0,
                };
            }
        };
        
        // Get recent blockhash
        let blockhash = match rpc_manager.get_latest_blockhash().await {
            Ok(hash) => hash,
            Err(e) => {
                error!("Failed to get blockhash: {:?}", e);
                return ExecutionResult {
                    success: false,
                    signature: None,
                    error: Some(format!("Failed to get blockhash: {}", e)),
                    gas_used: 0,
                    fee_paid: 0,
                };
            }
        };
        
        // Create transaction
        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&keeper_keypair.pubkey()),
            &[keeper_keypair],
            blockhash,
        );
        
        // Simulate transaction first if enabled
        if config.simulation_enabled() {
            match rpc_manager.simulate_transaction(&transaction).await {
                Ok(simulation) => {
                    if simulation.value.err.is_some() {
                        warn!("Transaction simulation failed for job {}: {:?}", 
                              job.job_id, simulation.value.err);
                        return ExecutionResult {
                            success: false,
                            signature: None,
                            error: Some(format!("Simulation failed: {:?}", simulation.value.err)),
                            gas_used: 0,
                            fee_paid: 0,
                        };
                    } else {
                        debug!("Transaction simulation succeeded for job {}", job.job_id);
                    }
                }
                Err(e) => {
                    warn!("Failed to simulate transaction for job {}: {:?}", job.job_id, e);
                    // Continue with execution anyway
                }
            }
        }
        
        // Execute transaction with retries
        let mut last_error = None;
        for attempt in 0..config.execution.max_retries {
            match rpc_manager.send_and_confirm_transaction(&transaction).await {
                Ok(signature) => {
                    info!("Job {} executed successfully: {}", job.job_id, signature);
                    return ExecutionResult {
                        success: true,
                        signature: Some(signature.to_string()),
                        error: None,
                        gas_used: 0, // Would need to parse transaction logs for actual value
                        fee_paid: 5000, // Placeholder - would calculate actual fee
                    };
                }
                Err(e) => {
                    last_error = Some(e.to_string());
                    warn!("Job {} execution failed (attempt {}): {:?}", 
                          job.job_id, attempt + 1, e);
                    
                    if attempt < config.execution.max_retries - 1 {
                        let delay = config.get_retry_delay() * (2_u32.pow(attempt));
                        debug!("Retrying job {} in {:?}", job.job_id, delay);
                        sleep(delay).await;
                    }
                }
            }
        }
        
        ExecutionResult {
            success: false,
            signature: None,
            error: last_error,
            gas_used: 0,
            fee_paid: 0,
        }
    }

    async fn build_execution_instruction(
        job: &crate::database::JobRecord,
        keeper_keypair: &Keypair,
    ) -> KeeperResult<Instruction> {
        // Build the execute_job instruction for the registry program
        let registry_program_id = "Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS"
            .parse::<Pubkey>()
            .map_err(|e| KeeperError::ConfigError(format!("Invalid program ID: {}", e)))?;

        let target_program_id = job.target_program.parse::<Pubkey>()
            .map_err(|e| KeeperError::InvalidJobError(format!("Invalid target program: {}", e)))?;

        // Derive PDAs
        let (registry_state, _) = Pubkey::find_program_address(
            &[b"registry"],
            &registry_program_id,
        );

        let (automation_job, _) = Pubkey::find_program_address(
            &[b"job", &job.job_id.to_le_bytes()],
            &registry_program_id,
        );

        let (keeper_account, _) = Pubkey::find_program_address(
            &[b"keeper", keeper_keypair.pubkey().as_ref()],
            &registry_program_id,
        );

        // This is a simplified version - in a full implementation, we'd need to:
        // 1. Get the current execution count from registry state
        // 2. Derive the execution record PDA properly
        // 3. Handle remaining accounts for the target program call
        
        let execution_count = 0u64; // Placeholder
        let (execution_record, _) = Pubkey::find_program_address(
            &[
                b"execution",
                &job.job_id.to_le_bytes(),
                &execution_count.to_le_bytes(),
            ],
            &registry_program_id,
        );

        let accounts = vec![
            AccountMeta::new(registry_state, false),
            AccountMeta::new(automation_job, false),
            AccountMeta::new(keeper_account, false),
            AccountMeta::new(execution_record, false),
            AccountMeta::new(keeper_keypair.pubkey(), true),
            AccountMeta::new_readonly(target_program_id, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ];

        // Build instruction data (discriminator + job_id)
        let mut instruction_data = Vec::new();
        // execute_job instruction discriminator (8 bytes)
        // This would need to be computed from the instruction name
        instruction_data.extend_from_slice(&[0u8; 8]); // Placeholder discriminator
        instruction_data.extend_from_slice(&job.job_id.to_le_bytes());

        Ok(Instruction {
            program_id: registry_program_id,
            accounts,
            data: instruction_data,
        })
    }

    async fn record_execution_result(
        request: &ExecutionRequest,
        result: ExecutionResult,
        database: &Database,
        keeper_address: &Pubkey,
    ) -> KeeperResult<()> {
        let execution_record = ExecutionRecord {
            id: 0, // Will be auto-generated
            job_id: request.job.job_id,
            keeper_address: keeper_address.to_string(),
            timestamp: Utc::now(),
            success: result.success,
            signature: result.signature,
            error: result.error,
            gas_used: Some(result.gas_used as i64),
            fee_paid: Some(result.fee_paid as i64),
        };

        database.record_execution(&execution_record).await?;
        
        // Update keeper stats
        let today = Utc::now().date_naive();
        database.update_keeper_stats(today, result.success, result.fee_paid as i64).await?;

        if result.success {
            info!("Execution recorded successfully for job {}", request.job.job_id);
        } else {
            warn!("Execution failure recorded for job {}", request.job.job_id);
        }

        Ok(())
    }

    pub async fn get_queue_stats(&self) -> (usize, ExecutionPriority) {
        let queue = self.execution_queue.lock().await;
        let size = queue.len();
        let highest_priority = queue.peek()
            .map(|e| e.request.priority.clone())
            .unwrap_or(ExecutionPriority::Low);
        
        (size, highest_priority)
    }

    pub async fn clear_queue(&self) -> usize {
        let mut queue = self.execution_queue.lock().await;
        let size = queue.len();
        queue.clear();
        size
    }
}