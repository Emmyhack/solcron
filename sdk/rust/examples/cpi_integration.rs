//! Example: Cross-Program Invocation (CPI) integration
//!
//! This example shows how to integrate SolCron automation into your own Solana program
//! using Cross-Program Invocation. This allows your program to register automation jobs
//! and manage them programmatically.

use anchor_lang::prelude::*;
use solcron_sdk::{
    cpi::{CPI, CPIValidation},
    types::{JobParams, TriggerType},
    accounts::{Accounts, JobRegistrationAccounts},
    utils::Utils,
    REGISTRY_PROGRAM_ID,
};

// This would be your program's ID
declare_id!("YourProgram11111111111111111111111111111111");

#[program]
pub mod my_defi_program {
    use super::*;

    /// Initialize your program and register automation
    pub fn initialize_with_automation(
        ctx: Context<InitializeWithAutomation>,
        harvest_interval: u64,
    ) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        vault.owner = ctx.accounts.owner.key();
        vault.balance = 0;
        vault.last_harvest = 0;
        vault.auto_harvest_enabled = true;

        // Register an automation job via CPI to SolCron
        let job_params = JobParams {
            target_program: crate::ID,
            target_instruction: "harvest_rewards".to_string(),
            trigger_type: TriggerType::TimeBased { 
                interval: harvest_interval 
            },
            trigger_params: Utils::serialize_trigger_params(&serde_json::json!({
                "vault": vault.key(),
                "min_rewards_threshold": 1_000_000 // 0.001 SOL
            })).map_err(|_| ErrorCode::SerializationFailed)?,
            gas_limit: 200_000,
            min_balance: 1_000_000, // 0.001 SOL
        };

        // Use PDA as the job owner so the program can manage it
        let vault_key = vault.key();
        let seeds = &[b"vault", vault_key.as_ref(), &[ctx.bumps.vault_authority]];
        
        CPI::register_job(
            &ctx.accounts.solcron_program.to_account_info(),
            &ctx.accounts.registry_state.to_account_info(),
            &ctx.accounts.automation_job.to_account_info(),
            &ctx.accounts.vault_authority.to_account_info(),
            &ctx.accounts.system_program.to_account_info(),
            job_params,
            50_000_000, // 0.05 SOL initial funding
            Some(&[seeds]),
        ).map_err(|_| ErrorCode::AutomationRegistrationFailed)?;

        vault.automation_job = ctx.accounts.automation_job.key();

        msg!("Vault initialized with automation job: {}", vault.automation_job);
        
        Ok(())
    }

    /// Deposit funds into the vault
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        
        // Transfer SOL to vault
        let cpi_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            anchor_lang::system_program::Transfer {
                from: ctx.accounts.user.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
            },
        );
        anchor_lang::system_program::transfer(cpi_context, amount)?;
        
        vault.balance += amount;
        
        msg!("Deposited {} lamports. New balance: {}", amount, vault.balance);
        
        Ok(())
    }

    /// Harvest rewards (called by SolCron automation)
    pub fn harvest_rewards(ctx: Context<HarvestRewards>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        let current_time = Clock::get()?.unix_timestamp as u64;
        
        // Simulate reward calculation (in practice, this would interact with DeFi protocols)
        let time_since_last_harvest = current_time - vault.last_harvest;
        let rewards = vault.balance * time_since_last_harvest / 100_000_000; // Simple APY calculation
        
        if rewards > 0 {
            vault.balance += rewards;
            vault.last_harvest = current_time;
            
            msg!("Harvested {} lamports in rewards. New balance: {}", rewards, vault.balance);
            
            // Fund the automation job to keep it running
            if rewards > 10_000_000 { // If we earned more than 0.01 SOL
                let funding_amount = rewards / 10; // Fund 10% of rewards back to automation
                
                CPI::fund_job(
                    &ctx.accounts.solcron_program.to_account_info(),
                    &ctx.accounts.automation_job.to_account_info(),
                    &ctx.accounts.vault.to_account_info(),
                    &ctx.accounts.system_program.to_account_info(),
                    funding_amount,
                    None, // Vault is a regular account, not PDA in this context
                ).map_err(|_| ErrorCode::AutomationFundingFailed)?;
                
                msg!("Funded automation job with {} lamports", funding_amount);
            }
        }
        
        Ok(())
    }

    /// Update automation parameters
    pub fn update_automation(
        ctx: Context<UpdateAutomation>,
        new_interval: Option<u64>,
    ) -> Result<()> {
        let vault = &ctx.accounts.vault;
        
        // Validate ownership
        require!(
            ctx.accounts.owner.key() == vault.owner,
            ErrorCode::Unauthorized
        );

        // Update trigger parameters if new interval provided
        let trigger_params = if let Some(interval) = new_interval {
            Some(Utils::serialize_trigger_params(&serde_json::json!({
                "vault": vault.key(),
                "interval": interval,
                "min_rewards_threshold": 1_000_000
            })).map_err(|_| ErrorCode::SerializationFailed)?)
        } else {
            None
        };

        // Update job via CPI
        let vault_key = vault.key();
        let seeds = &[b"vault", vault_key.as_ref(), &[ctx.bumps.vault_authority]];
        
        CPI::update_job(
            &ctx.accounts.solcron_program.to_account_info(),
            &ctx.accounts.automation_job.to_account_info(),
            &ctx.accounts.vault_authority.to_account_info(),
            None, // Keep existing gas limit
            None, // Keep existing min balance
            trigger_params,
            Some(&[seeds]),
        ).map_err(|_| ErrorCode::AutomationUpdateFailed)?;

        msg!("Automation parameters updated");
        
        Ok(())
    }

    /// Disable automation and withdraw remaining job balance
    pub fn disable_automation(ctx: Context<DisableAutomation>) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        
        // Validate ownership
        require!(
            ctx.accounts.owner.key() == vault.owner,
            ErrorCode::Unauthorized
        );

        // Cancel the automation job via CPI
        let vault_key = vault.key();
        let seeds = &[b"vault", vault_key.as_ref(), &[ctx.bumps.vault_authority]];
        
        CPI::cancel_job(
            &ctx.accounts.solcron_program.to_account_info(),
            &ctx.accounts.registry_state.to_account_info(),
            &ctx.accounts.automation_job.to_account_info(),
            &ctx.accounts.vault_authority.to_account_info(),
            &ctx.accounts.system_program.to_account_info(),
            Some(&[seeds]),
        ).map_err(|_| ErrorCode::AutomationCancellationFailed)?;

        vault.auto_harvest_enabled = false;
        vault.automation_job = Pubkey::default();
        
        msg!("Automation disabled and job cancelled");
        
        Ok(())
    }

    /// Withdraw funds from vault
    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let vault = &mut ctx.accounts.vault;
        
        require!(
            ctx.accounts.owner.key() == vault.owner,
            ErrorCode::Unauthorized
        );
        
        require!(vault.balance >= amount, ErrorCode::InsufficientBalance);
        
        // Transfer SOL from vault to user
        **ctx.accounts.vault.to_account_info().try_borrow_mut_lamports()? -= amount;
        **ctx.accounts.owner.to_account_info().try_borrow_mut_lamports()? += amount;
        
        vault.balance -= amount;
        
        msg!("Withdrew {} lamports. Remaining balance: {}", amount, vault.balance);
        
        Ok(())
    }
}

// Account structures
#[account]
pub struct Vault {
    pub owner: Pubkey,
    pub balance: u64,
    pub last_harvest: u64,
    pub auto_harvest_enabled: bool,
    pub automation_job: Pubkey,
}

impl Vault {
    pub const SIZE: usize = 8 + 32 + 8 + 8 + 1 + 32;
}

// Context structures
#[derive(Accounts)]
pub struct InitializeWithAutomation<'info> {
    #[account(
        init,
        payer = owner,
        space = Vault::SIZE,
        seeds = [b"vault", owner.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, Vault>,
    
    /// CHECK: PDA authority for the vault
    #[account(
        seeds = [b"vault_authority", vault.key().as_ref()],
        bump
    )]
    pub vault_authority: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
    
    /// SolCron registry state
    /// CHECK: Validated in CPI call
    pub registry_state: UncheckedAccount<'info>,
    
    /// Automation job account to be created
    /// CHECK: Validated in CPI call
    #[account(mut)]
    pub automation_job: UncheckedAccount<'info>,
    
    /// SolCron registry program
    /// CHECK: Program ID validated in CPI
    pub solcron_program: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub vault: Account<'info, Vault>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct HarvestRewards<'info> {
    #[account(mut)]
    pub vault: Account<'info, Vault>,
    
    /// Automation job account
    /// CHECK: Validated against vault.automation_job
    #[account(
        mut,
        constraint = automation_job.key() == vault.automation_job
    )]
    pub automation_job: UncheckedAccount<'info>,
    
    /// SolCron registry program
    /// CHECK: Program ID validated in CPI
    pub solcron_program: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateAutomation<'info> {
    #[account(mut)]
    pub vault: Account<'info, Vault>,
    
    /// CHECK: PDA authority for the vault
    #[account(
        seeds = [b"vault_authority", vault.key().as_ref()],
        bump
    )]
    pub vault_authority: UncheckedAccount<'info>,
    
    pub owner: Signer<'info>,
    
    /// Automation job account
    /// CHECK: Validated against vault.automation_job
    #[account(
        mut,
        constraint = automation_job.key() == vault.automation_job
    )]
    pub automation_job: UncheckedAccount<'info>,
    
    /// SolCron registry program
    /// CHECK: Program ID validated in CPI
    pub solcron_program: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct DisableAutomation<'info> {
    #[account(mut)]
    pub vault: Account<'info, Vault>,
    
    /// CHECK: PDA authority for the vault
    #[account(
        seeds = [b"vault_authority", vault.key().as_ref()],
        bump
    )]
    pub vault_authority: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
    
    /// SolCron registry state
    /// CHECK: Validated in CPI call
    pub registry_state: UncheckedAccount<'info>,
    
    /// Automation job account
    /// CHECK: Validated against vault.automation_job
    #[account(
        mut,
        constraint = automation_job.key() == vault.automation_job
    )]
    pub automation_job: UncheckedAccount<'info>,
    
    /// SolCron registry program
    /// CHECK: Program ID validated in CPI
    pub solcron_program: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub vault: Account<'info, Vault>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
}

// Error codes
#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized access")]
    Unauthorized,
    
    #[msg("Insufficient balance")]
    InsufficientBalance,
    
    #[msg("Serialization failed")]
    SerializationFailed,
    
    #[msg("Automation registration failed")]
    AutomationRegistrationFailed,
    
    #[msg("Automation funding failed")]
    AutomationFundingFailed,
    
    #[msg("Automation update failed")]
    AutomationUpdateFailed,
    
    #[msg("Automation cancellation failed")]
    AutomationCancellationFailed,
}

// Example client code to interact with this program
#[cfg(feature = "client")]
mod client_example {
    use super::*;
    use anchor_client::prelude::*;
    
    pub async fn example_usage() -> Result<()> {
        let program_id = crate::ID;
        let cluster = Cluster::Devnet;
        let payer = anchor_client::solana_sdk::signature::read_keypair_file("~/.config/solana/id.json")?;
        
        let client = Client::new(cluster, std::rc::Rc::new(payer));
        let program = client.program(program_id)?;

        // Initialize vault with automation
        let owner = anchor_client::solana_sdk::signature::Keypair::new();
        let harvest_interval = 3600; // 1 hour
        
        // Derive accounts
        let (vault, _vault_bump) = Pubkey::find_program_address(
            &[b"vault", owner.pubkey().as_ref()],
            &program_id,
        );
        
        let (vault_authority, _authority_bump) = Pubkey::find_program_address(
            &[b"vault_authority", vault.as_ref()],
            &program_id,
        );
        
        // Get SolCron accounts
        let (registry_state, _) = solcron_sdk::accounts::Accounts::registry_state()?;
        let registry = program.rpc().get_account_data(&registry_state)?;
        let next_job_id = u64::from_le_bytes(registry[64..72].try_into().unwrap()); // Extract next_job_id
        let (automation_job, _) = solcron_sdk::accounts::Accounts::automation_job(next_job_id)?;
        
        let signature = program
            .request()
            .accounts(InitializeWithAutomation {
                vault,
                vault_authority,
                owner: owner.pubkey(),
                registry_state,
                automation_job,
                solcron_program: REGISTRY_PROGRAM_ID,
                system_program: anchor_client::solana_sdk::system_program::ID,
            })
            .args(crate::instruction::InitializeWithAutomation {
                harvest_interval,
            })
            .signer(&owner)
            .send()?;
            
        println!("Vault initialized with automation: {}", signature);
        
        Ok(())
    }
}