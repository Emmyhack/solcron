//! SolCron Rust SDK
//!
//! A comprehensive Rust SDK for integrating with SolCron - the decentralized automation platform for Solana.
//!
//! ## Features
//!
//! - **High-Level Client Interface**: Easy-to-use async client for job management
//! - **Cross-Program Invocation (CPI)**: Direct program-to-program integration
//! - **Type Safety**: Comprehensive type system with validation
//! - **Error Handling**: Detailed error types with recovery information
//! - **PDA Management**: Automatic Program Derived Address handling
//! - **Batch Operations**: Efficient mass operations with automatic batching
//! - **Monitoring & Analytics**: Real-time metrics and performance analysis
//! - **Simulation Framework**: Load testing and development simulation tools
//! - **Performance**: Minimal overhead with optional features
//!
//! ## Quick Start
//!
//! Add to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! solcron-sdk = { path = "../sdk/rust", features = ["client"] }
//! ```
//!
//! ### Basic Usage
//!
//! ```rust,no_run
//! use solcron_sdk::{SolCronClient, types::TriggerType};
//! use solana_sdk::signature::Keypair;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let payer = Keypair::new();
//!     let client = SolCronClient::new_with_payer(
//!         "https://api.devnet.solana.com",
//!         payer,
//!         None,
//!     ).await?;
//!
//!     let job_id = client.register_job(
//!         program_id,
//!         "harvest".to_string(),
//!         TriggerType::TimeBased { interval: 3600 },
//!         200_000,
//!         solcron_sdk::utils::Utils::sol_to_lamports(0.01),
//!         &owner_keypair,
//!     ).await?;
//!
//!     println!("Job registered: {}", job_id);
//!     Ok(())
//! }
//! ```

use anchor_lang::prelude::*;

// Program IDs
pub const SOLCRON_REGISTRY_ID: Pubkey = Pubkey::new_from_array([
    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
    0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
    0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
    0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20,
]);

pub const SOLCRON_EXECUTION_ID: Pubkey = Pubkey::new_from_array([
    0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28,
    0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f, 0x30,
    0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38,
    0x39, 0x3a, 0x3b, 0x3c, 0x3d, 0x3e, 0x3f, 0x40,
]);

// Core modules
pub mod types;
pub mod error;
pub mod accounts;
pub mod instructions;
pub mod cpi;
pub mod utils;

// Advanced modules
pub mod batch;
pub mod monitoring;
pub mod simulation;

// Feature-gated modules
#[cfg(feature = "client")]
pub mod client;

// Re-exports for convenience
pub use types::*;
pub use error::{SolCronError, SolCronResult};
pub use accounts::Accounts;
pub use instructions::Instructions;
pub use cpi::CPI;
pub use utils::{Utils, TimeUtils};

// Advanced functionality
pub use batch::{BatchOperations, BatchConfig, BatchResult, BatchAnalysisReport};
pub use monitoring::{Monitor, MonitoringConfig, SystemMetrics, AlertType};
pub use simulation::{Simulator, SimulationConfig, SimulationResults, LoadTestConfig};

#[cfg(feature = "client")]
pub use client::SolCronClient;
//!
//! ## CPI Integration Example
//!
//! ```rust
//! use solcron_sdk::cpi;
//! use anchor_lang::prelude::*;
//! 
//! #[program]
//! pub mod my_program {
//!     use super::*;
//!     
//!     pub fn register_automation(
//!         ctx: Context<RegisterAutomation>,
//!         interval: u64,
//!         gas_limit: u64,
//!     ) -> Result<()> {
//!         // Register a SolCron job via CPI
//!         cpi::register_job(
//!             ctx.accounts.solcron_program.to_account_info(),
//!             ctx.accounts.registry_state.to_account_info(),
//!             ctx.accounts.automation_job.to_account_info(),
//!             ctx.accounts.owner.to_account_info(),
//!             cpi::JobParams {
//!                 target_program: crate::ID,
//!                 target_instruction: "harvest_rewards".to_string(),
//!                 trigger_type: cpi::TriggerType::TimeBased { interval },
//!                 gas_limit,
//!                 min_balance: 1_000_000,
//!             },
//!             50_000_000, // Initial funding
//!         )?;
//!         
//!         Ok(())
//!     }
//! }
//! ```

pub mod client;
pub mod types;
pub mod accounts;
pub mod instructions;
pub mod cpi;
pub mod utils;
pub mod error;

// Re-export commonly used types and functions
pub use client::*;
pub use types::*;
pub use accounts::*;
pub use instructions::*;
pub use error::*;

// Program IDs (these should match your deployed programs)
use solana_program::declare_id;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

/// The SolCron Registry Program ID
pub const REGISTRY_PROGRAM_ID: solana_program::pubkey::Pubkey = crate::ID;

/// The SolCron Execution Engine Program ID  
pub const EXECUTION_PROGRAM_ID: solana_program::pubkey::Pubkey = 
    solana_program::pubkey!("ExecNqpXiPPjs7m5wbuTCxZE8PJzgdW2cWEw23kcKJKm");

/// Current SDK version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_program_ids() {
        // Ensure program IDs are valid
        assert_ne!(REGISTRY_PROGRAM_ID.to_string(), "11111111111111111111111111111111");
        assert_ne!(EXECUTION_PROGRAM_ID.to_string(), "11111111111111111111111111111111");
    }
}