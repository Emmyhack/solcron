# SolCron Rust SDK

A comprehensive Rust SDK for integrating with SolCron - the decentralized automation platform for Solana.

## Features

- **🚀 High-Level Client Interface**: Easy-to-use async client for job management
- **🔗 Cross-Program Invocation (CPI)**: Direct program-to-program integration 
- **🛡️ Type Safety**: Comprehensive type system with validation
- **📊 Comprehensive Error Handling**: Detailed error types with recovery information
- **🎯 PDA Management**: Automatic Program Derived Address handling
- **⚡ Performance Optimized**: Minimal overhead with optional features
- **📖 Rich Documentation**: Extensive examples and guides

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
solcron-sdk = { path = "../sdk/rust", features = ["client"] }
```

### Basic Job Registration

```rust
use solcron_sdk::{SolCronClient, types::TriggerType};
use solana_sdk::signature::Keypair;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize client
    let payer = Keypair::new();
    let client = SolCronClient::new_with_payer(
        "https://api.devnet.solana.com",
        payer,
        None,
    ).await?;

    // Register a time-based automation job
    let job_id = client.register_job(
        program_id,                           // Target program
        "harvest".to_string(),               // Target instruction
        TriggerType::TimeBased { interval: 3600 }, // Execute every hour
        200_000,                            // Gas limit
        solcron_sdk::utils::Utils::sol_to_lamports(0.01), // Initial balance
        &owner_keypair,                     // Job owner
    ).await?;

    println!("Job registered with ID: {}", job_id);
    Ok(())
}
```

### Cross-Program Invocation (CPI)

For Solana programs that want to integrate SolCron automation:

```rust
use solcron_sdk::cpi;
use anchor_lang::prelude::*;

#[program]
pub mod my_defi_program {
    use super::*;
    
    pub fn setup_automation(
        ctx: Context<SetupAutomation>,
        interval: u64,
        gas_limit: u64,
        initial_balance: u64,
    ) -> Result<()> {
        // Register automation job via CPI
        let job_id = cpi::register_job(
            ctx.accounts.solcron_program.to_account_info(),
            ctx.accounts.registry.to_account_info(),
            ctx.accounts.job.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            // Job parameters
            ctx.program_id,
            "harvest".to_string(),
            solcron_sdk::types::TriggerType::TimeBased { interval },
            gas_limit,
            initial_balance,
            &[&ctx.accounts.authority.key().as_ref(), &[bump]],
        )?;
        
        // Store job ID for future reference
        ctx.accounts.vault.automation_job_id = Some(job_id);
        
        Ok(())
    }
}
```

## Architecture

The SDK is organized into several modules:

- **`client`**: High-level async client interface (feature-gated)
- **`cpi`**: Cross-program invocation utilities for Solana programs
- **`types`**: Complete type system for SolCron accounts and instructions
- **`accounts`**: PDA derivation and account management utilities
- **`instructions`**: Instruction builders for all SolCron operations
- **`utils`**: Validation, calculations, and helper functions
- **`error`**: Comprehensive error handling system

## Integration Patterns

### 1. Application Integration (TypeScript/Rust Apps)

Use the high-level client interface:

```rust
// Enable client features
solcron-sdk = { version = "0.1.0", features = ["client"] }
```

Perfect for:
- Web applications
- CLI tools  
- Desktop applications
- Backend services

### 2. Program Integration (Solana Smart Contracts)

Use CPI utilities without client dependencies:

```rust
// Minimal dependencies for on-chain programs
solcron-sdk = { version = "0.1.0", default-features = false }
```

Perfect for:
- DeFi protocols
- NFT projects
- Gaming platforms
- Any Solana program needing automation

### 3. Keeper Node Development

Full SDK with all features:

```rust
// All features enabled
solcron-sdk = { version = "0.1.0", features = ["client"] }
```

Perfect for:
- Custom keeper implementations
- Monitoring dashboards
- Performance analysis tools

## Examples

The `examples/` directory contains comprehensive examples:

### [`register_job.rs`](examples/register_job.rs)
Complete job lifecycle management including:
- Job registration with different trigger types
- Balance management and top-ups
- Job monitoring and statistics
- Error handling and recovery

### [`cpi_integration.rs`](examples/cpi_integration.rs)  
Full Anchor program showing:
- DeFi vault with automated harvesting
- CPI-based job registration and management
- Profit optimization and reinvestment
- Comprehensive account management

### [`keeper_node.rs`](examples/keeper_node.rs)
Professional keeper node implementation:
- Multi-job monitoring and execution
- Profitability analysis and optimization
- Statistics tracking and reporting
- Graceful error handling and recovery

### [`analytics_dashboard.rs`](examples/analytics_dashboard.rs)
Advanced monitoring and analytics system:
- Real-time metrics collection and display
- Trend analysis and performance insights  
- Alert management and optimization suggestions
- Comprehensive reporting and data export

Run examples with:

```bash
cargo run --example register_job
cargo run --example keeper_node
cargo run --example analytics_dashboard
```

## Types and Validation

### Trigger Types

```rust
use solcron_sdk::types::TriggerType;

// Time-based execution
let time_trigger = TriggerType::TimeBased { 
    interval: 3600  // Every hour
};

// Condition-based execution
let condition_trigger = TriggerType::Conditional { 
    condition: "price > 100".to_string() 
};

// Event-based execution  
let log_trigger = TriggerType::LogBased { 
    event_signature: "Transfer(address,address,uint256)".to_string() 
};

// Combined triggers
let hybrid_trigger = TriggerType::Hybrid { 
    triggers: vec![time_trigger, condition_trigger] 
};
```

### Account Management

```rust
use solcron_sdk::accounts::Accounts;

// Derive all necessary PDAs
let accounts = Accounts::derive_all(&program_id)?;

// Get specific account addresses
let registry_address = accounts.registry();
let job_address = accounts.job(job_id);
let keeper_address = accounts.keeper(&keeper_pubkey);
```

### Error Handling

```rust
use solcron_sdk::error::{SolCronError, SolCronResult};

match client.execute_job(job_id, &keeper).await {
    Ok(result) => println!("Execution successful: {:?}", result),
    Err(SolCronError::InsufficientBalance { current, required }) => {
        println!("Need to add {} more lamports", required - current);
        // Handle by topping up job balance
    },
    Err(SolCronError::JobNotFound { job_id }) => {
        println!("Job {} not found", job_id);
        // Handle by re-registering job
    },
    Err(e) => println!("Other error: {}", e),
}
```

## Utilities

### Fee Calculations

```rust
use solcron_sdk::utils::Utils;

// Calculate execution fees
let execution_fee = Utils::calculate_execution_fee(
    base_fee,      // Registry base fee
    gas_used,      // Estimated gas usage  
    gas_price,     // Current gas price
);

// Calculate keeper rewards
let keeper_reward = Utils::calculate_keeper_reward(
    execution_fee,     // Total execution fee
    protocol_fee_bps,  // Protocol fee in basis points
);

// Validate job parameters
Utils::validate_job_params(&job)?;
```

### Time Utilities

```rust
use solcron_sdk::utils::TimeUtils;

// Check if time-based job should execute
let should_execute = Utils::should_execute_time_trigger(
    interval,       // Trigger interval in seconds
    last_execution, // Last execution timestamp  
    current_time,   // Current timestamp
);

// Calculate next execution time
let next_time = TimeUtils::next_execution_time(interval, last_execution);

// Format durations
let formatted = Utils::format_duration(uptime_seconds);
```

## Advanced Features

The SDK provides several powerful modules for advanced use cases:

### Batch Operations

Efficiently manage multiple jobs and operations:

```rust
use solcron_sdk::{BatchOperations, BatchJobParams, BatchConfig};

let batch_ops = BatchOperations::new(client.clone(), Some(BatchConfig::default()));

// Register multiple jobs at once
let job_params = vec![
    BatchJobParams { /* job 1 */ },
    BatchJobParams { /* job 2 */ },
    // ... more jobs
];

let result = batch_ops.register_jobs(job_params, &payer).await?;
println!("Registered {}/{} jobs successfully", 
    result.successful.len(), result.total_operations());

// Analyze jobs for optimization
let analysis = batch_ops.analyze_jobs(job_ids).await?;
analysis.print_report();
```

### Real-time Monitoring & Analytics

Monitor system health and performance:

```rust
use solcron_sdk::{Monitor, MonitoringConfig, AlertThresholds};

let monitoring_config = MonitoringConfig {
    collection_interval: 30,
    alert_thresholds: AlertThresholds {
        min_job_balance: Utils::sol_to_lamports(0.001),
        max_failure_rate: 5.0,
        ..Default::default()
    },
    ..Default::default()
};

let mut monitor = Monitor::new(client.clone(), Some(monitoring_config));

// Collect current metrics
let metrics = monitor.collect_metrics().await?;
println!("Active jobs: {}, Success rate: {:.1}%", 
    metrics.registry_stats.active_jobs,
    metrics.job_stats.execution_success_rate);

// Check for alerts
for alert in monitor.get_current_alerts() {
    println!("Alert: {}", alert.message());
}

// Generate analytics reports
let report = monitor.generate_analytics_report(24);
report.print_report();
```

### Load Testing & Simulation

Test system performance and simulate scenarios:

```rust
use solcron_sdk::{Simulator, SimulationConfig, LoadTestConfig};

// Run comprehensive simulation
let simulator = Simulator::new(client.clone(), SimulationConfig {
    duration_seconds: 3600,
    job_count: 100,
    keeper_count: 10,
    failure_rate: 5.0,
    ..Default::default()
});

let results = simulator.run_simulation().await?;
results.print_summary();

// Run targeted load test
let load_test_results = simulator.run_load_test(LoadTestConfig {
    target_tps: 10.0,
    duration_seconds: 600,
    ..Default::default()
}).await?;

load_test_results.print_summary();
```

## Testing

Run the test suite:

```bash
# Unit tests
cargo test

# Integration tests (requires running Solana validator)
cargo test --features integration-tests

# Example tests
cargo test --examples
```

## Feature Flags

Control compilation and dependencies:

- **`client`**: Enables high-level client interface (requires `tokio`)
- **`no-client`**: Disables client features for minimal on-chain usage
- **`serde`**: Enables JSON serialization for types

## Performance

The SDK is designed for minimal overhead:

- **Zero-copy deserialization** where possible
- **Lazy PDA derivation** - computed only when needed
- **Optional async runtime** - only with client features
- **Minimal dependencies** for on-chain usage

## Error Recovery

The SDK provides detailed error information for automated recovery:

```rust
use solcron_sdk::error::SolCronError;

match error {
    // Recoverable errors
    SolCronError::InsufficientBalance { .. } => {
        // Auto-recovery: top up job balance
    },
    SolCronError::JobNotActive { .. } => {
        // Auto-recovery: reactivate job
    },
    SolCronError::KeeperNotActive { .. } => {
        // Auto-recovery: register keeper
    },
    
    // Non-recoverable errors
    SolCronError::InvalidJobId { .. } => {
        // Manual intervention required
    },
    _ => {
        // Handle other cases
    }
}
```

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/new-feature`
3. Make your changes with tests
4. Ensure all tests pass: `cargo test --all-features`
5. Submit a pull request

## Development Setup

```bash
# Clone repository
git clone https://github.com/your-org/solcron
cd solcron/sdk/rust

# Install dependencies
cargo build

# Run tests
cargo test

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy -- -D warnings
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Support

- **Documentation**: [Full API Documentation](https://docs.rs/solcron-sdk)
- **Examples**: See `examples/` directory
- **Issues**: [GitHub Issues](https://github.com/your-org/solcron/issues)
- **Discord**: [SolCron Community](https://discord.gg/solcron)

---

Built with ❤️ for the Solana ecosystem