//! Provider selection and management logic

use std::collections::HashMap;
use std::time::{Duration, Instant};

use anyhow::Context as _;
use tracing::{debug, info, warn};

use crate::config::fallback::{FallbackConfig, FallbackContext, FallbackDecision, FallbackEngine};
use crate::config::local_ai::{LocalAiConfig, ProviderHealthStatus};
use crate::health::HealthMonitor;

/// Provider selection and management service
pub struct ProviderSelector {
    local_config: LocalAiConfig,
    fallback_config: FallbackConfig,
    fallback_engine: FallbackEngine,
    health_monitor: HealthMonitor,
    provider_metrics: HashMap<String, ProviderMetrics>,
    current_provider: Option<String>,
    last_fallback_time: Option<Instant>,
}

/// Performance metrics for a provider
#[derive(Debug, Clone)]
pub struct ProviderMetrics {
    /// Total requests made
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Average response time
    pub avg_response_time: Duration,
    /// Last request timestamp
    pub last_request_time: Option<Instant>,
    /// Provider type (local or cloud)
    pub provider_type: ProviderType,
}

/// Type of provider
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderType {
    Local,
    Cloud,
}

/// Result of provider selection
#[derive(Debug, Clone)]
pub struct ProviderSelection {
    /// Selected provider name
    pub provider_name: String,
    /// Provider type
    pub provider_type: ProviderType,
    /// Reason for selection
    pub reason: String,
    /// Whether this is a fallback selection
    pub is_fallback: bool,
    /// Health status of local providers (if relevant)
    pub local_health: Option<HashMap<String, ProviderHealthStatus>>,
}

/// Provider selection context
#[derive(Debug, Clone)]
pub struct SelectionContext {
    /// Model being requested
    pub model_id: String,
    /// Whether streaming is required
    pub requires_streaming: bool,
    /// Whether tools are required
    pub requires_tools: bool,
    /// User preferences (if any)
    pub user_preferences: Option<UserPreferences>,
    /// Previous provider used
    pub previous_provider: Option<String>,
    /// Number of consecutive failures
    pub consecutive_failures: u32,
}

/// User preferences for provider selection
#[derive(Debug, Clone)]
pub struct UserPreferences {
    /// Preferred provider order
    pub preferred_providers: Vec<String>,
    /// Whether to allow fallback
    pub allow_fallback: bool,
    /// Maximum acceptable response time
    pub max_response_time: Option<Duration>,
    /// Prefer local providers
    pub prefer_local: bool,
}

impl ProviderSelector {
    /// Create a new provider selector
    pub async fn new(
        local_config: LocalAiConfig,
        fallback_config: FallbackConfig,
    ) -> anyhow::Result<Self> {
        let fallback_engine = FallbackEngine::new(fallback_config.clone(), local_config.clone());
        let health_monitor = HealthMonitor::new(local_config.clone()).await?;

        Ok(Self {
            local_config,
            fallback_config,
            fallback_engine,
            health_monitor,
            provider_metrics: HashMap::new(),
            current_provider: None,
            last_fallback_time: None,
        })
    }

    /// Initialize the provider selector
    pub async fn initialize(&mut self) -> anyhow::Result<()> {
        info!("Initializing provider selector");
        
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
        for provider_name in &self.fallback_config.cloud_providers {
            self.provider_metrics.insert(
                format!("cloud:{}", provider_name),
                ProviderMetrics::new(ProviderType::Cloud),
            );
        }

        info!("Provider selector initialized with {} providers", self.provider_metrics.len());
        Ok(())
    }

    /// Select the best provider for a request
    pub async fn select_provider(
        &mut self,
        context: SelectionContext,
    ) -> anyhow::Result<ProviderSelection> {
        info!(
            model = %context.model_id,
            streaming = context.requires_streaming,
            tools = context.requires_tools,
            "Selecting provider"
        );

        // Check if we should return to local provider
        if let Some(local_provider) = self.check_return_to_local().await {
            self.current_provider = Some(local_provider.clone());
            return Ok(ProviderSelection {
                provider_name: local_provider.clone(),
                provider_type: ProviderType::Local,
                reason: "Returned to healthy local provider".to_string(),
                is_fallback: false,
                local_health: Some(self.health_monitor.get_health_status().await),
            });
        }

        // Get current health status
        let local_health: Vec<_> = self.health_monitor.get_providers_by_health().await;

        // Create fallback context
        let fallback_context = FallbackContext::new(context.model_id.clone())
            .with_streaming(context.requires_streaming)
            .with_tools(context.requires_tools)
            .with_previous_provider(context.previous_provider.clone().unwrap_or_default())
            .with_consecutive_failures(context.consecutive_failures);

        // Make fallback decision
        let decision = self.fallback_engine
            .decide_provider(&fallback_context, &local_health)
            .await;

        // Convert decision to selection
        let selection = self.convert_decision_to_selection(decision, &local_health, &context)?;

        // Update current provider
        self.current_provider = Some(selection.provider_name.clone());

        // Update metrics
        self.update_selection_metrics(&selection);

        info!(
            provider = %selection.provider_name,
            provider_type = ?selection.provider_type,
            reason = %selection.reason,
            "Provider selected"
        );

        Ok(selection)
    }

    /// Check if we should return to a local provider
    async fn check_return_to_local(&self) -> Option<String> {
        // Only check if we're currently using a cloud provider
        if let Some(ref current) = self.current_provider {
            if current.starts_with("cloud:") {
                if let Some(fallback_time) = self.last_fallback_time {
                    let time_since_fallback = fallback_time.elapsed();
                    let local_health: Vec<_> = self.health_monitor.get_providers_by_health().await;
                    
                    return self.fallback_engine.should_return_to_local(
                        current,
                        &local_health,
                        time_since_fallback,
                    );
                }
            }
        }
        None
    }

    /// Convert fallback decision to provider selection
    fn convert_decision_to_selection(
        &mut self,
        decision: FallbackDecision,
        local_health: &[(String, ProviderHealthStatus)],
        _context: &SelectionContext,
    ) -> anyhow::Result<ProviderSelection> {
        match decision {
            FallbackDecision::UseLocal { provider_name, reason } => {
                Ok(ProviderSelection {
                    provider_name,
                    provider_type: ProviderType::Local,
                    reason,
                    is_fallback: false,
                    local_health: Some(local_health.iter().cloned().collect()),
                })
            }
            FallbackDecision::UseCloud { provider_name, reason, .. } => {
                // Mark fallback time
                self.last_fallback_time = Some(Instant::now());
                
                Ok(ProviderSelection {
                    provider_name: format!("cloud:{}", provider_name),
                    provider_type: ProviderType::Cloud,
                    reason,
                    is_fallback: true,
                    local_health: Some(local_health.iter().cloned().collect()),
                })
            }
            FallbackDecision::RequireManual { reason, available_options } => {
                anyhow::bail!(
                    "Manual provider selection required: {}. Available options: {:?}",
                    reason,
                    available_options
                );
            }
            FallbackDecision::NoProvider { reason, attempted_providers } => {
                anyhow::bail!(
                    "No suitable provider available: {}. Attempted: {:?}",
                    reason,
                    attempted_providers
                );
            }
        }
    }

    /// Update metrics after provider selection
    fn update_selection_metrics(&mut self, selection: &ProviderSelection) {
        if let Some(metrics) = self.provider_metrics.get_mut(&selection.provider_name) {
            metrics.total_requests += 1;
            metrics.last_request_time = Some(Instant::now());
        }
    }

    /// Record a successful request
    pub fn record_success(&mut self, provider_name: &str, response_time: Duration) {
        if let Some(metrics) = self.provider_metrics.get_mut(provider_name) {
            metrics.successful_requests += 1;
            
            // Update average response time (simple moving average)
            let total_requests = metrics.total_requests as f64;
            let current_avg = metrics.avg_response_time.as_millis() as f64;
            let new_time = response_time.as_millis() as f64;
            let new_avg = (current_avg * (total_requests - 1.0) + new_time) / total_requests;
            
            metrics.avg_response_time = Duration::from_millis(new_avg as u64);
        }

        debug!(
            provider = provider_name,
            response_time_ms = response_time.as_millis(),
            "Recorded successful request"
        );
    }

    /// Record a failed request
    pub fn record_failure(&mut self, provider_name: &str, error: &str) {
        warn!(
            provider = provider_name,
            error = error,
            "Recorded failed request"
        );
        
        // Metrics are already updated in update_selection_metrics
        // Failure tracking is handled by the health monitor
    }

    /// Get current provider metrics
    pub fn get_provider_metrics(&self) -> &HashMap<String, ProviderMetrics> {
        &self.provider_metrics
    }

    /// Get metrics for a specific provider
    pub fn get_provider_metric(&self, provider_name: &str) -> Option<&ProviderMetrics> {
        self.provider_metrics.get(provider_name)
    }

    /// Get current provider
    pub fn current_provider(&self) -> Option<&str> {
        self.current_provider.as_deref()
    }

    /// Force a health check for all providers
    pub async fn refresh_health(&self) -> anyhow::Result<HashMap<String, ProviderHealthStatus>> {
        self.health_monitor.force_check_all().await
    }

    /// Get current health status for all providers
    pub async fn get_health_status(&self) -> HashMap<String, ProviderHealthStatus> {
        self.health_monitor.get_health_status().await
    }

    /// Check if a specific provider is available
    pub async fn is_provider_available(&self, provider_name: &str) -> bool {
        if provider_name.starts_with("cloud:") {
            // For cloud providers, assume available unless we have metrics showing otherwise
            true
        } else {
            self.health_monitor.is_provider_usable(provider_name).await
        }
    }

    /// Get recommended providers for a specific model
    pub async fn get_recommended_providers(&self, model_id: &str) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        // First, add healthy local providers that support the model
        let local_health = self.health_monitor.get_providers_by_health().await;
        for (provider_name, status) in local_health {
            if status.is_usable() && self.provider_supports_model(&provider_name, model_id) {
                recommendations.push(provider_name);
            }
        }
        
        // Then add cloud providers
        for cloud_provider in &self.fallback_config.cloud_providers {
            recommendations.push(format!("cloud:{}", cloud_provider));
        }
        
        recommendations
    }

    /// Check if a provider supports a specific model
    fn provider_supports_model(&self, provider_name: &str, model_id: &str) -> bool {
        if let Some(provider_config) = self.local_config.providers.get(provider_name) {
            if provider_config.preferred_models.is_empty() {
                return true;
            }
            
            provider_config.preferred_models.iter().any(|preferred| {
                preferred == model_id || 
                model_id.starts_with(&preferred.replace(":latest", ""))
            })
        } else {
            // For cloud providers, assume model support
            true
        }
    }
}

impl ProviderMetrics {
    /// Create new provider metrics
    pub fn new(provider_type: ProviderType) -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            avg_response_time: Duration::from_millis(0),
            last_request_time: None,
            provider_type,
        }
    }

    /// Get success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        self.successful_requests as f64 / self.total_requests as f64
    }

    /// Check if provider is performing well
    pub fn is_performing_well(&self, min_success_rate: f64, max_response_time: Duration) -> bool {
        self.success_rate() >= min_success_rate && self.avg_response_time <= max_response_time
    }

    /// Get time since last request
    pub fn time_since_last_request(&self) -> Option<Duration> {
        self.last_request_time.map(|time| time.elapsed())
    }
}

impl SelectionContext {
    /// Create a new selection context
    pub fn new(model_id: String) -> Self {
        Self {
            model_id,
            requires_streaming: false,
            requires_tools: false,
            user_preferences: None,
            previous_provider: None,
            consecutive_failures: 0,
        }
    }

    /// Set streaming requirement
    pub fn with_streaming(mut self, streaming: bool) -> Self {
        self.requires_streaming = streaming;
        self
    }

    /// Set tools requirement
    pub fn with_tools(mut self, tools: bool) -> Self {
        self.requires_tools = tools;
        self
    }

    /// Set user preferences
    pub fn with_preferences(mut self, preferences: UserPreferences) -> Self {
        self.user_preferences = Some(preferences);
        self
    }

    /// Set previous provider
    pub fn with_previous_provider(mut self, provider: String) -> Self {
        self.previous_provider = Some(provider);
        self
    }

    /// Set consecutive failures
    pub fn with_consecutive_failures(mut self, failures: u32) -> Self {
        self.consecutive_failures = failures;
        self
    }
}

impl UserPreferences {
    /// Create default user preferences
    pub fn default() -> Self {
        Self {
            preferred_providers: vec![],
            allow_fallback: true,
            max_response_time: Some(Duration::from_secs(30)),
            prefer_local: true,
        }
    }

    /// Create preferences that prefer local providers
    pub fn prefer_local() -> Self {
        Self {
            preferred_providers: vec![],
            allow_fallback: true,
            max_response_time: Some(Duration::from_secs(10)),
            prefer_local: true,
        }
    }

    /// Create preferences that prefer cloud providers
    pub fn prefer_cloud() -> Self {
        Self {
            preferred_providers: vec!["cloud:openai".to_string(), "cloud:anthropic".to_string()],
            allow_fallback: false,
            max_response_time: Some(Duration::from_secs(30)),
            prefer_local: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::local_ai::LocalAiConfig;
    use crate::config::fallback::FallbackConfig;
    use pretty_assertions::assert_eq;
    use std::time::Duration;

    #[tokio::test]
    async fn test_provider_selector_creation() {
        let local_config = LocalAiConfig::with_default_ollama();
        let fallback_config = FallbackConfig::default();
        
        let actual = ProviderSelector::new(local_config, fallback_config).await;
        assert!(actual.is_ok());
    }

    #[test]
    fn test_provider_metrics_success_rate() {
        let mut fixture = ProviderMetrics::new(ProviderType::Local);
        fixture.total_requests = 10;
        fixture.successful_requests = 8;
        
        let actual = fixture.success_rate();
        let expected = 0.8;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_provider_metrics_performance() {
        let fixture = ProviderMetrics {
            total_requests: 5,
            successful_requests: 4,
            avg_response_time: Duration::from_millis(200),
            last_request_time: Some(Instant::now()),
            provider_type: ProviderType::Local,
        };

        assert!(fixture.is_performing_well(0.7, Duration::from_millis(300)));
        assert!(!fixture.is_performing_well(0.9, Duration::from_millis(300)));
        assert!(!fixture.is_performing_well(0.7, Duration::from_millis(100)));
    }

    #[test]
    fn test_selection_context_builder() {
        let fixture = SelectionContext::new("test-model".to_string())
            .with_streaming(true)
            .with_tools(true)
            .with_previous_provider("ollama".to_string())
            .with_consecutive_failures(2);

        assert_eq!(fixture.model_id, "test-model");
        assert_eq!(fixture.requires_streaming, true);
        assert_eq!(fixture.requires_tools, true);
        assert_eq!(fixture.previous_provider, Some("ollama".to_string()));
        assert_eq!(fixture.consecutive_failures, 2);
    }

    #[test]
    fn test_user_preferences_variants() {
        let local_pref = UserPreferences::prefer_local();
        assert_eq!(local_pref.prefer_local, true);
        assert_eq!(local_pref.allow_fallback, true);

        let cloud_pref = UserPreferences::prefer_cloud();
        assert_eq!(cloud_pref.prefer_local, false);
        assert_eq!(cloud_pref.allow_fallback, false);
        assert!(cloud_pref.preferred_providers.len() > 0);
    }

    #[test]
    fn test_provider_selection_properties() {
        let fixture = ProviderSelection {
            provider_name: "ollama".to_string(),
            provider_type: ProviderType::Local,
            reason: "Healthy local provider".to_string(),
            is_fallback: false,
            local_health: None,
        };

        assert_eq!(fixture.provider_type, ProviderType::Local);
        assert_eq!(fixture.is_fallback, false);
    }
}