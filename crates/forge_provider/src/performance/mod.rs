//! Performance monitoring and optimization for local AI providers

mod cli;
mod optimization;

pub use cli::*;
pub use optimization::*;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Context as _;
use derive_setters::Setters;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Performance metrics for a provider
#[derive(Debug, Clone, Serialize, Deserialize, Setters)]
#[setters(strip_option, into)]
pub struct ProviderMetrics {
    /// Provider name
    pub provider_name: String,
    /// Total requests made
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Average response time
    pub avg_response_time: Duration,
    /// Minimum response time
    pub min_response_time: Duration,
    /// Maximum response time
    pub max_response_time: Duration,
    /// 95th percentile response time
    pub p95_response_time: Duration,
    /// 99th percentile response time
    pub p99_response_time: Duration,
    /// Throughput (requests per second)
    pub throughput: f64,
    /// Model loading time (for local providers)
    pub model_loading_time: Option<Duration>,
    /// Memory usage (MB)
    pub memory_usage_mb: Option<u64>,
    /// CPU usage percentage
    pub cpu_usage_percent: Option<f64>,
    /// Last updated timestamp
    pub last_updated: Instant,
}

/// Performance measurement for a single request
#[derive(Debug, Clone)]
pub struct PerformanceMeasurement {
    /// Provider name
    pub provider_name: String,
    /// Request start time
    pub start_time: Instant,
    /// Request end time
    pub end_time: Instant,
    /// Whether the request was successful
    pub success: bool,
    /// Response size in bytes
    pub response_size_bytes: Option<usize>,
    /// Model used for the request
    pub model_name: Option<String>,
    /// Request type (inference, health_check, discovery)
    pub request_type: RequestType,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Type of request being measured
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestType {
    /// Model inference request
    Inference,
    /// Health check request
    HealthCheck,
    /// Model discovery request
    Discovery,
    /// Model loading operation
    ModelLoading,
}

/// Performance monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize, Setters)]
#[setters(strip_option, into)]
pub struct PerformanceConfig {
    /// Whether performance monitoring is enabled
    pub enabled: bool,
    /// Maximum number of measurements to keep in memory
    pub max_measurements: usize,
    /// Performance alert thresholds
    pub alert_thresholds: AlertThresholds,
    /// Benchmark targets
    pub benchmark_targets: BenchmarkTargets,
    /// Metrics collection interval
    pub collection_interval: Duration,
}

/// Alert thresholds for performance monitoring
#[derive(Debug, Clone, Serialize, Deserialize, Setters)]
#[setters(strip_option, into)]
pub struct AlertThresholds {
    /// Maximum acceptable response time
    pub max_response_time: Duration,
    /// Minimum acceptable success rate (0.0 to 1.0)
    pub min_success_rate: f64,
    /// Maximum acceptable memory usage (MB)
    pub max_memory_usage_mb: u64,
    /// Maximum acceptable CPU usage percentage
    pub max_cpu_usage_percent: f64,
    /// Minimum acceptable throughput (requests per second)
    pub min_throughput: f64,
}

/// Benchmark targets for comparison with cloud providers
#[derive(Debug, Clone, Serialize, Deserialize, Setters)]
#[setters(strip_option, into)]
pub struct BenchmarkTargets {
    /// Target response time to match cloud providers
    pub target_response_time: Duration,
    /// Target success rate
    pub target_success_rate: f64,
    /// Target throughput
    pub target_throughput: f64,
    /// Cloud provider baseline metrics for comparison
    pub cloud_baseline: Option<ProviderMetrics>,
}

/// Performance monitoring service
pub struct PerformanceMonitor {
    config: PerformanceConfig,
    metrics: Arc<RwLock<HashMap<String, ProviderMetrics>>>,
    measurements: Arc<RwLock<Vec<PerformanceMeasurement>>>,
}

/// Performance optimization recommendations
#[derive(Debug, Clone)]
pub struct OptimizationRecommendation {
    /// Provider name
    pub provider_name: String,
    /// Recommendation type
    pub recommendation_type: RecommendationType,
    /// Description of the issue
    pub description: String,
    /// Suggested action
    pub suggested_action: String,
    /// Expected impact
    pub expected_impact: String,
    /// Priority level
    pub priority: Priority,
}

/// Type of optimization recommendation
#[derive(Debug, Clone)]
pub enum RecommendationType {
    /// Model loading optimization
    ModelLoading,
    /// Memory optimization
    Memory,
    /// CPU optimization
    Cpu,
    /// Network optimization
    Network,
    /// Configuration optimization
    Configuration,
    /// Provider selection optimization
    ProviderSelection,
}

/// Priority level for recommendations
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new(config: PerformanceConfig) -> Self {
        Self {
            config,
            metrics: Arc::new(RwLock::new(HashMap::new())),
            measurements: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Start performance monitoring
    pub async fn start(&self) -> anyhow::Result<()> {
        if !self.config.enabled {
            info!("Performance monitoring is disabled");
            return Ok(());
        }

        info!("Starting performance monitoring");
        
        // Start metrics collection task
        self.start_metrics_collection().await;
        
        Ok(())
    }

    /// Start metrics collection background task
    async fn start_metrics_collection(&self) {
        let interval = self.config.collection_interval;
        let _metrics = Arc::clone(&self.metrics);
        let _measurements = Arc::clone(&self.measurements);

        // Note: In a real implementation, we would spawn this as a background task
        info!(
            "Would start metrics collection with interval {:?}",
            interval
        );
    }

    /// Record a performance measurement
    pub async fn record_measurement(&self, measurement: PerformanceMeasurement) {
        if !self.config.enabled {
            return;
        }

        debug!(
            "Recording measurement for {}: {:?} - {}ms",
            measurement.provider_name,
            measurement.request_type,
            measurement.duration().as_millis()
        );

        // Add measurement to history
        {
            let mut measurements = self.measurements.write().await;
            measurements.push(measurement.clone());

            // Limit measurements in memory
            if measurements.len() > self.config.max_measurements {
                measurements.remove(0);
            }
        }

        // Update provider metrics
        self.update_provider_metrics(&measurement).await;
    }

    /// Update provider metrics based on a new measurement
    async fn update_provider_metrics(&self, measurement: &PerformanceMeasurement) {
        let mut metrics = self.metrics.write().await;
        
        let provider_metrics = metrics
            .entry(measurement.provider_name.clone())
            .or_insert_with(|| ProviderMetrics::new(&measurement.provider_name));

        // Update counters
        provider_metrics.total_requests += 1;
        if measurement.success {
            provider_metrics.successful_requests += 1;
        } else {
            provider_metrics.failed_requests += 1;
        }

        let response_time = measurement.duration();

        // Update response time metrics
        if provider_metrics.total_requests == 1 {
            // First measurement
            provider_metrics.avg_response_time = response_time;
            provider_metrics.min_response_time = response_time;
            provider_metrics.max_response_time = response_time;
            provider_metrics.p95_response_time = response_time;
            provider_metrics.p99_response_time = response_time;
        } else {
            // Update running averages and extremes
            let total = provider_metrics.total_requests;
            let prev_avg = provider_metrics.avg_response_time;
            provider_metrics.avg_response_time = 
                Duration::from_nanos(
                    (prev_avg.as_nanos() * (total - 1) as u128 + response_time.as_nanos()) / total as u128
                );

            if response_time < provider_metrics.min_response_time {
                provider_metrics.min_response_time = response_time;
            }
            if response_time > provider_metrics.max_response_time {
                provider_metrics.max_response_time = response_time;
            }
        }

        // Calculate throughput (simplified)
        let time_window = Duration::from_secs(60); // 1 minute window
        provider_metrics.throughput = provider_metrics.total_requests as f64 / time_window.as_secs() as f64;

        provider_metrics.last_updated = Instant::now();
    }

    /// Get metrics for all providers
    pub async fn get_all_metrics(&self) -> HashMap<String, ProviderMetrics> {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }

    /// Get metrics for a specific provider
    pub async fn get_provider_metrics(&self, provider_name: &str) -> Option<ProviderMetrics> {
        let metrics = self.metrics.read().await;
        metrics.get(provider_name).cloned()
    }

    /// Get performance summary across all providers
    pub async fn get_performance_summary(&self) -> PerformanceSummary {
        let metrics = self.metrics.read().await;
        let measurements = self.measurements.read().await;

        let total_requests: u64 = metrics.values().map(|m| m.total_requests).sum();
        let total_successful: u64 = metrics.values().map(|m| m.successful_requests).sum();
        let overall_success_rate = if total_requests > 0 {
            total_successful as f64 / total_requests as f64
        } else {
            0.0
        };

        let avg_response_times: Vec<Duration> = metrics.values().map(|m| m.avg_response_time).collect();
        let overall_avg_response_time = if !avg_response_times.is_empty() {
            let total_nanos: u128 = avg_response_times.iter().map(|d| d.as_nanos()).sum();
            Duration::from_nanos(total_nanos / avg_response_times.len() as u128)
        } else {
            Duration::from_millis(0)
        };

        PerformanceSummary {
            total_providers: metrics.len(),
            total_requests,
            overall_success_rate,
            overall_avg_response_time,
            measurements_count: measurements.len(),
            active_providers: metrics.values().filter(|m| m.total_requests > 0).count(),
        }
    }

    /// Generate optimization recommendations
    pub async fn generate_recommendations(&self) -> Vec<OptimizationRecommendation> {
        let mut recommendations = Vec::new();
        let metrics = self.metrics.read().await;

        for (provider_name, provider_metrics) in metrics.iter() {
            // Check response time
            if provider_metrics.avg_response_time > self.config.alert_thresholds.max_response_time {
                recommendations.push(OptimizationRecommendation {
                    provider_name: provider_name.clone(),
                    recommendation_type: RecommendationType::Network,
                    description: format!(
                        "Average response time ({:?}) exceeds threshold ({:?})",
                        provider_metrics.avg_response_time,
                        self.config.alert_thresholds.max_response_time
                    ),
                    suggested_action: "Consider optimizing network configuration or switching to a faster provider".to_string(),
                    expected_impact: "Reduced response times and improved user experience".to_string(),
                    priority: Priority::High,
                });
            }

            // Check success rate
            let success_rate = if provider_metrics.total_requests > 0 {
                provider_metrics.successful_requests as f64 / provider_metrics.total_requests as f64
            } else {
                0.0
            };

            if success_rate < self.config.alert_thresholds.min_success_rate {
                recommendations.push(OptimizationRecommendation {
                    provider_name: provider_name.clone(),
                    recommendation_type: RecommendationType::ProviderSelection,
                    description: format!(
                        "Success rate ({:.2}%) is below threshold ({:.2}%)",
                        success_rate * 100.0,
                        self.config.alert_thresholds.min_success_rate * 100.0
                    ),
                    suggested_action: "Check provider health and consider fallback to alternative providers".to_string(),
                    expected_impact: "Improved reliability and reduced failure rates".to_string(),
                    priority: Priority::Critical,
                });
            }

            // Check memory usage
            if let Some(memory_usage) = provider_metrics.memory_usage_mb {
                if memory_usage > self.config.alert_thresholds.max_memory_usage_mb {
                    recommendations.push(OptimizationRecommendation {
                        provider_name: provider_name.clone(),
                        recommendation_type: RecommendationType::Memory,
                        description: format!(
                            "Memory usage ({}MB) exceeds threshold ({}MB)",
                            memory_usage,
                            self.config.alert_thresholds.max_memory_usage_mb
                        ),
                        suggested_action: "Consider using smaller models or optimizing memory allocation".to_string(),
                        expected_impact: "Reduced memory footprint and improved system stability".to_string(),
                        priority: Priority::Medium,
                    });
                }
            }

            // Check model loading time
            if let Some(loading_time) = provider_metrics.model_loading_time {
                if loading_time > Duration::from_secs(10) {
                    recommendations.push(OptimizationRecommendation {
                        provider_name: provider_name.clone(),
                        recommendation_type: RecommendationType::ModelLoading,
                        description: format!(
                            "Model loading time ({:?}) is slow",
                            loading_time
                        ),
                        suggested_action: "Consider model caching, preloading, or using faster storage".to_string(),
                        expected_impact: "Faster startup times and improved user experience".to_string(),
                        priority: Priority::Medium,
                    });
                }
            }
        }

        // Sort recommendations by priority
        recommendations.sort_by(|a, b| b.priority.cmp(&a.priority));
        recommendations
    }

    /// Compare performance against benchmark targets
    pub async fn benchmark_against_targets(&self) -> BenchmarkReport {
        let metrics = self.metrics.read().await;
        let mut provider_comparisons = HashMap::new();

        for (provider_name, provider_metrics) in metrics.iter() {
            let comparison = ProviderBenchmarkComparison {
                provider_name: provider_name.clone(),
                response_time_vs_target: self.compare_duration(
                    provider_metrics.avg_response_time,
                    self.config.benchmark_targets.target_response_time,
                ),
                success_rate_vs_target: self.compare_success_rate(
                    provider_metrics,
                    self.config.benchmark_targets.target_success_rate,
                ),
                throughput_vs_target: self.compare_throughput(
                    provider_metrics.throughput,
                    self.config.benchmark_targets.target_throughput,
                ),
                meets_targets: self.meets_all_targets(provider_metrics),
            };
            provider_comparisons.insert(provider_name.clone(), comparison);
        }

        BenchmarkReport {
            provider_comparisons,
            overall_performance_score: self.calculate_overall_score(&provider_comparisons).await,
            benchmark_timestamp: Instant::now(),
        }
    }

    /// Compare duration against target
    fn compare_duration(&self, actual: Duration, target: Duration) -> f64 {
        if target.as_nanos() == 0 {
            return 1.0;
        }
        target.as_nanos() as f64 / actual.as_nanos() as f64
    }

    /// Compare success rate against target
    fn compare_success_rate(&self, metrics: &ProviderMetrics, target: f64) -> f64 {
        let actual_rate = if metrics.total_requests > 0 {
            metrics.successful_requests as f64 / metrics.total_requests as f64
        } else {
            0.0
        };
        actual_rate / target
    }

    /// Compare throughput against target
    fn compare_throughput(&self, actual: f64, target: f64) -> f64 {
        if target == 0.0 {
            return 1.0;
        }
        actual / target
    }

    /// Check if provider meets all benchmark targets
    fn meets_all_targets(&self, metrics: &ProviderMetrics) -> bool {
        let response_time_ok = metrics.avg_response_time <= self.config.benchmark_targets.target_response_time;
        let success_rate = if metrics.total_requests > 0 {
            metrics.successful_requests as f64 / metrics.total_requests as f64
        } else {
            0.0
        };
        let success_rate_ok = success_rate >= self.config.benchmark_targets.target_success_rate;
        let throughput_ok = metrics.throughput >= self.config.benchmark_targets.target_throughput;

        response_time_ok && success_rate_ok && throughput_ok
    }

    /// Calculate overall performance score
    async fn calculate_overall_score(&self, comparisons: &HashMap<String, ProviderBenchmarkComparison>) -> f64 {
        if comparisons.is_empty() {
            return 0.0;
        }

        let total_score: f64 = comparisons.values().map(|comp| {
            // Weighted average of different metrics
            (comp.response_time_vs_target * 0.4) +
            (comp.success_rate_vs_target * 0.4) +
            (comp.throughput_vs_target * 0.2)
        }).sum();

        total_score / comparisons.len() as f64
    }
}

/// Performance summary across all providers
#[derive(Debug, Clone)]
pub struct PerformanceSummary {
    pub total_providers: usize,
    pub total_requests: u64,
    pub overall_success_rate: f64,
    pub overall_avg_response_time: Duration,
    pub measurements_count: usize,
    pub active_providers: usize,
}

/// Benchmark comparison report
#[derive(Debug, Clone)]
pub struct BenchmarkReport {
    pub provider_comparisons: HashMap<String, ProviderBenchmarkComparison>,
    pub overall_performance_score: f64,
    pub benchmark_timestamp: Instant,
}

/// Benchmark comparison for a single provider
#[derive(Debug, Clone)]
pub struct ProviderBenchmarkComparison {
    pub provider_name: String,
    pub response_time_vs_target: f64, // Ratio: target/actual (>1 is better)
    pub success_rate_vs_target: f64,  // Ratio: actual/target (>1 is better)
    pub throughput_vs_target: f64,    // Ratio: actual/target (>1 is better)
    pub meets_targets: bool,
}

impl PerformanceMeasurement {
    /// Create a new performance measurement
    pub fn new(
        provider_name: String,
        request_type: RequestType,
    ) -> Self {
        Self {
            provider_name,
            start_time: Instant::now(),
            end_time: Instant::now(),
            success: false,
            response_size_bytes: None,
            model_name: None,
            request_type,
            metadata: HashMap::new(),
        }
    }

    /// Mark the measurement as completed successfully
    pub fn complete_success(mut self) -> Self {
        self.end_time = Instant::now();
        self.success = true;
        self
    }

    /// Mark the measurement as completed with failure
    pub fn complete_failure(mut self) -> Self {
        self.end_time = Instant::now();
        self.success = false;
        self
    }

    /// Get the duration of the measurement
    pub fn duration(&self) -> Duration {
        self.end_time.duration_since(self.start_time)
    }

    /// Add metadata to the measurement
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Set the response size
    pub fn with_response_size(mut self, size_bytes: usize) -> Self {
        self.response_size_bytes = Some(size_bytes);
        self
    }

    /// Set the model name
    pub fn with_model(mut self, model_name: String) -> Self {
        self.model_name = Some(model_name);
        self
    }
}

impl ProviderMetrics {
    /// Create new provider metrics
    pub fn new(provider_name: &str) -> Self {
        Self {
            provider_name: provider_name.to_string(),
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            avg_response_time: Duration::from_millis(0),
            min_response_time: Duration::from_millis(0),
            max_response_time: Duration::from_millis(0),
            p95_response_time: Duration::from_millis(0),
            p99_response_time: Duration::from_millis(0),
            throughput: 0.0,
            model_loading_time: None,
            memory_usage_mb: None,
            cpu_usage_percent: None,
            last_updated: Instant::now(),
        }
    }

    /// Get success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        (self.successful_requests as f64 / self.total_requests as f64) * 100.0
    }

    /// Get failure rate as a percentage
    pub fn failure_rate(&self) -> f64 {
        100.0 - self.success_rate()
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_measurements: 10000,
            alert_thresholds: AlertThresholds::default(),
            benchmark_targets: BenchmarkTargets::default(),
            collection_interval: Duration::from_secs(60),
        }
    }
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            max_response_time: Duration::from_secs(5),
            min_success_rate: 0.95,
            max_memory_usage_mb: 2048,
            max_cpu_usage_percent: 80.0,
            min_throughput: 1.0,
        }
    }
}

impl Default for BenchmarkTargets {
    fn default() -> Self {
        Self {
            target_response_time: Duration::from_millis(500),
            target_success_rate: 0.99,
            target_throughput: 10.0,
            cloud_baseline: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn test_performance_monitor_creation() {
        let config = PerformanceConfig::default();
        let monitor = PerformanceMonitor::new(config);
        
        let metrics = monitor.get_all_metrics().await;
        assert_eq!(metrics.len(), 0);
    }

    #[tokio::test]
    async fn test_record_measurement() {
        let config = PerformanceConfig::default();
        let monitor = PerformanceMonitor::new(config);
        
        let measurement = PerformanceMeasurement::new(
            "test-provider".to_string(),
            RequestType::Inference,
        ).complete_success();
        
        monitor.record_measurement(measurement).await;
        
        let metrics = monitor.get_provider_metrics("test-provider").await;
        assert!(metrics.is_some());
        
        let metrics = metrics.unwrap();
        assert_eq!(metrics.total_requests, 1);
        assert_eq!(metrics.successful_requests, 1);
        assert_eq!(metrics.failed_requests, 0);
    }

    #[tokio::test]
    async fn test_performance_summary() {
        let config = PerformanceConfig::default();
        let monitor = PerformanceMonitor::new(config);
        
        // Record some measurements
        for i in 0..5 {
            let measurement = PerformanceMeasurement::new(
                format!("provider-{}", i),
                RequestType::Inference,
            ).complete_success();
            monitor.record_measurement(measurement).await;
        }
        
        let summary = monitor.get_performance_summary().await;
        assert_eq!(summary.total_providers, 5);
        assert_eq!(summary.total_requests, 5);
        assert_eq!(summary.active_providers, 5);
        assert_eq!(summary.overall_success_rate, 1.0);
    }

    #[test]
    fn test_provider_metrics_success_rate() {
        let mut metrics = ProviderMetrics::new("test");
        metrics.total_requests = 10;
        metrics.successful_requests = 8;
        metrics.failed_requests = 2;
        
        assert_eq!(metrics.success_rate(), 80.0);
        assert_eq!(metrics.failure_rate(), 20.0);
    }

    #[test]
    fn test_performance_measurement_duration() {
        let start = Instant::now();
        let measurement = PerformanceMeasurement {
            provider_name: "test".to_string(),
            start_time: start,
            end_time: start + Duration::from_millis(100),
            success: true,
            response_size_bytes: None,
            model_name: None,
            request_type: RequestType::Inference,
            metadata: HashMap::new(),
        };
        
        assert_eq!(measurement.duration(), Duration::from_millis(100));
    }

    #[tokio::test]
    async fn test_optimization_recommendations() {
        let mut config = PerformanceConfig::default();
        config.alert_thresholds.max_response_time = Duration::from_millis(100);
        config.alert_thresholds.min_success_rate = 0.9;
        
        let monitor = PerformanceMonitor::new(config);
        
        // Record a slow, failing measurement
        let measurement = PerformanceMeasurement {
            provider_name: "slow-provider".to_string(),
            start_time: Instant::now(),
            end_time: Instant::now() + Duration::from_millis(200),
            success: false,
            response_size_bytes: None,
            model_name: None,
            request_type: RequestType::Inference,
            metadata: HashMap::new(),
        };
        
        monitor.record_measurement(measurement).await;
        
        let recommendations = monitor.generate_recommendations().await;
        assert!(!recommendations.is_empty());
        
        // Should have recommendations for both slow response and low success rate
        let has_network_rec = recommendations.iter().any(|r| matches!(r.recommendation_type, RecommendationType::Network));
        let has_provider_rec = recommendations.iter().any(|r| matches!(r.recommendation_type, RecommendationType::ProviderSelection));
        
        assert!(has_network_rec);
        assert!(has_provider_rec);
    }

    #[tokio::test]
    async fn test_benchmark_comparison() {
        let mut config = PerformanceConfig::default();
        config.benchmark_targets.target_response_time = Duration::from_millis(100);
        config.benchmark_targets.target_success_rate = 0.95;
        config.benchmark_targets.target_throughput = 5.0;
        
        let monitor = PerformanceMonitor::new(config);
        
        // Record a good measurement
        let measurement = PerformanceMeasurement {
            provider_name: "fast-provider".to_string(),
            start_time: Instant::now(),
            end_time: Instant::now() + Duration::from_millis(50),
            success: true,
            response_size_bytes: None,
            model_name: None,
            request_type: RequestType::Inference,
            metadata: HashMap::new(),
        };
        
        monitor.record_measurement(measurement).await;
        
        let report = monitor.benchmark_against_targets().await;
        assert!(!report.provider_comparisons.is_empty());
        
        let comparison = report.provider_comparisons.get("fast-provider").unwrap();
        assert!(comparison.response_time_vs_target > 1.0); // Faster than target
        assert!(comparison.success_rate_vs_target > 0.0);
    }
}