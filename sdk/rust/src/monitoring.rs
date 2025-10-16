//! Monitoring and analytics utilities for SolCron automation jobs
//!
//! This module provides comprehensive monitoring capabilities including
//! real-time metrics collection, performance analysis, and alerting.

use crate::{
    client::SolCronClient,
    types::{AutomationJob, Keeper, RegistryState, TriggerType},
    error::{SolCronError, SolCronResult},
    utils::{Utils, TimeUtils},
};
use solana_sdk::pubkey::Pubkey;
use std::{
    collections::{HashMap, VecDeque},
    time::{SystemTime, UNIX_EPOCH},
};
use serde::{Serialize, Deserialize};

/// Configuration for monitoring operations
#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    /// How often to collect metrics (seconds)
    pub collection_interval: u64,
    /// Number of data points to retain in memory
    pub history_retention: usize,
    /// Alert thresholds
    pub alert_thresholds: AlertThresholds,
    /// Whether to enable detailed logging
    pub verbose_logging: bool,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            collection_interval: 60, // 1 minute
            history_retention: 1440, // 24 hours of minute-level data
            alert_thresholds: AlertThresholds::default(),
            verbose_logging: false,
        }
    }
}

/// Alert threshold configuration
#[derive(Debug, Clone)]
pub struct AlertThresholds {
    /// Minimum job balance (lamports)
    pub min_job_balance: u64,
    /// Maximum execution failure rate (percentage)
    pub max_failure_rate: f64,
    /// Maximum average execution time (seconds)
    pub max_execution_time: u64,
    /// Minimum keeper reputation score
    pub min_keeper_reputation: u32,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            min_job_balance: Utils::sol_to_lamports(0.001), // 0.001 SOL
            max_failure_rate: 10.0, // 10%
            max_execution_time: 30, // 30 seconds
            min_keeper_reputation: 8000, // 80%
        }
    }
}

/// Real-time metrics for the SolCron system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub timestamp: u64,
    pub registry_stats: RegistryMetrics,
    pub job_stats: JobMetrics,
    pub keeper_stats: KeeperMetrics,
    pub network_stats: NetworkMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryMetrics {
    pub total_jobs: u64,
    pub active_jobs: u64,
    pub total_keepers: u64,
    pub active_keepers: u64,
    pub total_executions: u64,
    pub successful_executions: u64,
    pub protocol_revenue: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobMetrics {
    pub avg_balance: u64,
    pub total_balance: u64,
    pub low_balance_count: u64,
    pub execution_success_rate: f64,
    pub avg_execution_time: f64,
    pub trigger_type_distribution: HashMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeeperMetrics {
    pub avg_reputation: f64,
    pub total_stake: u64,
    pub avg_stake: u64,
    pub keeper_utilization: f64, // Percentage of keepers actively executing
    pub top_performers: Vec<KeeperPerformance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeeperPerformance {
    pub keeper_address: String,
    pub reputation_score: u32,
    pub success_rate: f64,
    pub total_executions: u64,
    pub earnings: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub avg_transaction_time: f64,
    pub network_congestion: f64,
    pub gas_price_trend: f64,
}

/// Historical data point for trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsDataPoint {
    pub timestamp: u64,
    pub metrics: SystemMetrics,
}

/// Alert types that can be triggered
#[derive(Debug, Clone)]
pub enum AlertType {
    LowJobBalance { job_id: u64, current_balance: u64, threshold: u64 },
    HighFailureRate { job_id: u64, failure_rate: f64, threshold: f64 },
    SlowExecution { job_id: u64, execution_time: u64, threshold: u64 },
    KeeperUnderperforming { keeper: Pubkey, reputation: u32, threshold: u32 },
    SystemOverload { active_jobs: u64, active_keepers: u64 },
    NetworkCongestion { avg_time: f64 },
}

impl AlertType {
    pub fn severity(&self) -> AlertSeverity {
        match self {
            AlertType::LowJobBalance { .. } => AlertSeverity::Warning,
            AlertType::HighFailureRate { .. } => AlertSeverity::Critical,
            AlertType::SlowExecution { .. } => AlertSeverity::Warning,
            AlertType::KeeperUnderperforming { .. } => AlertSeverity::Info,
            AlertType::SystemOverload { .. } => AlertSeverity::Critical,
            AlertType::NetworkCongestion { .. } => AlertSeverity::Warning,
        }
    }
    
    pub fn message(&self) -> String {
        match self {
            AlertType::LowJobBalance { job_id, current_balance, threshold } => {
                format!("Job {} has low balance: {} < {} lamports", 
                    job_id, current_balance, threshold)
            },
            AlertType::HighFailureRate { job_id, failure_rate, threshold } => {
                format!("Job {} has high failure rate: {:.1}% > {:.1}%", 
                    job_id, failure_rate, threshold)
            },
            AlertType::SlowExecution { job_id, execution_time, threshold } => {
                format!("Job {} has slow execution: {}s > {}s", 
                    job_id, execution_time, threshold)
            },
            AlertType::KeeperUnderperforming { keeper, reputation, threshold } => {
                format!("Keeper {} underperforming: {} < {} reputation", 
                    keeper, reputation, threshold)
            },
            AlertType::SystemOverload { active_jobs, active_keepers } => {
                format!("System overloaded: {} jobs, {} keepers", 
                    active_jobs, active_keepers)
            },
            AlertType::NetworkCongestion { avg_time } => {
                format!("Network congestion detected: {:.1}s avg transaction time", avg_time)
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// Comprehensive monitoring system for SolCron
pub struct Monitor {
    client: SolCronClient,
    config: MonitoringConfig,
    metrics_history: VecDeque<MetricsDataPoint>,
    job_cache: HashMap<u64, AutomationJob>,
    keeper_cache: HashMap<Pubkey, Keeper>,
    alerts: Vec<AlertType>,
}

impl Monitor {
    /// Create a new monitoring system
    pub fn new(client: SolCronClient, config: Option<MonitoringConfig>) -> Self {
        Self {
            client,
            config: config.unwrap_or_default(),
            metrics_history: VecDeque::new(),
            job_cache: HashMap::new(),
            keeper_cache: HashMap::new(),
            alerts: Vec::new(),
        }
    }

    /// Start the monitoring loop
    pub async fn start_monitoring(&mut self) -> SolCronResult<()> {
        println!("🔍 Starting SolCron monitoring system...");
        
        loop {
            let start_time = std::time::Instant::now();
            
            // Collect current metrics
            match self.collect_metrics().await {
                Ok(metrics) => {
                    self.process_metrics(metrics).await?;
                },
                Err(e) => {
                    println!("❌ Failed to collect metrics: {}", e);
                }
            }
            
            // Sleep until next collection interval
            let elapsed = start_time.elapsed();
            let sleep_duration = std::time::Duration::from_secs(self.config.collection_interval)
                .saturating_sub(elapsed);
                
            if !sleep_duration.is_zero() {
                tokio::time::sleep(sleep_duration).await;
            }
        }
    }

    /// Collect comprehensive system metrics
    pub async fn collect_metrics(&mut self) -> SolCronResult<SystemMetrics> {
        if self.config.verbose_logging {
            println!("📊 Collecting system metrics...");
        }
        
        // Get registry state
        let registry = self.client.get_registry_state().await?;
        
        // Collect job metrics
        let job_metrics = self.collect_job_metrics(&registry).await?;
        
        // Collect keeper metrics  
        let keeper_metrics = self.collect_keeper_metrics(&registry).await?;
        
        // Collect network metrics
        let network_metrics = self.collect_network_metrics().await?;
        
        let registry_metrics = RegistryMetrics {
            total_jobs: registry.total_jobs,
            active_jobs: registry.active_jobs,
            total_keepers: registry.total_keepers,
            active_keepers: registry.active_keepers,
            total_executions: registry.total_executions,
            successful_executions: registry.successful_executions,
            protocol_revenue: registry.protocol_revenue,
        };
        
        Ok(SystemMetrics {
            timestamp: Utils::current_timestamp(),
            registry_stats: registry_metrics,
            job_stats: job_metrics,
            keeper_stats: keeper_metrics,
            network_stats: network_metrics,
        })
    }

    async fn collect_job_metrics(&mut self, registry: &RegistryState) -> SolCronResult<JobMetrics> {
        // In a real implementation, you would scan all job accounts
        // For this example, we'll use cached data and make estimates
        
        let mut total_balance = 0u64;
        let mut low_balance_count = 0u64;
        let mut successful_executions = 0u64;
        let mut total_executions = 0u64;
        let mut execution_times = Vec::new();
        let mut trigger_distribution: HashMap<String, u64> = HashMap::new();
        
        // Process cached jobs
        for (_, job) in &self.job_cache {
            total_balance += job.balance;
            
            if job.balance < self.config.alert_thresholds.min_job_balance {
                low_balance_count += 1;
            }
            
            total_executions += job.execution_count;
            // Assume 90% success rate for demo
            successful_executions += (job.execution_count as f64 * 0.9) as u64;
            
            // Simulate execution times (1-10 seconds)
            if job.execution_count > 0 {
                execution_times.push(5.0); // Average 5 seconds
            }
            
            // Count trigger types
            let trigger_name = match &job.trigger_type {
                TriggerType::TimeBased { .. } => "TimeBased",
                TriggerType::Conditional { .. } => "Conditional", 
                TriggerType::LogBased { .. } => "LogBased",
                TriggerType::Hybrid { .. } => "Hybrid",
            };
            *trigger_distribution.entry(trigger_name.to_string()).or_insert(0) += 1;
        }
        
        let avg_balance = if registry.total_jobs > 0 {
            total_balance / registry.total_jobs
        } else {
            0
        };
        
        let execution_success_rate = if total_executions > 0 {
            (successful_executions as f64 / total_executions as f64) * 100.0
        } else {
            100.0
        };
        
        let avg_execution_time = if !execution_times.is_empty() {
            execution_times.iter().sum::<f64>() / execution_times.len() as f64
        } else {
            0.0
        };
        
        Ok(JobMetrics {
            avg_balance,
            total_balance,
            low_balance_count,
            execution_success_rate,
            avg_execution_time,
            trigger_type_distribution: trigger_distribution,
        })
    }

    async fn collect_keeper_metrics(&mut self, registry: &RegistryState) -> SolCronResult<KeeperMetrics> {
        let mut total_stake = 0u64;
        let mut reputation_sum = 0u64;
        let mut top_performers = Vec::new();
        let active_count = registry.active_keepers;
        
        // Process cached keepers
        for (address, keeper) in &self.keeper_cache {
            total_stake += keeper.stake_amount;
            reputation_sum += keeper.reputation_score as u64;
            
            let performance = KeeperPerformance {
                keeper_address: address.to_string(),
                reputation_score: keeper.reputation_score,
                success_rate: keeper.success_rate(),
                total_executions: keeper.total_executions,
                earnings: keeper.total_earnings,
            };
            
            top_performers.push(performance);
        }
        
        // Sort by reputation and take top 10
        top_performers.sort_by(|a, b| b.reputation_score.cmp(&a.reputation_score));
        top_performers.truncate(10);
        
        let avg_reputation = if registry.total_keepers > 0 {
            reputation_sum as f64 / registry.total_keepers as f64
        } else {
            0.0
        };
        
        let avg_stake = if registry.total_keepers > 0 {
            total_stake / registry.total_keepers
        } else {
            0
        };
        
        let keeper_utilization = if registry.total_keepers > 0 {
            (active_count as f64 / registry.total_keepers as f64) * 100.0
        } else {
            0.0
        };
        
        Ok(KeeperMetrics {
            avg_reputation,
            total_stake,
            avg_stake,
            keeper_utilization,
            top_performers,
        })
    }

    async fn collect_network_metrics(&self) -> SolCronResult<NetworkMetrics> {
        // In a real implementation, you would measure actual network performance
        // For demo purposes, we'll simulate realistic values
        
        Ok(NetworkMetrics {
            avg_transaction_time: 2.5, // 2.5 seconds average
            network_congestion: 15.0,  // 15% congestion
            gas_price_trend: 1.0,      // Stable gas prices
        })
    }

    async fn process_metrics(&mut self, metrics: SystemMetrics) -> SolCronResult<()> {
        // Add to history
        self.add_metrics_to_history(metrics.clone());
        
        // Check for alerts
        self.check_alerts(&metrics).await?;
        
        // Log metrics if verbose
        if self.config.verbose_logging {
            self.print_metrics_summary(&metrics);
        }
        
        Ok(())
    }

    fn add_metrics_to_history(&mut self, metrics: SystemMetrics) {
        let data_point = MetricsDataPoint {
            timestamp: metrics.timestamp,
            metrics,
        };
        
        self.metrics_history.push_back(data_point);
        
        // Maintain history size limit
        while self.metrics_history.len() > self.config.history_retention {
            self.metrics_history.pop_front();
        }
    }

    async fn check_alerts(&mut self, metrics: &SystemMetrics) -> SolCronResult<()> {
        self.alerts.clear();
        
        // Check job balance alerts
        for (job_id, job) in &self.job_cache {
            if job.balance < self.config.alert_thresholds.min_job_balance {
                self.alerts.push(AlertType::LowJobBalance {
                    job_id: *job_id,
                    current_balance: job.balance,
                    threshold: self.config.alert_thresholds.min_job_balance,
                });
            }
        }
        
        // Check execution success rate
        if metrics.job_stats.execution_success_rate < (100.0 - self.config.alert_thresholds.max_failure_rate) {
            println!("⚠️  System-wide execution success rate: {:.1}%", 
                metrics.job_stats.execution_success_rate);
        }
        
        // Check keeper performance
        for (keeper_addr, keeper) in &self.keeper_cache {
            if keeper.reputation_score < self.config.alert_thresholds.min_keeper_reputation {
                self.alerts.push(AlertType::KeeperUnderperforming {
                    keeper: *keeper_addr,
                    reputation: keeper.reputation_score,
                    threshold: self.config.alert_thresholds.min_keeper_reputation,
                });
            }
        }
        
        // Check system overload
        let keeper_to_job_ratio = if metrics.registry_stats.active_jobs > 0 {
            metrics.registry_stats.active_keepers as f64 / metrics.registry_stats.active_jobs as f64
        } else {
            1.0
        };
        
        if keeper_to_job_ratio < 0.1 { // Less than 1 keeper per 10 jobs
            self.alerts.push(AlertType::SystemOverload {
                active_jobs: metrics.registry_stats.active_jobs,
                active_keepers: metrics.registry_stats.active_keepers,
            });
        }
        
        // Process alerts
        for alert in &self.alerts {
            self.handle_alert(alert.clone()).await?;
        }
        
        Ok(())
    }

    async fn handle_alert(&self, alert: AlertType) -> SolCronResult<()> {
        let severity_symbol = match alert.severity() {
            AlertSeverity::Info => "ℹ️",
            AlertSeverity::Warning => "⚠️",
            AlertSeverity::Critical => "🚨",
        };
        
        println!("{} {}", severity_symbol, alert.message());
        
        // In a real implementation, you might:
        // - Send notifications via email/SMS/Discord
        // - Write to logging systems
        // - Trigger automated remediation
        // - Update dashboards
        
        Ok(())
    }

    fn print_metrics_summary(&self, metrics: &SystemMetrics) {
        println!("\n📊 SolCron Metrics Summary ({})", 
            TimeUtils::format_timestamp(metrics.timestamp));
        println!("════════════════════════════════");
        
        // Registry stats
        println!("🏛️  Registry:");
        println!("   Jobs: {}/{} active", metrics.registry_stats.active_jobs, metrics.registry_stats.total_jobs);
        println!("   Keepers: {}/{} active", metrics.registry_stats.active_keepers, metrics.registry_stats.total_keepers);
        println!("   Executions: {}/{} successful", 
            metrics.registry_stats.successful_executions, 
            metrics.registry_stats.total_executions);
        
        // Job stats
        println!("🎯 Jobs:");
        println!("   Avg Balance: {} SOL", Utils::lamports_to_sol_string(metrics.job_stats.avg_balance, 6));
        println!("   Success Rate: {:.1}%", metrics.job_stats.execution_success_rate);
        println!("   Avg Execution Time: {:.1}s", metrics.job_stats.avg_execution_time);
        
        // Keeper stats
        println!("🛡️  Keepers:");
        println!("   Avg Reputation: {:.0}/10000", metrics.keeper_stats.avg_reputation);
        println!("   Utilization: {:.1}%", metrics.keeper_stats.keeper_utilization);
        println!("   Total Stake: {} SOL", Utils::lamports_to_sol_string(metrics.keeper_stats.total_stake, 3));
        
        // Network stats
        println!("🌐 Network:");
        println!("   Avg TX Time: {:.1}s", metrics.network_stats.avg_transaction_time);
        println!("   Congestion: {:.1}%", metrics.network_stats.network_congestion);
        
        println!();
    }

    /// Get historical metrics for trend analysis
    pub fn get_historical_metrics(&self, start_time: u64, end_time: u64) -> Vec<&MetricsDataPoint> {
        self.metrics_history
            .iter()
            .filter(|point| point.timestamp >= start_time && point.timestamp <= end_time)
            .collect()
    }

    /// Get current alerts
    pub fn get_current_alerts(&self) -> &[AlertType] {
        &self.alerts
    }

    /// Generate a comprehensive analytics report
    pub fn generate_analytics_report(&self, hours: u64) -> AnalyticsReport {
        let end_time = Utils::current_timestamp();
        let start_time = end_time - (hours * 3600);
        
        let historical_data = self.get_historical_metrics(start_time, end_time);
        
        AnalyticsReport::generate(historical_data, hours)
    }

    /// Update job cache (call this periodically or when jobs change)
    pub async fn update_job_cache(&mut self, job_ids: Vec<u64>) -> SolCronResult<()> {
        for job_id in job_ids {
            match self.client.get_job(job_id).await {
                Ok(job) => {
                    self.job_cache.insert(job_id, job);
                },
                Err(_) => {
                    // Job might have been deleted
                    self.job_cache.remove(&job_id);
                }
            }
        }
        Ok(())
    }

    /// Update keeper cache
    pub async fn update_keeper_cache(&mut self, keeper_addresses: Vec<Pubkey>) -> SolCronResult<()> {
        for address in keeper_addresses {
            match self.client.get_keeper(&address).await {
                Ok(keeper) => {
                    self.keeper_cache.insert(address, keeper);
                },
                Err(_) => {
                    // Keeper might have been removed
                    self.keeper_cache.remove(&address);
                }
            }
        }
        Ok(())
    }
}

/// Comprehensive analytics report
#[derive(Debug)]
pub struct AnalyticsReport {
    pub period_hours: u64,
    pub total_data_points: usize,
    pub job_trends: JobTrends,
    pub keeper_trends: KeeperTrends,
    pub system_health: SystemHealth,
    pub recommendations: Vec<String>,
}

impl AnalyticsReport {
    pub fn generate(data_points: Vec<&MetricsDataPoint>, hours: u64) -> Self {
        let mut report = Self {
            period_hours: hours,
            total_data_points: data_points.len(),
            job_trends: JobTrends::default(),
            keeper_trends: KeeperTrends::default(),
            system_health: SystemHealth::default(),
            recommendations: Vec::new(),
        };
        
        if !data_points.is_empty() {
            report.analyze_trends(&data_points);
            report.generate_recommendations();
        }
        
        report
    }
    
    fn analyze_trends(&mut self, data_points: &[&MetricsDataPoint]) {
        // Analyze job trends
        let job_counts: Vec<u64> = data_points.iter()
            .map(|point| point.metrics.registry_stats.active_jobs)
            .collect();
            
        if job_counts.len() >= 2 {
            let growth_rate = ((job_counts.last().unwrap() - job_counts.first().unwrap()) as f64 
                / job_counts.first().unwrap() as f64) * 100.0;
            self.job_trends.growth_rate = growth_rate;
        }
        
        // Analyze success rates
        let success_rates: Vec<f64> = data_points.iter()
            .map(|point| point.metrics.job_stats.execution_success_rate)
            .collect();
            
        if !success_rates.is_empty() {
            self.system_health.avg_success_rate = success_rates.iter().sum::<f64>() / success_rates.len() as f64;
            self.system_health.min_success_rate = success_rates.iter().cloned().fold(f64::INFINITY, f64::min);
            self.system_health.max_success_rate = success_rates.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        }
        
        // Calculate system health score (0-100)
        self.system_health.health_score = self.calculate_health_score();
    }
    
    fn calculate_health_score(&self) -> u8 {
        let mut score = 100u8;
        
        // Penalize low success rate
        if self.system_health.avg_success_rate < 95.0 {
            score = score.saturating_sub(10);
        }
        
        // Penalize negative growth
        if self.job_trends.growth_rate < -5.0 {
            score = score.saturating_sub(15);
        }
        
        score
    }
    
    fn generate_recommendations(&mut self) {
        if self.system_health.avg_success_rate < 95.0 {
            self.recommendations.push("Consider investigating execution failures and optimizing job parameters".to_string());
        }
        
        if self.job_trends.growth_rate > 20.0 {
            self.recommendations.push("High job growth detected - consider scaling keeper infrastructure".to_string());
        }
        
        if self.system_health.health_score < 80 {
            self.recommendations.push("System health below optimal - review monitoring alerts and take corrective action".to_string());
        }
    }
    
    pub fn print_report(&self) {
        println!("\n📈 SolCron Analytics Report ({} hours)", self.period_hours);
        println!("═══════════════════════════════════");
        
        println!("📊 Data Points: {}", self.total_data_points);
        println!("🎯 Job Growth Rate: {:.1}%", self.job_trends.growth_rate);
        println!("✅ Avg Success Rate: {:.1}%", self.system_health.avg_success_rate);
        println!("🏥 System Health: {}/100", self.system_health.health_score);
        
        if !self.recommendations.is_empty() {
            println!("\n💡 Recommendations:");
            for (i, rec) in self.recommendations.iter().enumerate() {
                println!("  {}. {}", i + 1, rec);
            }
        }
        
        println!();
    }
}

#[derive(Debug, Default)]
pub struct JobTrends {
    pub growth_rate: f64,
    pub avg_execution_frequency: f64,
    pub balance_trend: f64,
}

#[derive(Debug, Default)]
pub struct KeeperTrends {
    pub reputation_trend: f64,
    pub stake_growth: f64,
    pub activity_trend: f64,
}

#[derive(Debug, Default)]
pub struct SystemHealth {
    pub health_score: u8,
    pub avg_success_rate: f64,
    pub min_success_rate: f64,
    pub max_success_rate: f64,
}