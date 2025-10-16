//! Example: Register and manage an automation job
//! 
//! This example demonstrates how to register a new automation job,
//! fund it, and manage its lifecycle using the SolCron Rust SDK.

use solcron_sdk::{
    SolCronClient, JobParams, TriggerType, Utils,
    error::SolCronResult,
};
use solana_sdk::{
    signature::{Keypair, Signer},
    pubkey::Pubkey,
};

#[tokio::main]
async fn main() -> SolCronResult<()> {
    // Initialize logging
    env_logger::init();

    println!("🚀 SolCron Job Registration Example");
    println!("===================================");

    // 1. Create client connection
    println!("📡 Connecting to Solana cluster...");
    let cluster_url = "https://api.devnet.solana.com"; // Use devnet for testing
    let owner_keypair = Keypair::new(); // In practice, load from file
    
    let client = SolCronClient::new_with_payer(
        cluster_url,
        owner_keypair.insecure_clone(),
        None, // Use default commitment
    ).await?;

    println!("✅ Connected successfully!");
    println!("🔑 Owner pubkey: {}", owner_keypair.pubkey());

    // 2. Check registry state
    println!("\n📋 Checking registry state...");
    match client.get_registry_state().await {
        Ok(registry) => {
            println!("✅ Registry found:");
            println!("   - Admin: {}", registry.admin);
            println!("   - Total jobs: {}", registry.total_jobs);
            println!("   - Active jobs: {}", registry.active_jobs);
            println!("   - Base fee: {} lamports", registry.base_fee);
            println!("   - Min stake: {} SOL", Utils::lamports_to_sol_string(registry.min_stake, 3));
        }
        Err(e) => {
            println!("❌ Failed to get registry state: {}", e);
            println!("💡 Make sure the SolCron registry is deployed on this cluster");
            return Err(e);
        }
    }

    // 3. Define job parameters
    println!("\n⚙️  Defining job parameters...");
    let target_program = Pubkey::new_unique(); // Replace with your actual program ID
    
    let job_params = JobParams {
        target_program,
        target_instruction: "harvest_rewards".to_string(),
        trigger_type: TriggerType::TimeBased { 
            interval: 3600 // Execute every hour
        },
        trigger_params: Utils::serialize_trigger_params(&serde_json::json!({
            "description": "Harvest DeFi rewards every hour",
            "max_retries": 3
        }))?,
        gas_limit: 200_000,
        min_balance: Utils::sol_to_lamports(0.001), // 0.001 SOL minimum
    };

    // Validate parameters
    Utils::validate_job_params(&job_params)?;
    println!("✅ Job parameters validated");
    println!("   - Target: {}", job_params.target_program);
    println!("   - Instruction: {}", job_params.target_instruction);
    println!("   - Trigger: Every {} seconds", 
        if let TriggerType::TimeBased { interval } = &job_params.trigger_type {
            *interval
        } else {
            0
        }
    );
    println!("   - Gas limit: {}", job_params.gas_limit);
    println!("   - Min balance: {} SOL", Utils::lamports_to_sol_string(job_params.min_balance, 6));

    // 4. Register the job
    println!("\n📝 Registering automation job...");
    let initial_funding = Utils::sol_to_lamports(0.1); // Fund with 0.1 SOL
    
    // Note: In practice, you'd need to fund the owner account first
    println!("💰 Initial funding: {} SOL", Utils::lamports_to_sol_string(initial_funding, 3));
    
    match client.register_job(&job_params, initial_funding, &owner_keypair).await {
        Ok(job_id) => {
            println!("✅ Job registered successfully!");
            println!("🆔 Job ID: {}", job_id);
            
            // 5. Get job information
            println!("\n🔍 Fetching job details...");
            let job = client.get_job(job_id).await?;
            println!("✅ Job details:");
            println!("   - ID: {}", job.job_id);
            println!("   - Owner: {}", job.owner);
            println!("   - Active: {}", job.is_active);
            println!("   - Balance: {} SOL", Utils::lamports_to_sol_string(job.balance, 6));
            println!("   - Executions: {}", job.execution_count);
            println!("   - Created: {}", chrono::DateTime::from_timestamp(job.created_at as i64, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                .unwrap_or_else(|| "Unknown".to_string())
            );

            // 6. Fund the job with additional SOL
            println!("\n💰 Adding additional funding...");
            let additional_funding = Utils::sol_to_lamports(0.05); // Add 0.05 SOL
            let fund_signature = client.fund_job(job_id, additional_funding, &owner_keypair).await?;
            println!("✅ Job funded! Signature: {}", fund_signature);

            // 7. Get updated job information
            let updated_job = client.get_job(job_id).await?;
            println!("💰 Updated balance: {} SOL", 
                Utils::lamports_to_sol_string(updated_job.balance, 6)
            );

            // 8. Update job parameters
            println!("\n⚙️  Updating job parameters...");
            let new_gas_limit = Some(250_000); // Increase gas limit
            let new_min_balance = Some(Utils::sol_to_lamports(0.002)); // Increase min balance
            
            let update_signature = client.update_job(
                job_id,
                new_gas_limit,
                new_min_balance,
                None, // Keep existing trigger params
                &owner_keypair,
            ).await?;
            println!("✅ Job updated! Signature: {}", update_signature);

            // 9. Get job statistics
            println!("\n📊 Job statistics:");
            let job_stats = client.get_job_stats(job_id).await?;
            println!("   - Total executions: {}", job_stats.total_executions);
            println!("   - Success rate: {:.1}%", job_stats.success_rate * 100.0);
            println!("   - Current balance: {} SOL", 
                Utils::lamports_to_sol_string(job_stats.current_balance, 6)
            );
            
            if job_stats.last_execution > 0 {
                println!("   - Last execution: {}", 
                    chrono::DateTime::from_timestamp(job_stats.last_execution as i64, 0)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                        .unwrap_or_else(|| "Unknown".to_string())
                );
            } else {
                println!("   - Last execution: Never");
            }

            // 10. Demonstrate cancellation (commented out to preserve the job)
            println!("\n⚠️  Job management options:");
            println!("   - To cancel this job, uncomment the cancellation code");
            println!("   - To execute this job, run the keeper example");
            println!("   - Job will remain active until cancelled or funds depleted");

            /*
            // Uncomment to cancel the job
            println!("\n❌ Cancelling job...");
            let cancel_signature = client.cancel_job(job_id, &owner_keypair).await?;
            println!("✅ Job cancelled! Signature: {}", cancel_signature);
            println!("💰 Remaining balance refunded to owner");
            */

            println!("\n🎉 Example completed successfully!");
        }
        Err(e) => {
            println!("❌ Failed to register job: {}", e);
            
            // Provide helpful debugging information
            match &e {
                solcron_sdk::SolCronError::InsufficientBalance { required, available } => {
                    println!("💡 The owner account needs at least {} lamports", required);
                    println!("   Current balance: {} lamports", available);
                    println!("   Consider funding the account with: solana airdrop 1 {}", owner_keypair.pubkey());
                }
                solcron_sdk::SolCronError::ValidationError { field, reason } => {
                    println!("💡 Validation failed for field '{}': {}", field, reason);
                }
                _ => {
                    println!("💡 Check that the SolCron program is deployed and accessible");
                }
            }
            
            return Err(e);
        }
    }

    Ok(())
}

/// Helper function to demonstrate different trigger types
#[allow(dead_code)]
fn demonstrate_trigger_types() -> SolCronResult<Vec<JobParams>> {
    let target_program = Pubkey::new_unique();
    
    let jobs = vec![
        // Time-based trigger - execute every 30 minutes
        JobParams {
            target_program,
            target_instruction: "rebalance_portfolio".to_string(),
            trigger_type: TriggerType::TimeBased { interval: 1800 },
            trigger_params: Utils::serialize_trigger_params(&serde_json::json!({
                "description": "Rebalance investment portfolio every 30 minutes"
            }))?,
            gas_limit: 300_000,
            min_balance: Utils::sol_to_lamports(0.005),
        },
        
        // Conditional trigger - execute when price changes
        JobParams {
            target_program,
            target_instruction: "liquidate_position".to_string(),
            trigger_type: TriggerType::Conditional { 
                logic: "health_factor < 1.2".as_bytes().to_vec()
            },
            trigger_params: Utils::serialize_trigger_params(&serde_json::json!({
                "description": "Liquidate when health factor drops below 1.2",
                "account_to_monitor": "price_oracle_account",
                "threshold": 1.2
            }))?,
            gas_limit: 500_000,
            min_balance: Utils::sol_to_lamports(0.01),
        },
        
        // Log-based trigger - execute on specific events
        JobParams {
            target_program,
            target_instruction: "compound_rewards".to_string(),
            trigger_type: TriggerType::LogBased { 
                program_id: target_program,
                event_filter: "RewardsAccrued".to_string(),
            },
            trigger_params: Utils::serialize_trigger_params(&serde_json::json!({
                "description": "Compound rewards when RewardsAccrued event is emitted",
                "min_reward_threshold": 1000000
            }))?,
            gas_limit: 250_000,
            min_balance: Utils::sol_to_lamports(0.003),
        },
    ];

    Ok(jobs)
}