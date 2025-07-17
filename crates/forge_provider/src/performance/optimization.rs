//! Performance optimization utilities for local AI providers

use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Context as _;
use derive_setters::Setters;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::{PerformanceConfig, PerformanceMonitor, ProviderMetrics, RequestType};

/// Model loading optimizer for local providers
pub struct ModelLoadingOptimizer {
    config: OptimizationConfig,
    cache: Arc<RwLock<ModelCache>>,
    preloader: ModelPreloader,
}

/// Configuration for performance optimizations
#[derive(Debug, Clone, Serialize, Deserialize, Setters)]
#[setters(strip_option, into)]
pub struct OptimizationConfig {
    /// Enable model caching
    pub enable_model_caching: bool,
    /// Enable model preloading
    pub enable_model_preloading: bool,
    /// Maximum cache size in MB
    pub max_cache_size_mb: u64,
    /// Model cache TTL
    pub cache_ttl: Duration,
    /// Preload popular models
    pub preload_popular_models: bool,
    /// Memory optimization settings
    pub memory_optimization: MemoryOptimizationConfig,
    /// CPU optimization settings
    pub cpu_optimization: CpuOptimizationConfig,
}

/// Memory optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize, Setters)]
#[setters(strip_option, into)]
pub struct MemoryOptimizationConfig {
    /// Enable memory pooling
    pub enable_memory_pooling: bool,
    /// Maximum memory pool size in MB
    pub max_pool_size_mb: u64,
    /// Enable garbage collection optimization
    pub enable_gc_optimization: bool,
    /// Memory pressure threshold (0.0 to 1.0)
    pub memory_pressure_threshold: f64,
}

/// CPU optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize, Setters)]
#[setters(strip_option, into)]
pub struct CpuOptimizationConfig {
    /// Number of worker threads
    pub worker_threads: Option<usize>,
    /// Enable CPU affinity
    pub enable_cpu_affinity: bool,
    /// CPU usage threshold for throttling
    pub cpu_throttle_threshold: f64,
    /// Enable SIMD optimizations
    pub enable_simd: bool,
}

/// Model cache for faster loading
#[derive(Debug)]
struct ModelCache {
    /// Cached models with metadata
    models: std::collections::HashMap<String, CachedModel>,
    /// Total cache size in bytes
    total_size_bytes: u64,
    /// Maximum cache size in bytes
    max_size_bytes: u64,
}

/// Cached model information
#[derive(Debug, Clone)]
struct CachedModel {
    /// Model identifier
    model_id: String,
    /// Model size in bytes
    size_bytes: u64,
    /// When the model was cached
    cached_at: Instant,
    /// Last accessed time
    last_accessed: Instant,
    /// Access count
    access_count: u64,
    /// Cache TTL
    ttl: Duration,
}

/// Model preloader for anticipating usage
pub struct ModelPreloader {
    config: OptimizationConfig,
    usage_patterns: Arc<RwLock<UsagePatterns>>,
}

/// Usage patterns for predictive loading
#[derive(Debug, Default)]
struct UsagePatterns {
    /// Model usage frequency
    model_frequency: std::collections::HashMap<String, u64>,
    /// Time-based usage patterns
    time_patterns: std::collections::HashMap<String, Vec<Instant>>,
    /// Provider preference patterns
    provider_patterns: std::collections::HashMap<String, f64>,
}

/// Performance optimization result
#[derive(Debug, Clone)]
pub struct OptimizationResult {
    /// Optimization type applied
    pub optimization_type: OptimizationType,
    /// Performance improvement achieved
    pub improvement: PerformanceImprovement,
    /// Time taken to apply optimization
    pub optimization_time: Duration,
    /// Success status
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Type of optimization applied
#[derive(Debug, Clone)]
pub enum OptimizationType {
    /// Model caching optimization
    ModelCaching,
    /// Model preloading optimization
    ModelPreloading,
    /// Memory optimization
    MemoryOptimization,
    /// CPU optimization
    CpuOptimization,
    /// Network optimization
    NetworkOptimization,
}

/// Performance improvement metrics
#[derive(Debug, Clone)]
pub struct PerformanceImprovement {
    /// Response time improvement (before vs after)
    pub response_time_improvement: Duration,
    /// Memory usage improvement in MB
    pub memory_improvement_mb: i64,
    /// CPU usage improvement percentage
    pub cpu_improvement_percent: f64,
    /// Throughput improvement (requests per second)
    pub throughput_improvement: f64,
}

impl ModelLoadingOptimizer {
    /// Create a new model loading optimizer
    pub fn new(config: OptimizationConfig) -> Self {
        let cache = ModelCache::new(config.max_cache_size_mb * 1024 * 1024);
        let preloader = ModelPreloader::new(config.clone());

        Self {
            config,
            cache: Arc::new(RwLock::new(cache)),
            preloader,
        }
    }

    /// Optimize model loading for a provider
    pub async fn optimize_model_loading(
        &self,
        provider_name: &str,
        model_name: &str,
    ) -> anyhow::Result<OptimizationResult> {
        let start_time = Instant::now();
        
        info!(
            "Optimizing model loading for provider: {}, model: {}",
            provider_name, model_name
        );

        let mut improvements = PerformanceImprovement::default();
        let mut optimization_types = Vec::new();

        // Try model caching optimization
        if self.config.enable_model_caching {
            match self.apply_model_caching(provider_name, model_name).await {
                Ok(improvement) => {
                    improvements.merge(improvement);
                    optimization_types.push(OptimizationType::ModelCaching);
                }
                Err(e) => {
                    warn!("Model caching optimization failed: {}", e);
                }
            }
        }

        // Try model preloading optimization
        if self.config.enable_model_preloading {
            match self.apply_model_preloading(provider_name, model_name).await {
                Ok(improvement) => {
                    improvements.merge(improvement);
                    optimization_types.push(OptimizationType::ModelPreloading);
                }
                Err(e) => {
                    warn!("Model preloading optimization failed: {}", e);
                }
            }
        }

        let optimization_time = start_time.elapsed();

        Ok(OptimizationResult {
            optimization_type: OptimizationType::ModelCaching, // Primary type
            improvement: improvements,
            optimization_time,
            success: !optimization_types.is_empty(),
            error: None,
        })
    }

    /// Apply model caching optimization
    async fn apply_model_caching(
        &self,
        provider_name: &str,
        model_name: &str,
    ) -> anyhow::Result<PerformanceImprovement> {
        let cache_key = format!("{}:{}", provider_name, model_name);
        
        let mut cache = self.cache.write().await;
        
        // Check if model is already cached
        if let Some(cached_model) = cache.models.get_mut(&cache_key) {
            // Update access information
            cached_model.last_accessed = Instant::now();
            cached_model.access_count += 1;
            
            debug!("Model cache hit for: {}", cache_key);
            
            return Ok(PerformanceImprovement {
                response_time_improvement: Duration::from_millis(500), // Estimated improvement
                memory_improvement_mb: 0,
                cpu_improvement_percent: 0.0,
                throughput_improvement: 0.0,
            });
        }

        // Model not in cache, simulate caching
        debug!("Model cache miss for: {}, would cache model", cache_key);
        
        // Simulate model size (in a real implementation, this would be actual model size)
        let model_size = 1024 * 1024 * 100; // 100MB
        
        // Check if we have space in cache
        if cache.total_size_bytes + model_size > cache.max_size_bytes {
            // Evict least recently used models
            cache.evict_lru_models(model_size)?;
        }

        // Add model to cache
        let cached_model = CachedModel {
            model_id: cache_key.clone(),
            size_bytes: model_size,
            cached_at: Instant::now(),
            last_accessed: Instant::now(),
            access_count: 1,
            ttl: self.config.cache_ttl,
        };

        cache.models.insert(cache_key, cached_model);
        cache.total_size_bytes += model_size;

        Ok(PerformanceImprovement {
            response_time_improvement: Duration::from_millis(200), // Initial caching overhead
            memory_improvement_mb: -(model_size as i64 / 1024 / 1024), // Memory used for cache
            cpu_improvement_percent: 5.0, // CPU saved on subsequent loads
            throughput_improvement: 0.1,
        })
    }

    /// Apply model preloading optimization
    async fn apply_model_preloading(
        &self,
        provider_name: &str,
        model_name: &str,
    ) -> anyhow::Result<PerformanceImprovement> {
        // Update usage patterns
        self.preloader.record_usage(provider_name, model_name).await;
        
        // Check if we should preload related models
        let related_models = self.preloader.get_related_models(provider_name, model_name).await;
        
        debug!(
            "Would preload {} related models for {}:{}",
            related_models.len(),
            provider_name,
            model_name
        );

        Ok(PerformanceImprovement {
            response_time_improvement: Duration::from_millis(100),
            memory_improvement_mb: 0,
            cpu_improvement_percent: 2.0,
            throughput_improvement: 0.05,
        })
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> CacheStatistics {
        let cache = self.cache.read().await;
        
        let total_models = cache.models.len();
        let total_size_mb = cache.total_size_bytes / 1024 / 1024;
        let cache_utilization = cache.total_size_bytes as f64 / cache.max_size_bytes as f64;
        
        let total_accesses: u64 = cache.models.values().map(|m| m.access_count).sum();
        let avg_access_count = if total_models > 0 {
            total_accesses as f64 / total_models as f64
        } else {
            0.0
        };

        CacheStatistics {
            total_models,
            total_size_mb,
            cache_utilization,
            total_accesses,
            avg_access_count,
            hit_rate: 0.0, // Would be calculated from actual cache hits/misses
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStatistics {
    pub total_models: usize,
    pub total_size_mb: u64,
    pub cache_utilization: f64,
    pub total_accesses: u64,
    pub avg_access_count: f64,
    pub hit_rate: f64,
}

impl ModelCache {
    fn new(max_size_bytes: u64) -> Self {
        Self {
            models: std::collections::HashMap::new(),
            total_size_bytes: 0,
            max_size_bytes,
        }
    }

    fn evict_lru_models(&mut self, space_needed: u64) -> anyhow::Result<()> {
        // Sort models by last accessed time
        let mut models_by_access: Vec<_> = self.models.iter().collect();
        models_by_access.sort_by_key(|(_, model)| model.last_accessed);

        let mut space_freed = 0u64;
        let mut models_to_remove = Vec::new();

        for (model_id, model) in models_by_access {
            if space_freed >= space_needed {
                break;
            }
            
            models_to_remove.push(model_id.clone());
            space_freed += model.size_bytes;
        }

        // Remove selected models
        for model_id in models_to_remove {
            if let Some(model) = self.models.remove(&model_id) {
                self.total_size_bytes -= model.size_bytes;
                debug!("Evicted model from cache: {}", model_id);
            }
        }

        if space_freed < space_needed {
            anyhow::bail!(
                "Could not free enough space: needed {}, freed {}",
                space_needed,
                space_freed
            );
        }

        Ok(())
    }
}

impl ModelPreloader {
    fn new(config: OptimizationConfig) -> Self {
        Self {
            config,
            usage_patterns: Arc::new(RwLock::new(UsagePatterns::default())),
        }
    }

    async fn record_usage(&self, provider_name: &str, model_name: &str) {
        let mut patterns = self.usage_patterns.write().await;
        
        // Record model frequency
        let model_key = format!("{}:{}", provider_name, model_name);
        *patterns.model_frequency.entry(model_key.clone()).or_insert(0) += 1;
        
        // Record time pattern
        patterns.time_patterns
            .entry(model_key)
            .or_insert_with(Vec::new)
            .push(Instant::now());
        
        // Record provider preference
        *patterns.provider_patterns.entry(provider_name.to_string()).or_insert(0.0) += 1.0;
    }

    async fn get_related_models(&self, provider_name: &str, model_name: &str) -> Vec<String> {
        let patterns = self.usage_patterns.read().await;
        
        // Simple heuristic: return models from the same provider that are frequently used
        let mut related_models = Vec::new();
        
        for (model_key, frequency) in &patterns.model_frequency {
            if model_key.starts_with(provider_name) && !model_key.ends_with(model_name) && *frequency > 5 {
                related_models.push(model_key.clone());
            }
        }
        
        // Limit to top 3 related models
        related_models.sort_by_key(|key| std::cmp::Reverse(patterns.model_frequency.get(key).unwrap_or(&0)));
        related_models.truncate(3);
        
        related_models
    }
}

impl PerformanceImprovement {
    fn merge(&mut self, other: PerformanceImprovement) {
        self.response_time_improvement += other.response_time_improvement;
        self.memory_improvement_mb += other.memory_improvement_mb;
        self.cpu_improvement_percent += other.cpu_improvement_percent;
        self.throughput_improvement += other.throughput_improvement;
    }
}

impl Default for PerformanceImprovement {
    fn default() -> Self {
        Self {
            response_time_improvement: Duration::from_millis(0),
            memory_improvement_mb: 0,
            cpu_improvement_percent: 0.0,
            throughput_improvement: 0.0,
        }
    }
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            enable_model_caching: true,
            enable_model_preloading: true,
            max_cache_size_mb: 1024, // 1GB
            cache_ttl: Duration::from_secs(3600), // 1 hour
            preload_popular_models: true,
            memory_optimization: MemoryOptimizationConfig::default(),
            cpu_optimization: CpuOptimizationConfig::default(),
        }
    }
}

impl Default for MemoryOptimizationConfig {
    fn default() -> Self {
        Self {
            enable_memory_pooling: true,
            max_pool_size_mb: 512,
            enable_gc_optimization: true,
            memory_pressure_threshold: 0.8,
        }
    }
}

impl Default for CpuOptimizationConfig {
    fn default() -> Self {
        Self {
            worker_threads: None, // Use system default
            enable_cpu_affinity: false,
            cpu_throttle_threshold: 0.9,
            enable_simd: true,
        }
    }
}

/// System resource monitor for optimization decisions
pub struct ResourceMonitor {
    config: OptimizationConfig,
}

impl ResourceMonitor {
    pub fn new(config: OptimizationConfig) -> Self {
        Self { config }
    }

    /// Get current system resource usage
    pub async fn get_resource_usage(&self) -> ResourceUsage {
        // In a real implementation, this would use system APIs to get actual resource usage
        ResourceUsage {
            memory_usage_percent: 45.0,
            cpu_usage_percent: 30.0,
            available_memory_mb: 8192,
            disk_usage_percent: 60.0,
            network_bandwidth_mbps: 100.0,
        }
    }

    /// Check if system is under resource pressure
    pub async fn is_under_pressure(&self) -> bool {
        let usage = self.get_resource_usage().await;
        
        usage.memory_usage_percent / 100.0 > self.config.memory_optimization.memory_pressure_threshold ||
        usage.cpu_usage_percent / 100.0 > self.config.cpu_optimization.cpu_throttle_threshold
    }

    /// Get optimization recommendations based on resource usage
    pub async fn get_resource_recommendations(&self) -> Vec<ResourceRecommendation> {
        let usage = self.get_resource_usage().await;
        let mut recommendations = Vec::new();

        if usage.memory_usage_percent > 80.0 {
            recommendations.push(ResourceRecommendation {
                resource_type: ResourceType::Memory,
                severity: RecommendationSeverity::High,
                description: format!("Memory usage is high: {:.1}%", usage.memory_usage_percent),
                suggested_action: "Consider reducing model cache size or enabling memory optimization".to_string(),
            });
        }

        if usage.cpu_usage_percent > 90.0 {
            recommendations.push(ResourceRecommendation {
                resource_type: ResourceType::Cpu,
                severity: RecommendationSeverity::High,
                description: format!("CPU usage is high: {:.1}%", usage.cpu_usage_percent),
                suggested_action: "Consider reducing worker threads or enabling CPU throttling".to_string(),
            });
        }

        if usage.disk_usage_percent > 90.0 {
            recommendations.push(ResourceRecommendation {
                resource_type: ResourceType::Disk,
                severity: RecommendationSeverity::Medium,
                description: format!("Disk usage is high: {:.1}%", usage.disk_usage_percent),
                suggested_action: "Consider cleaning up model cache or reducing cache size".to_string(),
            });
        }

        recommendations
    }
}

/// Current system resource usage
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub memory_usage_percent: f64,
    pub cpu_usage_percent: f64,
    pub available_memory_mb: u64,
    pub disk_usage_percent: f64,
    pub network_bandwidth_mbps: f64,
}

/// Resource optimization recommendation
#[derive(Debug, Clone)]
pub struct ResourceRecommendation {
    pub resource_type: ResourceType,
    pub severity: RecommendationSeverity,
    pub description: String,
    pub suggested_action: String,
}

/// Type of system resource
#[derive(Debug, Clone)]
pub enum ResourceType {
    Memory,
    Cpu,
    Disk,
    Network,
}

/// Severity of recommendation
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum RecommendationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn test_model_loading_optimizer_creation() {
        let config = OptimizationConfig::default();
        let optimizer = ModelLoadingOptimizer::new(config);
        
        let stats = optimizer.get_cache_stats().await;
        assert_eq!(stats.total_models, 0);
        assert_eq!(stats.total_size_mb, 0);
    }

    #[tokio::test]
    async fn test_optimize_model_loading() {
        let config = OptimizationConfig::default();
        let optimizer = ModelLoadingOptimizer::new(config);
        
        let result = optimizer.optimize_model_loading("test-provider", "test-model").await;
        assert!(result.is_ok());
        
        let result = result.unwrap();
        assert!(result.success);
        assert!(result.improvement.response_time_improvement > Duration::from_millis(0));
    }

    #[tokio::test]
    async fn test_cache_statistics() {
        let config = OptimizationConfig::default();
        let optimizer = ModelLoadingOptimizer::new(config);
        
        // Optimize a model to populate cache
        let _result = optimizer.optimize_model_loading("test-provider", "test-model").await;
        
        let stats = optimizer.get_cache_stats().await;
        assert_eq!(stats.total_models, 1);
        assert!(stats.total_size_mb > 0);
        assert!(stats.cache_utilization > 0.0);
    }

    #[tokio::test]
    async fn test_resource_monitor() {
        let config = OptimizationConfig::default();
        let monitor = ResourceMonitor::new(config);
        
        let usage = monitor.get_resource_usage().await;
        assert!(usage.memory_usage_percent >= 0.0);
        assert!(usage.cpu_usage_percent >= 0.0);
        assert!(usage.available_memory_mb > 0);
        
        let is_under_pressure = monitor.is_under_pressure().await;
        assert!(!is_under_pressure); // Should not be under pressure with default values
    }

    #[tokio::test]
    async fn test_resource_recommendations() {
        let config = OptimizationConfig::default();
        let monitor = ResourceMonitor::new(config);
        
        let recommendations = monitor.get_resource_recommendations().await;
        // With default mock values, should not have high-severity recommendations
        let high_severity_count = recommendations.iter()
            .filter(|r| r.severity == RecommendationSeverity::High)
            .count();
        assert_eq!(high_severity_count, 0);
    }

    #[test]
    fn test_performance_improvement_merge() {
        let mut improvement1 = PerformanceImprovement {
            response_time_improvement: Duration::from_millis(100),
            memory_improvement_mb: 50,
            cpu_improvement_percent: 10.0,
            throughput_improvement: 2.0,
        };

        let improvement2 = PerformanceImprovement {
            response_time_improvement: Duration::from_millis(50),
            memory_improvement_mb: 25,
            cpu_improvement_percent: 5.0,
            throughput_improvement: 1.0,
        };

        improvement1.merge(improvement2);

        assert_eq!(improvement1.response_time_improvement, Duration::from_millis(150));
        assert_eq!(improvement1.memory_improvement_mb, 75);
        assert_eq!(improvement1.cpu_improvement_percent, 15.0);
        assert_eq!(improvement1.throughput_improvement, 3.0);
    }

    #[test]
    fn test_model_cache_eviction() {
        let mut cache = ModelCache::new(1024 * 1024 * 200); // 200MB max
        
        // Add a model that exceeds cache size
        let model1 = CachedModel {
            model_id: "model1".to_string(),
            size_bytes: 1024 * 1024 * 100, // 100MB
            cached_at: Instant::now(),
            last_accessed: Instant::now(),
            access_count: 1,
            ttl: Duration::from_secs(3600),
        };
        
        cache.models.insert("model1".to_string(), model1);
        cache.total_size_bytes = 1024 * 1024 * 100;
        
        // Try to add another model that would exceed cache size
        let space_needed = 1024 * 1024 * 150; // 150MB
        let result = cache.evict_lru_models(space_needed);
        
        assert!(result.is_ok());
        assert_eq!(cache.models.len(), 0); // Should have evicted the model
        assert_eq!(cache.total_size_bytes, 0);
    }
}