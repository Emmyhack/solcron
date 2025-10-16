use serde::{Deserialize, Serialize};
use std::fs;
use crate::error::KeeperError;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KeeperConfig {
    pub keeper: KeeperSettings,
    pub rpc: RpcSettings,
    pub monitoring: MonitoringSettings,
    pub execution: ExecutionSettings,
    pub database: DatabaseSettings,
    pub logging: LoggingSettings,
    pub metrics: MetricsSettings,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KeeperSettings {
    pub wallet_path: String,
    pub stake_amount: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RpcSettings {
    pub primary_url: String,
    pub fallback_urls: Vec<String>,
    pub request_timeout_ms: Option<u64>,
    pub max_retries: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MonitoringSettings {
    pub poll_interval_ms: u64,
    pub max_concurrent_jobs: usize,
    pub job_cache_ttl_seconds: u64,
    pub enable_websocket: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExecutionSettings {
    pub priority_fee_percentile: u32,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
    pub max_compute_units: u32,
    pub simulation_enabled: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseSettings {
    pub url: String,
    pub max_connections: Option<u32>,
    pub connection_timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingSettings {
    pub level: String,
    pub file_path: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetricsSettings {
    pub enabled: bool,
    pub port: Option<u16>,
}

impl KeeperConfig {
    pub fn load(path: &str) -> Result<Self, KeeperError> {
        let content = fs::read_to_string(path)
            .map_err(|e| KeeperError::ConfigError(format!("Failed to read config file {}: {}", path, e)))?;
        
        let config: KeeperConfig = toml::from_str(&content)
            .map_err(|e| KeeperError::ConfigError(format!("Failed to parse config: {}", e)))?;
        
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), KeeperError> {
        // Validate wallet path exists
        if !std::path::Path::new(&self.keeper.wallet_path).exists() {
            return Err(KeeperError::ConfigError(
                format!("Wallet file not found: {}", self.keeper.wallet_path)
            ));
        }

        // Validate RPC URLs
        if self.rpc.primary_url.is_empty() {
            return Err(KeeperError::ConfigError("Primary RPC URL cannot be empty".to_string()));
        }

        // Validate monitoring settings
        if self.monitoring.poll_interval_ms < 100 {
            return Err(KeeperError::ConfigError("Poll interval too small (min 100ms)".to_string()));
        }

        if self.monitoring.max_concurrent_jobs == 0 {
            return Err(KeeperError::ConfigError("Max concurrent jobs must be > 0".to_string()));
        }

        // Validate execution settings
        if self.execution.priority_fee_percentile > 100 {
            return Err(KeeperError::ConfigError("Priority fee percentile must be <= 100".to_string()));
        }

        if self.execution.max_compute_units > 1_400_000 {
            return Err(KeeperError::ConfigError("Max compute units too high (max 1.4M)".to_string()));
        }

        // Validate database URL
        if !self.database.url.starts_with("postgresql://") {
            return Err(KeeperError::ConfigError("Only PostgreSQL databases are supported".to_string()));
        }

        Ok(())
    }

    pub fn get_rpc_urls(&self) -> Vec<String> {
        let mut urls = vec![self.rpc.primary_url.clone()];
        urls.extend(self.rpc.fallback_urls.clone());
        urls
    }

    pub fn get_request_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(
            self.rpc.request_timeout_ms.unwrap_or(30_000)
        )
    }

    pub fn get_max_rpc_retries(&self) -> u32 {
        self.rpc.max_retries.unwrap_or(3)
    }

    pub fn get_poll_interval(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.monitoring.poll_interval_ms)
    }

    pub fn get_retry_delay(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.execution.retry_delay_ms)
    }

    pub fn websocket_enabled(&self) -> bool {
        self.monitoring.enable_websocket.unwrap_or(true)
    }

    pub fn simulation_enabled(&self) -> bool {
        self.execution.simulation_enabled.unwrap_or(true)
    }

    pub fn get_max_db_connections(&self) -> u32 {
        self.database.max_connections.unwrap_or(10)
    }

    pub fn get_db_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(
            self.database.connection_timeout_ms.unwrap_or(10_000)
        )
    }

    pub fn get_metrics_port(&self) -> u16 {
        self.metrics.port.unwrap_or(9090)
    }
}