use anchor_lang::prelude::*;

#[error_code]
pub enum SolCronError {
    #[msg("Unauthorized: Only admin can perform this action")]
    Unauthorized,
    
    #[msg("Invalid job: Job not found or inactive")]
    InvalidJob,
    
    #[msg("Invalid keeper: Keeper not found or inactive")]
    InvalidKeeper,
    
    #[msg("Insufficient balance: Job balance too low")]
    InsufficientBalance,
    
    #[msg("Insufficient stake: Keeper stake below minimum")]
    InsufficientStake,
    
    #[msg("Invalid trigger: Trigger conditions not met")]
    InvalidTrigger,
    
    #[msg("Gas limit exceeded: Job execution would exceed gas limit")]
    GasLimitExceeded,
    
    #[msg("Execution failed: Target program execution failed")]
    ExecutionFailed,
    
    #[msg("Invalid parameters: One or more parameters are invalid")]
    InvalidParameters,
    
    #[msg("Job already exists: Job ID already in use")]
    JobAlreadyExists,
    
    #[msg("Keeper already registered: Keeper address already exists")]
    KeeperAlreadyRegistered,
    
    #[msg("Cooldown period: Action not allowed during cooldown")]
    CooldownPeriod,
    
    #[msg("Rate limit exceeded: Too many operations in time window")]
    RateLimitExceeded,
    
    #[msg("Invalid fee: Fee calculation error")]
    InvalidFee,
    
    #[msg("Slashing failed: Cannot slash keeper")]
    SlashingFailed,
    
    #[msg("Rewards claim failed: No rewards to claim")]
    NoRewardsToClaim,
    
    #[msg("Job execution too early: Time-based trigger not ready")]
    ExecutionTooEarly,
    
    #[msg("Target program error: CPI call failed")]
    TargetProgramError,
    
    #[msg("Math overflow: Calculation resulted in overflow")]
    MathOverflow,
    
    #[msg("Invalid instruction data: Malformed instruction parameters")]
    InvalidInstructionData,
}