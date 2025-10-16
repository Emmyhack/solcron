use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;

// Register Keeper
#[derive(Accounts)]
pub struct RegisterKeeper<'info> {
    #[account(
        mut,
        seeds = [b"registry"],
        bump = registry_state.bump
    )]
    pub registry_state: Account<'info, RegistryState>,
    
    #[account(
        init,
        payer = keeper_account,
        space = Keeper::MAX_SIZE,
        seeds = [b"keeper", keeper_account.key().as_ref()],
        bump
    )]
    pub keeper: Account<'info, Keeper>,
    
    #[account(mut)]
    pub keeper_account: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn register_keeper(ctx: Context<RegisterKeeper>, stake_amount: u64) -> Result<()> {
    let registry_state = &mut ctx.accounts.registry_state;
    require!(stake_amount >= registry_state.min_stake, SolCronError::InsufficientStake);

    let keeper = &mut ctx.accounts.keeper;
    let clock = Clock::get()?;

    // Transfer stake to keeper account
    let transfer_instruction = anchor_lang::solana_program::system_instruction::transfer(
        &ctx.accounts.keeper_account.key(),
        &keeper.key(),
        stake_amount,
    );
    
    anchor_lang::solana_program::program::invoke(
        &transfer_instruction,
        &[
            ctx.accounts.keeper_account.to_account_info(),
            keeper.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    // Initialize keeper
    keeper.address = ctx.accounts.keeper_account.key();
    keeper.stake_amount = stake_amount;
    keeper.reputation_score = 5000; // Starting reputation (50%)
    keeper.is_active = true;
    keeper.total_executions = 0;
    keeper.successful_executions = 0;
    keeper.total_earnings = 0;
    keeper.pending_rewards = 0;
    keeper.last_execution_time = 0;
    keeper.registered_at = clock.unix_timestamp;
    keeper.bump = ctx.bumps.keeper;

    // Update registry stats
    registry_state.total_keepers += 1;
    registry_state.active_keepers += 1;

    emit!(KeeperRegistered {
        address: keeper.address,
        stake_amount,
    });

    msg!("Keeper {} registered with stake: {} lamports", keeper.address, stake_amount);

    Ok(())
}

// Unregister Keeper
#[derive(Accounts)]
pub struct UnregisterKeeper<'info> {
    #[account(
        mut,
        seeds = [b"registry"],
        bump = registry_state.bump
    )]
    pub registry_state: Account<'info, RegistryState>,
    
    #[account(
        mut,
        seeds = [b"keeper", keeper_account.key().as_ref()],
        bump = keeper.bump,
        constraint = keeper.is_active @ SolCronError::InvalidKeeper,
        constraint = keeper.address == keeper_account.key() @ SolCronError::Unauthorized
    )]
    pub keeper: Account<'info, Keeper>,
    
    #[account(mut)]
    pub keeper_account: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn unregister_keeper(ctx: Context<UnregisterKeeper>) -> Result<()> {
    let keeper = &mut ctx.accounts.keeper;
    let registry_state = &mut ctx.accounts.registry_state;
    let clock = Clock::get()?;

    // Check cooldown period (24 hours)
    require!(
        clock.unix_timestamp - keeper.last_execution_time >= 86400,
        SolCronError::CooldownPeriod
    );

    // Return stake + pending rewards to keeper
    let total_refund = keeper.stake_amount + keeper.pending_rewards;
    
    if total_refund > 0 {
        **keeper.to_account_info().try_borrow_mut_lamports()? -= total_refund;
        **ctx.accounts.keeper_account.to_account_info().try_borrow_mut_lamports()? += total_refund;
    }

    // Mark keeper as inactive
    keeper.is_active = false;
    keeper.stake_amount = 0;
    keeper.pending_rewards = 0;

    // Update registry stats
    registry_state.active_keepers -= 1;

    emit!(KeeperUnregistered {
        address: keeper.address,
        refunded_amount: total_refund,
    });

    msg!("Keeper {} unregistered, {} lamports refunded", keeper.address, total_refund);

    Ok(())
}

// Claim Rewards
#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(
        mut,
        seeds = [b"keeper", keeper_account.key().as_ref()],
        bump = keeper.bump,
        constraint = keeper.is_active @ SolCronError::InvalidKeeper,
        constraint = keeper.address == keeper_account.key() @ SolCronError::Unauthorized
    )]
    pub keeper: Account<'info, Keeper>,
    
    #[account(mut)]
    pub keeper_account: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
    let keeper = &mut ctx.accounts.keeper;
    
    require!(keeper.pending_rewards > 0, SolCronError::NoRewardsToClaim);

    let rewards_amount = keeper.pending_rewards;

    // Transfer rewards to keeper's wallet
    **keeper.to_account_info().try_borrow_mut_lamports()? -= rewards_amount;
    **ctx.accounts.keeper_account.to_account_info().try_borrow_mut_lamports()? += rewards_amount;

    keeper.pending_rewards = 0;

    emit!(RewardsClaimed {
        keeper: keeper.address,
        amount: rewards_amount,
    });

    msg!("Keeper {} claimed {} lamports in rewards", keeper.address, rewards_amount);

    Ok(())
}

// Events
#[event]
pub struct KeeperRegistered {
    pub address: Pubkey,
    pub stake_amount: u64,
}

#[event]
pub struct KeeperUnregistered {
    pub address: Pubkey,
    pub refunded_amount: u64,
}

#[event]
pub struct RewardsClaimed {
    pub keeper: Pubkey,
    pub amount: u64,
}