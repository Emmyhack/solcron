use solana_program::pubkey::Pubkey;
use crate::{
    types::*,
    error::{SolCronError, SolCronResult},
};

/// Utility functions for SolCron operations
pub struct Utils;

impl Utils {
    /// Convert a string to bytes with maximum length
    /// 
    /// # Arguments
    /// * `s` - String to convert
    /// * `max_len` - Maximum allowed length
    /// 
    /// Returns bytes padded/truncated to max_len
    pub fn string_to_bytes(s: &str, max_len: usize) -> Vec<u8> {
        let mut bytes = s.as_bytes().to_vec();
        bytes.resize(max_len, 0);
        bytes
    }

    /// Convert bytes to string, removing null padding
    pub fn bytes_to_string(bytes: &[u8]) -> String {
        // Find the first null byte or use full length
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        String::from_utf8_lossy(&bytes[..end]).to_string()
    }

    /// Validate a public key string and convert to Pubkey
    pub fn parse_pubkey(s: &str) -> SolCronResult<Pubkey> {
        s.parse::<Pubkey>()
            .map_err(|e| SolCronError::ValidationError {
                field: "pubkey".to_string(),
                reason: format!("Invalid public key format: {}", e),
            })
    }

    /// Format lamports as SOL string with precision
    pub fn lamports_to_sol_string(lamports: u64, precision: usize) -> String {
        let sol = lamports as f64 / solana_program::native_token::LAMPORTS_PER_SOL as f64;
        format!("{:.precision$}", sol, precision = precision)
    }

    /// Convert SOL to lamports
    pub fn sol_to_lamports(sol: f64) -> u64 {
        (sol * solana_program::native_token::LAMPORTS_PER_SOL as f64) as u64
    }

    /// Calculate execution fee based on registry parameters
    pub fn calculate_execution_fee(base_fee: u64, gas_used: u64, gas_price: u64) -> u64 {
        base_fee + (gas_used * gas_price)
    }

    /// Calculate protocol fee from execution fee
    pub fn calculate_protocol_fee(execution_fee: u64, fee_bps: u16) -> u64 {
        (execution_fee * fee_bps as u64) / 10000
    }

    /// Calculate keeper reward (execution fee - protocol fee)
    pub fn calculate_keeper_reward(execution_fee: u64, fee_bps: u16) -> u64 {
        let protocol_fee = Self::calculate_protocol_fee(execution_fee, fee_bps);
        execution_fee - protocol_fee
    }

    /// Validate trigger parameters for different trigger types
    pub fn validate_trigger_params(
        trigger_type: &TriggerType,
        trigger_params: &[u8],
    ) -> SolCronResult<()> {
        match trigger_type {
            TriggerType::TimeBased { interval } => {
                if *interval == 0 {
                    return Err(SolCronError::InvalidTrigger {
                        reason: "Time interval cannot be zero".to_string(),
                    });
                }
                if *interval > 365 * 24 * 3600 {
                    return Err(SolCronError::InvalidTrigger {
                        reason: "Time interval too long (max 1 year)".to_string(),
                    });
                }
            }
            TriggerType::Conditional { logic } => {
                if logic.is_empty() {
                    return Err(SolCronError::InvalidTrigger {
                        reason: "Conditional logic cannot be empty".to_string(),
                    });
                }
                if logic.len() > 1024 {
                    return Err(SolCronError::InvalidTrigger {
                        reason: "Conditional logic too long (max 1024 bytes)".to_string(),
                    });
                }
            }
            TriggerType::LogBased { program_id, event_filter } => {
                if event_filter.is_empty() {
                    return Err(SolCronError::InvalidTrigger {
                        reason: "Event filter cannot be empty".to_string(),
                    });
                }
            }
            TriggerType::Hybrid { conditions, operator } => {
                if conditions.is_empty() {
                    return Err(SolCronError::InvalidTrigger {
                        reason: "Hybrid trigger must have at least one condition".to_string(),
                    });
                }
                if operator != "AND" && operator != "OR" {
                    return Err(SolCronError::InvalidTrigger {
                        reason: "Hybrid operator must be 'AND' or 'OR'".to_string(),
                    });
                }
            }
        }

        // Validate trigger params length
        if trigger_params.len() > 64 {
            return Err(SolCronError::InvalidTrigger {
                reason: "Trigger params too long (max 64 bytes)".to_string(),
            });
        }

        Ok(())
    }

    /// Validate job parameters
    pub fn validate_job_params(params: &JobParams) -> SolCronResult<()> {
        // Validate target instruction name
        if params.target_instruction.is_empty() {
            return Err(SolCronError::ValidationError {
                field: "target_instruction".to_string(),
                reason: "Cannot be empty".to_string(),
            });
        }

        if params.target_instruction.len() > 32 {
            return Err(SolCronError::ValidationError {
                field: "target_instruction".to_string(),
                reason: "Too long (max 32 characters)".to_string(),
            });
        }

        // Validate gas limit
        if params.gas_limit == 0 {
            return Err(SolCronError::ValidationError {
                field: "gas_limit".to_string(),
                reason: "Cannot be zero".to_string(),
            });
        }

        const MAX_GAS_LIMIT: u64 = 1_400_000; // Solana transaction compute limit
        if params.gas_limit > MAX_GAS_LIMIT {
            return Err(SolCronError::ValidationError {
                field: "gas_limit".to_string(),
                reason: format!("Exceeds maximum ({})", MAX_GAS_LIMIT),
            });
        }

        // Validate minimum balance
        const MIN_BALANCE_THRESHOLD: u64 = 1000; // 0.000001 SOL
        if params.min_balance < MIN_BALANCE_THRESHOLD {
            return Err(SolCronError::ValidationError {
                field: "min_balance".to_string(),
                reason: format!("Below minimum threshold ({})", MIN_BALANCE_THRESHOLD),
            });
        }

        // Validate trigger
        Self::validate_trigger_params(&params.trigger_type, &params.trigger_params)?;

        Ok(())
    }

    /// Validate keeper registration parameters
    pub fn validate_keeper_params(stake_amount: u64, min_stake: u64) -> SolCronResult<()> {
        if stake_amount < min_stake {
            return Err(SolCronError::InsufficientStake {
                required: min_stake,
                provided: stake_amount,
            });
        }

        const MAX_STAKE: u64 = 1000 * solana_program::native_token::LAMPORTS_PER_SOL; // 1000 SOL
        if stake_amount > MAX_STAKE {
            return Err(SolCronError::ValidationError {
                field: "stake_amount".to_string(),
                reason: format!("Exceeds maximum stake ({})", MAX_STAKE),
            });
        }

        Ok(())
    }

    /// Validate registry parameters
    pub fn validate_registry_params(
        base_fee: u64,
        min_stake: u64,
        protocol_fee_bps: u16,
    ) -> SolCronResult<()> {
        // Validate base fee
        const MAX_BASE_FEE: u64 = 100_000; // 0.0001 SOL max
        if base_fee > MAX_BASE_FEE {
            return Err(SolCronError::ValidationError {
                field: "base_fee".to_string(),
                reason: format!("Exceeds maximum ({})", MAX_BASE_FEE),
            });
        }

        // Validate minimum stake
        const MIN_STAKE_THRESHOLD: u64 = solana_program::native_token::LAMPORTS_PER_SOL / 10; // 0.1 SOL
        if min_stake < MIN_STAKE_THRESHOLD {
            return Err(SolCronError::ValidationError {
                field: "min_stake".to_string(),
                reason: format!("Below minimum threshold ({})", MIN_STAKE_THRESHOLD),
            });
        }

        // Validate protocol fee (max 10% = 1000 bps)
        const MAX_PROTOCOL_FEE_BPS: u16 = 1000;
        if protocol_fee_bps > MAX_PROTOCOL_FEE_BPS {
            return Err(SolCronError::ValidationError {
                field: "protocol_fee_bps".to_string(),
                reason: format!("Exceeds maximum ({})", MAX_PROTOCOL_FEE_BPS),
            });
        }

        Ok(())
    }

    /// Check if a time-based trigger should execute
    pub fn should_execute_time_trigger(
        interval: u64,
        last_execution: u64,
        current_time: u64,
    ) -> bool {
        if last_execution == 0 {
            return true; // First execution
        }
        current_time >= last_execution + interval
    }

    /// Estimate gas usage for different instruction types
    pub fn estimate_gas_usage(instruction_type: &str) -> u64 {
        match instruction_type.to_lowercase().as_str() {
            "transfer" => 300,
            "mint" => 1000,
            "swap" => 5000,
            "harvest" => 10000,
            "liquidate" => 15000,
            "compound" => 8000,
            _ => 5000, // Default estimate
        }
    }

    /// Calculate reputation score change based on execution result
    pub fn calculate_reputation_change(
        current_reputation: u64,
        success: bool,
        consecutive_successes: u64,
        consecutive_failures: u64,
    ) -> i64 {
        const BASE_CHANGE: u64 = 100;
        const MAX_REPUTATION: u64 = 10000;

        if success {
            // Bonus for consecutive successes
            let bonus = std::cmp::min(consecutive_successes * 10, 200);
            let change = BASE_CHANGE + bonus;
            
            // Don't exceed maximum reputation
            if current_reputation + change > MAX_REPUTATION {
                (MAX_REPUTATION - current_reputation) as i64
            } else {
                change as i64
            }
        } else {
            // Penalty increases with consecutive failures
            let penalty = BASE_CHANGE * 2 + (consecutive_failures * 50);
            let change = std::cmp::min(penalty, current_reputation);
            -(change as i64)
        }
    }

    /// Format duration in human-readable format
    pub fn format_duration(seconds: u64) -> String {
        const MINUTE: u64 = 60;
        const HOUR: u64 = 60 * MINUTE;
        const DAY: u64 = 24 * HOUR;

        if seconds >= DAY {
            format!("{} day(s)", seconds / DAY)
        } else if seconds >= HOUR {
            format!("{} hour(s)", seconds / HOUR)
        } else if seconds >= MINUTE {
            format!("{} minute(s)", seconds / MINUTE)
        } else {
            format!("{} second(s)", seconds)
        }
    }

    /// Generate a unique seed for PDA derivation
    pub fn generate_unique_seed() -> [u8; 8] {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        timestamp.to_le_bytes()
    }

    /// Serialize trigger parameters to JSON
    pub fn serialize_trigger_params<T: serde::Serialize>(params: &T) -> SolCronResult<Vec<u8>> {
        serde_json::to_vec(params)
            .map_err(|e| SolCronError::SerializationError {
                reason: format!("Failed to serialize trigger params: {}", e),
            })
    }

    /// Deserialize trigger parameters from JSON
    pub fn deserialize_trigger_params<T: serde::de::DeserializeOwned>(
        data: &[u8],
    ) -> SolCronResult<T> {
        serde_json::from_slice(data)
            .map_err(|e| SolCronError::DeserializationError {
                reason: format!("Failed to deserialize trigger params: {}", e),
            })
    }

    /// Check if an account has sufficient balance for an operation
    pub fn check_sufficient_balance(
        available: u64,
        required: u64,
        operation: &str,
    ) -> SolCronResult<()> {
        if available < required {
            Err(SolCronError::InsufficientBalance {
                required,
                available,
            })
        } else {
            Ok(())
        }
    }

    /// Get current Unix timestamp
    pub fn current_timestamp() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Convert basis points to percentage
    pub fn bps_to_percentage(bps: u16) -> f64 {
        bps as f64 / 100.0
    }

    /// Convert percentage to basis points
    pub fn percentage_to_bps(percentage: f64) -> u16 {
        (percentage * 100.0) as u16
    }
}

/// Time utilities
pub struct TimeUtils;

impl TimeUtils {
    /// Get next execution time for a time-based trigger
    pub fn next_execution_time(interval: u64, last_execution: u64) -> u64 {
        if last_execution == 0 {
            Utils::current_timestamp()
        } else {
            last_execution + interval
        }
    }

    /// Check if enough time has passed since last execution
    pub fn can_execute_now(interval: u64, last_execution: u64) -> bool {
        Utils::should_execute_time_trigger(interval, last_execution, Utils::current_timestamp())
    }

    /// Calculate time remaining until next execution
    pub fn time_until_execution(interval: u64, last_execution: u64) -> u64 {
        let next_time = Self::next_execution_time(interval, last_execution);
        let current_time = Utils::current_timestamp();
        
        if current_time >= next_time {
            0
        } else {
            next_time - current_time
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_to_bytes() {
        let result = Utils::string_to_bytes("hello", 10);
        assert_eq!(result.len(), 10);
        assert_eq!(&result[..5], b"hello");
        assert_eq!(&result[5..], &[0; 5]);
    }

    #[test]
    fn test_bytes_to_string() {
        let bytes = vec![104, 101, 108, 108, 111, 0, 0, 0]; // "hello" + nulls
        let result = Utils::bytes_to_string(&bytes);
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_lamports_conversion() {
        let lamports = 1_000_000_000; // 1 SOL
        let sol_string = Utils::lamports_to_sol_string(lamports, 3);
        assert_eq!(sol_string, "1.000");

        let sol = 1.5;
        let converted_lamports = Utils::sol_to_lamports(sol);
        assert_eq!(converted_lamports, 1_500_000_000);
    }

    #[test]
    fn test_fee_calculations() {
        let base_fee = 5000;
        let gas_used = 10000;
        let gas_price = 1;
        
        let execution_fee = Utils::calculate_execution_fee(base_fee, gas_used, gas_price);
        assert_eq!(execution_fee, 15000);

        let protocol_fee = Utils::calculate_protocol_fee(execution_fee, 250); // 2.5%
        assert_eq!(protocol_fee, 375);

        let keeper_reward = Utils::calculate_keeper_reward(execution_fee, 250);
        assert_eq!(keeper_reward, 14625);
    }

    #[test]
    fn test_time_trigger() {
        let interval = 3600; // 1 hour
        let last_execution = 1000000;
        let current_time = 1003700; // 1 hour 1 minute later

        assert!(Utils::should_execute_time_trigger(interval, last_execution, current_time));
        
        let too_early = 1003500; // 58 minutes later
        assert!(!Utils::should_execute_time_trigger(interval, last_execution, too_early));
    }

    #[test]
    fn test_reputation_change() {
        let current_reputation = 5000;
        
        // Successful execution
        let change = Utils::calculate_reputation_change(current_reputation, true, 5, 0);
        assert!(change > 0);

        // Failed execution
        let change = Utils::calculate_reputation_change(current_reputation, false, 0, 3);
        assert!(change < 0);
    }

    #[test]
    fn test_validation() {
        let valid_params = JobParams {
            target_program: Pubkey::new_unique(),
            target_instruction: "test".to_string(),
            trigger_type: TriggerType::TimeBased { interval: 3600 },
            trigger_params: vec![],
            gas_limit: 200_000,
            min_balance: 1_000_000,
        };

        assert!(Utils::validate_job_params(&valid_params).is_ok());

        // Test invalid gas limit
        let mut invalid_params = valid_params.clone();
        invalid_params.gas_limit = 0;
        assert!(Utils::validate_job_params(&invalid_params).is_err());
    }
}