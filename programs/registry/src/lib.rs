use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

pub mod instructions;
pub mod state;
pub mod errors;

use instructions::*;
use state::*;

#[program]
pub mod solcron_registry {
    use super::*;

    /// Initialize the registry with admin and global parameters
    pub fn initialize_registry(
        ctx: Context<InitializeRegistry>,
        admin: Pubkey,
        base_fee: u64,
        min_stake: u64,
        protocol_fee_bps: u16,
        treasury: Pubkey,
    ) -> Result<()> {
        instructions::initialize_registry(ctx, admin, base_fee, min_stake, protocol_fee_bps, treasury)
    }

    /// Register a new automation job
    pub fn register_job(
        ctx: Context<RegisterJob>,
        target_program: Pubkey,
        target_instruction: String,
        trigger_type: TriggerType,
        trigger_params: Vec<u8>,
        gas_limit: u64,
        min_balance: u64,
        initial_funding: u64,
    ) -> Result<()> {
        instructions::register_job(
            ctx,
            target_program,
            target_instruction,
            trigger_type,
            trigger_params,
            gas_limit,
            min_balance,
            initial_funding,
        )
    }

    /// Fund an existing job
    pub fn fund_job(ctx: Context<FundJob>, amount: u64) -> Result<()> {
        instructions::fund_job(ctx, amount)
    }

    /// Cancel a job and withdraw remaining funds
    pub fn cancel_job(ctx: Context<CancelJob>) -> Result<()> {
        instructions::cancel_job(ctx)
    }

    /// Update job parameters
    pub fn update_job(
        ctx: Context<UpdateJob>,
        gas_limit: Option<u64>,
        min_balance: Option<u64>,
        trigger_params: Option<Vec<u8>>,
    ) -> Result<()> {
        instructions::update_job(ctx, gas_limit, min_balance, trigger_params)
    }

    /// Register as a keeper
    pub fn register_keeper(ctx: Context<RegisterKeeper>, stake_amount: u64) -> Result<()> {
        instructions::register_keeper(ctx, stake_amount)
    }

    /// Unregister as a keeper
    pub fn unregister_keeper(ctx: Context<UnregisterKeeper>) -> Result<()> {
        instructions::unregister_keeper(ctx)
    }

    /// Execute an automation job
    pub fn execute_job(ctx: Context<ExecuteJob>, job_id: u64) -> Result<()> {
        instructions::execute_job(ctx, job_id)
    }

    /// Claim accumulated keeper rewards
    pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
        instructions::claim_rewards(ctx)
    }

    /// Admin function to slash a malicious keeper
    pub fn slash_keeper(
        ctx: Context<SlashKeeper>,
        keeper: Pubkey,
        slash_amount: u64,
        reason: String,
    ) -> Result<()> {
        instructions::slash_keeper(ctx, keeper, slash_amount, reason)
    }

    /// Admin function to update registry parameters
    pub fn update_registry_params(
        ctx: Context<UpdateRegistryParams>,
        base_fee: Option<u64>,
        min_stake: Option<u64>,
        protocol_fee_bps: Option<u16>,
    ) -> Result<()> {
        instructions::update_registry_params(ctx, base_fee, min_stake, protocol_fee_bps)
    }
}