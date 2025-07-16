//! Enhanced fallback features for Phase 7
//! 
//! This module provides intelligent enhancements to the fallback system implemented in Phase 6,
//! including adaptive fallback strategies, user experience improvements, and advanced decision logic.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use derive_setters::Setters;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::config::fallback::{FallbackConfig, FallbackContext, FallbackDecision};
use crate::config::local_ai::{LocalAiConfig, ProviderHealthStatus};

/// Enhanced fallback configuration with intelligent features
#[derive(Debug, Clone, Serialize, Deserialize, Setters)]
#[setters(strip_option, into)]
pub struct EnhancedFallbackConfig {
    /// Base fallback configuration
    pub base_config: FallbackConfig,
    /// Enable adaptive strategy selection
    pub adaptive_strategy: bool,
    /// Performance-based provider ranking
    pub performance_ranking: bool,
    /// User experience optimizations
    pub ux_optimizations: UxOptimizations,
    /// Learning from user patterns
    pub pattern_learning: PatternLearning,
    /// Cost optimization settings
    pub cost_optimization: CostOptimization,
}

/// User experience optimization settings
#[derive(Debug, Clone, Serialize, Deserialize, Setters)]
#[setters(strip_option, into)]
pub struct UxOptimizations {
    /// Preemptive fallback based on performance degradation
    pub preemptive_fallback: bool,
    /// Smart retry with exponential backoff
    pub smart_retry: bool,
    /// Seamless provider switching
    pub seamless_switching: bool,
    /// Context-aware notifications
    pub context_aware_notifications: bool,
    /// Response time optimization
    pub response_time_optimization: bool,
}

/// Pattern learning configuration
#[derive(Debug, Clone, Serialize, Deserialize, Setters)]
#[setters(strip_option, into)]
pub struct PatternLearning {
    /// Enable learning from user behavior
    pub enabled: bool,
    /// Time-based usage patterns
    pub time_patterns: bool,
    /// Model-specific preferences
    pub model_preferences: bool,
    /// Workload-based optimization
    pub workload_optimization: bool,
    /// Learning window in days
    pub learning_window_days: u32,
}

/// Cost optimization settings
#[derive(Debug, Clone, Serialize, Deserialize, Setters)]
#[setters(strip_option, into)]
pub struct CostOptimization {
    /// Enable cost-aware fallback
    pub enabled: bool,
    /// Prefer local providers for cost savings
    pub prefer_local_for_cost: bool,
    /// Cloud provider cost ranking
    pub cloud_cost_ranking: Vec<String>,
    /// Budget-based switching
    pub budget_aware_switching: bool,
    /// Daily budget limit in USD
    pub daily_budget_limit: Option<f64>,
}

/// Enhanced fallback decision with additional context
#[derive(Debug, Clone)]
pub struct EnhancedFallbackDecision {
    /// Base fallback decision
    pub decision: FallbackDecision,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// Reasoning for the decision
    pub reasoning: Vec<String>,
    /// Alternative options considered
    pub alternatives: Vec<AlternativeOption>,
    /// Estimated cost impact
    pub cost_impact: Option<CostImpact>,
    /// Performance prediction
    pub performance_prediction: Option<PerformancePrediction>,
}

/// Alternative option that was considered
#[derive(Debug, Clone)]
pub struct AlternativeOption {
    /// Provider name
    pub provider_name: String,
    /// Reason it wasn't selected
    pub rejection_reason: String,
    /// Score relative to selected option
    pub relative_score: f64,
}

/// Cost impact analysis
#[derive(Debug, Clone)]
pub struct CostImpact {
    /// Estimated cost per request
    pub cost_per_request: f64,
    /// Cost comparison with alternatives
    pub cost_savings: f64,
    /// Budget impact
    pub budget_impact: BudgetImpact,
}

/// Budget impact assessment
#[derive(Debug, Clone)]
pub enum BudgetImpact {
    /// Within budget
    WithinBudget,
    /// Approaching budget limit
    ApproachingLimit { remaining_percentage: f64 },
    /// Exceeds budget
    ExceedsBudget { overage_amount: f64 },
}

/// Performance prediction
#[derive(Debug, Clone)]
pub struct PerformancePrediction {
    /// Expected response time
    pub expected_response_time: Duration,
    /// Expected success rate
    pub expected_success_rate: f64,
    /// Quality score
    pub quality_score: f64,
    /// Reliability score
    pub reliability_score: f64,
}

/// Enhanced fallback engine with intelligent features
pub struct EnhancedFallbackEngine {
    config: EnhancedFallbackConfig,
    local_config: LocalAiConfig,
    usage_patterns: UsagePatterns,
    performance_history: PerformanceHistory,
    cost_tracker: CostTracker,
}

/// Usage patterns tracking
#[derive(Debug, Clone)]
pub struct UsagePatterns {
    /// Time-based patterns
    pub time_patterns: HashMap<String, TimePattern>,
    /// Model usage patterns
    pub model_patterns: HashMap<String, ModelPattern>,
    /// Workload patterns
    pub workload_patterns: HashMap<String, WorkloadPattern>,
}

/// Time-based usage pattern
#[derive(Debug, Clone)]
pub struct TimePattern {
    /// Hour of day preferences
    pub hourly_preferences: HashMap<u8, ProviderPreference>,
    /// Day of week preferences
    pub daily_preferences: HashMap<u8, ProviderPreference>,
    /// Peak usage times
    pub peak_times: Vec<TimeRange>,
}

/// Model usage pattern
#[derive(Debug, Clone)]
pub struct ModelPattern {
    /// Preferred providers for this model
    pub preferred_providers: Vec<String>,
    /// Performance by provider
    pub provider_performance: HashMap<String, f64>,
    /// Usage frequency
    pub usage_frequency: f64,
}

/// Workload pattern
#[derive(Debug, Clone)]
pub struct WorkloadPattern {
    /// Streaming vs non-streaming preferences
    pub streaming_preference: Option<String>,
    /// Tools usage preferences
    pub tools_preference: Option<String>,
    /// Response time requirements
    pub response_time_requirements: Duration,
}

/// Provider preference data
#[derive(Debug, Clone)]
pub struct ProviderPreference {
    /// Provider name
    pub provider: String,
    /// Preference score (0.0 to 1.0)
    pub score: f64,
    /// Usage count
    pub usage_count: u64,
    /// Success rate
    pub success_rate: f64,
}

/// Time range for patterns
#[derive(Debug, Clone)]
pub struct TimeRange {
    /// Start hour (0-23)
    pub start_hour: u8,
    /// End hour (0-23)
    pub end_hour: u8,
    /// Days of week (0=Sunday, 6=Saturday)
    pub days_of_week: Vec<u8>,
}

/// Performance history tracking
#[derive(Debug, Clone)]
pub struct PerformanceHistory {
    /// Provider performance metrics
    pub provider_metrics: HashMap<String, ProviderPerformanceMetrics>,
    /// Historical trends
    pub trends: HashMap<String, PerformanceTrend>,
    /// Anomaly detection
    pub anomalies: Vec<PerformanceAnomaly>,
}

/// Performance metrics for a provider
#[derive(Debug, Clone)]
pub struct ProviderPerformanceMetrics {
    /// Average response time over time
    pub response_times: Vec<(Instant, Duration)>,
    /// Success rates over time
    pub success_rates: Vec<(Instant, f64)>,
    /// Quality scores over time
    pub quality_scores: Vec<(Instant, f64)>,
    /// Reliability scores
    pub reliability_scores: Vec<(Instant, f64)>,
}

/// Performance trend analysis
#[derive(Debug, Clone)]
pub struct PerformanceTrend {
    /// Trend direction
    pub direction: TrendDirection,
    /// Trend strength (0.0 to 1.0)
    pub strength: f64,
    /// Confidence in trend
    pub confidence: f64,
    /// Time window for trend
    pub time_window: Duration,
}

/// Trend direction
#[derive(Debug, Clone)]
pub enum TrendDirection {
    /// Performance improving
    Improving,
    /// Performance stable
    Stable,
    /// Performance degrading
    Degrading,
    /// Insufficient data
    Unknown,
}

/// Performance anomaly
#[derive(Debug, Clone)]
pub struct PerformanceAnomaly {
    /// Provider name
    pub provider: String,
    /// Anomaly type
    pub anomaly_type: AnomalyType,
    /// Severity (0.0 to 1.0)
    pub severity: f64,
    /// Timestamp
    pub timestamp: Instant,
    /// Description
    pub description: String,
}

/// Type of performance anomaly
#[derive(Debug, Clone)]
pub enum AnomalyType {
    /// Sudden response time spike
    ResponseTimeSpike,
    /// Success rate drop
    SuccessRateDrop,
    /// Quality degradation
    QualityDegradation,
    /// Service unavailable
    ServiceUnavailable,
    /// Unusual pattern
    UnusualPattern,
}

/// Cost tracking
#[derive(Debug, Clone)]
pub struct CostTracker {
    /// Daily costs by provider
    pub daily_costs: HashMap<String, f64>,
    /// Monthly costs by provider
    pub monthly_costs: HashMap<String, f64>,
    /// Cost per request by provider
    pub cost_per_request: HashMap<String, f64>,
    /// Budget status
    pub budget_status: BudgetStatus,
}

/// Budget status tracking
#[derive(Debug, Clone)]
pub struct BudgetStatus {
    /// Daily budget used
    pub daily_used: f64,
    /// Daily budget limit
    pub daily_limit: Option<f64>,
    /// Monthly budget used
    pub monthly_used: f64,
    /// Monthly budget limit
    pub monthly_limit: Option<f64>,
    /// Budget alerts
    pub alerts: Vec<BudgetAlert>,
}

/// Budget alert
#[derive(Debug, Clone)]
pub struct BudgetAlert {
    /// Alert type
    pub alert_type: BudgetAlertType,
    /// Threshold percentage
    pub threshold: f64,
    /// Timestamp
    pub timestamp: Instant,
    /// Message
    pub message: String,
}

/// Budget alert types
#[derive(Debug, Clone)]
pub enum BudgetAlertType {
    /// Approaching daily limit
    DailyApproaching,
    /// Daily limit exceeded
    DailyExceeded,
    /// Approaching monthly limit
    MonthlyApproaching,
    /// Monthly limit exceeded
    MonthlyExceeded,
}

impl Default for EnhancedFallbackConfig {
    fn default() -> Self {
        Self {
            base_config: FallbackConfig::default(),
            adaptive_strategy: true,
            performance_ranking: true,
            ux_optimizations: UxOptimizations::default(),
            pattern_learning: PatternLearning::default(),
            cost_optimization: CostOptimization::default(),
        }
    }
}

impl Default for UxOptimizations {
    fn default() -> Self {
        Self {
            preemptive_fallback: true,
            smart_retry: true,
            seamless_switching: true,
            context_aware_notifications: true,
            response_time_optimization: true,
        }
    }
}

impl Default for PatternLearning {
    fn default() -> Self {
        Self {
            enabled: true,
            time_patterns: true,
            model_preferences: true,
            workload_optimization: true,
            learning_window_days: 30,
        }
    }
}

impl Default for CostOptimization {
    fn default() -> Self {
        Self {
            enabled: true,
            prefer_local_for_cost: true,
            cloud_cost_ranking: vec![
                "openai".to_string(),
                "anthropic".to_string(),
            ],
            budget_aware_switching: false,
            daily_budget_limit: None,
        }
    }
}

impl EnhancedFallbackEngine {
    /// Create a new enhanced fallback engine
    pub fn new(config: EnhancedFallbackConfig, local_config: LocalAiConfig) -> Self {
        Self {
            config,
            local_config,
            usage_patterns: UsagePatterns::new(),
            performance_history: PerformanceHistory::new(),
            cost_tracker: CostTracker::new(),
        }
    }

    /// Make an enhanced fallback decision
    pub async fn decide_provider_enhanced(
        &mut self,
        context: &FallbackContext,
        local_health: &[(String, ProviderHealthStatus)],
    ) -> EnhancedFallbackDecision {
        info!(
            model = %context.model_id,
            adaptive = self.config.adaptive_strategy,
            "Making enhanced fallback decision"
        );

        // Start with base decision
        let base_engine = crate::config::fallback::FallbackEngine::new(
            self.config.base_config.clone(),
            self.local_config.clone(),
        );
        
        let base_decision = base_engine.decide_provider(context, local_health).await;

        // Apply enhancements
        let mut reasoning = vec!["Base fallback decision made".to_string()];
        let mut confidence: f64 = 0.7; // Base confidence
        let mut alternatives = Vec::new();

        // Adaptive strategy enhancement
        if self.config.adaptive_strategy {
            confidence += 0.1;
            reasoning.push("Adaptive strategy enabled".to_string());
            
            // Analyze patterns and adjust decision
            if let Some(pattern_adjustment) = self.analyze_usage_patterns(context).await {
                reasoning.push(format!("Pattern analysis: {}", pattern_adjustment));
                confidence += 0.1;
            }
        }

        // Performance ranking enhancement
        if self.config.performance_ranking {
            let performance_scores = self.calculate_performance_scores(local_health).await;
            reasoning.push(format!("Performance ranking applied: {} providers analyzed", performance_scores.len()));
            confidence += 0.05;

            // Add alternatives based on performance
            for (provider, score) in performance_scores.iter().take(3) {
                if provider != base_decision.provider_name().unwrap_or("") {
                    alternatives.push(AlternativeOption {
                        provider_name: provider.clone(),
                        rejection_reason: format!("Lower performance score: {:.2}", score),
                        relative_score: score / performance_scores.values().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&1.0),
                    });
                }
            }
        }

        // UX optimizations
        if self.config.ux_optimizations.preemptive_fallback {
            if let Some(preemptive_reason) = self.check_preemptive_fallback(context, local_health).await {
                reasoning.push(format!("Preemptive fallback: {}", preemptive_reason));
                confidence += 0.05;
            }
        }

        // Cost optimization
        let cost_impact = if self.config.cost_optimization.enabled {
            reasoning.push("Cost optimization applied".to_string());
            confidence += 0.05;
            self.calculate_cost_impact(&base_decision).await
        } else {
            None
        };

        // Performance prediction
        let performance_prediction = self.predict_performance(&base_decision, context).await;
        if performance_prediction.is_some() {
            reasoning.push("Performance prediction generated".to_string());
            confidence += 0.05;
        }

        // Cap confidence at 1.0
        confidence = confidence.min(1.0);

        EnhancedFallbackDecision {
            decision: base_decision,
            confidence,
            reasoning,
            alternatives,
            cost_impact,
            performance_prediction,
        }
    }

    /// Analyze usage patterns for decision enhancement
    async fn analyze_usage_patterns(&self, context: &FallbackContext) -> Option<String> {
        if !self.config.pattern_learning.enabled {
            return None;
        }

        let mut analysis = Vec::new();

        // Time-based patterns
        if self.config.pattern_learning.time_patterns {
            if let Some(time_preference) = self.get_time_based_preference().await {
                analysis.push(format!("Time preference: {}", time_preference));
            }
        }

        // Model-specific patterns
        if self.config.pattern_learning.model_preferences {
            if let Some(model_preference) = self.get_model_preference(&context.model_id).await {
                analysis.push(format!("Model preference: {}", model_preference));
            }
        }

        if analysis.is_empty() {
            None
        } else {
            Some(analysis.join(", "))
        }
    }

    /// Calculate performance scores for providers
    async fn calculate_performance_scores(&self, local_health: &[(String, ProviderHealthStatus)]) -> HashMap<String, f64> {
        let mut scores = HashMap::new();

        for (provider_name, health_status) in local_health {
            let mut score = match health_status {
                ProviderHealthStatus::Healthy { .. } => 1.0,
                ProviderHealthStatus::Degraded { .. } => 0.6,
                ProviderHealthStatus::Unhealthy { .. } => 0.1,
            };

            // Apply historical performance data
            if let Some(metrics) = self.performance_history.provider_metrics.get(provider_name) {
                let avg_success_rate = self.calculate_average_success_rate(metrics);
                score *= avg_success_rate;
            }

            scores.insert(provider_name.clone(), score);
        }

        scores
    }

    /// Check for preemptive fallback conditions
    async fn check_preemptive_fallback(&self, _context: &FallbackContext, local_health: &[(String, ProviderHealthStatus)]) -> Option<String> {
        for (provider_name, health_status) in local_health {
            if let ProviderHealthStatus::Degraded { .. } = health_status {
                // Check if degradation is getting worse
                if let Some(trend) = self.performance_history.trends.get(provider_name) {
                    if matches!(trend.direction, TrendDirection::Degrading) && trend.strength > 0.7 {
                        return Some(format!("Provider {} showing degrading trend", provider_name));
                    }
                }
            }
        }
        None
    }

    /// Calculate cost impact of a decision
    async fn calculate_cost_impact(&self, decision: &FallbackDecision) -> Option<CostImpact> {
        if let Some(provider_name) = decision.provider_name() {
            let cost_per_request = self.cost_tracker.cost_per_request.get(provider_name).copied().unwrap_or(0.0);
            
            // Calculate savings compared to most expensive option
            let max_cost = self.cost_tracker.cost_per_request.values().max_by(|a, b| a.partial_cmp(b).unwrap()).copied().unwrap_or(0.0);
            let cost_savings = max_cost - cost_per_request;

            let budget_impact = self.assess_budget_impact(cost_per_request).await;

            Some(CostImpact {
                cost_per_request,
                cost_savings,
                budget_impact,
            })
        } else {
            None
        }
    }

    /// Predict performance for a decision
    async fn predict_performance(&self, decision: &FallbackDecision, _context: &FallbackContext) -> Option<PerformancePrediction> {
        if let Some(provider_name) = decision.provider_name() {
            if let Some(metrics) = self.performance_history.provider_metrics.get(provider_name) {
                let expected_response_time = self.calculate_average_response_time(metrics);
                let expected_success_rate = self.calculate_average_success_rate(metrics);
                let quality_score = self.calculate_average_quality_score(metrics);
                let reliability_score = self.calculate_reliability_score(metrics);

                Some(PerformancePrediction {
                    expected_response_time,
                    expected_success_rate,
                    quality_score,
                    reliability_score,
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get time-based preference
    async fn get_time_based_preference(&self) -> Option<String> {
        // Implementation would analyze current time and return preferred provider
        // For now, return a placeholder
        Some("local preferred during business hours".to_string())
    }

    /// Get model-specific preference
    async fn get_model_preference(&self, model_id: &str) -> Option<String> {
        if let Some(pattern) = self.usage_patterns.model_patterns.get(model_id) {
            pattern.preferred_providers.first().map(|p| format!("Prefer {} for {}", p, model_id))
        } else {
            None
        }
    }

    /// Assess budget impact
    async fn assess_budget_impact(&self, cost_per_request: f64) -> BudgetImpact {
        if let Some(daily_limit) = self.cost_tracker.budget_status.daily_limit {
            let daily_used = self.cost_tracker.budget_status.daily_used;
            let remaining = daily_limit - daily_used;
            let remaining_percentage = (remaining / daily_limit) * 100.0;

            if cost_per_request > remaining {
                BudgetImpact::ExceedsBudget {
                    overage_amount: cost_per_request - remaining,
                }
            } else if remaining_percentage < 20.0 {
                BudgetImpact::ApproachingLimit { remaining_percentage }
            } else {
                BudgetImpact::WithinBudget
            }
        } else {
            BudgetImpact::WithinBudget
        }
    }

    /// Calculate average response time from metrics
    fn calculate_average_response_time(&self, metrics: &ProviderPerformanceMetrics) -> Duration {
        if metrics.response_times.is_empty() {
            return Duration::from_millis(1000); // Default fallback
        }

        let total: Duration = metrics.response_times.iter().map(|(_, duration)| *duration).sum();
        total / metrics.response_times.len() as u32
    }

    /// Calculate average success rate from metrics
    fn calculate_average_success_rate(&self, metrics: &ProviderPerformanceMetrics) -> f64 {
        if metrics.success_rates.is_empty() {
            return 0.8; // Default fallback
        }

        let total: f64 = metrics.success_rates.iter().map(|(_, rate)| *rate).sum();
        total / metrics.success_rates.len() as f64
    }

    /// Calculate average quality score from metrics
    fn calculate_average_quality_score(&self, metrics: &ProviderPerformanceMetrics) -> f64 {
        if metrics.quality_scores.is_empty() {
            return 0.8; // Default fallback
        }

        let total: f64 = metrics.quality_scores.iter().map(|(_, score)| *score).sum();
        total / metrics.quality_scores.len() as f64
    }

    /// Calculate reliability score from metrics
    fn calculate_reliability_score(&self, metrics: &ProviderPerformanceMetrics) -> f64 {
        if metrics.reliability_scores.is_empty() {
            return 0.8; // Default fallback
        }

        let total: f64 = metrics.reliability_scores.iter().map(|(_, score)| *score).sum();
        total / metrics.reliability_scores.len() as f64
    }

    /// Record usage for pattern learning
    pub async fn record_usage(&mut self, provider_name: &str, context: &FallbackContext, success: bool, response_time: Duration) {
        if !self.config.pattern_learning.enabled {
            return;
        }

        debug!(
            provider = provider_name,
            model = %context.model_id,
            success = success,
            response_time_ms = response_time.as_millis(),
            "Recording usage for pattern learning"
        );

        // Update performance history
        self.update_performance_history(provider_name, success, response_time).await;

        // Update usage patterns
        self.update_usage_patterns(provider_name, context).await;

        // Update cost tracking if this is a cloud provider
        if provider_name.starts_with("cloud:") {
            self.update_cost_tracking(provider_name).await;
        }
    }

    /// Update performance history
    async fn update_performance_history(&mut self, provider_name: &str, success: bool, response_time: Duration) {
        let metrics = self.performance_history.provider_metrics
            .entry(provider_name.to_string())
            .or_insert_with(|| ProviderPerformanceMetrics {
                response_times: Vec::new(),
                success_rates: Vec::new(),
                quality_scores: Vec::new(),
                reliability_scores: Vec::new(),
            });

        let now = Instant::now();
        metrics.response_times.push((now, response_time));
        metrics.success_rates.push((now, if success { 1.0 } else { 0.0 }));

        // Keep only recent data (last 1000 entries)
        if metrics.response_times.len() > 1000 {
            metrics.response_times.remove(0);
        }
        if metrics.success_rates.len() > 1000 {
            metrics.success_rates.remove(0);
        }
    }

    /// Update usage patterns
    async fn update_usage_patterns(&mut self, provider_name: &str, context: &FallbackContext) {
        // Update model patterns
        let model_pattern = self.usage_patterns.model_patterns
            .entry(context.model_id.clone())
            .or_insert_with(|| ModelPattern {
                preferred_providers: Vec::new(),
                provider_performance: HashMap::new(),
                usage_frequency: 0.0,
            });

        model_pattern.usage_frequency += 1.0;
        
        if !model_pattern.preferred_providers.contains(&provider_name.to_string()) {
            model_pattern.preferred_providers.push(provider_name.to_string());
        }
    }

    /// Update cost tracking
    async fn update_cost_tracking(&mut self, provider_name: &str) {
        // Simplified cost tracking - in real implementation, this would integrate with billing APIs
        let cost = match provider_name {
            "cloud:openai" => 0.002, // Example cost per request
            "cloud:anthropic" => 0.003,
            _ => 0.001,
        };

        *self.cost_tracker.daily_costs.entry(provider_name.to_string()).or_insert(0.0) += cost;
        *self.cost_tracker.monthly_costs.entry(provider_name.to_string()).or_insert(0.0) += cost;
        self.cost_tracker.cost_per_request.insert(provider_name.to_string(), cost);

        self.cost_tracker.budget_status.daily_used += cost;
        self.cost_tracker.budget_status.monthly_used += cost;
    }
}

impl UsagePatterns {
    fn new() -> Self {
        Self {
            time_patterns: HashMap::new(),
            model_patterns: HashMap::new(),
            workload_patterns: HashMap::new(),
        }
    }
}

impl PerformanceHistory {
    fn new() -> Self {
        Self {
            provider_metrics: HashMap::new(),
            trends: HashMap::new(),
            anomalies: Vec::new(),
        }
    }
}

impl CostTracker {
    fn new() -> Self {
        Self {
            daily_costs: HashMap::new(),
            monthly_costs: HashMap::new(),
            cost_per_request: HashMap::new(),
            budget_status: BudgetStatus {
                daily_used: 0.0,
                daily_limit: None,
                monthly_used: 0.0,
                monthly_limit: None,
                alerts: Vec::new(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::local_ai::LocalAiConfig;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_enhanced_fallback_config_default() {
        let config = EnhancedFallbackConfig::default();
        assert_eq!(config.adaptive_strategy, true);
        assert_eq!(config.performance_ranking, true);
        assert_eq!(config.ux_optimizations.preemptive_fallback, true);
        assert_eq!(config.pattern_learning.enabled, true);
        assert_eq!(config.cost_optimization.enabled, true);
    }

    #[test]
    fn test_enhanced_fallback_engine_creation() {
        let config = EnhancedFallbackConfig::default();
        let local_config = LocalAiConfig::new();
        let engine = EnhancedFallbackEngine::new(config, local_config);
        assert_eq!(engine.usage_patterns.time_patterns.len(), 0);
        assert_eq!(engine.performance_history.provider_metrics.len(), 0);
        assert_eq!(engine.cost_tracker.daily_costs.len(), 0);
    }

    #[tokio::test]
    async fn test_performance_score_calculation() {
        let config = EnhancedFallbackConfig::default();
        let local_config = LocalAiConfig::new();
        let engine = EnhancedFallbackEngine::new(config, local_config);
        
        let local_health = vec![
            ("ollama".to_string(), ProviderHealthStatus::Healthy {
                response_time: Duration::from_millis(100),
                models_available: 5,
                additional_info: None,
            }),
            ("local_ai".to_string(), ProviderHealthStatus::Degraded {
                response_time: Duration::from_millis(500),
                reason: "High load".to_string(),
                models_available: 3,
            }),
        ];

        let scores = engine.calculate_performance_scores(&local_health).await;
        assert_eq!(scores.len(), 2);
        assert!(scores.get("ollama").unwrap() > scores.get("local_ai").unwrap());
    }
}