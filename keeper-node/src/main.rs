use clap::{Parser, Subcommand};
use log::{info, error};
use std::sync::Arc;

mod config;
mod monitor;
mod executor;
mod evaluator;
mod rpc;
mod database;
mod keeper;
mod error;

use config::KeeperConfig;
use keeper::KeeperNode;
use error::KeeperError;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to configuration file
    #[arg(short, long, default_value = "keeper-config.toml")]
    config: String,

    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    log_level: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the keeper node
    Start,
    /// Register as a keeper on-chain
    Register {
        /// Stake amount in SOL
        #[arg(short, long)]
        stake: f64,
    },
    /// Check keeper status
    Status,
    /// Claim accumulated rewards
    Claim,
    /// Unregister as keeper
    Unregister,
    /// Generate example configuration file
    GenConfig,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Initialize logging
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or(&cli.log_level)
    ).init();

    info!("SolCron Keeper Node v{}", env!("CARGO_PKG_VERSION"));

    match cli.command {
        Commands::Start => {
            let config = KeeperConfig::load(&cli.config)?;
            start_keeper(config).await?;
        }
        Commands::Register { stake } => {
            let config = KeeperConfig::load(&cli.config)?;
            register_keeper(config, stake).await?;
        }
        Commands::Status => {
            let config = KeeperConfig::load(&cli.config)?;
            show_status(config).await?;
        }
        Commands::Claim => {
            let config = KeeperConfig::load(&cli.config)?;
            claim_rewards(config).await?;
        }
        Commands::Unregister => {
            let config = KeeperConfig::load(&cli.config)?;
            unregister_keeper(config).await?;
        }
        Commands::GenConfig => {
            generate_config(&cli.config)?;
        }
    }

    Ok(())
}

async fn start_keeper(config: KeeperConfig) -> Result<(), KeeperError> {
    info!("Starting keeper node...");
    
    let keeper_node = Arc::new(KeeperNode::new(config).await?);
    
    // Start monitoring and execution services
    let node = keeper_node.clone();
    let monitoring_task = tokio::spawn(async move {
        node.start_monitoring().await
    });

    let node = keeper_node.clone();
    let execution_task = tokio::spawn(async move {
        node.start_execution().await
    });

    // Wait for shutdown signal
    tokio::select! {
        result = monitoring_task => {
            if let Err(e) = result {
                error!("Monitoring task failed: {:?}", e);
            }
        }
        result = execution_task => {
            if let Err(e) = result {
                error!("Execution task failed: {:?}", e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Received shutdown signal, stopping keeper node...");
        }
    }

    info!("Keeper node stopped");
    Ok(())
}

async fn register_keeper(config: KeeperConfig, stake: f64) -> Result<(), KeeperError> {
    info!("Registering keeper with stake: {} SOL", stake);
    
    let keeper_node = KeeperNode::new(config).await?;
    let stake_lamports = (stake * 1_000_000_000.0) as u64;
    
    let signature = keeper_node.register_keeper(stake_lamports).await?;
    info!("Keeper registered successfully! Transaction: {}", signature);
    
    Ok(())
}

async fn show_status(config: KeeperConfig) -> Result<(), KeeperError> {
    let keeper_node = KeeperNode::new(config).await?;
    let status = keeper_node.get_status().await?;
    
    println!("=== Keeper Status ===");
    println!("Address: {}", status.address);
    println!("Active: {}", status.is_active);
    println!("Stake: {} SOL", status.stake_amount as f64 / 1_000_000_000.0);
    println!("Reputation: {}/10000", status.reputation_score);
    println!("Successful Executions: {}", status.successful_executions);
    println!("Failed Executions: {}", status.failed_executions);
    println!("Total Earnings: {} SOL", status.total_earnings as f64 / 1_000_000_000.0);
    println!("Pending Rewards: {} SOL", status.pending_rewards as f64 / 1_000_000_000.0);
    
    Ok(())
}

async fn claim_rewards(config: KeeperConfig) -> Result<(), KeeperError> {
    info!("Claiming keeper rewards...");
    
    let keeper_node = KeeperNode::new(config).await?;
    let signature = keeper_node.claim_rewards().await?;
    
    info!("Rewards claimed successfully! Transaction: {}", signature);
    Ok(())
}

async fn unregister_keeper(config: KeeperConfig) -> Result<(), KeeperError> {
    info!("Unregistering keeper...");
    
    let keeper_node = KeeperNode::new(config).await?;
    let signature = keeper_node.unregister_keeper().await?;
    
    info!("Keeper unregistered successfully! Transaction: {}", signature);
    Ok(())
}

fn generate_config(path: &str) -> Result<(), KeeperError> {
    let example_config = r#"[keeper]
wallet_path = "/path/to/keeper-keypair.json"
stake_amount = 1000000000  # 1 SOL in lamports

[rpc]
primary_url = "https://api.mainnet-beta.solana.com"
fallback_urls = [
    "https://solana-api.projectserum.com",
    "https://rpc.ankr.com/solana"
]

[monitoring]
poll_interval_ms = 1000
max_concurrent_jobs = 10
job_cache_ttl_seconds = 60

[execution]
priority_fee_percentile = 50
max_retries = 3
retry_delay_ms = 2000
max_compute_units = 1400000

[database]
url = "postgresql://user:pass@localhost/solcron"

[logging]
level = "info"
file_path = "keeper.log"

[metrics]
enabled = true
port = 9090
"#;

    std::fs::write(path, example_config)
        .map_err(|e| KeeperError::ConfigError(format!("Failed to write config: {}", e)))?;
    
    println!("Example configuration generated at: {}", path);
    Ok(())
}