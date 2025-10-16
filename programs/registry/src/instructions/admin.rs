use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;

// Slash Keeper
#[derive(Accounts)]
#[instruction(keeper_pubkey: Pubkey)]
pub struct SlashKeeper<'info> {
    #[account(
        seeds = [b"registry"],
        bump = registry_state.bump,
        constraint = registry_state.admin == admin.key() @ SolCronError::Unauthorized
    )]
    pub registry_state: Account<'info, RegistryState>,
    
    #[account(
        mut,
        seeds = [b"keeper", keeper_pubkey.as_ref()],
        bump = keeper.bump,
        constraint = keeper.is_active @ SolCronError::InvalidKeeper
    )]
    pub keeper: Account<'info, Keeper>,
    
    #[account(mut)]
    pub admin: Signer<'info>,
    
    /// CHECK: Treasury account to receive slashed funds
    #[account(
        mut,
        constraint = treasury.key() == registry_state.treasury @ SolCronError::InvalidParameters
    )]
    pub treasury: AccountInfo<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn slash_keeper(
    ctx: Context<SlashKeeper>,
    _keeper_pubkey: Pubkey,
    slash_amount: u64,
    reason: String,
) -> Result<()> {
    require!(slash_amount > 0, SolCronError::InvalidParameters);
    require!(!reason.is_empty(), SolCronError::InvalidParameters);

    let keeper = &mut ctx.accounts.keeper;
    
    // Ensure we don't slash more than available stake + rewards
    let available_balance = keeper.stake_amount + keeper.pending_rewards;
    let actual_slash_amount = std::cmp::min(slash_amount, available_balance);
    
    require!(actual_slash_amount > 0, SolCronError::SlashingFailed);

    // Transfer slashed funds to treasury
    if actual_slash_amount > 0 {
        **keeper.to_account_info().try_borrow_mut_lamports()? -= actual_slash_amount;
        **ctx.accounts.treasury.try_borrow_mut_lamports()? += actual_slash_amount;
    }

    // Reduce keeper's stake and rewards
    if keeper.stake_amount >= actual_slash_amount {
        keeper.stake_amount -= actual_slash_amount;
    } else {
        let remaining_slash = actual_slash_amount - keeper.stake_amount;
        keeper.stake_amount = 0;
        keeper.pending_rewards = keeper.pending_rewards.saturating_sub(remaining_slash);
    }

    // Severely impact reputation
    keeper.reputation_score = keeper.reputation_score.saturating_sub(2000); // -20%
    
    // If stake is below minimum, deactivate keeper
    if keeper.stake_amount < ctx.accounts.registry_state.min_stake {
        keeper.is_active = false;
    }

    emit!(KeeperSlashed {
        keeper: keeper.address,
        slash_amount: actual_slash_amount,
        reason: reason.clone(),
        new_stake: keeper.stake_amount,
        new_reputation: keeper.reputation_score,
    });

    msg!("Keeper {} slashed {} lamports for: {}", 
         keeper.address, actual_slash_amount, reason);

    Ok(())
}

// Update Registry Parameters
#[derive(Accounts)]
pub struct UpdateRegistryParams<'info> {
    #[account(
        mut,
        seeds = [b"registry"],
        bump = registry_state.bump,
        constraint = registry_state.admin == admin.key() @ SolCronError::Unauthorized
    )]
    pub registry_state: Account<'info, RegistryState>,
    
    pub admin: Signer<'info>,
}

pub fn update_registry_params(
    ctx: Context<UpdateRegistryParams>,
    base_fee: Option<u64>,
    min_stake: Option<u64>,
    protocol_fee_bps: Option<u16>,
) -> Result<()> {
    let registry_state = &mut ctx.accounts.registry_state;

    if let Some(base_fee) = base_fee {
        require!(base_fee > 0, SolCronError::InvalidParameters);
        registry_state.base_fee = base_fee;
    }

    if let Some(min_stake) = min_stake {
        require!(min_stake > 0, SolCronError::InvalidParameters);
        registry_state.min_stake = min_stake;
    }

    if let Some(protocol_fee_bps) = protocol_fee_bps {
        require!(protocol_fee_bps <= 1000, SolCronError::InvalidParameters); // Max 10%
        registry_state.protocol_fee_bps = protocol_fee_bps;
    }

    emit!(RegistryParamsUpdated {
        base_fee: registry_state.base_fee,
        min_stake: registry_state.min_stake,
        protocol_fee_bps: registry_state.protocol_fee_bps,
    });

    msg!("Registry parameters updated by admin: {}", registry_state.admin);

    Ok(())
}

// Transfer Admin
#[derive(Accounts)]
pub struct TransferAdmin<'info> {
    #[account(
        mut,
        seeds = [b"registry"],
        bump = registry_state.bump,
        constraint = registry_state.admin == current_admin.key() @ SolCronError::Unauthorized
    )]
    pub registry_state: Account<'info, RegistryState>,
    
    pub current_admin: Signer<'info>,
    
    /// CHECK: New admin account
    pub new_admin: AccountInfo<'info>,
}

pub fn transfer_admin(ctx: Context<TransferAdmin>) -> Result<()> {
    let registry_state = &mut ctx.accounts.registry_state;
    let old_admin = registry_state.admin;
    
    registry_state.admin = ctx.accounts.new_admin.key();

    emit!(AdminTransferred {
        old_admin,
        new_admin: registry_state.admin,
    });

    msg!("Admin transferred from {} to {}", old_admin, registry_state.admin);

    Ok(())
}

// Events
#[event]
pub struct KeeperSlashed {
    pub keeper: Pubkey,
    pub slash_amount: u64,
    pub reason: String,
    pub new_stake: u64,
    pub new_reputation: u64,
}

#[event]
pub struct RegistryParamsUpdated {
    pub base_fee: u64,
    pub min_stake: u64,
    pub protocol_fee_bps: u16,
}

#[event]
pub struct AdminTransferred {
    pub old_admin: Pubkey,
    pub new_admin: Pubkey,
}