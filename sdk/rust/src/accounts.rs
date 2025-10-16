use solana_program::pubkey::Pubkey;
use crate::{REGISTRY_PROGRAM_ID, error::SolCronResult, types::*};

/// Account address derivation utilities for SolCron PDAs
pub struct Accounts;

impl Accounts {
    /// Derive the registry state account PDA
    /// 
    /// Returns the PDA and bump seed for the main registry state account
    pub fn registry_state() -> SolCronResult<(Pubkey, u8)> {
        Pubkey::try_find_program_address(
            &[b"registry"],
            &REGISTRY_PROGRAM_ID,
        ).ok_or_else(|| crate::error::SolCronError::PDADerivationError {
            reason: "Failed to derive registry state PDA".to_string(),
        })
    }

    /// Derive an automation job account PDA
    /// 
    /// # Arguments
    /// * `job_id` - The unique job identifier
    /// 
    /// Returns the PDA and bump seed for the job account
    pub fn automation_job(job_id: u64) -> SolCronResult<(Pubkey, u8)> {
        let job_id_bytes = job_id.to_le_bytes();
        Pubkey::try_find_program_address(
            &[b"job", &job_id_bytes],
            &REGISTRY_PROGRAM_ID,
        ).ok_or_else(|| crate::error::SolCronError::PDADerivationError {
            reason: format!("Failed to derive job PDA for job_id: {}", job_id),
        })
    }

    /// Derive a keeper account PDA
    /// 
    /// # Arguments
    /// * `keeper_address` - The keeper's public key
    /// 
    /// Returns the PDA and bump seed for the keeper account
    pub fn keeper(keeper_address: &Pubkey) -> SolCronResult<(Pubkey, u8)> {
        Pubkey::try_find_program_address(
            &[b"keeper", keeper_address.as_ref()],
            &REGISTRY_PROGRAM_ID,
        ).ok_or_else(|| crate::error::SolCronError::PDADerivationError {
            reason: format!("Failed to derive keeper PDA for address: {}", keeper_address),
        })
    }

    /// Derive an execution record account PDA
    /// 
    /// # Arguments
    /// * `job_id` - The job identifier
    /// * `execution_count` - The execution sequence number
    /// 
    /// Returns the PDA and bump seed for the execution record
    pub fn execution_record(job_id: u64, execution_count: u64) -> SolCronResult<(Pubkey, u8)> {
        let job_id_bytes = job_id.to_le_bytes();
        let execution_count_bytes = execution_count.to_le_bytes();
        
        Pubkey::try_find_program_address(
            &[b"execution", &job_id_bytes, &execution_count_bytes],
            &REGISTRY_PROGRAM_ID,
        ).ok_or_else(|| crate::error::SolCronError::PDADerivationError {
            reason: format!(
                "Failed to derive execution record PDA for job_id: {}, execution: {}", 
                job_id, execution_count
            ),
        })
    }

    /// Derive the execution authority PDA for the execution engine
    pub fn execution_authority() -> SolCronResult<(Pubkey, u8)> {
        Pubkey::try_find_program_address(
            &[b"execution_authority"],
            &crate::EXECUTION_PROGRAM_ID,
        ).ok_or_else(|| crate::error::SolCronError::PDADerivationError {
            reason: "Failed to derive execution authority PDA".to_string(),
        })
    }

    /// Get all accounts required for job registration
    /// 
    /// # Arguments
    /// * `owner` - The job owner's public key
    /// * `job_id` - The job identifier (from registry state)
    /// 
    /// Returns a struct with all required account addresses
    pub fn job_registration_accounts(
        owner: &Pubkey,
        job_id: u64,
    ) -> SolCronResult<JobRegistrationAccounts> {
        let (registry_state, _) = Self::registry_state()?;
        let (automation_job, _) = Self::automation_job(job_id)?;

        Ok(JobRegistrationAccounts {
            registry_state,
            automation_job,
            owner: *owner,
            system_program: solana_program::system_program::ID,
        })
    }

    /// Get all accounts required for keeper registration
    /// 
    /// # Arguments
    /// * `keeper_address` - The keeper's public key
    /// 
    /// Returns a struct with all required account addresses
    pub fn keeper_registration_accounts(
        keeper_address: &Pubkey,
    ) -> SolCronResult<KeeperRegistrationAccounts> {
        let (registry_state, _) = Self::registry_state()?;
        let (keeper, _) = Self::keeper(keeper_address)?;

        Ok(KeeperRegistrationAccounts {
            registry_state,
            keeper,
            keeper_account: *keeper_address,
            system_program: solana_program::system_program::ID,
        })
    }

    /// Get all accounts required for job execution
    /// 
    /// # Arguments
    /// * `job_id` - The job identifier
    /// * `keeper_address` - The executing keeper's public key
    /// * `target_program` - The target program to execute
    /// 
    /// Returns a struct with all required account addresses
    pub fn job_execution_accounts(
        job_id: u64,
        keeper_address: &Pubkey,
        target_program: &Pubkey,
        execution_count: u64,
    ) -> SolCronResult<JobExecutionAccounts> {
        let (registry_state, _) = Self::registry_state()?;
        let (automation_job, _) = Self::automation_job(job_id)?;
        let (keeper, _) = Self::keeper(keeper_address)?;
        let (execution_record, _) = Self::execution_record(job_id, execution_count)?;

        Ok(JobExecutionAccounts {
            registry_state,
            automation_job,
            keeper,
            execution_record,
            keeper_account: *keeper_address,
            target_program: *target_program,
            system_program: solana_program::system_program::ID,
        })
    }

    /// Validate that a PDA was derived correctly
    /// 
    /// # Arguments
    /// * `address` - The PDA to validate
    /// * `seeds` - The seeds used for derivation
    /// * `program_id` - The program that owns the PDA
    /// 
    /// Returns true if the PDA is valid
    pub fn validate_pda(
        address: &Pubkey,
        seeds: &[&[u8]],
        program_id: &Pubkey,
    ) -> bool {
        if let Ok((derived_address, _)) = Pubkey::try_find_program_address(seeds, program_id) {
            *address == derived_address
        } else {
            false
        }
    }
}

/// Account addresses required for job registration
#[derive(Debug, Clone)]
pub struct JobRegistrationAccounts {
    pub registry_state: Pubkey,
    pub automation_job: Pubkey,
    pub owner: Pubkey,
    pub system_program: Pubkey,
}

/// Account addresses required for keeper registration
#[derive(Debug, Clone)]
pub struct KeeperRegistrationAccounts {
    pub registry_state: Pubkey,
    pub keeper: Pubkey,
    pub keeper_account: Pubkey,
    pub system_program: Pubkey,
}

/// Account addresses required for job execution
#[derive(Debug, Clone)]
pub struct JobExecutionAccounts {
    pub registry_state: Pubkey,
    pub automation_job: Pubkey,
    pub keeper: Pubkey,
    pub execution_record: Pubkey,
    pub keeper_account: Pubkey,
    pub target_program: Pubkey,
    pub system_program: Pubkey,
}

/// Account addresses required for reward claiming
#[derive(Debug, Clone)]
pub struct RewardClaimAccounts {
    pub keeper: Pubkey,
    pub keeper_account: Pubkey,
    pub system_program: Pubkey,
}

impl RewardClaimAccounts {
    /// Create reward claim accounts for a keeper
    pub fn new(keeper_address: &Pubkey) -> SolCronResult<Self> {
        let (keeper, _) = Accounts::keeper(keeper_address)?;
        
        Ok(Self {
            keeper,
            keeper_account: *keeper_address,
            system_program: solana_program::system_program::ID,
        })
    }
}

/// Account addresses required for admin operations
#[derive(Debug, Clone)]
pub struct AdminAccounts {
    pub registry_state: Pubkey,
    pub admin: Pubkey,
}

impl AdminAccounts {
    /// Create admin accounts
    pub fn new(admin: &Pubkey) -> SolCronResult<Self> {
        let (registry_state, _) = Accounts::registry_state()?;
        
        Ok(Self {
            registry_state,
            admin: *admin,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_program::pubkey::Pubkey;

    #[test]
    fn test_registry_state_derivation() {
        let result = Accounts::registry_state();
        assert!(result.is_ok());
        
        let (pda, bump) = result.unwrap();
        assert_ne!(pda, Pubkey::default());
        assert!(bump <= 255);
    }

    #[test]
    fn test_job_pda_derivation() {
        let job_id = 123;
        let result = Accounts::automation_job(job_id);
        assert!(result.is_ok());
        
        let (pda, bump) = result.unwrap();
        assert_ne!(pda, Pubkey::default());
        assert!(bump <= 255);
        
        // Derive again and ensure consistency
        let result2 = Accounts::automation_job(job_id);
        assert_eq!(result.unwrap().0, result2.unwrap().0);
    }

    #[test]
    fn test_keeper_pda_derivation() {
        let keeper_address = Pubkey::new_unique();
        let result = Accounts::keeper(&keeper_address);
        assert!(result.is_ok());
        
        let (pda, bump) = result.unwrap();
        assert_ne!(pda, Pubkey::default());
        assert_ne!(pda, keeper_address);
        assert!(bump <= 255);
    }

    #[test]
    fn test_execution_record_derivation() {
        let job_id = 456;
        let execution_count = 10;
        let result = Accounts::execution_record(job_id, execution_count);
        assert!(result.is_ok());
        
        let (pda, bump) = result.unwrap();
        assert_ne!(pda, Pubkey::default());
        assert!(bump <= 255);
    }

    #[test]
    fn test_pda_validation() {
        let job_id = 789;
        let (pda, _) = Accounts::automation_job(job_id).unwrap();
        
        let job_id_bytes = job_id.to_le_bytes();
        let seeds = &[b"job", &job_id_bytes];
        
        let is_valid = Accounts::validate_pda(&pda, seeds, &REGISTRY_PROGRAM_ID);
        assert!(is_valid);
        
        // Test with wrong seeds
        let wrong_seeds = &[b"wrong", &job_id_bytes];
        let is_invalid = Accounts::validate_pda(&pda, wrong_seeds, &REGISTRY_PROGRAM_ID);
        assert!(!is_invalid);
    }

    #[test]
    fn test_job_registration_accounts() {
        let owner = Pubkey::new_unique();
        let job_id = 999;
        
        let result = Accounts::job_registration_accounts(&owner, job_id);
        assert!(result.is_ok());
        
        let accounts = result.unwrap();
        assert_eq!(accounts.owner, owner);
        assert_ne!(accounts.registry_state, Pubkey::default());
        assert_ne!(accounts.automation_job, Pubkey::default());
    }
}