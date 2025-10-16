//! Example: Advanced monitoring and analytics dashboard
//!
//! This example shows how to build a comprehensive monitoring system
//! for SolCron using the advanced monitoring and analytics capabilities.

use solcron_sdk::{
    SolCronClient, Monitor, MonitoringConfig, AlertThresholds,
    Simulator, SimulationConfig, BatchOperations, BatchConfig,
    types::TriggerType, utils::Utils, error::SolCronResult,
};
use solana_sdk::{signature::Keypair, pubkey::Pubkey};
use tokio::{time::{sleep, Duration}, select};
use std::sync::Arc;

/// Configuration for the analytics dashboard
#[derive(Debug)]
struct DashboardConfig {
    /// Solana cluster RPC URL
    cluster_url: String,
    /// How often to update analytics (seconds)
    refresh_interval: u64,
    /// Enable simulation mode for demo data
    simulation_mode: bool,
    /// Job IDs to monitor (empty = monitor all)
    monitored_jobs: Vec<u64>,
    /// Keeper addresses to monitor (empty = monitor all)
    monitored_keepers: Vec<Pubkey>,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            cluster_url: "https://api.devnet.solana.com".to_string(),
            refresh_interval: 30,
            simulation_mode: true, // For demo purposes
            monitored_jobs: vec![],
            monitored_keepers: vec![],
        }
    }
}

/// Advanced analytics dashboard for SolCron
struct AnalyticsDashboard {
    client: SolCronClient,
    monitor: Monitor,
    batch_ops: BatchOperations,
    config: DashboardConfig,
    simulator: Option<Simulator>,
}

impl AnalyticsDashboard {
    /// Create a new analytics dashboard
    async fn new(config: DashboardConfig) -> SolCronResult<Self> {
        println!("🚀 Initializing SolCron Analytics Dashboard");
        println!("==========================================");
        
        // Initialize client
        let payer = Keypair::new(); // In production, load from secure storage
        let client = SolCronClient::new_with_payer(
            &config.cluster_url,
            payer,
            None,
        ).await?;
        
        // Configure monitoring with custom alert thresholds
        let monitoring_config = MonitoringConfig {
            collection_interval: 30,
            history_retention: 2880, // 24 hours of 30-second intervals
            alert_thresholds: AlertThresholds {
                min_job_balance: Utils::sol_to_lamports(0.001),
                max_failure_rate: 5.0, // 5% max failure rate
                max_execution_time: 15, // 15 seconds max execution time
                min_keeper_reputation: 8500, // 85% minimum reputation
            },
            verbose_logging: true,
        };
        
        let monitor = Monitor::new(client.clone(), Some(monitoring_config));
        
        // Initialize batch operations
        let batch_config = BatchConfig {
            max_ops_per_tx: 8,
            max_concurrent_txs: 3,
            retry_attempts: 2,
            retry_delay_ms: 500,
        };
        let batch_ops = BatchOperations::new(client.clone(), Some(batch_config));
        
        // Initialize simulator if in simulation mode
        let simulator = if config.simulation_mode {
            let sim_config = SimulationConfig {
                duration_seconds: 7200, // 2 hours
                job_count: 50,
                keeper_count: 8,
                base_interval: 180, // 3 minutes
                failure_rate: 3.0, // 3% failure rate
                network_latency_ms: 75,
                verbose: false,
            };
            Some(Simulator::new(client.clone(), sim_config))
        } else {
            None
        };
        
        println!("✅ Dashboard initialized");
        println!("🌐 Cluster: {}", config.cluster_url);
        println!("📊 Monitoring enabled with {} second intervals", 
            monitoring_config.collection_interval);
        
        if config.simulation_mode {
            println!("🎬 Simulation mode enabled for demo data");
        }
        
        Ok(Self {
            client,
            monitor,
            batch_ops,
            config,
            simulator,
        })
    }
    
    /// Start the analytics dashboard
    async fn start(&mut self) -> SolCronResult<()> {
        println!("\n🎯 Starting Analytics Dashboard...");
        
        // Start simulation if enabled
        if let Some(simulator) = &self.simulator {
            println!("🎬 Running simulation for demo data...");
            
            tokio::spawn({
                let sim = simulator.clone();
                async move {
                    if let Err(e) = sim.run_simulation().await {
                        println!("❌ Simulation failed: {}", e);
                    }
                }
            });
            
            // Give simulation time to generate some data
            sleep(Duration::from_secs(10)).await;
        }
        
        // Main dashboard loop
        loop {
            let cycle_start = std::time::Instant::now();
            
            select! {
                // Main analytics cycle
                _ = self.analytics_cycle() => {
                    // Continue loop
                }
                
                // Handle Ctrl+C
                _ = tokio::signal::ctrl_c() => {
                    println!("\n👋 Shutting down analytics dashboard...");
                    self.generate_final_report().await?;
                    break;
                }
            }
            
            // Maintain refresh interval
            let elapsed = cycle_start.elapsed();
            if elapsed < Duration::from_secs(self.config.refresh_interval) {
                let sleep_duration = Duration::from_secs(self.config.refresh_interval) - elapsed;
                sleep(sleep_duration).await;
            }
        }
        
        Ok(())
    }
    
    /// Main analytics collection and reporting cycle
    async fn analytics_cycle(&mut self) -> SolCronResult<()> {
        println!("\n🔄 Running analytics cycle...");
        
        // 1. Collect current metrics
        let current_metrics = match self.monitor.collect_metrics().await {
            Ok(metrics) => {
                self.display_current_metrics(&metrics);
                metrics
            }
            Err(e) => {
                println!("❌ Failed to collect metrics: {}", e);
                return Ok(());
            }
        };
        
        // 2. Check for alerts
        let alerts = self.monitor.get_current_alerts();
        if !alerts.is_empty() {
            self.display_alerts(alerts);
        }
        
        // 3. Generate trend analysis every 5 minutes
        static mut LAST_TREND_ANALYSIS: u64 = 0;
        let current_time = Utils::current_timestamp();
        
        unsafe {
            if current_time - LAST_TREND_ANALYSIS >= 300 { // 5 minutes
                self.generate_trend_analysis().await?;
                LAST_TREND_ANALYSIS = current_time;
            }
        }
        
        // 4. Batch operations analysis every 10 minutes
        static mut LAST_BATCH_ANALYSIS: u64 = 0;
        
        unsafe {
            if current_time - LAST_BATCH_ANALYSIS >= 600 { // 10 minutes
                self.run_batch_analysis().await?;
                LAST_BATCH_ANALYSIS = current_time;
            }
        }
        
        // 5. Performance optimization suggestions
        self.suggest_optimizations(&current_metrics).await?;
        
        Ok(())
    }
    
    /// Display current system metrics in a dashboard format
    fn display_current_metrics(&self, metrics: &solcron_sdk::monitoring::SystemMetrics) {
        println!("\n📊 SolCron Real-Time Dashboard");
        println!("═════════════════════════════════════════════════════════");
        
        // System Overview
        println!("🏛️  SYSTEM OVERVIEW");
        println!("   ├── Registry Status: {} jobs ({} active), {} keepers ({} active)",
            metrics.registry_stats.total_jobs,
            metrics.registry_stats.active_jobs,
            metrics.registry_stats.total_keepers,
            metrics.registry_stats.active_keepers
        );
        
        let success_rate = if metrics.registry_stats.total_executions > 0 {
            (metrics.registry_stats.successful_executions as f64 / 
             metrics.registry_stats.total_executions as f64) * 100.0
        } else {
            100.0
        };
        
        println!("   ├── Execution Rate: {:.1}% ({}/{})",
            success_rate,
            metrics.registry_stats.successful_executions,
            metrics.registry_stats.total_executions
        );
        
        println!("   └── Protocol Revenue: {} SOL",
            Utils::lamports_to_sol_string(metrics.registry_stats.protocol_revenue, 6)
        );
        
        // Job Metrics
        println!("\n🎯 JOB METRICS");
        println!("   ├── Average Balance: {} SOL",
            Utils::lamports_to_sol_string(metrics.job_stats.avg_balance, 6)
        );
        println!("   ├── Total Balance: {} SOL",
            Utils::lamports_to_sol_string(metrics.job_stats.total_balance, 3)
        );
        println!("   ├── Low Balance Jobs: {}", metrics.job_stats.low_balance_count);
        println!("   ├── Success Rate: {:.1}%", metrics.job_stats.execution_success_rate);
        println!("   └── Avg Execution Time: {:.1}s", metrics.job_stats.avg_execution_time);
        
        // Trigger Type Distribution
        if !metrics.job_stats.trigger_type_distribution.is_empty() {
            println!("\n   📈 Trigger Distribution:");
            for (trigger_type, count) in &metrics.job_stats.trigger_type_distribution {
                let percentage = (*count as f64 / metrics.registry_stats.total_jobs as f64) * 100.0;
                println!("      ├── {}: {} ({:.1}%)", trigger_type, count, percentage);
            }
        }
        
        // Keeper Metrics
        println!("\n🛡️  KEEPER METRICS");
        println!("   ├── Average Reputation: {:.0}/10000", metrics.keeper_stats.avg_reputation);
        println!("   ├── Total Stake: {} SOL",
            Utils::lamports_to_sol_string(metrics.keeper_stats.total_stake, 3)
        );
        println!("   ├── Average Stake: {} SOL",
            Utils::lamports_to_sol_string(metrics.keeper_stats.avg_stake, 6)
        );
        println!("   └── Utilization: {:.1}%", metrics.keeper_stats.keeper_utilization);
        
        // Top Performers
        if !metrics.keeper_stats.top_performers.is_empty() {
            println!("\n   🏆 Top Performing Keepers:");
            for (i, performer) in metrics.keeper_stats.top_performers.iter().take(3).enumerate() {
                println!("      {}. {} - {}/10000 reputation, {:.1}% success, {} SOL earned",
                    i + 1,
                    &performer.keeper_address[..8],
                    performer.reputation_score,
                    performer.success_rate,
                    Utils::lamports_to_sol_string(performer.earnings, 4)
                );
            }
        }
        
        // Network Metrics
        println!("\n🌐 NETWORK METRICS");
        println!("   ├── Avg Transaction Time: {:.1}s", metrics.network_stats.avg_transaction_time);
        println!("   ├── Network Congestion: {:.1}%", metrics.network_stats.network_congestion);
        println!("   └── Gas Price Trend: {:.1}x", metrics.network_stats.gas_price_trend);
        
        println!("═════════════════════════════════════════════════════════");
    }
    
    /// Display current alerts
    fn display_alerts(&self, alerts: &[solcron_sdk::monitoring::AlertType]) {
        println!("\n🚨 ACTIVE ALERTS ({}):", alerts.len());
        
        for (i, alert) in alerts.iter().enumerate() {
            let severity_icon = match alert.severity() {
                solcron_sdk::monitoring::AlertSeverity::Critical => "🔴",
                solcron_sdk::monitoring::AlertSeverity::Warning => "🟡",
                solcron_sdk::monitoring::AlertSeverity::Info => "🔵",
            };
            
            println!("   {}. {} {}", i + 1, severity_icon, alert.message());
        }
    }
    
    /// Generate comprehensive trend analysis
    async fn generate_trend_analysis(&self) -> SolCronResult<()> {
        println!("\n📈 Generating trend analysis...");
        
        // Generate 24-hour analytics report
        let report = self.monitor.generate_analytics_report(24);
        
        println!("📊 24-Hour Trend Analysis");
        println!("─────────────────────────");
        report.print_report();
        
        Ok(())
    }
    
    /// Run batch operations analysis
    async fn run_batch_analysis(&mut self) -> SolCronResult<()> {
        println!("🔍 Running batch analysis...");
        
        // Analyze jobs for optimization opportunities
        let job_ids: Vec<u64> = (1..=50).collect(); // Sample job IDs
        
        match self.batch_ops.analyze_jobs(job_ids).await {
            Ok(analysis) => {
                analysis.print_report();
            }
            Err(e) => {
                println!("❌ Batch analysis failed: {}", e);
            }
        }
        
        Ok(())
    }
    
    /// Suggest performance optimizations
    async fn suggest_optimizations(&self, metrics: &solcron_sdk::monitoring::SystemMetrics) -> SolCronResult<()> {
        let mut suggestions = Vec::new();
        
        // Check keeper to job ratio
        if metrics.registry_stats.active_keepers > 0 {
            let job_per_keeper_ratio = metrics.registry_stats.active_jobs as f64 / 
                metrics.registry_stats.active_keepers as f64;
            
            if job_per_keeper_ratio > 15.0 {
                suggestions.push("🔧 Consider onboarding more keepers - current ratio is high");
            } else if job_per_keeper_ratio < 3.0 {
                suggestions.push("💡 System may be over-provisioned with keepers");
            }
        }
        
        // Check success rates
        if metrics.job_stats.execution_success_rate < 95.0 {
            suggestions.push("⚠️  Investigate execution failures - success rate below target");
        }
        
        // Check balance health
        if metrics.job_stats.low_balance_count > 0 {
            suggestions.push(format!(
                "💰 {} jobs have low balances - consider implementing auto-topup",
                metrics.job_stats.low_balance_count
            ));
        }
        
        // Check execution times
        if metrics.job_stats.avg_execution_time > 10.0 {
            suggestions.push("⚡ Average execution time is high - investigate performance bottlenecks");
        }
        
        // Check network congestion
        if metrics.network_stats.network_congestion > 25.0 {
            suggestions.push("🌐 High network congestion detected - consider adjusting execution timing");
        }
        
        // Display suggestions
        if !suggestions.is_empty() {
            println!("\n💡 OPTIMIZATION SUGGESTIONS:");
            for (i, suggestion) in suggestions.iter().enumerate() {
                println!("   {}. {}", i + 1, suggestion);
            }
        }
        
        Ok(())
    }
    
    /// Generate final comprehensive report
    async fn generate_final_report(&self) -> SolCronResult<()> {
        println!("\n📋 Generating Final Analytics Report...");
        
        // 24-hour comprehensive report
        let report = self.monitor.generate_analytics_report(24);
        
        println!("\n" + "=".repeat(60).as_str());
        println!("           SOLCRON ANALYTICS FINAL REPORT");
        println!("=".repeat(60));
        
        report.print_report();
        
        // Historical metrics summary
        let end_time = Utils::current_timestamp();
        let start_time = end_time - 86400; // 24 hours ago
        let historical_data = self.monitor.get_historical_metrics(start_time, end_time);
        
        if !historical_data.is_empty() {
            println!("📊 Historical Data Summary:");
            println!("   ├── Data Points Collected: {}", historical_data.len());
            println!("   ├── Monitoring Period: {} hours", 
                (end_time - start_time) / 3600);
            
            // Calculate trends from historical data
            if historical_data.len() >= 2 {
                let first_point = &historical_data[0].metrics;
                let last_point = &historical_data[historical_data.len() - 1].metrics;
                
                let job_growth = if first_point.registry_stats.total_jobs > 0 {
                    ((last_point.registry_stats.total_jobs as f64 - 
                      first_point.registry_stats.total_jobs as f64) / 
                     first_point.registry_stats.total_jobs as f64) * 100.0
                } else {
                    0.0
                };
                
                println!("   ├── Job Growth: {:.1}%", job_growth);
                
                let execution_growth = if first_point.registry_stats.total_executions > 0 {
                    ((last_point.registry_stats.total_executions as f64 - 
                      first_point.registry_stats.total_executions as f64) / 
                     first_point.registry_stats.total_executions as f64) * 100.0
                } else {
                    0.0
                };
                
                println!("   └── Execution Growth: {:.1}%", execution_growth);
            }
        }
        
        println!("\n🎉 Analytics dashboard session complete!");
        println!("Thank you for using SolCron Analytics Dashboard");
        println!("=".repeat(60));
        
        Ok(())
    }
}

/// Main function demonstrating the analytics dashboard
#[tokio::main]
async fn main() -> SolCronResult<()> {
    env_logger::init();
    
    println!("🌟 Welcome to SolCron Analytics Dashboard");
    println!("=========================================");
    
    let config = DashboardConfig::default();
    
    let mut dashboard = AnalyticsDashboard::new(config).await?;
    
    println!("\n🎛️  Dashboard Controls:");
    println!("   • Press Ctrl+C to generate final report and exit");
    println!("   • Monitoring updates every 30 seconds");
    println!("   • Trend analysis every 5 minutes");
    println!("   • Batch analysis every 10 minutes");
    
    dashboard.start().await?;
    
    Ok(())
}

/// Additional utility functions for dashboard customization
impl AnalyticsDashboard {
    /// Add custom job monitoring
    pub fn add_monitored_job(&mut self, job_id: u64) {
        if !self.config.monitored_jobs.contains(&job_id) {
            self.config.monitored_jobs.push(job_id);
            println!("📌 Added job {} to monitoring list", job_id);
        }
    }
    
    /// Add custom keeper monitoring  
    pub fn add_monitored_keeper(&mut self, keeper_address: Pubkey) {
        if !self.config.monitored_keepers.contains(&keeper_address) {
            self.config.monitored_keepers.push(keeper_address);
            println!("📌 Added keeper {} to monitoring list", &keeper_address.to_string()[..8]);
        }
    }
    
    /// Export metrics to JSON format
    pub async fn export_metrics_json(&self, hours: u64) -> SolCronResult<String> {
        let end_time = Utils::current_timestamp();
        let start_time = end_time - (hours * 3600);
        let historical_data = self.monitor.get_historical_metrics(start_time, end_time);
        
        // In a real implementation, you would serialize to JSON
        // For this example, we'll return a formatted summary
        let json_summary = format!(
            r#"{{
  "period_hours": {},
  "total_data_points": {},
  "start_time": {},
  "end_time": {},
  "summary": "SolCron metrics export"
}}"#,
            hours,
            historical_data.len(),
            start_time,
            end_time
        );
        
        Ok(json_summary)
    }
}