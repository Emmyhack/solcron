use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::*;

// Execute Job
#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct ExecuteJob<'info> {
    #[account(
        mut,
        seeds = [b"registry"],
        bump = registry_state.bump
    )]
    pub registry_state: Account<'info, RegistryState>,
    
    #[account(
        mut,
        seeds = [b"job", job_id.to_le_bytes().as_ref()],
        bump = automation_job.bump,
        constraint = automation_job.is_active @ SolCronError::InvalidJob,
        constraint = automation_job.job_id == job_id @ SolCronError::InvalidJob
    )]
    pub automation_job: Account<'info, AutomationJob>,
    
    #[account(
        mut,
        seeds = [b"keeper", keeper_account.key().as_ref()],
        bump = keeper.bump,
        constraint = keeper.is_active @ SolCronError::InvalidKeeper,
        constraint = keeper.address == keeper_account.key() @ SolCronError::Unauthorized
    )]
    pub keeper: Account<'info, Keeper>,
    
    #[account(
        init,
        payer = keeper_account,
        space = ExecutionRecord::MAX_SIZE,
        seeds = [
            b"execution", 
            job_id.to_le_bytes().as_ref(),
            registry_state.total_executions.to_le_bytes().as_ref()
        ],
        bump
    )]
    pub execution_record: Account<'info, ExecutionRecord>,
    
    #[account(mut)]
    pub keeper_account: Signer<'info>,
    
    /// CHECK: Target program to execute
    pub target_program: AccountInfo<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn execute_job(ctx: Context<ExecuteJob>, job_id: u64) -> Result<()> {
    let automation_job = &mut ctx.accounts.automation_job;
    let keeper = &mut ctx.accounts.keeper;
    let registry_state = &mut ctx.accounts.registry_state;
    let execution_record = &mut ctx.accounts.execution_record;
    let clock = Clock::get()?;

    // Verify target program matches
    require!(
        ctx.accounts.target_program.key() == automation_job.target_program,
        SolCronError::InvalidParameters
    );

    // Check if execution is allowed based on trigger type
    let execution_allowed = match &automation_job.trigger_type {
        TriggerType::TimeBased => {
            // Parse interval from trigger_params (simplified for now)
            let min_interval = if automation_job.trigger_params.len() >= 8 {
                i64::from_le_bytes(automation_job.trigger_params[0..8].try_into().unwrap_or([0; 8]))
            } else {
                60 // Default 1 minute
            };
            clock.unix_timestamp - automation_job.last_execution >= min_interval
        },
        TriggerType::Conditional => {
            // For now, we'll implement basic conditional logic
            // In a full implementation, this would evaluate custom conditions
            evaluate_conditional_trigger(automation_job, &clock)?
        },
        TriggerType::LogTrigger => {
            // Log-based triggers would require additional event monitoring
            // For now, we'll allow execution if enough time has passed
            clock.unix_timestamp - automation_job.last_execution >= 60
        },
        TriggerType::Hybrid => {
            // Hybrid triggers combine multiple conditions
            evaluate_hybrid_trigger(automation_job, &clock)?
        }
    };

    require!(execution_allowed, SolCronError::InvalidTrigger);

    // Check job has sufficient balance
    let execution_fee = calculate_execution_fee(registry_state, automation_job)?;
    require!(automation_job.balance >= execution_fee, SolCronError::InsufficientBalance);
    require!(
        automation_job.balance - execution_fee >= automation_job.min_balance,
        SolCronError::InsufficientBalance
    );

    // Initialize execution record
    execution_record.job_id = job_id;
    execution_record.keeper = keeper.address;
    execution_record.timestamp = clock.unix_timestamp;
    execution_record.success = false; // Will be updated based on execution result
    execution_record.gas_used = 0; // Would be measured in actual implementation
    execution_record.fee_paid = execution_fee;
    execution_record.error_code = None;
    execution_record.bump = ctx.bumps.execution_record;

    // Attempt to execute the target instruction via CPI
    let execution_result = execute_target_instruction(
        &automation_job,
        &ctx.accounts.target_program,
        ctx.remaining_accounts,
    );

    let success = execution_result.is_ok();
    let error_code = if execution_result.is_err() {
        Some(1u32) // Generic error code, could be more specific based on error type
    } else {
        None
    };

    // Update execution record
    execution_record.success = success;
    execution_record.error_code = error_code;

    // Update job state
    automation_job.last_execution = clock.unix_timestamp;
    automation_job.execution_count += 1;
    automation_job.balance -= execution_fee;

    keeper.total_executions += 1;
    if success {
        keeper.successful_executions += 1;
    }

    // Distribute fees
    distribute_execution_fees(
        registry_state,
        keeper,
        execution_fee,
        &ctx.accounts.system_program,
    )?;

    // Update keeper reputation and last execution time
    keeper.reputation_score = keeper.calculate_reputation();
    keeper.last_execution_time = clock.unix_timestamp;

    // Update registry stats
    registry_state.total_executions += 1;

    // Deactivate job if balance is too low
    if automation_job.balance < automation_job.min_balance {
        automation_job.is_active = false;
        registry_state.active_jobs -= 1;
        
        emit!(JobDeactivated {
            job_id: automation_job.job_id,
            reason: "Insufficient balance".to_string(),
        });
    }

    emit!(JobExecuted {
        job_id: automation_job.job_id,
        keeper: keeper.address,
        success,
        fee_paid: execution_fee,
        gas_used: execution_record.gas_used,
    });

    if success {
        msg!("Job {} executed successfully by keeper {}", job_id, keeper.address);
    } else {
        msg!("Job {} execution failed by keeper {}, error: {:?}", job_id, keeper.address, error_code);
    }

    Ok(())
}

// Helper functions
fn calculate_execution_fee(registry_state: &RegistryState, _job: &AutomationJob) -> Result<u64> {
    // Simple fee calculation - in a full implementation this would be more sophisticated
    Ok(registry_state.base_fee)
}

fn evaluate_conditional_trigger(_job: &AutomationJob, _clock: &Clock) -> Result<bool> {
    // Placeholder for conditional trigger evaluation
    // In a full implementation, this would parse and evaluate the condition logic
    Ok(true)
}

fn evaluate_hybrid_trigger(_job: &AutomationJob, clock: &Clock) -> Result<bool> {
    // Placeholder for hybrid trigger evaluation
    // This would combine multiple trigger conditions
    Ok(clock.unix_timestamp % 60 == 0) // Simple example
}

fn execute_target_instruction(
    job: &AutomationJob,
    target_program: &AccountInfo,
    remaining_accounts: &[AccountInfo],
) -> Result<()> {
    // This is a simplified version - in a full implementation, this would:
    // 1. Parse the target instruction name and parameters
    // 2. Build the appropriate instruction data
    // 3. Perform CPI call to the target program
    // 4. Handle the response and any errors
    
    msg!("Executing instruction: {} on program: {}", 
         job.target_instruction, 
         target_program.key());
    
    // For now, we'll simulate a successful execution
    // In reality, this would involve complex CPI logic
    if remaining_accounts.is_empty() {
        return Err(SolCronError::InvalidParameters.into());
    }
    
    Ok(())
}

fn distribute_execution_fees(
    registry_state: &RegistryState,
    keeper: &mut Keeper,
    total_fee: u64,
    _system_program: &Program<System>,
) -> Result<()> {
    // Calculate fee distribution
    let protocol_fee = (total_fee as u128 * registry_state.protocol_fee_bps as u128 / 10000) as u64;
    let keeper_fee = total_fee - protocol_fee;

    // Add keeper fee to pending rewards
    keeper.pending_rewards = keeper.pending_rewards
        .checked_add(keeper_fee)
        .ok_or(SolCronError::MathOverflow)?;
    
    keeper.total_earnings = keeper.total_earnings
        .checked_add(keeper_fee)
        .ok_or(SolCronError::MathOverflow)?;

    // Protocol fee would be transferred to treasury in a full implementation
    msg!("Fees distributed - Keeper: {}, Protocol: {}", keeper_fee, protocol_fee);

    Ok(())
}

// Events
#[event]
pub struct JobExecuted {
    pub job_id: u64,
    pub keeper: Pubkey,
    pub success: bool,
    pub fee_paid: u64,
    pub gas_used: u64,
}

#[event]
pub struct JobDeactivated {
    pub job_id: u64,
    pub reason: String,
}