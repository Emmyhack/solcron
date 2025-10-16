//! High-level client for interacting with SolCron from applications
//!
//! This module provides a comprehensive client interface for applications to interact
//! with the SolCron automation platform. It includes methods for job management,
//! keeper operations, and monitoring.

#[cfg(feature = "client")]
use {
    anchor_client::{Client, Cluster, Program},
    solana_client::rpc_client::RpcClient,
    solana_sdk::{
        commitment_config::CommitmentConfig,
        pubkey::Pubkey,
        signature::{Keypair, Signature},
        signer::Signer,
        transaction::Transaction,
    },
    tokio::time::{sleep, Duration},
    std::sync::Arc,
};

use crate::{
    types::*,
    accounts::*,
    instructions::*,
    error::{SolCronError, SolCronResult},
    utils::*,
};

/// High-level client for SolCron operations
#[cfg(feature = "client")]
pub struct SolCronClient {
    /// Anchor client for program interactions
    client: Client<Arc<Keypair>>,
    /// SolCron registry program
    program: Program<Arc<Keypair>>,
    /// RPC client for direct Solana operations
    rpc_client: RpcClient,
    /// Default commitment level for transactions
    commitment: CommitmentConfig,
}

#[cfg(feature = "client")]
impl SolCronClient {
    /// Create a new SolCron client
    /// 
    /// # Arguments
    /// * `cluster_url` - Solana cluster RPC URL
    /// * `payer` - Keypair for transaction fees (optional, uses default if None)
    /// * `commitment` - Transaction commitment level (optional, uses confirmed if None)
    /// 
    /// # Example
    /// ```rust,no_run
    /// use solcron_sdk::SolCronClient;
    /// use solana_sdk::signature::Keypair;
    /// 
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let payer = Keypair::new();
    ///     let client = SolCronClient::new_with_payer(
    ///         "https://api.mainnet-beta.solana.com",
    ///         payer,
    ///         None
    ///     ).await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn new(cluster_url: &str) -> SolCronResult<Self> {
        let payer = Keypair::new();
        Self::new_with_payer(cluster_url, payer, None).await
    }

    /// Create a new SolCron client with custom payer
    pub async fn new_with_payer(
        cluster_url: &str,
        payer: Keypair,
        commitment: Option<CommitmentConfig>,
    ) -> SolCronResult<Self> {
        let commitment = commitment.unwrap_or(CommitmentConfig::confirmed());
        let cluster = Cluster::Custom(cluster_url.to_string(), cluster_url.to_string());
        
        let client = Client::new_with_options(
            cluster,
            Arc::new(payer),
            commitment,
        );

        let program = client.program(crate::REGISTRY_PROGRAM_ID)
            .map_err(|e| SolCronError::ConnectionError {
                source: format!("Failed to connect to SolCron program: {}", e),
            })?;

        let rpc_client = RpcClient::new_with_commitment(cluster_url, commitment);

        Ok(Self {
            client,
            program,
            rpc_client,
            commitment,
        })
    }

    /// Get the registry state
    pub async fn get_registry_state(&self) -> SolCronResult<RegistryState> {
        let (registry_state_address, _) = Accounts::registry_state()?;
        
        self.program
            .account::<RegistryState>(registry_state_address)
            .await
            .map_err(|e| SolCronError::AccountNotFound {
                account: format!("Registry state: {}", e),
            })
    }

    /// Register a new automation job
    /// 
    /// # Arguments
    /// * `job_params` - Job configuration parameters
    /// * `initial_funding` - Initial funding amount in lamports
    /// * `owner` - Job owner keypair
    /// 
    /// Returns the job ID of the created job
    /// 
    /// # Example
    /// ```rust,no_run
    /// use solcron_sdk::{SolCronClient, JobParams, TriggerType};
    /// use solana_sdk::{pubkey::Pubkey, signature::Keypair};
    /// 
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = SolCronClient::new("https://api.devnet.solana.com").await?;
    ///     let owner = Keypair::new();
    ///     
    ///     let job_params = JobParams {
    ///         target_program: Pubkey::new_unique(),
    ///         target_instruction: "harvest".to_string(),
    ///         trigger_type: TriggerType::TimeBased { interval: 3600 },
    ///         trigger_params: vec![],
    ///         gas_limit: 200_000,
    ///         min_balance: 1_000_000,
    ///     };
    ///     
    ///     let job_id = client.register_job(&job_params, 100_000_000, &owner).await?;
    ///     println!("Registered job with ID: {}", job_id);
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn register_job(
        &self,
        job_params: &JobParams,
        initial_funding: u64,
        owner: &Keypair,
    ) -> SolCronResult<u64> {
        // Get current registry state for next job ID
        let registry_state = self.get_registry_state().await?;
        let job_id = registry_state.next_job_id;

        // Get required accounts
        let accounts = Accounts::job_registration_accounts(&owner.pubkey(), job_id)?;

        // Build and send transaction
        let tx = self.program
            .request()
            .accounts(crate::accounts::RegisterJob {
                registry_state: accounts.registry_state,
                automation_job: accounts.automation_job,
                owner: accounts.owner,
                system_program: accounts.system_program,
            })
            .args(crate::instruction::RegisterJob {
                target_program: job_params.target_program,
                target_instruction: job_params.target_instruction.clone(),
                trigger_type: job_params.trigger_type.clone(),
                trigger_params: job_params.trigger_params.clone(),
                gas_limit: job_params.gas_limit,
                min_balance: job_params.min_balance,
                initial_funding,
            })
            .signer(owner)
            .send()
            .await
            .map_err(|e| SolCronError::TransactionExecutionError {
                reason: format!("Failed to register job: {}", e),
            })?;

        // Confirm transaction
        self.confirm_transaction(tx).await?;

        Ok(job_id)
    }

    /// Get job information
    /// 
    /// # Arguments
    /// * `job_id` - Job identifier
    /// 
    /// Returns the job account data
    pub async fn get_job(&self, job_id: u64) -> SolCronResult<AutomationJob> {
        let (job_address, _) = Accounts::automation_job(job_id)?;
        
        self.program
            .account::<AutomationJob>(job_address)
            .await
            .map_err(|e| SolCronError::JobNotFound { job_id })
    }

    /// Fund an existing job
    /// 
    /// # Arguments
    /// * `job_id` - Job identifier
    /// * `amount` - Additional funding amount in lamports
    /// * `funder` - Account providing the funding
    pub async fn fund_job(
        &self,
        job_id: u64,
        amount: u64,
        funder: &Keypair,
    ) -> SolCronResult<Signature> {
        let (job_address, _) = Accounts::automation_job(job_id)?;

        let tx = self.program
            .request()
            .accounts(crate::accounts::FundJob {
                automation_job: job_address,
                funder: funder.pubkey(),
                system_program: solana_sdk::system_program::ID,
            })
            .args(crate::instruction::FundJob { amount })
            .signer(funder)
            .send()
            .await
            .map_err(|e| SolCronError::TransactionExecutionError {
                reason: format!("Failed to fund job: {}", e),
            })?;

        self.confirm_transaction(tx).await
    }

    /// Update job parameters
    /// 
    /// # Arguments
    /// * `job_id` - Job identifier
    /// * `gas_limit` - New gas limit (optional)
    /// * `min_balance` - New minimum balance (optional)
    /// * `trigger_params` - New trigger parameters (optional)
    /// * `owner` - Job owner keypair
    pub async fn update_job(
        &self,
        job_id: u64,
        gas_limit: Option<u64>,
        min_balance: Option<u64>,
        trigger_params: Option<Vec<u8>>,
        owner: &Keypair,
    ) -> SolCronResult<Signature> {
        let (job_address, _) = Accounts::automation_job(job_id)?;

        let tx = self.program
            .request()
            .accounts(crate::accounts::UpdateJob {
                automation_job: job_address,
                owner: owner.pubkey(),
            })
            .args(crate::instruction::UpdateJob {
                gas_limit,
                min_balance,
                trigger_params,
            })
            .signer(owner)
            .send()
            .await
            .map_err(|e| SolCronError::TransactionExecutionError {
                reason: format!("Failed to update job: {}", e),
            })?;

        self.confirm_transaction(tx).await
    }

    /// Cancel a job and refund remaining balance
    /// 
    /// # Arguments
    /// * `job_id` - Job identifier
    /// * `owner` - Job owner keypair
    pub async fn cancel_job(&self, job_id: u64, owner: &Keypair) -> SolCronResult<Signature> {
        let (registry_state, _) = Accounts::registry_state()?;
        let (job_address, _) = Accounts::automation_job(job_id)?;

        let tx = self.program
            .request()
            .accounts(crate::accounts::CancelJob {
                registry_state,
                automation_job: job_address,
                owner: owner.pubkey(),
                system_program: solana_sdk::system_program::ID,
            })
            .args(crate::instruction::CancelJob)
            .signer(owner)
            .send()
            .await
            .map_err(|e| SolCronError::TransactionExecutionError {
                reason: format!("Failed to cancel job: {}", e),
            })?;

        self.confirm_transaction(tx).await
    }

    /// Register as a keeper
    /// 
    /// # Arguments
    /// * `stake_amount` - Amount to stake in lamports
    /// * `keeper` - Keeper keypair
    pub async fn register_keeper(
        &self,
        stake_amount: u64,
        keeper: &Keypair,
    ) -> SolCronResult<Signature> {
        let accounts = Accounts::keeper_registration_accounts(&keeper.pubkey())?;

        let tx = self.program
            .request()
            .accounts(crate::accounts::RegisterKeeper {
                registry_state: accounts.registry_state,
                keeper: accounts.keeper,
                keeper_account: accounts.keeper_account,
                system_program: accounts.system_program,
            })
            .args(crate::instruction::RegisterKeeper { stake_amount })
            .signer(keeper)
            .send()
            .await
            .map_err(|e| SolCronError::TransactionExecutionError {
                reason: format!("Failed to register keeper: {}", e),
            })?;

        self.confirm_transaction(tx).await
    }

    /// Get keeper information
    /// 
    /// # Arguments
    /// * `keeper_address` - Keeper's public key
    pub async fn get_keeper(&self, keeper_address: &Pubkey) -> SolCronResult<Keeper> {
        let (keeper_account, _) = Accounts::keeper(keeper_address)?;
        
        self.program
            .account::<Keeper>(keeper_account)
            .await
            .map_err(|e| SolCronError::KeeperNotFound {
                keeper: keeper_address.to_string(),
            })
    }

    /// Claim keeper rewards
    /// 
    /// # Arguments
    /// * `keeper` - Keeper keypair
    pub async fn claim_rewards(&self, keeper: &Keypair) -> SolCronResult<Signature> {
        let reward_accounts = RewardClaimAccounts::new(&keeper.pubkey())?;

        let tx = self.program
            .request()
            .accounts(crate::accounts::ClaimRewards {
                keeper: reward_accounts.keeper,
                keeper_account: reward_accounts.keeper_account,
                system_program: reward_accounts.system_program,
            })
            .args(crate::instruction::ClaimRewards)
            .signer(keeper)
            .send()
            .await
            .map_err(|e| SolCronError::TransactionExecutionError {
                reason: format!("Failed to claim rewards: {}", e),
            })?;

        self.confirm_transaction(tx).await
    }

    /// Execute a job (called by keepers)
    /// 
    /// # Arguments
    /// * `job_id` - Job identifier
    /// * `keeper` - Keeper keypair
    pub async fn execute_job(&self, job_id: u64, keeper: &Keypair) -> SolCronResult<ExecutionResult> {
        // Get job info to determine execution count and target program
        let job = self.get_job(job_id).await?;
        let execution_count = job.execution_count;

        let accounts = Accounts::job_execution_accounts(
            job_id,
            &keeper.pubkey(),
            &job.target_program,
            execution_count,
        )?;

        let tx = self.program
            .request()
            .accounts(crate::accounts::ExecuteJob {
                registry_state: accounts.registry_state,
                automation_job: accounts.automation_job,
                keeper: accounts.keeper,
                execution_record: accounts.execution_record,
                keeper_account: accounts.keeper_account,
                target_program: accounts.target_program,
                system_program: accounts.system_program,
            })
            .args(crate::instruction::ExecuteJob { job_id })
            .signer(keeper)
            .send()
            .await
            .map_err(|e| SolCronError::ExecutionFailed {
                job_id,
                error: e.to_string(),
            })?;

        let signature = self.confirm_transaction(tx).await?;

        // Return execution result
        // Note: In a real implementation, you might parse transaction logs
        // to get the actual execution details
        Ok(ExecutionResult {
            job_id,
            success: true,
            gas_used: 0, // Would be extracted from logs
            fee_charged: 0, // Would be extracted from logs
            execution_time: chrono::Utc::now().timestamp() as u64,
            error: None,
        })
    }

    /// Get job statistics
    pub async fn get_job_stats(&self, job_id: u64) -> SolCronResult<JobStats> {
        let job = self.get_job(job_id).await?;
        
        Ok(JobStats {
            job_id: job.job_id,
            owner: job.owner,
            total_executions: job.execution_count,
            total_fees_paid: 0, // Would need to calculate from execution records
            last_execution: job.last_execution,
            is_active: job.is_active,
            current_balance: job.balance,
            success_rate: 1.0, // Would need to calculate from execution records
        })
    }

    /// Get keeper statistics
    pub async fn get_keeper_stats(&self, keeper_address: &Pubkey) -> SolCronResult<KeeperStats> {
        let keeper = self.get_keeper(keeper_address).await?;
        
        Ok(KeeperStats {
            keeper: keeper.address,
            total_executions: keeper.successful_executions + keeper.failed_executions,
            successful_executions: keeper.successful_executions,
            failed_executions: keeper.failed_executions,
            success_rate: keeper.success_rate() / 100.0,
            total_earnings: keeper.total_earnings,
            reputation_score: keeper.reputation_score,
            stake_amount: keeper.stake_amount,
            is_active: keeper.is_active,
        })
    }

    /// Get overall registry statistics
    pub async fn get_registry_stats(&self) -> SolCronResult<RegistryStats> {
        let registry = self.get_registry_state().await?;
        
        Ok(RegistryStats {
            total_jobs: registry.total_jobs,
            active_jobs: registry.active_jobs,
            total_keepers: registry.total_keepers,
            active_keepers: registry.active_keepers,
            total_executions: registry.total_executions,
            total_fees_collected: registry.total_fees_collected,
            average_success_rate: 0.95, // Would need to calculate from all keepers
        })
    }

    /// List all jobs for an owner
    pub async fn list_jobs(&self, owner: &Pubkey) -> SolCronResult<Vec<AutomationJob>> {
        // Note: This would need to scan accounts or maintain an index
        // For now, return empty vec - in practice you'd implement account scanning
        // or use a program-derived account list
        Ok(vec![])
    }

    /// Monitor jobs for execution opportunities (used by keepers)
    pub async fn monitor_jobs(&self) -> SolCronResult<Vec<(u64, TriggerEvaluation)>> {
        // This would implement the core keeper logic:
        // 1. Scan all active jobs
        // 2. Evaluate trigger conditions
        // 3. Return jobs ready for execution
        
        // For now, return empty vec - full implementation would be in keeper node
        Ok(vec![])
    }

    /// Wait for transaction confirmation with retry logic
    async fn confirm_transaction(&self, signature: Signature) -> SolCronResult<Signature> {
        const MAX_RETRIES: usize = 30;
        const RETRY_DELAY: Duration = Duration::from_secs(2);

        for attempt in 0..MAX_RETRIES {
            match self.rpc_client.confirm_transaction(&signature) {
                Ok(_) => return Ok(signature),
                Err(e) if attempt < MAX_RETRIES - 1 => {
                    log::debug!(
                        "Transaction confirmation attempt {} failed: {}, retrying...", 
                        attempt + 1, 
                        e
                    );
                    sleep(RETRY_DELAY).await;
                }
                Err(e) => {
                    return Err(SolCronError::TransactionExecutionError {
                        reason: format!("Transaction confirmation failed after {} attempts: {}", MAX_RETRIES, e),
                    });
                }
            }
        }

        unreachable!()
    }

    /// Get the underlying RPC client for advanced operations
    pub fn rpc_client(&self) -> &RpcClient {
        &self.rpc_client
    }

    /// Get the program interface for advanced operations
    pub fn program(&self) -> &Program<Arc<Keypair>> {
        &self.program
    }
}

// For non-client builds, provide stubs
#[cfg(not(feature = "client"))]
pub struct SolCronClient;

#[cfg(not(feature = "client"))]
impl SolCronClient {
    pub async fn new(_cluster_url: &str) -> SolCronResult<Self> {
        Err(SolCronError::NotImplemented {
            feature: "Client functionality requires 'client' feature".to_string(),
        })
    }
}