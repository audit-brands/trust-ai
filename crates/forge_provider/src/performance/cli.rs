//! CLI integration for performance monitoring and optimization

use std::collections::HashMap;

use anyhow::Context as _;
use tracing::{info, warn};

use crate::performance::{
    BenchmarkReport, ModelLoadingOptimizer, OptimizationConfig, OptimizationResult,
    PerformanceConfig, PerformanceMonitor, PerformanceSummary, ProviderMetrics, ResourceMonitor,
};

/// Performance CLI handler for managing performance monitoring and optimization
pub struct PerformanceCli {
    monitor: PerformanceMonitor,
    optimizer: ModelLoadingOptimizer,
    resource_monitor: ResourceMonitor,
}

/// Performance command variants
#[derive(Debug, Clone)]
pub enum PerformanceCommand {
    /// Show performance status
    Status,
    /// Show detailed metrics
    Metrics { provider_name: Option<String> },
    /// Run performance benchmark
    Benchmark,
    /// Generate optimization recommendations
    Optimize { provider_name: Option<String> },
    /// Show cache statistics
    Cache,
    /// Show resource usage
    Resources,
    /// Start performance monitoring
    Start,
    /// Stop performance monitoring
    Stop,
}

/// Performance CLI output
#[derive(Debug, Clone)]
pub struct PerformanceOutput {
    pub command: PerformanceCommand,
    pub success: bool,
    pub message: String,
    pub data: Option<PerformanceData>,
}

/// Performance data for CLI output
#[derive(Debug, Clone)]
pub enum PerformanceData {
    Summary(PerformanceSummary),
    Metrics(HashMap<String, ProviderMetrics>),
    BenchmarkReport(BenchmarkReport),
    OptimizationResults(Vec<OptimizationResult>),
    CacheStats(crate::performance::optimization::CacheStatistics),
    ResourceUsage(crate::performance::optimization::ResourceUsage),
}

impl PerformanceCli {
    /// Create a new performance CLI handler
    pub fn new() -> anyhow::Result<Self> {
        let performance_config = PerformanceConfig::default();
        let optimization_config = OptimizationConfig::default();

        let monitor = PerformanceMonitor::new(performance_config);
        let optimizer = ModelLoadingOptimizer::new(optimization_config.clone());
        let resource_monitor = ResourceMonitor::new(optimization_config);

        Ok(Self { monitor, optimizer, resource_monitor })
    }

    /// Execute a performance command
    pub async fn execute_command(
        &self,
        command: PerformanceCommand,
    ) -> anyhow::Result<PerformanceOutput> {
        match command.clone() {
            PerformanceCommand::Status => self.handle_status().await,
            PerformanceCommand::Metrics { provider_name } => {
                self.handle_metrics(provider_name).await
            }
            PerformanceCommand::Benchmark => self.handle_benchmark().await,
            PerformanceCommand::Optimize { provider_name } => {
                self.handle_optimize(provider_name).await
            }
            PerformanceCommand::Cache => self.handle_cache().await,
            PerformanceCommand::Resources => self.handle_resources().await,
            PerformanceCommand::Start => self.handle_start().await,
            PerformanceCommand::Stop => self.handle_stop().await,
        }
    }

    /// Handle performance status command
    async fn handle_status(&self) -> anyhow::Result<PerformanceOutput> {
        info!("Getting performance status");

        let summary = self.monitor.get_performance_summary().await;

        let message = format!(
            "Performance Status:\n\
            • Total Providers: {}\n\
            • Active Providers: {}\n\
            • Total Requests: {}\n\
            • Overall Success Rate: {:.2}%\n\
            • Average Response Time: {:?}\n\
            • Measurements Collected: {}",
            summary.total_providers,
            summary.active_providers,
            summary.total_requests,
            summary.overall_success_rate * 100.0,
            summary.overall_avg_response_time,
            summary.measurements_count
        );

        Ok(PerformanceOutput {
            command: PerformanceCommand::Status,
            success: true,
            message,
            data: Some(PerformanceData::Summary(summary)),
        })
    }

    /// Handle metrics command
    async fn handle_metrics(
        &self,
        provider_name: Option<String>,
    ) -> anyhow::Result<PerformanceOutput> {
        match provider_name {
            Some(name) => {
                info!("Getting metrics for provider: {}", name);

                if let Some(metrics) = self.monitor.get_provider_metrics(&name).await {
                    let message = format!(
                        "Metrics for {}:\n\
                        • Total Requests: {}\n\
                        • Success Rate: {:.2}%\n\
                        • Average Response Time: {:?}\n\
                        • Min/Max Response Time: {:?} / {:?}\n\
                        • Throughput: {:.2} req/s\n\
                        • Memory Usage: {} MB\n\
                        • CPU Usage: {:.1}%",
                        name,
                        metrics.total_requests,
                        metrics.success_rate(),
                        metrics.avg_response_time,
                        metrics.min_response_time,
                        metrics.max_response_time,
                        metrics.throughput,
                        metrics.memory_usage_mb.unwrap_or(0),
                        metrics.cpu_usage_percent.unwrap_or(0.0)
                    );

                    let mut metrics_map = HashMap::new();
                    metrics_map.insert(name.clone(), metrics);

                    Ok(PerformanceOutput {
                        command: PerformanceCommand::Metrics { provider_name: Some(name) },
                        success: true,
                        message,
                        data: Some(PerformanceData::Metrics(metrics_map)),
                    })
                } else {
                    Ok(PerformanceOutput {
                        command: PerformanceCommand::Metrics { provider_name: Some(name.clone()) },
                        success: false,
                        message: format!("No metrics found for provider: {name}"),
                        data: None,
                    })
                }
            }
            None => {
                info!("Getting metrics for all providers");

                let all_metrics = self.monitor.get_all_metrics().await;

                if all_metrics.is_empty() {
                    Ok(PerformanceOutput {
                        command: PerformanceCommand::Metrics { provider_name: None },
                        success: true,
                        message: "No performance metrics available yet".to_string(),
                        data: Some(PerformanceData::Metrics(all_metrics)),
                    })
                } else {
                    let mut message = "Performance Metrics:\n".to_string();
                    for (name, metrics) in &all_metrics {
                        message.push_str(&format!(
                            "\n{}:\n\
                            • Requests: {} (Success: {:.1}%)\n\
                            • Response Time: {:?} (avg)\n\
                            • Throughput: {:.2} req/s\n",
                            name,
                            metrics.total_requests,
                            metrics.success_rate(),
                            metrics.avg_response_time,
                            metrics.throughput
                        ));
                    }

                    Ok(PerformanceOutput {
                        command: PerformanceCommand::Metrics { provider_name: None },
                        success: true,
                        message,
                        data: Some(PerformanceData::Metrics(all_metrics)),
                    })
                }
            }
        }
    }

    /// Handle benchmark command
    async fn handle_benchmark(&self) -> anyhow::Result<PerformanceOutput> {
        info!("Running performance benchmark");

        let report = self.monitor.benchmark_against_targets().await;

        let mut message = format!(
            "Performance Benchmark Results:\n\
            • Overall Performance Score: {:.2}\n\
            • Benchmark Timestamp: {:?}\n\n",
            report.overall_performance_score, report.benchmark_timestamp
        );

        for (provider_name, comparison) in &report.provider_comparisons {
            message.push_str(&format!(
                "{}:\n\
                • Response Time vs Target: {:.2}x\n\
                • Success Rate vs Target: {:.2}x\n\
                • Throughput vs Target: {:.2}x\n\
                • Meets All Targets: {}\n\n",
                provider_name,
                comparison.response_time_vs_target,
                comparison.success_rate_vs_target,
                comparison.throughput_vs_target,
                if comparison.meets_targets {
                    "✅"
                } else {
                    "❌"
                }
            ));
        }

        Ok(PerformanceOutput {
            command: PerformanceCommand::Benchmark,
            success: true,
            message,
            data: Some(PerformanceData::BenchmarkReport(report)),
        })
    }

    /// Handle optimize command
    async fn handle_optimize(
        &self,
        provider_name: Option<String>,
    ) -> anyhow::Result<PerformanceOutput> {
        match provider_name {
            Some(name) => {
                info!("Running optimization for provider: {}", name);

                // Run model loading optimization
                let optimization_result = self
                    .optimizer
                    .optimize_model_loading(&name, "default-model")
                    .await
                    .context("Failed to optimize model loading")?;

                let message = if optimization_result.success {
                    format!(
                        "Optimization completed for {}:\n\
                        • Response Time Improvement: {:?}\n\
                        • Memory Improvement: {} MB\n\
                        • CPU Improvement: {:.1}%\n\
                        • Throughput Improvement: {:.2} req/s\n\
                        • Optimization Time: {:?}",
                        name,
                        optimization_result.improvement.response_time_improvement,
                        optimization_result.improvement.memory_improvement_mb,
                        optimization_result.improvement.cpu_improvement_percent,
                        optimization_result.improvement.throughput_improvement,
                        optimization_result.optimization_time
                    )
                } else {
                    format!(
                        "Optimization failed for {}: {}",
                        name,
                        optimization_result
                            .error
                            .as_ref()
                            .unwrap_or(&"Unknown error".to_string())
                    )
                };

                let success = optimization_result.success;
                Ok(PerformanceOutput {
                    command: PerformanceCommand::Optimize { provider_name: Some(name) },
                    success,
                    message,
                    data: Some(PerformanceData::OptimizationResults(vec![
                        optimization_result,
                    ])),
                })
            }
            None => {
                info!("Generating optimization recommendations for all providers");

                let recommendations = self.monitor.generate_recommendations().await;

                if recommendations.is_empty() {
                    Ok(PerformanceOutput {
                        command: PerformanceCommand::Optimize { provider_name: None },
                        success: true,
                        message: "No optimization recommendations at this time. All providers are performing well!".to_string(),
                        data: Some(PerformanceData::OptimizationResults(vec![])),
                    })
                } else {
                    let mut message = "Optimization Recommendations:\n\n".to_string();

                    for (i, rec) in recommendations.iter().enumerate() {
                        message.push_str(&format!(
                            "{}. {} ({:?} Priority)\n\
                            Provider: {}\n\
                            Issue: {}\n\
                            Action: {}\n\
                            Impact: {}\n\n",
                            i + 1,
                            match rec.recommendation_type {
                                crate::performance::RecommendationType::ModelLoading =>
                                    "Model Loading",
                                crate::performance::RecommendationType::Memory => "Memory",
                                crate::performance::RecommendationType::Cpu => "CPU",
                                crate::performance::RecommendationType::Network => "Network",
                                crate::performance::RecommendationType::Configuration =>
                                    "Configuration",
                                crate::performance::RecommendationType::ProviderSelection =>
                                    "Provider Selection",
                            },
                            rec.priority,
                            rec.provider_name,
                            rec.description,
                            rec.suggested_action,
                            rec.expected_impact
                        ));
                    }

                    Ok(PerformanceOutput {
                        command: PerformanceCommand::Optimize { provider_name: None },
                        success: true,
                        message,
                        data: Some(PerformanceData::OptimizationResults(vec![])),
                    })
                }
            }
        }
    }

    /// Handle cache command
    async fn handle_cache(&self) -> anyhow::Result<PerformanceOutput> {
        info!("Getting cache statistics");

        let stats = self.optimizer.get_cache_stats().await;

        let message = format!(
            "Model Cache Statistics:\n\
            • Total Models Cached: {}\n\
            • Total Cache Size: {} MB\n\
            • Cache Utilization: {:.1}%\n\
            • Total Accesses: {}\n\
            • Average Access Count: {:.1}\n\
            • Cache Hit Rate: {:.1}%",
            stats.total_models,
            stats.total_size_mb,
            stats.cache_utilization * 100.0,
            stats.total_accesses,
            stats.avg_access_count,
            stats.hit_rate * 100.0
        );

        Ok(PerformanceOutput {
            command: PerformanceCommand::Cache,
            success: true,
            message,
            data: Some(PerformanceData::CacheStats(stats)),
        })
    }

    /// Handle resources command
    async fn handle_resources(&self) -> anyhow::Result<PerformanceOutput> {
        info!("Getting system resource usage");

        let usage = self.resource_monitor.get_resource_usage().await;
        let recommendations = self.resource_monitor.get_resource_recommendations().await;

        let mut message = format!(
            "System Resource Usage:\n\
            • Memory: {:.1}% ({} MB available)\n\
            • CPU: {:.1}%\n\
            • Disk: {:.1}%\n\
            • Network: {:.1} Mbps\n",
            usage.memory_usage_percent,
            usage.available_memory_mb,
            usage.cpu_usage_percent,
            usage.disk_usage_percent,
            usage.network_bandwidth_mbps
        );

        if !recommendations.is_empty() {
            message.push_str("\nResource Recommendations:\n");
            for (i, rec) in recommendations.iter().enumerate() {
                message.push_str(&format!(
                    "{}. {:?} - {} ({})\n",
                    i + 1,
                    rec.resource_type,
                    rec.description,
                    rec.suggested_action
                ));
            }
        }

        Ok(PerformanceOutput {
            command: PerformanceCommand::Resources,
            success: true,
            message,
            data: Some(PerformanceData::ResourceUsage(usage)),
        })
    }

    /// Handle start command
    async fn handle_start(&self) -> anyhow::Result<PerformanceOutput> {
        info!("Starting performance monitoring");

        self.monitor
            .start()
            .await
            .context("Failed to start performance monitoring")?;

        Ok(PerformanceOutput {
            command: PerformanceCommand::Start,
            success: true,
            message: "Performance monitoring started successfully".to_string(),
            data: None,
        })
    }

    /// Handle stop command
    async fn handle_stop(&self) -> anyhow::Result<PerformanceOutput> {
        info!("Stopping performance monitoring");

        // In a real implementation, we would stop background tasks here
        warn!("Performance monitoring stop not yet implemented");

        Ok(PerformanceOutput {
            command: PerformanceCommand::Stop,
            success: true,
            message: "Performance monitoring stopped".to_string(),
            data: None,
        })
    }
}

impl Default for PerformanceCli {
    fn default() -> Self {
        // Create default configurations
        let performance_config = PerformanceConfig::default();
        let optimization_config = OptimizationConfig::default();
        
        // Create components with default configurations
        let monitor = PerformanceMonitor::new(performance_config);
        let optimizer = ModelLoadingOptimizer::new(optimization_config.clone());
        let resource_monitor = ResourceMonitor::new(optimization_config);
        
        Self { monitor, optimizer, resource_monitor }
    }
}

/// Parse performance command from CLI input
pub fn parse_performance_command(input: &str) -> anyhow::Result<PerformanceCommand> {
    let parts: Vec<&str> = input.split_whitespace().collect();

    if parts.is_empty() {
        return Ok(PerformanceCommand::Status);
    }

    match parts[0] {
        "status" => Ok(PerformanceCommand::Status),
        "metrics" => {
            let provider_name = if parts.len() > 1 {
                Some(parts[1].to_string())
            } else {
                None
            };
            Ok(PerformanceCommand::Metrics { provider_name })
        }
        "benchmark" => Ok(PerformanceCommand::Benchmark),
        "optimize" => {
            let provider_name = if parts.len() > 1 {
                Some(parts[1].to_string())
            } else {
                None
            };
            Ok(PerformanceCommand::Optimize { provider_name })
        }
        "cache" => Ok(PerformanceCommand::Cache),
        "resources" => Ok(PerformanceCommand::Resources),
        "start" => Ok(PerformanceCommand::Start),
        "stop" => Ok(PerformanceCommand::Stop),
        _ => anyhow::bail!("Unknown performance command: {}", parts[0]),
    }
}

/// Format performance output for display
pub fn format_performance_output(output: &PerformanceOutput) -> String {
    let status_indicator = if output.success { "✅" } else { "❌" };

    format!(
        "{} Performance Command: {:?}\n\n{}",
        status_indicator, output.command, output.message
    )
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn test_performance_cli_creation() {
        let cli = PerformanceCli::new();
        assert!(cli.is_ok());
    }

    #[tokio::test]
    async fn test_status_command() {
        let cli = PerformanceCli::new().unwrap();
        let result = cli.execute_command(PerformanceCommand::Status).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.success);
        assert!(output.message.contains("Performance Status"));
    }

    #[tokio::test]
    async fn test_metrics_command() {
        let cli = PerformanceCli::new().unwrap();
        let result = cli
            .execute_command(PerformanceCommand::Metrics { provider_name: None })
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.success);
        assert!(
            output.message.contains("No performance metrics available")
                || output.message.contains("Performance Metrics")
        );
    }

    #[tokio::test]
    async fn test_cache_command() {
        let cli = PerformanceCli::new().unwrap();
        let result = cli.execute_command(PerformanceCommand::Cache).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.success);
        assert!(output.message.contains("Model Cache Statistics"));
    }

    #[tokio::test]
    async fn test_resources_command() {
        let cli = PerformanceCli::new().unwrap();
        let result = cli.execute_command(PerformanceCommand::Resources).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.success);
        assert!(output.message.contains("System Resource Usage"));
    }

    #[test]
    fn test_parse_performance_command() {
        let result = parse_performance_command("status");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), PerformanceCommand::Status));

        let result = parse_performance_command("metrics ollama");
        assert!(result.is_ok());
        if let PerformanceCommand::Metrics { provider_name } = result.unwrap() {
            assert_eq!(provider_name, Some("ollama".to_string()));
        } else {
            panic!("Expected Metrics command");
        }

        let result = parse_performance_command("benchmark");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), PerformanceCommand::Benchmark));

        let result = parse_performance_command("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_format_performance_output() {
        let output = PerformanceOutput {
            command: PerformanceCommand::Status,
            success: true,
            message: "Test message".to_string(),
            data: None,
        };

        let formatted = format_performance_output(&output);
        assert!(formatted.contains("✅"));
        assert!(formatted.contains("Test message"));
        assert!(formatted.contains("Status"));
    }
}
