use sqlx::{PgPool, Pool, Postgres, Row};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::config::KeeperConfig;
use crate::error::{KeeperError, KeeperResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRecord {
    pub job_id: i64,
    pub owner: String,
    pub target_program: String,
    pub target_instruction: String,
    pub trigger_type: String,
    pub trigger_params: serde_json::Value,
    pub balance: i64,
    pub gas_limit: i64,
    pub min_balance: i64,
    pub is_active: bool,
    pub last_checked: Option<DateTime<Utc>>,
    pub last_executed: Option<DateTime<Utc>>,
    pub execution_count: i64,
    pub failed_count: i64,
    pub cached_data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    pub id: i32,
    pub job_id: i64,
    pub keeper_address: String,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub signature: Option<String>,
    pub error: Option<String>,
    pub gas_used: Option<i64>,
    pub fee_paid: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeeperStats {
    pub date: chrono::NaiveDate,
    pub successful_executions: i32,
    pub failed_executions: i32,
    pub total_fees_earned: i64,
}

pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(config: &KeeperConfig) -> KeeperResult<Self> {
        let pool = PgPool::connect(&config.database.url).await?;
        
        // Run migrations
        Self::run_migrations(&pool).await?;
        
        Ok(Database { pool })
    }

    async fn run_migrations(pool: &PgPool) -> KeeperResult<()> {
        // Create jobs table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS jobs (
                job_id BIGINT PRIMARY KEY,
                owner TEXT NOT NULL,
                target_program TEXT NOT NULL,
                target_instruction TEXT NOT NULL,
                trigger_type TEXT NOT NULL,
                trigger_params JSONB NOT NULL,
                balance BIGINT NOT NULL,
                gas_limit BIGINT NOT NULL,
                min_balance BIGINT NOT NULL,
                is_active BOOLEAN NOT NULL DEFAULT true,
                last_checked TIMESTAMP WITH TIME ZONE,
                last_executed TIMESTAMP WITH TIME ZONE,
                execution_count BIGINT NOT NULL DEFAULT 0,
                failed_count BIGINT NOT NULL DEFAULT 0,
                cached_data JSONB,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            )
            "#
        ).execute(pool).await?;

        // Create executions table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS executions (
                id SERIAL PRIMARY KEY,
                job_id BIGINT NOT NULL,
                keeper_address TEXT NOT NULL,
                timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
                success BOOLEAN NOT NULL,
                signature TEXT,
                error TEXT,
                gas_used BIGINT,
                fee_paid BIGINT,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            )
            "#
        ).execute(pool).await?;

        // Create keeper_stats table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS keeper_stats (
                date DATE PRIMARY KEY,
                successful_executions INTEGER NOT NULL DEFAULT 0,
                failed_executions INTEGER NOT NULL DEFAULT 0,
                total_fees_earned BIGINT NOT NULL DEFAULT 0,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
            )
            "#
        ).execute(pool).await?;

        // Create indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_jobs_active ON jobs(is_active) WHERE is_active = true")
            .execute(pool).await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_jobs_last_checked ON jobs(last_checked)")
            .execute(pool).await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_executions_job_id ON executions(job_id)")
            .execute(pool).await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_executions_timestamp ON executions(timestamp)")
            .execute(pool).await?;

        Ok(())
    }

    pub async fn upsert_job(&self, job: &JobRecord) -> KeeperResult<()> {
        sqlx::query(
            r#"
            INSERT INTO jobs (
                job_id, owner, target_program, target_instruction, trigger_type, 
                trigger_params, balance, gas_limit, min_balance, is_active,
                last_executed, execution_count, failed_count, cached_data
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            ON CONFLICT (job_id) DO UPDATE SET
                owner = EXCLUDED.owner,
                target_program = EXCLUDED.target_program,
                target_instruction = EXCLUDED.target_instruction,
                trigger_type = EXCLUDED.trigger_type,
                trigger_params = EXCLUDED.trigger_params,
                balance = EXCLUDED.balance,
                gas_limit = EXCLUDED.gas_limit,
                min_balance = EXCLUDED.min_balance,
                is_active = EXCLUDED.is_active,
                last_executed = EXCLUDED.last_executed,
                execution_count = EXCLUDED.execution_count,
                failed_count = EXCLUDED.failed_count,
                cached_data = EXCLUDED.cached_data,
                updated_at = NOW()
            "#
        )
        .bind(job.job_id)
        .bind(&job.owner)
        .bind(&job.target_program)
        .bind(&job.target_instruction)
        .bind(&job.trigger_type)
        .bind(&job.trigger_params)
        .bind(job.balance)
        .bind(job.gas_limit)
        .bind(job.min_balance)
        .bind(job.is_active)
        .bind(job.last_executed)
        .bind(job.execution_count)
        .bind(job.failed_count)
        .bind(&job.cached_data)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_active_jobs(&self) -> KeeperResult<Vec<JobRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT job_id, owner, target_program, target_instruction, trigger_type,
                   trigger_params, balance, gas_limit, min_balance, is_active,
                   last_checked, last_executed, execution_count, failed_count, cached_data
            FROM jobs 
            WHERE is_active = true 
            ORDER BY last_checked ASC NULLS FIRST
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut jobs = Vec::new();
        for row in rows {
            jobs.push(JobRecord {
                job_id: row.get("job_id"),
                owner: row.get("owner"),
                target_program: row.get("target_program"),
                target_instruction: row.get("target_instruction"),
                trigger_type: row.get("trigger_type"),
                trigger_params: row.get("trigger_params"),
                balance: row.get("balance"),
                gas_limit: row.get("gas_limit"),
                min_balance: row.get("min_balance"),
                is_active: row.get("is_active"),
                last_checked: row.get("last_checked"),
                last_executed: row.get("last_executed"),
                execution_count: row.get("execution_count"),
                failed_count: row.get("failed_count"),
                cached_data: row.get("cached_data"),
            });
        }

        Ok(jobs)
    }

    pub async fn get_eligible_jobs(&self, keeper_address: &str) -> KeeperResult<Vec<JobRecord>> {
        let now = Utc::now();
        
        let rows = sqlx::query(
            r#"
            SELECT job_id, owner, target_program, target_instruction, trigger_type,
                   trigger_params, balance, gas_limit, min_balance, is_active,
                   last_checked, last_executed, execution_count, failed_count, cached_data
            FROM jobs 
            WHERE is_active = true 
              AND balance > min_balance
              AND (last_checked IS NULL OR last_checked < $1 - INTERVAL '30 seconds')
            ORDER BY 
              CASE WHEN last_executed IS NULL THEN 0 ELSE 1 END,
              last_executed ASC NULLS FIRST
            LIMIT 50
            "#
        )
        .bind(now)
        .fetch_all(&self.pool)
        .await?;

        let mut jobs = Vec::new();
        for row in rows {
            jobs.push(JobRecord {
                job_id: row.get("job_id"),
                owner: row.get("owner"),
                target_program: row.get("target_program"),
                target_instruction: row.get("target_instruction"),
                trigger_type: row.get("trigger_type"),
                trigger_params: row.get("trigger_params"),
                balance: row.get("balance"),
                gas_limit: row.get("gas_limit"),
                min_balance: row.get("min_balance"),
                is_active: row.get("is_active"),
                last_checked: row.get("last_checked"),
                last_executed: row.get("last_executed"),
                execution_count: row.get("execution_count"),
                failed_count: row.get("failed_count"),
                cached_data: row.get("cached_data"),
            });
        }

        Ok(jobs)
    }

    pub async fn update_job_checked(&self, job_id: i64) -> KeeperResult<()> {
        sqlx::query(
            "UPDATE jobs SET last_checked = NOW(), updated_at = NOW() WHERE job_id = $1"
        )
        .bind(job_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn record_execution(&self, execution: &ExecutionRecord) -> KeeperResult<()> {
        sqlx::query(
            r#"
            INSERT INTO executions (
                job_id, keeper_address, timestamp, success, 
                signature, error, gas_used, fee_paid
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#
        )
        .bind(execution.job_id)
        .bind(&execution.keeper_address)
        .bind(execution.timestamp)
        .bind(execution.success)
        .bind(&execution.signature)
        .bind(&execution.error)
        .bind(execution.gas_used)
        .bind(execution.fee_paid)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_keeper_stats(&self, date: chrono::NaiveDate, success: bool, fee_earned: i64) -> KeeperResult<()> {
        if success {
            sqlx::query(
                r#"
                INSERT INTO keeper_stats (date, successful_executions, total_fees_earned)
                VALUES ($1, 1, $2)
                ON CONFLICT (date) DO UPDATE SET
                    successful_executions = keeper_stats.successful_executions + 1,
                    total_fees_earned = keeper_stats.total_fees_earned + $2
                "#
            )
            .bind(date)
            .bind(fee_earned)
            .execute(&self.pool)
            .await?;
        } else {
            sqlx::query(
                r#"
                INSERT INTO keeper_stats (date, failed_executions)
                VALUES ($1, 1)
                ON CONFLICT (date) DO UPDATE SET
                    failed_executions = keeper_stats.failed_executions + 1
                "#
            )
            .bind(date)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    pub async fn get_execution_history(&self, job_id: i64, limit: i64) -> KeeperResult<Vec<ExecutionRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT id, job_id, keeper_address, timestamp, success, 
                   signature, error, gas_used, fee_paid
            FROM executions 
            WHERE job_id = $1 
            ORDER BY timestamp DESC 
            LIMIT $2
            "#
        )
        .bind(job_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut executions = Vec::new();
        for row in rows {
            executions.push(ExecutionRecord {
                id: row.get("id"),
                job_id: row.get("job_id"),
                keeper_address: row.get("keeper_address"),
                timestamp: row.get("timestamp"),
                success: row.get("success"),
                signature: row.get("signature"),
                error: row.get("error"),
                gas_used: row.get("gas_used"),
                fee_paid: row.get("fee_paid"),
            });
        }

        Ok(executions)
    }

    pub async fn cleanup_old_data(&self, days: i32) -> KeeperResult<()> {
        // Clean up old execution records
        sqlx::query(
            "DELETE FROM executions WHERE created_at < NOW() - INTERVAL '$1 days'"
        )
        .bind(days)
        .execute(&self.pool)
        .await?;

        // Clean up old keeper stats (keep yearly data)
        sqlx::query(
            "DELETE FROM keeper_stats WHERE date < CURRENT_DATE - INTERVAL '365 days'"
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}