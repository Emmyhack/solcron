use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;

// Register Job
#[derive(Accounts)]
#[instruction(target_program: Pubkey, target_instruction: String)]
pub struct RegisterJob<'info> {
    #[account(
        mut,
        seeds = [b"registry"],
        bump = registry_state.bump
    )]
    pub registry_state: Account<'info, RegistryState>,
    
    #[account(
        init,
        payer = owner,
        space = AutomationJob::MAX_SIZE,
        seeds = [b"job", registry_state.next_job_id.to_le_bytes().as_ref()],
        bump
    )]
    pub automation_job: Account<'info, AutomationJob>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

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
    require!(target_instruction.len() <= 50, SolCronError::InvalidParameters);
    require!(trigger_params.len() <= 256, SolCronError::InvalidParameters);
    require!(gas_limit > 0 && gas_limit <= 1_400_000, SolCronError::InvalidParameters); // Max compute units
    require!(initial_funding >= min_balance, SolCronError::InsufficientBalance);

    let registry_state = &mut ctx.accounts.registry_state;
    let automation_job = &mut ctx.accounts.automation_job;
    let clock = Clock::get()?;

    // Validate trigger type parameters
    match &trigger_type {
        TriggerType::TimeBased => {
            require!(!trigger_params.is_empty(), SolCronError::InvalidParameters);
            // trigger_params should contain interval in bytes
        },
        TriggerType::Conditional => {
            require!(!trigger_params.is_empty(), SolCronError::InvalidParameters);
            // trigger_params should contain condition logic
        },
        TriggerType::LogTrigger => {
            require!(!trigger_params.is_empty(), SolCronError::InvalidParameters);
            // trigger_params should contain event signature
        },
        TriggerType::Hybrid => {
            require!(!trigger_params.is_empty(), SolCronError::InvalidParameters);
            // trigger_params should contain hybrid configuration
        }
    }

    // Transfer initial funding from owner
    if initial_funding > 0 {
        let transfer_instruction = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.owner.key(),
            &automation_job.key(),
            initial_funding,
        );
        
        anchor_lang::solana_program::program::invoke(
            &transfer_instruction,
            &[
                ctx.accounts.owner.to_account_info(),
                automation_job.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;
    }

    // Initialize job
    automation_job.job_id = registry_state.next_job_id;
    automation_job.owner = ctx.accounts.owner.key();
    automation_job.target_program = target_program;
    automation_job.target_instruction = target_instruction;
    automation_job.trigger_type = trigger_type;
    automation_job.trigger_params = trigger_params;
    automation_job.gas_limit = gas_limit;
    automation_job.balance = initial_funding;
    automation_job.min_balance = min_balance;
    automation_job.is_active = true;
    automation_job.execution_count = 0;
    automation_job.last_execution = 0;
    automation_job.created_at = clock.unix_timestamp;
    automation_job.updated_at = clock.unix_timestamp;
    automation_job.bump = ctx.bumps.automation_job;

    // Update registry state
    registry_state.next_job_id += 1;
    registry_state.total_jobs += 1;
    registry_state.active_jobs += 1;

    emit!(JobRegistered {
        job_id: automation_job.job_id,
        owner: automation_job.owner,
        target_program,
        initial_funding,
    });

    msg!("Job {} registered by {}", automation_job.job_id, automation_job.owner);

    Ok(())
}

// Fund Job
#[derive(Accounts)]
pub struct FundJob<'info> {
    #[account(
        mut,
        seeds = [b"job", automation_job.job_id.to_le_bytes().as_ref()],
        bump = automation_job.bump,
        constraint = automation_job.is_active @ SolCronError::InvalidJob
    )]
    pub automation_job: Account<'info, AutomationJob>,
    
    #[account(
        mut,
        constraint = funder.key() == automation_job.owner @ SolCronError::Unauthorized
    )]
    pub funder: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn fund_job(ctx: Context<FundJob>, amount: u64) -> Result<()> {
    require!(amount > 0, SolCronError::InvalidParameters);

    let automation_job = &mut ctx.accounts.automation_job;

    // Transfer funds to job account
    let transfer_instruction = anchor_lang::solana_program::system_instruction::transfer(
        &ctx.accounts.funder.key(),
        &automation_job.key(),
        amount,
    );
    
    anchor_lang::solana_program::program::invoke(
        &transfer_instruction,
        &[
            ctx.accounts.funder.to_account_info(),
            automation_job.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    automation_job.balance = automation_job.balance
        .checked_add(amount)
        .ok_or(SolCronError::MathOverflow)?;

    emit!(JobFunded {
        job_id: automation_job.job_id,
        amount,
        new_balance: automation_job.balance,
    });

    msg!("Job {} funded with {} lamports", automation_job.job_id, amount);

    Ok(())
}

// Cancel Job
#[derive(Accounts)]
pub struct CancelJob<'info> {
    #[account(
        mut,
        seeds = [b"registry"],
        bump = registry_state.bump
    )]
    pub registry_state: Account<'info, RegistryState>,
    
    #[account(
        mut,
        seeds = [b"job", automation_job.job_id.to_le_bytes().as_ref()],
        bump = automation_job.bump,
        constraint = automation_job.is_active @ SolCronError::InvalidJob,
        constraint = automation_job.owner == owner.key() @ SolCronError::Unauthorized
    )]
    pub automation_job: Account<'info, AutomationJob>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn cancel_job(ctx: Context<CancelJob>) -> Result<()> {
    let automation_job = &mut ctx.accounts.automation_job;
    let registry_state = &mut ctx.accounts.registry_state;

    // Transfer remaining balance back to owner
    if automation_job.balance > 0 {
        **automation_job.to_account_info().try_borrow_mut_lamports()? -= automation_job.balance;
        **ctx.accounts.owner.to_account_info().try_borrow_mut_lamports()? += automation_job.balance;
    }

    let refunded_amount = automation_job.balance;
    
    // Mark job as inactive
    automation_job.is_active = false;
    automation_job.balance = 0;
    
    // Update registry stats
    registry_state.active_jobs -= 1;

    emit!(JobCancelled {
        job_id: automation_job.job_id,
        owner: automation_job.owner,
        refunded_amount,
    });

    msg!("Job {} cancelled, {} lamports refunded", automation_job.job_id, refunded_amount);

    Ok(())
}

// Update Job
#[derive(Accounts)]
pub struct UpdateJob<'info> {
    #[account(
        mut,
        seeds = [b"job", automation_job.job_id.to_le_bytes().as_ref()],
        bump = automation_job.bump,
        constraint = automation_job.is_active @ SolCronError::InvalidJob,
        constraint = automation_job.owner == owner.key() @ SolCronError::Unauthorized
    )]
    pub automation_job: Account<'info, AutomationJob>,
    
    pub owner: Signer<'info>,
}

pub fn update_job(
    ctx: Context<UpdateJob>,
    gas_limit: Option<u64>,
    min_balance: Option<u64>,
    trigger_params: Option<Vec<u8>>,
) -> Result<()> {
    let automation_job = &mut ctx.accounts.automation_job;
    let clock = Clock::get()?;

    if let Some(gas_limit) = gas_limit {
        require!(gas_limit > 0 && gas_limit <= 1_400_000, SolCronError::InvalidParameters);
        automation_job.gas_limit = gas_limit;
    }

    if let Some(min_balance) = min_balance {
        require!(automation_job.balance >= min_balance, SolCronError::InsufficientBalance);
        automation_job.min_balance = min_balance;
    }

    if let Some(trigger_params) = trigger_params {
        require!(trigger_params.len() <= 256, SolCronError::InvalidParameters);
        automation_job.trigger_params = trigger_params;
    }

    automation_job.updated_at = clock.unix_timestamp;

    emit!(JobUpdated {
        job_id: automation_job.job_id,
    });

    msg!("Job {} updated", automation_job.job_id);

    Ok(())
}

// Events
#[event]
pub struct JobRegistered {
    pub job_id: u64,
    pub owner: Pubkey,
    pub target_program: Pubkey,
    pub initial_funding: u64,
}

#[event]
pub struct JobFunded {
    pub job_id: u64,
    pub amount: u64,
    pub new_balance: u64,
}

#[event]
pub struct JobCancelled {
    pub job_id: u64,
    pub owner: Pubkey,
    pub refunded_amount: u64,
}

#[event]
pub struct JobUpdated {
    pub job_id: u64,
}