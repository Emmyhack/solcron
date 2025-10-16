use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;

#[derive(Accounts)]
pub struct InitializeRegistry<'info> {
    #[account(
        init,
        payer = payer,
        space = RegistryState::MAX_SIZE,
        seeds = [b"registry"],
        bump
    )]
    pub registry_state: Account<'info, RegistryState>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn initialize_registry(
    ctx: Context<InitializeRegistry>,
    admin: Pubkey,
    base_fee: u64,
    min_stake: u64,
    protocol_fee_bps: u16,
    treasury: Pubkey,
) -> Result<()> {
    require!(protocol_fee_bps <= 1000, SolCronError::InvalidParameters); // Max 10%
    require!(base_fee > 0, SolCronError::InvalidParameters);
    require!(min_stake > 0, SolCronError::InvalidParameters);

    let registry_state = &mut ctx.accounts.registry_state;
    
    registry_state.admin = admin;
    registry_state.base_fee = base_fee;
    registry_state.min_stake = min_stake;
    registry_state.protocol_fee_bps = protocol_fee_bps;
    registry_state.treasury = treasury;
    registry_state.total_jobs = 0;
    registry_state.active_jobs = 0;
    registry_state.total_keepers = 0;
    registry_state.active_keepers = 0;
    registry_state.total_executions = 0;
    registry_state.successful_executions = 0;
    registry_state.protocol_revenue = 0;
    registry_state.next_job_id = 1;
    registry_state.is_paused = false;
    registry_state.bump = ctx.bumps.registry_state;

    msg!("SolCron Registry initialized with admin: {}", admin);
    msg!("Base fee: {} lamports, Min stake: {} lamports", base_fee, min_stake);
    msg!("Protocol fee: {} bps, Treasury: {}", protocol_fee_bps, treasury);

    Ok(())
}