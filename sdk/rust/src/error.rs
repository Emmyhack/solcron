use thiserror::Error;
use solana_program::program_error::ProgramError;

/// Errors that can occur when using the SolCron SDK
#[derive(Error, Debug, Clone, PartialEq)]
pub enum SolCronError {
    /// Generic error with message
    #[error("SolCron error: {message}")]
    Generic { message: String },

    /// Client connection errors
    #[error("Failed to connect to Solana RPC: {source}")]
    ConnectionError { source: String },

    /// Account not found
    #[error("Account not found: {account}")]
    AccountNotFound { account: String },

    /// Invalid account data
    #[error("Invalid account data for {account_type}: {reason}")]
    InvalidAccountData { 
        account_type: String, 
        reason: String 
    },

    /// Insufficient balance for operation
    #[error("Insufficient balance: required {required}, available {available}")]
    InsufficientBalance { 
        required: u64, 
        available: u64 
    },

    /// Job not found
    #[error("Job not found: {job_id}")]
    JobNotFound { job_id: u64 },

    /// Job already exists
    #[error("Job already exists: {job_id}")]
    JobAlreadyExists { job_id: u64 },

    /// Job is not active
    #[error("Job is not active: {job_id}")]
    JobNotActive { job_id: u64 },

    /// Keeper not found
    #[error("Keeper not found: {keeper}")]
    KeeperNotFound { keeper: String },

    /// Keeper already registered
    #[error("Keeper already registered: {keeper}")]
    KeeperAlreadyRegistered { keeper: String },

    /// Keeper not active
    #[error("Keeper not active: {keeper}")]
    KeeperNotActive { keeper: String },

    /// Insufficient stake for keeper operations
    #[error("Insufficient stake: required {required}, provided {provided}")]
    InsufficientStake { 
        required: u64, 
        provided: u64 
    },

    /// Trigger condition not met
    #[error("Trigger condition not met for job {job_id}: {reason}")]
    TriggerNotMet { 
        job_id: u64, 
        reason: String 
    },

    /// Invalid trigger configuration
    #[error("Invalid trigger configuration: {reason}")]
    InvalidTrigger { reason: String },

    /// Execution failed
    #[error("Execution failed for job {job_id}: {error}")]
    ExecutionFailed { 
        job_id: u64, 
        error: String 
    },

    /// Unauthorized operation
    #[error("Unauthorized: {operation} requires {required_role}")]
    Unauthorized { 
        operation: String, 
        required_role: String 
    },

    /// Invalid program ID
    #[error("Invalid program ID: expected {expected}, got {actual}")]
    InvalidProgramId { 
        expected: String, 
        actual: String 
    },

    /// Serialization error
    #[error("Serialization error: {reason}")]
    SerializationError { reason: String },

    /// Deserialization error
    #[error("Deserialization error: {reason}")]
    DeserializationError { reason: String },

    /// Transaction building error
    #[error("Failed to build transaction: {reason}")]
    TransactionBuildError { reason: String },

    /// Transaction execution error
    #[error("Transaction execution failed: {reason}")]
    TransactionExecutionError { reason: String },

    /// Invalid instruction data
    #[error("Invalid instruction data: {reason}")]
    InvalidInstructionData { reason: String },

    /// Account creation error
    #[error("Failed to create account: {reason}")]
    AccountCreationError { reason: String },

    /// PDA derivation error
    #[error("Failed to derive PDA: {reason}")]
    PDADerivationError { reason: String },

    /// Configuration error
    #[error("Configuration error: {reason}")]
    ConfigError { reason: String },

    /// Network error
    #[error("Network error: {reason}")]
    NetworkError { reason: String },

    /// Timeout error
    #[error("Operation timed out: {operation}")]
    TimeoutError { operation: String },

    /// Rate limit exceeded
    #[error("Rate limit exceeded for {resource}")]
    RateLimitExceeded { resource: String },

    /// Validation error
    #[error("Validation failed: {field} {reason}")]
    ValidationError { 
        field: String, 
        reason: String 
    },

    /// Math overflow/underflow
    #[error("Math error: {operation} caused {error_type}")]
    MathError { 
        operation: String, 
        error_type: String 
    },

    /// Feature not implemented
    #[error("Feature not implemented: {feature}")]
    NotImplemented { feature: String },

    /// Internal SDK error
    #[error("Internal SDK error: {reason}")]
    InternalError { reason: String },
}

impl From<std::io::Error> for SolCronError {
    fn from(error: std::io::Error) -> Self {
        SolCronError::Generic { 
            message: format!("IO error: {}", error) 
        }
    }
}

impl From<serde_json::Error> for SolCronError {
    fn from(error: serde_json::Error) -> Self {
        SolCronError::SerializationError { 
            reason: error.to_string() 
        }
    }
}

impl From<solana_client::client_error::ClientError> for SolCronError {
    fn from(error: solana_client::client_error::ClientError) -> Self {
        SolCronError::ConnectionError { 
            source: error.to_string() 
        }
    }
}

impl From<solana_sdk::transport::TransportError> for SolCronError {
    fn from(error: solana_sdk::transport::TransportError) -> Self {
        SolCronError::NetworkError { 
            reason: error.to_string() 
        }
    }
}

impl From<anchor_client::ClientError> for SolCronError {
    fn from(error: anchor_client::ClientError) -> Self {
        SolCronError::Generic { 
            message: format!("Anchor client error: {}", error) 
        }
    }
}

impl From<ProgramError> for SolCronError {
    fn from(error: ProgramError) -> Self {
        SolCronError::Generic { 
            message: format!("Program error: {:?}", error) 
        }
    }
}

/// Specialized result type for SolCron operations
pub type SolCronResult<T> = Result<T, SolCronError>;

/// Error codes that match the on-chain program errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum SolCronErrorCode {
    /// Invalid instruction
    InvalidInstruction = 0,
    /// Insufficient balance
    InsufficientBalance = 1,
    /// Invalid job
    InvalidJob = 2,
    /// Job not active
    JobNotActive = 3,
    /// Keeper not found
    KeeperNotFound = 4,
    /// Insufficient stake
    InsufficientStake = 5,
    /// Invalid trigger
    InvalidTrigger = 6,
    /// Execution failed
    ExecutionFailed = 7,
    /// Unauthorized
    Unauthorized = 8,
    /// Invalid program ID
    InvalidProgramId = 9,
    /// Account already exists
    AccountAlreadyExists = 10,
    /// No rewards to claim
    NoRewardsToClaim = 11,
    /// Math overflow
    MathOverflow = 12,
    /// Invalid fee rate
    InvalidFeeRate = 13,
    /// Keeper already registered
    KeeperAlreadyRegistered = 14,
    /// Registry not initialized
    RegistryNotInitialized = 15,
}

impl SolCronErrorCode {
    pub fn as_u32(self) -> u32 {
        self as u32
    }

    pub fn from_u32(code: u32) -> Option<Self> {
        match code {
            0 => Some(Self::InvalidInstruction),
            1 => Some(Self::InsufficientBalance),
            2 => Some(Self::InvalidJob),
            3 => Some(Self::JobNotActive),
            4 => Some(Self::KeeperNotFound),
            5 => Some(Self::InsufficientStake),
            6 => Some(Self::InvalidTrigger),
            7 => Some(Self::ExecutionFailed),
            8 => Some(Self::Unauthorized),
            9 => Some(Self::InvalidProgramId),
            10 => Some(Self::AccountAlreadyExists),
            11 => Some(Self::NoRewardsToClaim),
            12 => Some(Self::MathOverflow),
            13 => Some(Self::InvalidFeeRate),
            14 => Some(Self::KeeperAlreadyRegistered),
            15 => Some(Self::RegistryNotInitialized),
            _ => None,
        }
    }
}

impl std::fmt::Display for SolCronErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidInstruction => "Invalid instruction",
            Self::InsufficientBalance => "Insufficient balance",
            Self::InvalidJob => "Invalid job",
            Self::JobNotActive => "Job not active",
            Self::KeeperNotFound => "Keeper not found",
            Self::InsufficientStake => "Insufficient stake",
            Self::InvalidTrigger => "Invalid trigger",
            Self::ExecutionFailed => "Execution failed",
            Self::Unauthorized => "Unauthorized",
            Self::InvalidProgramId => "Invalid program ID",
            Self::AccountAlreadyExists => "Account already exists",
            Self::NoRewardsToClaim => "No rewards to claim",
            Self::MathOverflow => "Math overflow",
            Self::InvalidFeeRate => "Invalid fee rate",
            Self::KeeperAlreadyRegistered => "Keeper already registered",
            Self::RegistryNotInitialized => "Registry not initialized",
        };
        write!(f, "{}", message)
    }
}

/// Helper functions for error handling
impl SolCronError {
    /// Create a validation error
    pub fn validation(field: &str, reason: &str) -> Self {
        Self::ValidationError {
            field: field.to_string(),
            reason: reason.to_string(),
        }
    }

    /// Create a generic error with message
    pub fn generic(message: &str) -> Self {
        Self::Generic {
            message: message.to_string(),
        }
    }

    /// Create an internal error
    pub fn internal(reason: &str) -> Self {
        Self::InternalError {
            reason: reason.to_string(),
        }
    }

    /// Check if error is recoverable (can be retried)
    pub fn is_recoverable(&self) -> bool {
        match self {
            Self::ConnectionError { .. } |
            Self::NetworkError { .. } |
            Self::TimeoutError { .. } |
            Self::RateLimitExceeded { .. } => true,
            
            Self::AccountNotFound { .. } |
            Self::InvalidAccountData { .. } |
            Self::JobNotFound { .. } |
            Self::KeeperNotFound { .. } |
            Self::Unauthorized { .. } |
            Self::ValidationError { .. } => false,
            
            _ => false,
        }
    }

    /// Get error code if available
    pub fn error_code(&self) -> Option<SolCronErrorCode> {
        match self {
            Self::InsufficientBalance { .. } => Some(SolCronErrorCode::InsufficientBalance),
            Self::JobNotFound { .. } => Some(SolCronErrorCode::InvalidJob),
            Self::JobNotActive { .. } => Some(SolCronErrorCode::JobNotActive),
            Self::KeeperNotFound { .. } => Some(SolCronErrorCode::KeeperNotFound),
            Self::InsufficientStake { .. } => Some(SolCronErrorCode::InsufficientStake),
            Self::InvalidTrigger { .. } => Some(SolCronErrorCode::InvalidTrigger),
            Self::ExecutionFailed { .. } => Some(SolCronErrorCode::ExecutionFailed),
            Self::Unauthorized { .. } => Some(SolCronErrorCode::Unauthorized),
            Self::InvalidProgramId { .. } => Some(SolCronErrorCode::InvalidProgramId),
            _ => None,
        }
    }
}