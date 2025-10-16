use serde_json::Value;
use chrono::{DateTime, Utc};
use log::{debug, warn};
use crate::database::JobRecord;
use crate::rpc::RpcManager;
use crate::error::{KeeperError, KeeperResult};

pub struct TriggerEvaluator {
    rpc_manager: RpcManager,
}

#[derive(Debug, Clone)]
pub struct EvaluationResult {
    pub should_execute: bool,
    pub reason: String,
    pub next_check_time: Option<DateTime<Utc>>,
}

impl TriggerEvaluator {
    pub fn new(rpc_manager: RpcManager) -> Self {
        Self { rpc_manager }
    }

    pub async fn evaluate_job(&self, job: &JobRecord) -> KeeperResult<EvaluationResult> {
        let now = Utc::now();
        
        // Basic checks first
        if !job.is_active {
            return Ok(EvaluationResult {
                should_execute: false,
                reason: "Job is not active".to_string(),
                next_check_time: None,
            });
        }

        if job.balance <= job.min_balance {
            return Ok(EvaluationResult {
                should_execute: false,
                reason: "Insufficient balance".to_string(),
                next_check_time: None,
            });
        }

        // Evaluate based on trigger type
        match job.trigger_type.as_str() {
            "time" => self.evaluate_time_trigger(job, now).await,
            "conditional" => self.evaluate_conditional_trigger(job, now).await,
            "log" => self.evaluate_log_trigger(job, now).await,
            "hybrid" => self.evaluate_hybrid_trigger(job, now).await,
            _ => {
                warn!("Unknown trigger type: {}", job.trigger_type);
                Ok(EvaluationResult {
                    should_execute: false,
                    reason: format!("Unknown trigger type: {}", job.trigger_type),
                    next_check_time: None,
                })
            }
        }
    }

    async fn evaluate_time_trigger(
        &self,
        job: &JobRecord,
        now: DateTime<Utc>,
    ) -> KeeperResult<EvaluationResult> {
        let params = &job.trigger_params;
        
        // Extract interval from trigger params
        let interval_seconds = params
            .get("interval")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| KeeperError::InvalidTriggerError(
                "Missing or invalid interval in time trigger".to_string()
            ))?;

        debug!("Evaluating time trigger for job {}: interval={}", job.job_id, interval_seconds);

        let should_execute = if let Some(last_executed) = job.last_executed {
            let elapsed = now.signed_duration_since(last_executed);
            elapsed.num_seconds() >= interval_seconds
        } else {
            // First execution
            true
        };

        let next_check_time = if should_execute {
            None // Execute now
        } else if let Some(last_executed) = job.last_executed {
            Some(last_executed + chrono::Duration::seconds(interval_seconds))
        } else {
            Some(now + chrono::Duration::seconds(interval_seconds))
        };

        let reason = if should_execute {
            "Time interval elapsed".to_string()
        } else {
            format!("Waiting for interval ({}s)", interval_seconds)
        };

        Ok(EvaluationResult {
            should_execute,
            reason,
            next_check_time,
        })
    }

    async fn evaluate_conditional_trigger(
        &self,
        job: &JobRecord,
        _now: DateTime<Utc>,
    ) -> KeeperResult<EvaluationResult> {
        let params = &job.trigger_params;
        
        // Extract condition logic from trigger params
        let condition = params
            .get("condition")
            .and_then(|v| v.as_str())
            .ok_or_else(|| KeeperError::InvalidTriggerError(
                "Missing condition in conditional trigger".to_string()
            ))?;

        debug!("Evaluating conditional trigger for job {}: condition={}", job.job_id, condition);

        // For now, implement basic condition evaluation
        // In a full implementation, this would be a more sophisticated condition parser/evaluator
        let result = self.evaluate_condition(job, condition).await?;

        Ok(EvaluationResult {
            should_execute: result.0,
            reason: result.1,
            next_check_time: Some(Utc::now() + chrono::Duration::seconds(60)), // Check again in 1 minute
        })
    }

    async fn evaluate_log_trigger(
        &self,
        job: &JobRecord,
        _now: DateTime<Utc>,
    ) -> KeeperResult<EvaluationResult> {
        let params = &job.trigger_params;
        
        let event_signature = params
            .get("event_signature")
            .and_then(|v| v.as_str())
            .ok_or_else(|| KeeperError::InvalidTriggerError(
                "Missing event_signature in log trigger".to_string()
            ))?;

        debug!("Evaluating log trigger for job {}: event={}", job.job_id, event_signature);

        // For now, we'll implement a simple check based on time
        // In a full implementation, this would monitor transaction logs for specific events
        let should_execute = job.last_executed.is_none() || 
            job.last_executed
                .map(|t| Utc::now().signed_duration_since(t).num_seconds() > 300)
                .unwrap_or(false); // 5 minutes

        Ok(EvaluationResult {
            should_execute,
            reason: if should_execute {
                "Event condition met".to_string()
            } else {
                "Waiting for event".to_string()
            },
            next_check_time: Some(Utc::now() + chrono::Duration::seconds(30)), // Check again in 30 seconds
        })
    }

    async fn evaluate_hybrid_trigger(
        &self,
        job: &JobRecord,
        now: DateTime<Utc>,
    ) -> KeeperResult<EvaluationResult> {
        let params = &job.trigger_params;
        
        debug!("Evaluating hybrid trigger for job {}", job.job_id);

        // Hybrid triggers combine multiple conditions
        let time_condition = params.get("time_interval").and_then(|v| v.as_i64());
        let custom_condition = params.get("condition").and_then(|v| v.as_str());
        let event_signature = params.get("event_signature").and_then(|v| v.as_str());

        let mut should_execute = true;
        let mut reasons = Vec::new();

        // Check time condition if present
        if let Some(interval) = time_condition {
            let time_check = if let Some(last_executed) = job.last_executed {
                now.signed_duration_since(last_executed).num_seconds() >= interval
            } else {
                true
            };
            
            if !time_check {
                should_execute = false;
                reasons.push(format!("Time interval not met ({}s)", interval));
            } else {
                reasons.push("Time interval met".to_string());
            }
        }

        // Check custom condition if present
        if let Some(condition) = custom_condition {
            let (condition_result, condition_reason) = self.evaluate_condition(job, condition).await?;
            if !condition_result {
                should_execute = false;
            }
            reasons.push(condition_reason);
        }

        // Check event condition if present (simplified)
        if event_signature.is_some() {
            // For now, just check if enough time has passed
            let event_check = job.last_executed.is_none() || 
                job.last_executed
                    .map(|t| now.signed_duration_since(t).num_seconds() > 60)
                    .unwrap_or(false);
            
            if !event_check {
                should_execute = false;
                reasons.push("Event condition not met".to_string());
            } else {
                reasons.push("Event condition met".to_string());
            }
        }

        Ok(EvaluationResult {
            should_execute,
            reason: reasons.join("; "),
            next_check_time: Some(now + chrono::Duration::seconds(30)),
        })
    }

    async fn evaluate_condition(&self, job: &JobRecord, condition: &str) -> KeeperResult<(bool, String)> {
        // Simple condition evaluation - in a full implementation this would be much more sophisticated
        // For now, we'll support basic conditions like:
        // - "balance > 1000000" (balance greater than 1 SOL)
        // - "account_exists:PublicKey" (check if account exists)
        // - "token_balance > 100:TokenMintAddress" (token balance check)
        
        debug!("Evaluating condition: {}", condition);
        
        if condition.starts_with("balance >") {
            return self.evaluate_balance_condition(job, condition).await;
        }
        
        if condition.starts_with("account_exists:") {
            return self.evaluate_account_exists_condition(condition).await;
        }
        
        if condition.contains("token_balance >") {
            return self.evaluate_token_balance_condition(condition).await;
        }
        
        // Default case - always true for unknown conditions
        warn!("Unknown condition format: {}", condition);
        Ok((true, "Unknown condition - defaulting to true".to_string()))
    }

    async fn evaluate_balance_condition(&self, job: &JobRecord, condition: &str) -> KeeperResult<(bool, String)> {
        // Parse "balance > 1000000" format
        let parts: Vec<&str> = condition.split_whitespace().collect();
        if parts.len() != 3 {
            return Ok((false, "Invalid balance condition format".to_string()));
        }

        let threshold: u64 = parts[2].parse()
            .map_err(|_| KeeperError::InvalidTriggerError("Invalid balance threshold".to_string()))?;

        // Check the job's own balance
        let current_balance = job.balance as u64;
        let result = current_balance > threshold;
        
        Ok((result, format!("Balance {} {} {}", current_balance, parts[1], threshold)))
    }

    async fn evaluate_account_exists_condition(&self, condition: &str) -> KeeperResult<(bool, String)> {
        // Parse "account_exists:PublicKeyString" format
        let pubkey_str = condition.strip_prefix("account_exists:")
            .ok_or_else(|| KeeperError::InvalidTriggerError("Invalid account_exists format".to_string()))?;

        let pubkey = pubkey_str.parse::<solana_sdk::pubkey::Pubkey>()
            .map_err(|_| KeeperError::InvalidTriggerError("Invalid public key".to_string()))?;

        match self.rpc_manager.get_account_data(&pubkey).await {
            Ok(Some(_)) => Ok((true, "Account exists".to_string())),
            Ok(None) => Ok((false, "Account does not exist".to_string())),
            Err(e) => {
                warn!("Error checking account existence: {:?}", e);
                Ok((false, "Error checking account".to_string()))
            }
        }
    }

    async fn evaluate_token_balance_condition(&self, condition: &str) -> KeeperResult<(bool, String)> {
        // Parse "token_balance > 100:TokenMintAddress" format
        // This is a placeholder implementation
        debug!("Token balance condition evaluation not fully implemented: {}", condition);
        Ok((true, "Token condition evaluation placeholder".to_string()))
    }
}