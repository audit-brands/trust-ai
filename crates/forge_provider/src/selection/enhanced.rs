//! Enhanced provider selection with intelligent fallback features
//! 
//! This module extends the provider selection system from Phase 6 with Phase 7 enhancements,
//! providing adaptive decision-making, pattern learning, and improved user experience.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use anyhow::Result;
use tracing::{debug, info, warn};

use crate::config::enhanced::{
    EnhancedFallbackConfig, EnhancedFallbackDecision, EnhancedFallbackEngine,
};
use crate::config::fallback::{FallbackContext, FallbackDecision};
use crate::config::local_ai::{LocalAiConfig, ProviderHealthStatus};
use crate::health::HealthMonitor;
use crate::selection::{ProviderMetrics, ProviderSelection, ProviderType, SelectionContext};

/// Enhanced provider selector with intelligent features
pub struct EnhancedProviderSelector {
    local_config: LocalAiConfig,
    enhanced_config: EnhancedFallbackConfig,
    enhanced_engine: EnhancedFallbackEngine,
    health_monitor: HealthMonitor,
    provider_metrics: HashMap<String, ProviderMetrics>,
    current_provider: Option<String>,
    last_fallback_time: Option<Instant>,
    selection_history: Vec<SelectionHistoryEntry>,
    user_feedback: HashMap<String, UserFeedback>,
}

/// Selection history entry for learning
#[derive(Debug, Clone)]
pub struct SelectionHistoryEntry {
    /// Timestamp of selection
    pub timestamp: Instant,
    /// Context of the selection
    pub context: SelectionContext,
    /// Decision made
    pub decision: EnhancedFallbackDecision,
    /// Actual outcome
    pub outcome: Option<SelectionOutcome>,
}

/// Outcome of a provider selection
#[derive(Debug, Clone)]
pub struct SelectionOutcome {
    /// Whether the selection was successful
    pub success: bool,
    /// Actual response time
    pub response_time: Duration,
    /// User satisfaction score (0.0 to 1.0)
    pub user_satisfaction: Option<f64>,
    /// Quality assessment
    pub quality_score: Option<f64>,
    /// Error message if failed
    pub error_message: Option<String>,
}

/// User feedback on provider selections
#[derive(Debug, Clone)]
pub struct UserFeedback {
    /// Provider name
    pub provider_name: String,
    /// Feedback type
    pub feedback_type: FeedbackType,
    /// Rating (1-5)
    pub rating: u8,
    /// Comments
    pub comments: Option<String>,
    /// Timestamp
    pub timestamp: Instant,
}

/// Type of user feedback
#[derive(Debug, Clone)]
pub enum FeedbackType {
    /// General satisfaction
    Satisfaction,
    /// Performance feedback
    Performance,
    /// Quality feedback
    Quality,
    /// Cost feedback
    Cost,
    /// Reliability feedback
    Reliability,
}

/// Enhanced provider selection result
#[derive(Debug, Clone)]
pub struct EnhancedProviderSelection {
    /// Base provider selection
    pub selection: ProviderSelection,
    /// Enhanced decision details
    pub enhanced_decision: EnhancedFallbackDecision,
    /// Recommendation strength (0.0 to 1.0)
    pub recommendation_strength: f64,
    /// User notification message
    pub user_notification: Option<String>,
    /// Suggested alternatives
    pub suggested_alternatives: Vec<String>,
    /// Learning insights
    pub learning_insights: Vec<String>,
}

/// Smart retry configuration
#[derive(Debug, Clone)]
pub struct SmartRetryConfig {
    /// Maximum retry attempts
    pub max_attempts: u32,
    /// Base delay between retries
    pub base_delay: Duration,
    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Whether to try alternative providers
    pub try_alternatives: bool,
}

impl EnhancedProviderSelector {
    /// Create a new enhanced provider selector
    pub async fn new(
        local_config: LocalAiConfig,
        enhanced_config: EnhancedFallbackConfig,
    ) -> Result<Self> {
        let enhanced_engine = EnhancedFallbackEngine::new(enhanced_config.clone(), local_config.clone());
        let health_monitor = HealthMonitor::new(local_config.clone()).await?;

        Ok(Self {
            local_config,
            enhanced_config,
            enhanced_engine,
            health_monitor,
            provider_metrics: HashMap::new(),
            current_provider: None,
            last_fallback_time: None,
            selection_history: Vec::new(),
            user_feedback: HashMap::new(),
        })
    }

    /// Initialize the enhanced provider selector
    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing enhanced provider selector");
        
        // Start health monitoring
        self.health_monitor.start().await?;
        
        // Initialize metrics for all configured providers
        for provider_name in self.local_config.providers.keys() {
            self.provider_metrics.insert(
                provider_name.clone(),
                ProviderMetrics::new(ProviderType::Local),
            );
        }

        // Initialize metrics for cloud providers
        for provider_name in &self.enhanced_config.base_config.cloud_providers {
            self.provider_metrics.insert(
                format!("cloud:{}", provider_name),
                ProviderMetrics::new(ProviderType::Cloud),
            );
        }

        info!("Enhanced provider selector initialized with {} providers", self.provider_metrics.len());
        Ok(())
    }

    /// Select provider with enhanced intelligence
    pub async fn select_provider_enhanced(
        &mut self,
        context: SelectionContext,
    ) -> Result<EnhancedProviderSelection> {
        info!(
            model = %context.model_id,
            streaming = context.requires_streaming,
            tools = context.requires_tools,
            "Selecting provider with enhanced intelligence"
        );

        // Check for seamless switching opportunities
        if self.enhanced_config.ux_optimizations.seamless_switching {
            if let Some(seamless_switch) = self.check_seamless_switching(&context).await {
                return Ok(seamless_switch);
            }
        }

        // Get current health status
        let local_health: Vec<_> = self.health_monitor.get_providers_by_health().await;

        // Create fallback context
        let fallback_context = FallbackContext::new(context.model_id.clone())
            .with_streaming(context.requires_streaming)
            .with_tools(context.requires_tools)
            .with_previous_provider(context.previous_provider.clone().unwrap_or_default())
            .with_consecutive_failures(context.consecutive_failures);

        // Make enhanced fallback decision
        let enhanced_decision = self.enhanced_engine
            .decide_provider_enhanced(&fallback_context, &local_health)
            .await;

        // Convert to enhanced selection
        let enhanced_selection = self.convert_to_enhanced_selection(
            enhanced_decision,
            &local_health,
            &context,
        ).await?;

        // Record selection in history
        self.record_selection_history(&context, &enhanced_selection).await;

        // Update current provider
        self.current_provider = Some(enhanced_selection.selection.provider_name.clone());

        // Generate user notification if needed
        let user_notification = self.generate_user_notification(&enhanced_selection).await;

        info!(
            provider = %enhanced_selection.selection.provider_name,
            confidence = enhanced_selection.enhanced_decision.confidence,
            recommendation_strength = enhanced_selection.recommendation_strength,
            "Enhanced provider selection completed"
        );

        Ok(EnhancedProviderSelection {
            selection: enhanced_selection.selection,
            enhanced_decision: enhanced_selection.enhanced_decision,
            recommendation_strength: enhanced_selection.recommendation_strength,
            user_notification,
            suggested_alternatives: enhanced_selection.suggested_alternatives,
            learning_insights: enhanced_selection.learning_insights,
        })
    }

    /// Check for seamless switching opportunities
    async fn check_seamless_switching(&self, _context: &SelectionContext) -> Option<EnhancedProviderSelection> {
        if let Some(ref current) = self.current_provider {
            // Check if current provider is still optimal
            if let Some(metrics) = self.provider_metrics.get(current) {
                if metrics.is_performing_well(0.8, Duration::from_secs(5)) {
                    // Current provider is still good, continue using it
                    debug!("Seamless switching: continuing with current provider {}", current);
                    
                    // Create a simple selection to continue with current provider
                    // This would be implemented based on the actual selection logic
                    return None; // Simplified for now
                }
            }
        }
        None
    }

    /// Convert enhanced decision to enhanced selection
    async fn convert_to_enhanced_selection(
        &self,
        enhanced_decision: EnhancedFallbackDecision,
        local_health: &[(String, ProviderHealthStatus)],
        context: &SelectionContext,
    ) -> Result<EnhancedProviderSelection> {
        // Convert base decision to provider selection
        let selection = match &enhanced_decision.decision {
            FallbackDecision::UseLocal { provider_name, reason } => {
                ProviderSelection {
                    provider_name: provider_name.clone(),
                    provider_type: ProviderType::Local,
                    reason: reason.clone(),
                    is_fallback: false,
                    local_health: Some(local_health.iter().cloned().collect()),
                }
            }
            FallbackDecision::UseCloud { provider_name, reason, .. } => {
                ProviderSelection {
                    provider_name: format!("cloud:{}", provider_name),
                    provider_type: ProviderType::Cloud,
                    reason: reason.clone(),
                    is_fallback: true,
                    local_health: Some(local_health.iter().cloned().collect()),
                }
            }
            FallbackDecision::RequireManual { reason, available_options } => {
                return Err(anyhow::anyhow!(
                    "Manual provider selection required: {}. Available options: {:?}",
                    reason,
                    available_options
                ));
            }
            FallbackDecision::NoProvider { reason, attempted_providers } => {
                return Err(anyhow::anyhow!(
                    "No suitable provider available: {}. Attempted: {:?}",
                    reason,
                    attempted_providers
                ));
            }
        };

        // Calculate recommendation strength
        let recommendation_strength = self.calculate_recommendation_strength(&enhanced_decision, context).await;

        // Generate suggested alternatives
        let suggested_alternatives = enhanced_decision.alternatives
            .iter()
            .take(3)
            .map(|alt| alt.provider_name.clone())
            .collect();

        // Generate learning insights
        let learning_insights = self.generate_learning_insights(&enhanced_decision).await;

        Ok(EnhancedProviderSelection {
            selection,
            enhanced_decision,
            recommendation_strength,
            user_notification: None, // Will be set later
            suggested_alternatives,
            learning_insights,
        })
    }

    /// Calculate recommendation strength
    async fn calculate_recommendation_strength(&self, enhanced_decision: &EnhancedFallbackDecision, _context: &SelectionContext) -> f64 {
        let mut strength = enhanced_decision.confidence;

        // Boost strength based on performance prediction
        if let Some(ref perf_pred) = enhanced_decision.performance_prediction {
            if perf_pred.expected_success_rate > 0.9 {
                strength += 0.1;
            }
            if perf_pred.expected_response_time < Duration::from_secs(2) {
                strength += 0.05;
            }
        }

        // Boost strength based on cost optimization
        if let Some(ref cost_impact) = enhanced_decision.cost_impact {
            if cost_impact.cost_savings > 0.0 {
                strength += 0.05;
            }
        }

        // Cap at 1.0
        strength.min(1.0)
    }

    /// Generate learning insights
    async fn generate_learning_insights(&self, enhanced_decision: &EnhancedFallbackDecision) -> Vec<String> {
        let mut insights = Vec::new();

        // Confidence-based insights
        if enhanced_decision.confidence > 0.9 {
            insights.push("High confidence in provider selection".to_string());
        } else if enhanced_decision.confidence < 0.6 {
            insights.push("Low confidence - consider manual review".to_string());
        }

        // Cost-based insights
        if let Some(ref cost_impact) = enhanced_decision.cost_impact {
            if cost_impact.cost_savings > 0.001 {
                insights.push(format!("Cost savings: ${:.4} per request", cost_impact.cost_savings));
            }
        }

        // Performance-based insights
        if let Some(ref perf_pred) = enhanced_decision.performance_prediction {
            if perf_pred.expected_response_time < Duration::from_millis(500) {
                insights.push("Fast response time expected".to_string());
            }
            if perf_pred.expected_success_rate > 0.95 {
                insights.push("High reliability expected".to_string());
            }
        }

        insights
    }

    /// Record selection in history for learning
    async fn record_selection_history(&mut self, context: &SelectionContext, selection: &EnhancedProviderSelection) {
        let entry = SelectionHistoryEntry {
            timestamp: Instant::now(),
            context: context.clone(),
            decision: selection.enhanced_decision.clone(),
            outcome: None, // Will be updated when outcome is known
        };

        self.selection_history.push(entry);

        // Keep only recent history (last 1000 entries)
        if self.selection_history.len() > 1000 {
            self.selection_history.remove(0);
        }
    }

    /// Generate user notification
    async fn generate_user_notification(&self, selection: &EnhancedProviderSelection) -> Option<String> {
        if !self.enhanced_config.ux_optimizations.context_aware_notifications {
            return None;
        }

        let mut notifications = Vec::new();

        // Fallback notification
        if selection.selection.is_fallback {
            notifications.push("Switched to cloud provider due to local unavailability".to_string());
        }

        // Performance notification
        if let Some(ref perf_pred) = selection.enhanced_decision.performance_prediction {
            if perf_pred.expected_response_time > Duration::from_secs(10) {
                notifications.push("Slower response expected due to provider load".to_string());
            }
        }

        // Cost notification
        if let Some(ref cost_impact) = selection.enhanced_decision.cost_impact {
            if cost_impact.cost_savings > 0.01 {
                notifications.push(format!("Cost-optimized selection saves ${:.3}", cost_impact.cost_savings));
            }
        }

        // Low confidence notification
        if selection.enhanced_decision.confidence < 0.6 {
            notifications.push("Provider selection has low confidence - manual review recommended".to_string());
        }

        if notifications.is_empty() {
            None
        } else {
            Some(notifications.join("; "))
        }
    }

    /// Record successful request with enhanced learning
    pub async fn record_success_enhanced(
        &mut self,
        provider_name: &str,
        context: &SelectionContext,
        response_time: Duration,
        quality_score: Option<f64>,
    ) {
        // Record in base metrics
        if let Some(metrics) = self.provider_metrics.get_mut(provider_name) {
            metrics.total_requests += 1;
            metrics.successful_requests += 1;
            
            // Update average response time
            let total_requests = metrics.total_requests as f64;
            let current_avg = metrics.avg_response_time.as_millis() as f64;
            let new_time = response_time.as_millis() as f64;
            let new_avg = (current_avg * (total_requests - 1.0) + new_time) / total_requests;
            
            metrics.avg_response_time = Duration::from_millis(new_avg as u64);
            metrics.last_request_time = Some(Instant::now());
        }

        // Record in enhanced engine for pattern learning
        let fallback_context = FallbackContext::new(context.model_id.clone())
            .with_streaming(context.requires_streaming)
            .with_tools(context.requires_tools)
            .with_previous_provider(context.previous_provider.clone().unwrap_or_default())
            .with_consecutive_failures(context.consecutive_failures);

        self.enhanced_engine.record_usage(provider_name, &fallback_context, true, response_time).await;

        // Update selection history outcome
        if let Some(last_entry) = self.selection_history.last_mut() {
            if last_entry.outcome.is_none() {
                last_entry.outcome = Some(SelectionOutcome {
                    success: true,
                    response_time,
                    user_satisfaction: None,
                    quality_score,
                    error_message: None,
                });
            }
        }

        debug!(
            provider = provider_name,
            response_time_ms = response_time.as_millis(),
            quality_score = ?quality_score,
            "Enhanced success recording completed"
        );
    }

    /// Record failed request with enhanced learning
    pub async fn record_failure_enhanced(
        &mut self,
        provider_name: &str,
        context: &SelectionContext,
        error: &str,
        response_time: Option<Duration>,
    ) {
        // Record in base metrics
        if let Some(metrics) = self.provider_metrics.get_mut(provider_name) {
            metrics.total_requests += 1;
            metrics.last_request_time = Some(Instant::now());
        }

        // Record in enhanced engine for pattern learning
        let fallback_context = FallbackContext::new(context.model_id.clone())
            .with_streaming(context.requires_streaming)
            .with_tools(context.requires_tools)
            .with_previous_provider(context.previous_provider.clone().unwrap_or_default())
            .with_consecutive_failures(context.consecutive_failures);

        self.enhanced_engine.record_usage(
            provider_name,
            &fallback_context,
            false,
            response_time.unwrap_or(Duration::from_secs(30))
        ).await;

        // Update selection history outcome
        if let Some(last_entry) = self.selection_history.last_mut() {
            if last_entry.outcome.is_none() {
                last_entry.outcome = Some(SelectionOutcome {
                    success: false,
                    response_time: response_time.unwrap_or(Duration::from_secs(30)),
                    user_satisfaction: None,
                    quality_score: None,
                    error_message: Some(error.to_string()),
                });
            }
        }

        warn!(
            provider = provider_name,
            error = error,
            response_time_ms = response_time.map(|d| d.as_millis()),
            "Enhanced failure recording completed"
        );
    }

    /// Record user feedback for learning
    pub async fn record_user_feedback(&mut self, feedback: UserFeedback) {
        info!(
            provider = %feedback.provider_name,
            feedback_type = ?feedback.feedback_type,
            rating = feedback.rating,
            "Recording user feedback"
        );

        self.user_feedback.insert(
            format!("{}_{}", feedback.provider_name, feedback.timestamp.elapsed().as_secs()),
            feedback,
        );

        // Keep only recent feedback (last 100 entries)
        if self.user_feedback.len() > 100 {
            // Remove oldest entry
            if let Some(oldest_key) = self.user_feedback.keys().next().cloned() {
                self.user_feedback.remove(&oldest_key);
            }
        }
    }

    /// Get smart retry configuration
    pub fn get_smart_retry_config(&self) -> SmartRetryConfig {
        SmartRetryConfig {
            max_attempts: if self.enhanced_config.ux_optimizations.smart_retry { 5 } else { 3 },
            base_delay: Duration::from_millis(500),
            backoff_multiplier: 1.5,
            max_delay: Duration::from_secs(10),
            try_alternatives: self.enhanced_config.ux_optimizations.seamless_switching,
        }
    }

    /// Get provider recommendations based on learning
    pub async fn get_provider_recommendations(&self, _context: &SelectionContext) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Add providers based on historical success
        for (provider_name, metrics) in &self.provider_metrics {
            if metrics.success_rate() > 0.8 && metrics.total_requests > 10 {
                recommendations.push(provider_name.clone());
            }
        }

        // Sort by success rate and response time
        recommendations.sort_by(|a, b| {
            let metrics_a = self.provider_metrics.get(a).unwrap();
            let metrics_b = self.provider_metrics.get(b).unwrap();
            
            let score_a = metrics_a.success_rate() - (metrics_a.avg_response_time.as_millis() as f64 / 10000.0);
            let score_b = metrics_b.success_rate() - (metrics_b.avg_response_time.as_millis() as f64 / 10000.0);
            
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        recommendations
    }

    /// Get learning insights from historical data
    pub async fn get_learning_insights(&self) -> Vec<String> {
        let mut insights = Vec::new();

        // Analyze selection history
        if self.selection_history.len() > 10 {
            let recent_selections = &self.selection_history[self.selection_history.len().saturating_sub(10)..];
            
            // Find most successful provider
            let mut provider_success: HashMap<String, (u32, u32)> = HashMap::new();
            for entry in recent_selections {
                if let Some(ref outcome) = entry.outcome {
                    let provider_name = entry.decision.decision.provider_name().unwrap_or("unknown");
                    let (successes, total) = provider_success.entry(provider_name.to_string()).or_insert((0, 0));
                    *total += 1;
                    if outcome.success {
                        *successes += 1;
                    }
                }
            }

            if let Some((best_provider, (successes, total))) = provider_success.iter()
                .max_by_key(|(_, (s, t))| (*s as f64 / *t as f64 * 1000.0) as u32) {
                if *total > 3 {
                    insights.push(format!(
                        "Provider {} has {:.1}% success rate in recent requests",
                        best_provider,
                        (*successes as f64 / *total as f64) * 100.0
                    ));
                }
            }
        }

        // Analyze user feedback
        if !self.user_feedback.is_empty() {
            let avg_rating: f64 = self.user_feedback.values()
                .map(|f| f.rating as f64)
                .sum::<f64>() / self.user_feedback.len() as f64;
            
            insights.push(format!("Average user rating: {:.1}/5.0", avg_rating));
        }

        insights
    }
}

impl Default for SmartRetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(1000),
            backoff_multiplier: 2.0,
            max_delay: Duration::from_secs(30),
            try_alternatives: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::local_ai::LocalAiConfig;
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn test_enhanced_provider_selector_creation() {
        let local_config = LocalAiConfig::with_default_ollama();
        let enhanced_config = EnhancedFallbackConfig::default();
        
        let actual = EnhancedProviderSelector::new(local_config, enhanced_config).await;
        assert!(actual.is_ok());
    }

    #[test]
    fn test_smart_retry_config_default() {
        let config = SmartRetryConfig::default();
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.base_delay, Duration::from_millis(1000));
        assert_eq!(config.backoff_multiplier, 2.0);
        assert_eq!(config.try_alternatives, true);
    }

    #[test]
    fn test_selection_outcome_creation() {
        let outcome = SelectionOutcome {
            success: true,
            response_time: Duration::from_millis(500),
            user_satisfaction: Some(0.9),
            quality_score: Some(0.85),
            error_message: None,
        };
        
        assert_eq!(outcome.success, true);
        assert_eq!(outcome.response_time, Duration::from_millis(500));
        assert_eq!(outcome.user_satisfaction, Some(0.9));
    }
}