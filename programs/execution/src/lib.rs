use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::AccountMeta as SolanaAccountMeta;

declare_id!("ExecNqpXiPPjs7m5wbuTCxZE8PJzgdW2cWEw23kcKJKm");

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AccountMeta {
    pub pubkey: Pubkey,
    pub is_signer: bool,
    pub is_writable: bool,
}

impl From<AccountMeta> for SolanaAccountMeta {
    fn from(meta: AccountMeta) -> Self {
        if meta.is_signer {
            if meta.is_writable {
                SolanaAccountMeta::new(meta.pubkey, true)
            } else {
                SolanaAccountMeta::new_readonly(meta.pubkey, true)
            }
        } else {
            if meta.is_writable {
                SolanaAccountMeta::new(meta.pubkey, false)
            } else {
                SolanaAccountMeta::new_readonly(meta.pubkey, false)
            }
        }
    }
}

#[program]
pub mod solcron_execution {
    use super::*;

    /// Execute a cross-program invocation with proper validation and error handling
    pub fn execute_cpi_call(
        ctx: Context<ExecuteCpiCall>,
        program_id: Pubkey,
        data: Vec<u8>,
        accounts: Vec<AccountMeta>,
    ) -> Result<()> {
        // Validate instruction data size
        require!(data.len() <= 1024, ExecutionError::InstructionDataTooLarge);
        require!(!data.is_empty(), ExecutionError::EmptyInstructionData);
        
        // Validate accounts
        require!(!accounts.is_empty(), ExecutionError::NoAccountsProvided);
        require!(accounts.len() <= 32, ExecutionError::TooManyAccounts);

        // Convert custom AccountMeta to Solana AccountMeta
        let solana_accounts: Vec<SolanaAccountMeta> = accounts.into_iter().map(|meta| meta.into()).collect();
        
        // Build the instruction
        let instruction = anchor_lang::solana_program::instruction::Instruction {
            program_id,
            accounts: solana_accounts,
            data,
        };

        // Prepare account infos for CPI
        let mut account_infos = Vec::new();
        account_infos.push(ctx.accounts.target_program.to_account_info());
        
        // Add remaining accounts
        for account in ctx.remaining_accounts {
            account_infos.push(account.to_account_info());
        }

        // Execute the CPI call with error handling
        match anchor_lang::solana_program::program::invoke(&instruction, &account_infos) {
            Ok(()) => {
                emit!(CpiExecutionSuccess {
                    target_program: ctx.accounts.target_program.key(),
                    executor: ctx.accounts.execution_authority.key(),
                });
                
                msg!("CPI call executed successfully to program: {}", ctx.accounts.target_program.key());
                Ok(())
            }
            Err(err) => {
                emit!(CpiExecutionFailed {
                    target_program: ctx.accounts.target_program.key(),
                    executor: ctx.accounts.execution_authority.key(),
                    error_code: 1, // Generic error code
                });
                
                msg!("CPI call failed to program: {}, error: {:?}", 
                     ctx.accounts.target_program.key(), err);
                Err(ExecutionError::CpiCallFailed.into())
            }
        }
    }

    /// Execute a CPI call with signed accounts (for PDA authorities)
    pub fn execute_cpi_call_with_seeds(
        ctx: Context<ExecuteCpiCallWithSeeds>,
        program_id: Pubkey,
        data: Vec<u8>,
        accounts: Vec<AccountMeta>,
        seeds: Vec<Vec<u8>>,
    ) -> Result<()> {
        // Validate inputs
        require!(data.len() <= 1024, ExecutionError::InstructionDataTooLarge);
        require!(!data.is_empty(), ExecutionError::EmptyInstructionData);
        require!(!accounts.is_empty(), ExecutionError::NoAccountsProvided);
        require!(accounts.len() <= 32, ExecutionError::TooManyAccounts);

        // Convert custom AccountMeta to Solana AccountMeta
        let solana_accounts: Vec<SolanaAccountMeta> = accounts.into_iter().map(|meta| meta.into()).collect();
        
        // Build the instruction
        let instruction = anchor_lang::solana_program::instruction::Instruction {
            program_id,
            accounts: solana_accounts,
            data,
        };

        // Prepare account infos and signer seeds
        let mut account_infos = Vec::new();
        account_infos.push(ctx.accounts.target_program.to_account_info());
        
        for account in ctx.remaining_accounts {
            account_infos.push(account.to_account_info());
        }

        // Convert seeds to the correct format for invoke_signed
        let seeds_refs: Vec<&[u8]> = seeds.iter().map(|seed| seed.as_slice()).collect();
        let signer_seeds = &[seeds_refs.as_slice()];

        // Execute signed CPI call
        match anchor_lang::solana_program::program::invoke_signed(
            &instruction,
            &account_infos,
            signer_seeds,
        ) {
            Ok(()) => {
                emit!(SignedCpiExecutionSuccess {
                    target_program: ctx.accounts.target_program.key(),
                    authority: ctx.accounts.execution_authority.key(),
                });
                
                msg!("Signed CPI call executed successfully to program: {}", 
                     ctx.accounts.target_program.key());
                Ok(())
            }
            Err(err) => {
                emit!(SignedCpiExecutionFailed {
                    target_program: ctx.accounts.target_program.key(),
                    authority: ctx.accounts.execution_authority.key(),
                    error_code: 1, // Generic error code
                });
                
                msg!("Signed CPI call failed to program: {}, error: {:?}", 
                     ctx.accounts.target_program.key(), err);
                Err(ExecutionError::CpiCallFailed.into())
            }
        }
    }

    /// Validate that a program can be called (security check)
    pub fn validate_target_program(ctx: Context<ValidateTargetProgram>) -> Result<bool> {
        let program_key = ctx.accounts.target_program.key();
        
        // Basic validation - check if program account exists and is executable
        require!(ctx.accounts.target_program.executable, ExecutionError::ProgramNotExecutable);
        
        // Additional security checks could be added here:
        // - Whitelist/blacklist of allowed programs
        // - Program ownership verification
        // - Program upgrade authority checks
        
        msg!("Program {} validated for execution", program_key);
        Ok(true)
    }
}

#[derive(Accounts)]
pub struct ExecuteCpiCall<'info> {
    pub execution_authority: Signer<'info>,
    /// CHECK: Target program to invoke
    pub target_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct ExecuteCpiCallWithSeeds<'info> {
    pub execution_authority: Signer<'info>,
    /// CHECK: Target program to invoke
    pub target_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct ValidateTargetProgram<'info> {
    /// CHECK: Program to validate
    pub target_program: AccountInfo<'info>,
}

// Events
#[event]
pub struct CpiExecutionSuccess {
    pub target_program: Pubkey,
    pub executor: Pubkey,
}

#[event]
pub struct CpiExecutionFailed {
    pub target_program: Pubkey,
    pub executor: Pubkey,
    pub error_code: u32,
}

#[event]
pub struct SignedCpiExecutionSuccess {
    pub target_program: Pubkey,
    pub authority: Pubkey,
}

#[event]
pub struct SignedCpiExecutionFailed {
    pub target_program: Pubkey,
    pub authority: Pubkey,
    pub error_code: u32,
}

// Errors
#[error_code]
pub enum ExecutionError {
    #[msg("Instruction data is too large (max 1024 bytes)")]
    InstructionDataTooLarge,
    
    #[msg("Instruction data cannot be empty")]
    EmptyInstructionData,
    
    #[msg("No accounts provided for CPI call")]
    NoAccountsProvided,
    
    #[msg("Too many accounts provided (max 32)")]
    TooManyAccounts,
    
    #[msg("CPI call to target program failed")]
    CpiCallFailed,
    
    #[msg("Target program is not executable")]
    ProgramNotExecutable,
    
    #[msg("Unauthorized execution attempt")]
    UnauthorizedExecution,
    
    #[msg("Invalid signer seeds provided")]
    InvalidSignerSeeds,
}