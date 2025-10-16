use anchor_lang::prelude::*;

/// Job configuration and state
#[account]
pub struct AutomationJob {
    pub job_id: u64,                    // Unique job identifier
    pub owner: Pubkey,                  // Job creator's wallet
    pub target_program: Pubkey,         // Program to call
    pub target_instruction: String,     // Instruction name to invoke
    pub trigger_type: TriggerType,      // Type of trigger
    pub trigger_params: Vec<u8>,        // Serialized trigger parameters
    pub gas_limit: u64,                 // Max compute units per execution
    pub balance: u64,                   // Remaining SOL balance
    pub min_balance: u64,               // Minimum balance threshold
    pub is_active: bool,                // Job status
    pub execution_count: u64,           // Total executions
    pub last_execution: i64,            // Last execution timestamp
    pub created_at: i64,                // Creation timestamp
    pub updated_at: i64,                // Last update timestamp
    pub bump: u8,                       // PDA bump seed
}

impl AutomationJob {
    pub const MAX_SIZE: usize = 8 + // discriminator
        8 + // job_id
        32 + // owner
        32 + // target_program
        (4 + 50) + // target_instruction (max 50 chars)
        1 + 8 + // trigger_type enum
        (4 + 256) + // trigger_params (max 256 bytes)
        8 + // gas_limit
        8 + // balance
        8 + // min_balance
        1 + // is_active
        8 + // execution_count
        8 + // last_execution
        8 + // created_at
        8 + // updated_at
        1; // bump
}

/// Trigger type configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum TriggerType {
    /// Execute on time-based schedule
    TimeBased,
    /// Execute based on conditions
    Conditional,
    /// Execute based on log events
    LogTrigger,
    /// Hybrid trigger combining multiple conditions
    Hybrid,
}

/// Keeper registration and reputation
#[account]
pub struct Keeper {
    pub address: Pubkey,                // Keeper's wallet address
    pub stake_amount: u64,              // Staked SOL amount
    pub reputation_score: u64,          // Performance score (0-10000)
    pub is_active: bool,                // Active status
    pub total_executions: u64,          // Total job executions
    pub successful_executions: u64,     // Successful job executions
    pub total_earnings: u64,            // Total fees earned
    pub pending_rewards: u64,           // Unclaimed rewards
    pub last_execution_time: i64,       // Last execution timestamp
    pub registered_at: i64,             // Registration timestamp
    pub bump: u8,                       // PDA bump seed
}

impl Keeper {
    pub const MAX_SIZE: usize = 8 + // discriminator
        32 + // address
        8 + // stake_amount
        8 + // reputation_score
        1 + // is_active
        8 + // total_executions
        8 + // successful_executions
        8 + // total_earnings
        8 + // pending_rewards
        8 + // last_execution_time
        8 + // registered_at
        1; // bump

    /// Calculate reputation score based on performance
    pub fn calculate_reputation(&self) -> u64 {
        if self.total_executions == 0 {
            return 5000; // Default score for new keepers
        }
        
        let success_rate = (self.successful_executions * 10000) / self.total_executions;
        
        // Adjust for volume (bonus for high-volume keepers)
        let volume_bonus = if self.total_executions > 1000 {
            500
        } else if self.total_executions > 100 {
            200
        } else {
            0
        };
        
        std::cmp::min(success_rate + volume_bonus, 10000)
    }
}

/// Global registry state
#[account]
pub struct RegistryState {
    pub admin: Pubkey,                  // Protocol admin
    pub base_fee: u64,                  // Base execution fee in lamports
    pub min_stake: u64,                 // Minimum keeper stake in lamports
    pub protocol_fee_bps: u16,          // Protocol fee in basis points
    pub treasury: Pubkey,               // Treasury wallet
    pub total_jobs: u64,                // Total jobs created
    pub active_jobs: u64,               // Currently active jobs
    pub total_keepers: u64,             // Total registered keepers
    pub active_keepers: u64,            // Currently active keepers
    pub total_executions: u64,          // Total job executions
    pub successful_executions: u64,     // Successful job executions
    pub protocol_revenue: u64,          // Total protocol revenue
    pub next_job_id: u64,               // Next job ID counter
    pub is_paused: bool,                // Emergency pause status
    pub bump: u8,                       // PDA bump seed
}

impl RegistryState {
    pub const MAX_SIZE: usize = 8 + // discriminator
        32 + // admin
        8 + // base_fee
        8 + // min_stake
        2 + // protocol_fee_bps
        32 + // treasury
        8 + // total_jobs
        8 + // active_jobs
        8 + // total_keepers
        8 + // active_keepers
        8 + // total_executions
        8 + // successful_executions
        8 + // protocol_revenue
        8 + // next_job_id
        1 + // is_paused
        1; // bump
}

/// Execution record for tracking
#[account]
pub struct ExecutionRecord {
    pub job_id: u64,                    // Associated job ID
    pub keeper: Pubkey,                 // Executing keeper
    pub timestamp: i64,                 // Execution timestamp
    pub success: bool,                  // Execution result
    pub gas_used: u64,                  // Compute units consumed
    pub fee_paid: u64,                  // Fee paid to keeper
    pub error_code: Option<u32>,        // Error code if failed
    pub bump: u8,                       // PDA bump seed
}

impl ExecutionRecord {
    pub const MAX_SIZE: usize = 8 + // discriminator
        8 + // job_id
        32 + // keeper
        8 + // timestamp
        1 + // success
        8 + // gas_used
        8 + // fee_paid
        (1 + 4) + // error_code (Option<u32>)
        1; // bump
}