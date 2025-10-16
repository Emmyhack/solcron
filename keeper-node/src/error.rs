use thiserror::Error;

#[derive(Error, Debug)]
pub enum KeeperError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("RPC error: {0}")]
    RpcError(String),

    #[error("Solana client error: {0}")]
    SolanaClientError(#[from] solana_client::client_error::ClientError),

    #[error("Anchor client error: {0}")]
    AnchorClientError(#[from] anchor_client::ClientError),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] bincode::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("HTTP request error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("WebSocket error: {0}")]
    WebSocketError(String),

    #[error("Monitoring error: {0}")]
    MonitoringError(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("Evaluation error: {0}")]
    EvaluationError(String),

    #[error("Invalid job: {0}")]
    InvalidJobError(String),

    #[error("Insufficient balance: {0}")]
    InsufficientBalanceError(String),

    #[error("Transaction failed: {0}")]
    TransactionError(String),

    #[error("Keeper not registered")]
    KeeperNotRegistered,

    #[error("Keeper already registered")]
    KeeperAlreadyRegistered,

    #[error("Invalid trigger condition: {0}")]
    InvalidTriggerError(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<anchor_lang::error::Error> for KeeperError {
    fn from(error: anchor_lang::error::Error) -> Self {
        KeeperError::AnchorClientError(anchor_client::ClientError::AnchorError(error))
    }
}

pub type KeeperResult<T> = Result<T, KeeperError>;