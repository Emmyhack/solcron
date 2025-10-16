//! Cross-Program Invocation (CPI) utilities for integrating with SolCron from other programs
//!
//! This module provides helper functions for Solana programs to interact with SolCron
//! via Cross-Program Invocation. This enables other programs to register automation jobs
//! and manage their SolCron integration programmatically.

use anchor_lang::prelude::*;
use solana_program::{
    account_info::AccountInfo,
    program::invoke_signed,
    program::invoke,
    instruction::Instruction,
};
use crate::{
    types::*,
    instructions::*,
    accounts::*,
    error::{SolCronError, SolCronResult},
    REGISTRY_PROGRAM_ID,
};

/// CPI helper functions for SolCron integration
pub struct CPI;

impl CPI {
    /// Register an automation job via CPI
    /// 
    /// # Arguments
    /// * `program_info` - SolCron registry program account
    /// * `registry_state_info` - Registry state account
    /// * `job_info` - Job account to be created
    /// * `owner_info` - Job owner account
    /// * `system_program_info` - System program account
    /// * `job_params` - Job configuration
    /// * `initial_funding` - Initial funding amount
    /// * `signer_seeds` - Optional seeds for PDA signing
    /// 
    /// # Example
    /// ```rust,ignore
    /// use solcron_sdk::cpi::CPI;
    /// use solcron_sdk::types::{JobParams, TriggerType};
    /// 
    /// // In your program instruction handler
    /// pub fn register_automation_job(ctx: Context<RegisterAutomation>) -> Result<()> {
    ///     let job_params = JobParams {
    ///         target_program: crate::ID,
    ///         target_instruction: "harvest_rewards".to_string(),
    ///         trigger_type: TriggerType::TimeBased { interval: 3600 },
    ///         trigger_params: vec![],
    ///         gas_limit: 200_000,
    ///         min_balance: 1_000_000,
    ///     };
    ///     
    ///     CPI::register_job(
    ///         &ctx.accounts.solcron_program,
    ///         &ctx.accounts.registry_state,
    ///         &ctx.accounts.automation_job,
    ///         &ctx.accounts.owner,
    ///         &ctx.accounts.system_program,
    ///         job_params,
    ///         50_000_000, // 0.05 SOL initial funding
    ///         None,
    ///     )?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn register_job<'info>(
        program_info: &AccountInfo<'info>,
        registry_state_info: &AccountInfo<'info>,
        job_info: &AccountInfo<'info>,
        owner_info: &AccountInfo<'info>,
        system_program_info: &AccountInfo<'info>,
        job_params: JobParams,
        initial_funding: u64,
        signer_seeds: Option<&[&[&[u8]]]>,
    ) -> SolCronResult<()> {
        // Validate program ID
        if *program_info.key != REGISTRY_PROGRAM_ID {
            return Err(SolCronError::InvalidProgramId {
                expected: REGISTRY_PROGRAM_ID.to_string(),
                actual: program_info.key.to_string(),
            });
        }

        // Build instruction data
        let data = RegisterJobData {
            target_program: job_params.target_program,
            target_instruction: job_params.target_instruction,
            trigger_type: job_params.trigger_type,
            trigger_params: job_params.trigger_params,
            gas_limit: job_params.gas_limit,
            min_balance: job_params.min_balance,
            initial_funding,
        };

        let instruction_data = InstructionData::RegisterJob(data).try_to_vec()?;

        let instruction = Instruction {
            program_id: REGISTRY_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(*registry_state_info.key, false),
                AccountMeta::new(*job_info.key, false),
                AccountMeta::new(*owner_info.key, true),
                AccountMeta::new_readonly(*system_program_info.key, false),
            ],
            data: instruction_data,
        };

        let account_infos = &[
            program_info.clone(),
            registry_state_info.clone(),
            job_info.clone(),
            owner_info.clone(),
            system_program_info.clone(),
        ];

        if let Some(seeds) = signer_seeds {
            invoke_signed(&instruction, account_infos, seeds)?;
        } else {
            invoke(&instruction, account_infos)?;
        }

        Ok(())
    }

    /// Fund an existing job via CPI
    /// 
    /// # Arguments
    /// * `program_info` - SolCron registry program account
    /// * `job_info` - Job account to fund
    /// * `funder_info` - Account providing the funding
    /// * `system_program_info` - System program account
    /// * `amount` - Amount to fund (lamports)
    /// * `signer_seeds` - Optional seeds for PDA signing
    pub fn fund_job<'info>(
        program_info: &AccountInfo<'info>,
        job_info: &AccountInfo<'info>,
        funder_info: &AccountInfo<'info>,
        system_program_info: &AccountInfo<'info>,
        amount: u64,
        signer_seeds: Option<&[&[&[u8]]]>,
    ) -> SolCronResult<()> {
        if *program_info.key != REGISTRY_PROGRAM_ID {
            return Err(SolCronError::InvalidProgramId {
                expected: REGISTRY_PROGRAM_ID.to_string(),
                actual: program_info.key.to_string(),
            });
        }

        let data = FundJobData { amount };
        let instruction_data = InstructionData::FundJob(data).try_to_vec()?;

        let instruction = Instruction {
            program_id: REGISTRY_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(*job_info.key, false),
                AccountMeta::new(*funder_info.key, true),
                AccountMeta::new_readonly(*system_program_info.key, false),
            ],
            data: instruction_data,
        };

        let account_infos = &[
            program_info.clone(),
            job_info.clone(),
            funder_info.clone(),
            system_program_info.clone(),
        ];

        if let Some(seeds) = signer_seeds {
            invoke_signed(&instruction, account_infos, seeds)?;
        } else {
            invoke(&instruction, account_infos)?;
        }

        Ok(())
    }

    /// Update job parameters via CPI
    /// 
    /// # Arguments
    /// * `program_info` - SolCron registry program account
    /// * `job_info` - Job account to update
    /// * `owner_info` - Job owner account
    /// * `gas_limit` - New gas limit (optional)
    /// * `min_balance` - New minimum balance (optional)
    /// * `trigger_params` - New trigger parameters (optional)
    /// * `signer_seeds` - Optional seeds for PDA signing
    pub fn update_job<'info>(
        program_info: &AccountInfo<'info>,
        job_info: &AccountInfo<'info>,
        owner_info: &AccountInfo<'info>,
        gas_limit: Option<u64>,
        min_balance: Option<u64>,
        trigger_params: Option<Vec<u8>>,
        signer_seeds: Option<&[&[&[u8]]]>,
    ) -> SolCronResult<()> {
        if *program_info.key != REGISTRY_PROGRAM_ID {
            return Err(SolCronError::InvalidProgramId {
                expected: REGISTRY_PROGRAM_ID.to_string(),
                actual: program_info.key.to_string(),
            });
        }

        let data = UpdateJobData {
            gas_limit,
            min_balance,
            trigger_params,
        };
        let instruction_data = InstructionData::UpdateJob(data).try_to_vec()?;

        let instruction = Instruction {
            program_id: REGISTRY_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(*job_info.key, false),
                AccountMeta::new_readonly(*owner_info.key, true),
            ],
            data: instruction_data,
        };

        let account_infos = &[
            program_info.clone(),
            job_info.clone(),
            owner_info.clone(),
        ];

        if let Some(seeds) = signer_seeds {
            invoke_signed(&instruction, account_infos, seeds)?;
        } else {
            invoke(&instruction, account_infos)?;
        }

        Ok(())
    }

    /// Cancel a job via CPI
    /// 
    /// # Arguments
    /// * `program_info` - SolCron registry program account
    /// * `registry_state_info` - Registry state account
    /// * `job_info` - Job account to cancel
    /// * `owner_info` - Job owner account
    /// * `system_program_info` - System program account
    /// * `signer_seeds` - Optional seeds for PDA signing
    pub fn cancel_job<'info>(
        program_info: &AccountInfo<'info>,
        registry_state_info: &AccountInfo<'info>,
        job_info: &AccountInfo<'info>,
        owner_info: &AccountInfo<'info>,
        system_program_info: &AccountInfo<'info>,
        signer_seeds: Option<&[&[&[u8]]]>,
    ) -> SolCronResult<()> {
        if *program_info.key != REGISTRY_PROGRAM_ID {
            return Err(SolCronError::InvalidProgramId {
                expected: REGISTRY_PROGRAM_ID.to_string(),
                actual: program_info.key.to_string(),
            });
        }

        let instruction_data = InstructionData::CancelJob.try_to_vec()?;

        let instruction = Instruction {
            program_id: REGISTRY_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(*registry_state_info.key, false),
                AccountMeta::new(*job_info.key, false),
                AccountMeta::new(*owner_info.key, true),
                AccountMeta::new_readonly(*system_program_info.key, false),
            ],
            data: instruction_data,
        };

        let account_infos = &[
            program_info.clone(),
            registry_state_info.clone(),
            job_info.clone(),
            owner_info.clone(),
            system_program_info.clone(),
        ];

        if let Some(seeds) = signer_seeds {
            invoke_signed(&instruction, account_infos, seeds)?;
        } else {
            invoke(&instruction, account_infos)?;
        }

        Ok(())
    }

    /// Register as a keeper via CPI (typically used by protocol treasuries)
    /// 
    /// # Arguments
    /// * `program_info` - SolCron registry program account
    /// * `registry_state_info` - Registry state account
    /// * `keeper_info` - Keeper account to be created
    /// * `keeper_account_info` - Keeper's wallet account
    /// * `system_program_info` - System program account
    /// * `stake_amount` - Amount to stake (lamports)
    /// * `signer_seeds` - Optional seeds for PDA signing
    pub fn register_keeper<'info>(
        program_info: &AccountInfo<'info>,
        registry_state_info: &AccountInfo<'info>,
        keeper_info: &AccountInfo<'info>,
        keeper_account_info: &AccountInfo<'info>,
        system_program_info: &AccountInfo<'info>,
        stake_amount: u64,
        signer_seeds: Option<&[&[&[u8]]]>,
    ) -> SolCronResult<()> {
        if *program_info.key != REGISTRY_PROGRAM_ID {
            return Err(SolCronError::InvalidProgramId {
                expected: REGISTRY_PROGRAM_ID.to_string(),
                actual: program_info.key.to_string(),
            });
        }

        let data = RegisterKeeperData { stake_amount };
        let instruction_data = InstructionData::RegisterKeeper(data).try_to_vec()?;

        let instruction = Instruction {
            program_id: REGISTRY_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(*registry_state_info.key, false),
                AccountMeta::new(*keeper_info.key, false),
                AccountMeta::new(*keeper_account_info.key, true),
                AccountMeta::new_readonly(*system_program_info.key, false),
            ],
            data: instruction_data,
        };

        let account_infos = &[
            program_info.clone(),
            registry_state_info.clone(),
            keeper_info.clone(),
            keeper_account_info.clone(),
            system_program_info.clone(),
        ];

        if let Some(seeds) = signer_seeds {
            invoke_signed(&instruction, account_infos, seeds)?;
        } else {
            invoke(&instruction, account_infos)?;
        }

        Ok(())
    }
}

/// Account validation helpers for CPI
pub struct CPIValidation;

impl CPIValidation {
    /// Validate that an account is a valid SolCron job account
    pub fn validate_job_account(account_info: &AccountInfo) -> SolCronResult<AutomationJob> {
        // Check owner
        if account_info.owner != &REGISTRY_PROGRAM_ID {
            return Err(SolCronError::InvalidAccountData {
                account_type: "AutomationJob".to_string(),
                reason: "Invalid owner".to_string(),
            });
        }

        // Check data length
        if account_info.data_len() < AutomationJob::ACCOUNT_SIZE {
            return Err(SolCronError::InvalidAccountData {
                account_type: "AutomationJob".to_string(),
                reason: "Insufficient data length".to_string(),
            });
        }

        // Deserialize account data
        let data = account_info.try_borrow_data()?;
        AutomationJob::try_deserialize(&mut &data[..])
            .map_err(|e| SolCronError::DeserializationError {
                reason: format!("Failed to deserialize AutomationJob: {}", e),
            })
    }

    /// Validate that an account is a valid SolCron keeper account
    pub fn validate_keeper_account(account_info: &AccountInfo) -> SolCronResult<Keeper> {
        if account_info.owner != &REGISTRY_PROGRAM_ID {
            return Err(SolCronError::InvalidAccountData {
                account_type: "Keeper".to_string(),
                reason: "Invalid owner".to_string(),
            });
        }

        if account_info.data_len() < Keeper::ACCOUNT_SIZE {
            return Err(SolCronError::InvalidAccountData {
                account_type: "Keeper".to_string(),
                reason: "Insufficient data length".to_string(),
            });
        }

        let data = account_info.try_borrow_data()?;
        Keeper::try_deserialize(&mut &data[..])
            .map_err(|e| SolCronError::DeserializationError {
                reason: format!("Failed to deserialize Keeper: {}", e),
            })
    }

    /// Validate that an account is the valid SolCron registry state
    pub fn validate_registry_state(account_info: &AccountInfo) -> SolCronResult<RegistryState> {
        if account_info.owner != &REGISTRY_PROGRAM_ID {
            return Err(SolCronError::InvalidAccountData {
                account_type: "RegistryState".to_string(),
                reason: "Invalid owner".to_string(),
            });
        }

        // Verify this is the correct registry state PDA
        let (expected_registry_state, _) = Accounts::registry_state()?;
        if *account_info.key != expected_registry_state {
            return Err(SolCronError::InvalidAccountData {
                account_type: "RegistryState".to_string(),
                reason: "Invalid registry state address".to_string(),
            });
        }

        let data = account_info.try_borrow_data()?;
        RegistryState::try_deserialize(&mut &data[..])
            .map_err(|e| SolCronError::DeserializationError {
                reason: format!("Failed to deserialize RegistryState: {}", e),
            })
    }

    /// Check if a job can be executed (trigger conditions met, sufficient balance)
    pub fn can_execute_job(
        job: &AutomationJob, 
        current_time: u64, 
        execution_fee: u64
    ) -> SolCronResult<bool> {
        // Check if job is active
        if !job.is_active {
            return Ok(false);
        }

        // Check balance
        if !job.can_execute(execution_fee) {
            return Ok(false);
        }

        // Check trigger conditions
        match &job.trigger_type {
            TriggerType::TimeBased { interval } => {
                if job.last_execution == 0 {
                    return Ok(true); // First execution
                }
                Ok(current_time >= job.last_execution + interval)
            }
            TriggerType::Conditional { .. } => {
                // For conditional triggers, the caller needs to implement
                // the specific logic based on trigger_params
                Ok(true)
            }
            TriggerType::LogBased { .. } => {
                // For log-based triggers, the caller needs to implement
                // the specific logic based on trigger_params
                Ok(true)
            }
            TriggerType::Hybrid { .. } => {
                // For hybrid triggers, more complex logic is needed
                Ok(true)
            }
        }
    }
}

// Re-export commonly used CPI types
pub use crate::types::{TriggerType, JobParams, TriggerCondition};
pub use crate::accounts::{Accounts, JobRegistrationAccounts, KeeperRegistrationAccounts};
pub use crate::instructions::{RegisterJobData, FundJobData, UpdateJobData, RegisterKeeperData};