use anchor_lang::prelude::*;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};
use crate::{
    types::*,
    accounts::*,
    error::{SolCronError, SolCronResult},
    REGISTRY_PROGRAM_ID,
};

/// Instruction builders for SolCron program interactions
pub struct Instructions;

impl Instructions {
    /// Create an instruction to initialize the registry
    /// 
    /// # Arguments
    /// * `admin` - The registry administrator
    /// * `base_fee` - Base fee for job execution (lamports)
    /// * `min_stake` - Minimum stake required for keepers (lamports)
    /// * `protocol_fee_bps` - Protocol fee in basis points (0-10000)
    /// * `treasury` - Treasury account for protocol fees
    /// * `payer` - Account that pays for initialization
    pub fn initialize_registry(
        admin: Pubkey,
        base_fee: u64,
        min_stake: u64,
        protocol_fee_bps: u16,
        treasury: Pubkey,
        payer: Pubkey,
    ) -> SolCronResult<Instruction> {
        let (registry_state, _) = Accounts::registry_state()?;

        let accounts = vec![
            AccountMeta::new(registry_state, false),
            AccountMeta::new(payer, true),
            AccountMeta::new_readonly(system_program::ID, false),
        ];

        let data = InitializeRegistryData {
            admin,
            base_fee,
            min_stake,
            protocol_fee_bps,
            treasury,
        };

        Ok(Instruction {
            program_id: REGISTRY_PROGRAM_ID,
            accounts,
            data: InstructionData::InitializeRegistry(data).try_to_vec()?,
        })
    }

    /// Create an instruction to register a new automation job
    /// 
    /// # Arguments
    /// * `job_params` - Job configuration parameters
    /// * `initial_funding` - Initial funding amount (lamports)
    /// * `owner` - Job owner
    /// * `job_id` - Job identifier (from registry state)
    pub fn register_job(
        job_params: JobParams,
        initial_funding: u64,
        owner: Pubkey,
        job_id: u64,
    ) -> SolCronResult<Instruction> {
        let accounts_info = Accounts::job_registration_accounts(&owner, job_id)?;

        let accounts = vec![
            AccountMeta::new(accounts_info.registry_state, false),
            AccountMeta::new(accounts_info.automation_job, false),
            AccountMeta::new(accounts_info.owner, true),
            AccountMeta::new_readonly(accounts_info.system_program, false),
        ];

        let data = RegisterJobData {
            target_program: job_params.target_program,
            target_instruction: job_params.target_instruction,
            trigger_type: job_params.trigger_type,
            trigger_params: job_params.trigger_params,
            gas_limit: job_params.gas_limit,
            min_balance: job_params.min_balance,
            initial_funding,
        };

        Ok(Instruction {
            program_id: REGISTRY_PROGRAM_ID,
            accounts,
            data: InstructionData::RegisterJob(data).try_to_vec()?,
        })
    }

    /// Create an instruction to fund an existing job
    /// 
    /// # Arguments
    /// * `job_id` - Job identifier
    /// * `amount` - Funding amount (lamports)
    /// * `funder` - Account funding the job
    pub fn fund_job(
        job_id: u64,
        amount: u64,
        funder: Pubkey,
    ) -> SolCronResult<Instruction> {
        let (automation_job, _) = Accounts::automation_job(job_id)?;

        let accounts = vec![
            AccountMeta::new(automation_job, false),
            AccountMeta::new(funder, true),
            AccountMeta::new_readonly(system_program::ID, false),
        ];

        let data = FundJobData { amount };

        Ok(Instruction {
            program_id: REGISTRY_PROGRAM_ID,
            accounts,
            data: InstructionData::FundJob(data).try_to_vec()?,
        })
    }

    /// Create an instruction to update job parameters
    /// 
    /// # Arguments
    /// * `job_id` - Job identifier
    /// * `gas_limit` - New gas limit (optional)
    /// * `min_balance` - New minimum balance (optional)
    /// * `trigger_params` - New trigger parameters (optional)
    /// * `owner` - Job owner
    pub fn update_job(
        job_id: u64,
        gas_limit: Option<u64>,
        min_balance: Option<u64>,
        trigger_params: Option<Vec<u8>>,
        owner: Pubkey,
    ) -> SolCronResult<Instruction> {
        let (automation_job, _) = Accounts::automation_job(job_id)?;

        let accounts = vec![
            AccountMeta::new(automation_job, false),
            AccountMeta::new_readonly(owner, true),
        ];

        let data = UpdateJobData {
            gas_limit,
            min_balance,
            trigger_params,
        };

        Ok(Instruction {
            program_id: REGISTRY_PROGRAM_ID,
            accounts,
            data: InstructionData::UpdateJob(data).try_to_vec()?,
        })
    }

    /// Create an instruction to cancel a job
    /// 
    /// # Arguments
    /// * `job_id` - Job identifier
    /// * `owner` - Job owner
    pub fn cancel_job(job_id: u64, owner: Pubkey) -> SolCronResult<Instruction> {
        let (registry_state, _) = Accounts::registry_state()?;
        let (automation_job, _) = Accounts::automation_job(job_id)?;

        let accounts = vec![
            AccountMeta::new(registry_state, false),
            AccountMeta::new(automation_job, false),
            AccountMeta::new(owner, true),
            AccountMeta::new_readonly(system_program::ID, false),
        ];

        Ok(Instruction {
            program_id: REGISTRY_PROGRAM_ID,
            accounts,
            data: InstructionData::CancelJob.try_to_vec()?,
        })
    }

    /// Create an instruction to register a keeper
    /// 
    /// # Arguments
    /// * `stake_amount` - Amount to stake (lamports)
    /// * `keeper_address` - Keeper's public key
    pub fn register_keeper(
        stake_amount: u64,
        keeper_address: Pubkey,
    ) -> SolCronResult<Instruction> {
        let accounts_info = Accounts::keeper_registration_accounts(&keeper_address)?;

        let accounts = vec![
            AccountMeta::new(accounts_info.registry_state, false),
            AccountMeta::new(accounts_info.keeper, false),
            AccountMeta::new(accounts_info.keeper_account, true),
            AccountMeta::new_readonly(accounts_info.system_program, false),
        ];

        let data = RegisterKeeperData { stake_amount };

        Ok(Instruction {
            program_id: REGISTRY_PROGRAM_ID,
            accounts,
            data: InstructionData::RegisterKeeper(data).try_to_vec()?,
        })
    }

    /// Create an instruction to unregister a keeper
    /// 
    /// # Arguments
    /// * `keeper_address` - Keeper's public key
    pub fn unregister_keeper(keeper_address: Pubkey) -> SolCronResult<Instruction> {
        let (registry_state, _) = Accounts::registry_state()?;
        let (keeper, _) = Accounts::keeper(&keeper_address)?;

        let accounts = vec![
            AccountMeta::new(registry_state, false),
            AccountMeta::new(keeper, false),
            AccountMeta::new(keeper_address, true),
            AccountMeta::new_readonly(system_program::ID, false),
        ];

        Ok(Instruction {
            program_id: REGISTRY_PROGRAM_ID,
            accounts,
            data: InstructionData::UnregisterKeeper.try_to_vec()?,
        })
    }

    /// Create an instruction to execute a job
    /// 
    /// # Arguments
    /// * `job_id` - Job identifier
    /// * `keeper_address` - Executing keeper
    /// * `target_program` - Target program to execute
    /// * `execution_count` - Current execution count
    pub fn execute_job(
        job_id: u64,
        keeper_address: Pubkey,
        target_program: Pubkey,
        execution_count: u64,
    ) -> SolCronResult<Instruction> {
        let accounts_info = Accounts::job_execution_accounts(
            job_id,
            &keeper_address,
            &target_program,
            execution_count,
        )?;

        let accounts = vec![
            AccountMeta::new(accounts_info.registry_state, false),
            AccountMeta::new(accounts_info.automation_job, false),
            AccountMeta::new(accounts_info.keeper, false),
            AccountMeta::new(accounts_info.execution_record, false),
            AccountMeta::new_readonly(accounts_info.keeper_account, true),
            AccountMeta::new_readonly(accounts_info.target_program, false),
            AccountMeta::new_readonly(accounts_info.system_program, false),
        ];

        let data = ExecuteJobData { job_id };

        Ok(Instruction {
            program_id: REGISTRY_PROGRAM_ID,
            accounts,
            data: InstructionData::ExecuteJob(data).try_to_vec()?,
        })
    }

    /// Create an instruction to claim keeper rewards
    /// 
    /// # Arguments
    /// * `keeper_address` - Keeper's public key
    pub fn claim_rewards(keeper_address: Pubkey) -> SolCronResult<Instruction> {
        let reward_accounts = RewardClaimAccounts::new(&keeper_address)?;

        let accounts = vec![
            AccountMeta::new(reward_accounts.keeper, false),
            AccountMeta::new(reward_accounts.keeper_account, true),
            AccountMeta::new_readonly(reward_accounts.system_program, false),
        ];

        Ok(Instruction {
            program_id: REGISTRY_PROGRAM_ID,
            accounts,
            data: InstructionData::ClaimRewards.try_to_vec()?,
        })
    }

    /// Create an instruction to update registry parameters (admin only)
    /// 
    /// # Arguments
    /// * `base_fee` - New base fee (optional)
    /// * `min_stake` - New minimum stake (optional)
    /// * `protocol_fee_bps` - New protocol fee rate (optional)
    /// * `admin` - Registry admin
    pub fn update_registry_params(
        base_fee: Option<u64>,
        min_stake: Option<u64>,
        protocol_fee_bps: Option<u16>,
        admin: Pubkey,
    ) -> SolCronResult<Instruction> {
        let admin_accounts = AdminAccounts::new(&admin)?;

        let accounts = vec![
            AccountMeta::new(admin_accounts.registry_state, false),
            AccountMeta::new_readonly(admin_accounts.admin, true),
        ];

        let data = UpdateRegistryParamsData {
            base_fee,
            min_stake,
            protocol_fee_bps,
        };

        Ok(Instruction {
            program_id: REGISTRY_PROGRAM_ID,
            accounts,
            data: InstructionData::UpdateRegistryParams(data).try_to_vec()?,
        })
    }

    /// Create an instruction to slash a keeper (admin only)
    /// 
    /// # Arguments
    /// * `keeper_address` - Keeper to slash
    /// * `slash_amount` - Amount to slash (lamports)
    /// * `reason` - Reason for slashing
    /// * `admin` - Registry admin
    /// * `treasury` - Treasury to receive slashed funds
    pub fn slash_keeper(
        keeper_address: Pubkey,
        slash_amount: u64,
        reason: String,
        admin: Pubkey,
        treasury: Pubkey,
    ) -> SolCronResult<Instruction> {
        let (registry_state, _) = Accounts::registry_state()?;
        let (keeper, _) = Accounts::keeper(&keeper_address)?;

        let accounts = vec![
            AccountMeta::new(registry_state, false),
            AccountMeta::new(keeper, false),
            AccountMeta::new_readonly(admin, true),
            AccountMeta::new(treasury, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ];

        let data = SlashKeeperData {
            keeper_address,
            slash_amount,
            reason,
        };

        Ok(Instruction {
            program_id: REGISTRY_PROGRAM_ID,
            accounts,
            data: InstructionData::SlashKeeper(data).try_to_vec()?,
        })
    }
}

/// Instruction data structures
#[derive(AnchorSerialize, AnchorDeserialize)]
pub enum InstructionData {
    InitializeRegistry(InitializeRegistryData),
    RegisterJob(RegisterJobData),
    FundJob(FundJobData),
    UpdateJob(UpdateJobData),
    CancelJob,
    RegisterKeeper(RegisterKeeperData),
    UnregisterKeeper,
    ExecuteJob(ExecuteJobData),
    ClaimRewards,
    UpdateRegistryParams(UpdateRegistryParamsData),
    SlashKeeper(SlashKeeperData),
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeRegistryData {
    pub admin: Pubkey,
    pub base_fee: u64,
    pub min_stake: u64,
    pub protocol_fee_bps: u16,
    pub treasury: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct RegisterJobData {
    pub target_program: Pubkey,
    pub target_instruction: String,
    pub trigger_type: TriggerType,
    pub trigger_params: Vec<u8>,
    pub gas_limit: u64,
    pub min_balance: u64,
    pub initial_funding: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct FundJobData {
    pub amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct UpdateJobData {
    pub gas_limit: Option<u64>,
    pub min_balance: Option<u64>,
    pub trigger_params: Option<Vec<u8>>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct RegisterKeeperData {
    pub stake_amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ExecuteJobData {
    pub job_id: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct UpdateRegistryParamsData {
    pub base_fee: Option<u64>,
    pub min_stake: Option<u64>,
    pub protocol_fee_bps: Option<u16>,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct SlashKeeperData {
    pub keeper_address: Pubkey,
    pub slash_amount: u64,
    pub reason: String,
}

impl InstructionData {
    /// Serialize instruction data
    pub fn try_to_vec(&self) -> SolCronResult<Vec<u8>> {
        self.try_to_vec().map_err(|e| SolCronError::SerializationError {
            reason: format!("Failed to serialize instruction data: {}", e),
        })
    }
}