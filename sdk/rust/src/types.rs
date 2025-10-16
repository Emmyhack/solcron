use anchor_lang::prelude::*;
use serde::{Deserialize, Serialize};

/// Represents different types of automation triggers
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize)]
pub enum TriggerType {
    /// Time-based trigger that executes at regular intervals
    TimeBased { 
        /// Interval in seconds between executions
        interval: u64 
    },
    /// Conditional trigger based on account state or external data
    Conditional { 
        /// Logic expression as bytes (e.g., serialized condition)
        logic: Vec<u8> 
    },
    /// Log-based trigger that monitors on-chain events
    LogBased { 
        /// Program ID to monitor
        program_id: Pubkey,
        /// Event filter criteria
        event_filter: String,
    },
    /// Hybrid trigger combining multiple conditions
    Hybrid {
        /// Multiple trigger conditions
        conditions: Vec<TriggerCondition>,
        /// Logic operator: "AND" or "OR"
        operator: String,
    },
}

/// Individual trigger condition for hybrid triggers
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize)]
pub struct TriggerCondition {
    pub trigger_type: TriggerType,
    pub weight: u8, // 0-100, for priority weighting
}

/// Parameters for registering a new automation job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobParams {
    /// The target program to call
    pub target_program: Pubkey,
    /// The instruction name to call on the target program
    pub target_instruction: String,
    /// The trigger configuration
    pub trigger_type: TriggerType,
    /// Serialized trigger parameters
    pub trigger_params: Vec<u8>,
    /// Maximum gas/compute units for execution
    pub gas_limit: u64,
    /// Minimum balance to maintain in the job account
    pub min_balance: u64,
}

/// Automation job account state
#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize)]
pub struct AutomationJob {
    /// Unique job identifier
    pub job_id: u64,
    /// Job owner's public key
    pub owner: Pubkey,
    /// Target program to execute
    pub target_program: Pubkey,
    /// Target instruction name (max 32 bytes)
    pub target_instruction: String,
    /// Trigger configuration
    pub trigger_type: TriggerType,
    /// Serialized trigger parameters (max 64 bytes)
    pub trigger_params: Vec<u8>,
    /// Maximum compute units for execution
    pub gas_limit: u64,
    /// Current job balance (lamports)
    pub balance: u64,
    /// Minimum balance threshold (lamports)
    pub min_balance: u64,
    /// Whether the job is active
    pub is_active: bool,
    /// Number of times executed
    pub execution_count: u64,
    /// Timestamp of last execution
    pub last_execution: u64,
    /// Job creation timestamp
    pub created_at: u64,
}

/// Keeper account state
#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize)]
pub struct Keeper {
    /// Keeper's public key
    pub address: Pubkey,
    /// Amount staked by the keeper (lamports)
    pub stake_amount: u64,
    /// Reputation score (0-10000, basis points)
    pub reputation_score: u64,
    /// Whether the keeper is active
    pub is_active: bool,
    /// Total successful executions
    pub successful_executions: u64,
    /// Total failed executions
    pub failed_executions: u64,
    /// Pending rewards (lamports)
    pub pending_rewards: u64,
    /// Total earnings over time (lamports)
    pub total_earnings: u64,
    /// Registration timestamp
    pub registered_at: u64,
}

/// Registry state account
#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize)]
pub struct RegistryState {
    /// Registry admin
    pub admin: Pubkey,
    /// Base fee for job execution (lamports)
    pub base_fee: u64,
    /// Minimum stake required for keepers (lamports)
    pub min_stake: u64,
    /// Protocol fee in basis points (0-10000)
    pub protocol_fee_bps: u16,
    /// Treasury account for protocol fees
    pub treasury: Pubkey,
    /// Next available job ID
    pub next_job_id: u64,
    /// Total number of jobs registered
    pub total_jobs: u64,
    /// Number of active jobs
    pub active_jobs: u64,
    /// Total number of keepers
    pub total_keepers: u64,
    /// Number of active keepers
    pub active_keepers: u64,
    /// Total executions performed
    pub total_executions: u64,
    /// Total fees collected (lamports)
    pub total_fees_collected: u64,
    /// Registry creation timestamp
    pub created_at: u64,
}

/// Execution record for tracking job runs
#[derive(Debug, Clone, Serialize, Deserialize, AnchorSerialize, AnchorDeserialize)]
pub struct ExecutionRecord {
    /// Job ID that was executed
    pub job_id: u64,
    /// Keeper who executed the job
    pub keeper: Pubkey,
    /// Execution timestamp
    pub executed_at: u64,
    /// Whether execution was successful
    pub success: bool,
    /// Gas used in execution
    pub gas_used: u64,
    /// Fee charged for execution
    pub fee_charged: u64,
    /// Error message if execution failed
    pub error_message: String,
}

/// Statistics for a keeper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeeperStats {
    pub keeper: Pubkey,
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub success_rate: f64, // 0.0 - 1.0
    pub total_earnings: u64,
    pub reputation_score: u64,
    pub stake_amount: u64,
    pub is_active: bool,
}

/// Statistics for a job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStats {
    pub job_id: u64,
    pub owner: Pubkey,
    pub total_executions: u64,
    pub total_fees_paid: u64,
    pub last_execution: u64,
    pub is_active: bool,
    pub current_balance: u64,
    pub success_rate: f64,
}

/// Overall registry statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStats {
    pub total_jobs: u64,
    pub active_jobs: u64,
    pub total_keepers: u64,
    pub active_keepers: u64,
    pub total_executions: u64,
    pub total_fees_collected: u64,
    pub average_success_rate: f64,
}

/// Job execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub job_id: u64,
    pub success: bool,
    pub gas_used: u64,
    pub fee_charged: u64,
    pub execution_time: u64,
    pub error: Option<String>,
}

/// Trigger evaluation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerEvaluation {
    pub should_execute: bool,
    pub reason: String,
    pub next_evaluation: Option<u64>, // Timestamp for next check
}

impl AutomationJob {
    /// Calculate the space required for this account
    pub const ACCOUNT_SIZE: usize = 8 + // discriminator
        8 +  // job_id
        32 + // owner
        32 + // target_program
        32 + // target_instruction (max)
        64 + // trigger_type (max enum variant)
        64 + // trigger_params
        8 +  // gas_limit
        8 +  // balance
        8 +  // min_balance
        1 +  // is_active
        8 +  // execution_count
        8 +  // last_execution
        8;   // created_at

    /// Check if the job can be executed (has sufficient balance)
    pub fn can_execute(&self, execution_fee: u64) -> bool {
        self.is_active && 
        self.balance >= execution_fee && 
        self.balance >= self.min_balance
    }

    /// Get time until next execution for time-based jobs
    pub fn time_until_next_execution(&self, current_time: u64) -> Option<u64> {
        match &self.trigger_type {
            TriggerType::TimeBased { interval } => {
                if self.last_execution == 0 {
                    return Some(0); // Can execute immediately
                }
                let next_execution = self.last_execution + interval;
                if current_time >= next_execution {
                    Some(0) // Can execute now
                } else {
                    Some(next_execution - current_time)
                }
            }
            _ => None,
        }
    }
}

impl Keeper {
    /// Calculate the space required for this account
    pub const ACCOUNT_SIZE: usize = 8 + // discriminator
        32 + // address
        8 +  // stake_amount
        8 +  // reputation_score
        1 +  // is_active
        8 +  // successful_executions
        8 +  // failed_executions
        8 +  // pending_rewards
        8 +  // total_earnings
        8;   // registered_at

    /// Calculate success rate as a percentage (0-100)
    pub fn success_rate(&self) -> f64 {
        let total = self.successful_executions + self.failed_executions;
        if total == 0 {
            0.0
        } else {
            (self.successful_executions as f64 / total as f64) * 100.0
        }
    }

    /// Update reputation based on execution result
    pub fn update_reputation(&mut self, success: bool) {
        const MAX_REPUTATION: u64 = 10000;
        const MIN_REPUTATION: u64 = 0;
        const REPUTATION_DELTA: u64 = 100;

        if success {
            self.reputation_score = std::cmp::min(
                MAX_REPUTATION, 
                self.reputation_score + REPUTATION_DELTA
            );
            self.successful_executions += 1;
        } else {
            self.reputation_score = std::cmp::max(
                MIN_REPUTATION, 
                self.reputation_score.saturating_sub(REPUTATION_DELTA * 2)
            );
            self.failed_executions += 1;
        }
    }

    /// Check if keeper is eligible to execute jobs
    pub fn is_eligible(&self, min_reputation: u64) -> bool {
        self.is_active && self.reputation_score >= min_reputation
    }
}

impl RegistryState {
    /// Calculate the space required for this account
    pub const ACCOUNT_SIZE: usize = 8 + // discriminator
        32 + // admin
        8 +  // base_fee
        8 +  // min_stake
        2 +  // protocol_fee_bps
        32 + // treasury
        8 +  // next_job_id
        8 +  // total_jobs
        8 +  // active_jobs
        8 +  // total_keepers
        8 +  // active_keepers
        8 +  // total_executions
        8 +  // total_fees_collected
        8;   // created_at

    /// Calculate execution fee for a job
    pub fn calculate_execution_fee(&self, gas_used: u64) -> u64 {
        self.base_fee + gas_used // Simple fee model: base fee + gas cost
    }

    /// Calculate protocol fee from total execution fee
    pub fn calculate_protocol_fee(&self, execution_fee: u64) -> u64 {
        (execution_fee * self.protocol_fee_bps as u64) / 10000
    }

    /// Calculate keeper reward (execution fee - protocol fee)
    pub fn calculate_keeper_reward(&self, execution_fee: u64) -> u64 {
        execution_fee - self.calculate_protocol_fee(execution_fee)
    }
}

impl Default for TriggerType {
    fn default() -> Self {
        TriggerType::TimeBased { interval: 3600 } // 1 hour default
    }
}