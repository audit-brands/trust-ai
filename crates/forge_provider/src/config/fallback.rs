use std::time::Duration;

use derive_setters::Setters;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use super::local_ai::{LocalAiConfig, ProviderHealthStatus};

/// Configuration for provider fallback behavior
#[derive(Debug, Clone, Serialize, Deserialize, Setters)]
#[setters(strip_option, into)]
pub struct FallbackConfig {
    /// Fallback strategy to use
    pub strategy: FallbackStrategy,
    /// Cloud providers to fallback to
    pub cloud_providers: Vec<String>,
    /// Whether to notify user about fallback
    pub notify_user: bool,
    /// Maximum retry attempts before fallback
    pub max_retries: u32,
    /// Delay between retry attempts in milliseconds
    pub retry_delay_ms: u64,
    /// Timeout for fallback decision in seconds
    pub decision_timeout_seconds: u64,
    /// Whether to automatically return to local when available
    pub auto_return_to_local: bool,
    /// Minimum time to wait before returning to local in seconds
    pub local_recovery_delay_seconds: u64,
}

/// Fallback strategy options
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FallbackStrategy {
    /// Graceful fallback with user notification
    Graceful,
    /// Immediate fallback without delay
    Immediate,
    /// Manual fallback requiring user confirmation
    Manual,
    /// No fallback, fail if local unavailable
    None,
}

/// Result of a fallback decision
#[derive(Debug, Clone)]
pub enum FallbackDecision {
    /// Use local provider
    UseLocal {
        provider_name: String,
        reason: String,
    },
    /// Fallback to cloud provider
    UseCloud {
        provider_name: String,
        reason: String,
        local_status: Option<ProviderHealthStatus>,
    },
    /// Manual intervention required
    RequireManual {
        reason: String,
        available_options: Vec<String>,
    },
    /// No suitable provider available
    NoProvider {
        reason: String,
        attempted_providers: Vec<String>,
    },
}

/// Context for fallback decisions
#[derive(Debug, Clone)]
pub struct FallbackContext {
    /// Current model being requested
    pub model_id: String,
    /// Whether this is a streaming request
    pub is_streaming: bool,
    /// Whether tools are required
    pub requires_tools: bool,
    /// Previous provider used (if any)
    pub previous_provider: Option<String>,
    /// Number of consecutive failures
    pub consecutive_failures: u32,
    /// Time since last successful request
    pub time_since_last_success: Option<Duration>,
}

impl Default for FallbackConfig {
    fn default() -> Self {
        Self {
            strategy: FallbackStrategy::Graceful,
            cloud_providers: vec!["openai".to_string(), "anthropic".to_string()],
            notify_user: true,
            max_retries: 3,
            retry_delay_ms: 1000,
            decision_timeout_seconds: 10,
            auto_return_to_local: true,
            local_recovery_delay_seconds: 60,
        }
    }
}

impl FallbackConfig {
    /// Create a new fallback configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate the fallback configuration
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.max_retries > 10 {
            warn!("Max retries of {} is very high", self.max_retries);
        }

        if self.retry_delay_ms > 10000 {
            warn!("Retry delay of {} ms is very high", self.retry_delay_ms);
        }

        if self.decision_timeout_seconds == 0 {
            anyhow::bail!("Decision timeout cannot be zero");
        }

        if self.decision_timeout_seconds > 60 {
            warn!(
                "Decision timeout of {} seconds is very high",
                self.decision_timeout_seconds
            );
        }

        if self.cloud_providers.is_empty() && self.strategy != FallbackStrategy::None {
            warn!("No cloud providers configured for fallback");
        }

        debug!("Fallback configuration validated successfully");
        Ok(())
    }

    /// Get retry delay as Duration
    pub fn retry_delay(&self) -> Duration {
        Duration::from_millis(self.retry_delay_ms)
    }

    /// Get decision timeout as Duration
    pub fn decision_timeout(&self) -> Duration {
        Duration::from_secs(self.decision_timeout_seconds)
    }

    /// Get local recovery delay as Duration
    pub fn local_recovery_delay(&self) -> Duration {
        Duration::from_secs(self.local_recovery_delay_seconds)
    }
}

/// Fallback decision engine
pub struct FallbackEngine {
    config: FallbackConfig,
    local_config: LocalAiConfig,
}

impl FallbackEngine {
    /// Create a new fallback engine
    pub fn new(config: FallbackConfig, local_config: LocalAiConfig) -> Self {
        Self { config, local_config }
    }

    /// Make a fallback decision based on current context and provider health
    pub async fn decide_provider(
        &self,
        context: &FallbackContext,
        local_health: &[(String, ProviderHealthStatus)],
    ) -> FallbackDecision {
        info!(
            strategy = ?self.config.strategy,
            model = %context.model_id,
            consecutive_failures = context.consecutive_failures,
            "Making fallback decision"
        );

        match self.config.strategy {
            FallbackStrategy::None => self.decide_local_only(context, local_health).await,
            FallbackStrategy::Manual => self.decide_manual(context, local_health).await,
            FallbackStrategy::Immediate => self.decide_immediate(context, local_health).await,
            FallbackStrategy::Graceful => self.decide_graceful(context, local_health).await,
        }
    }

    /// Decide provider when only local is allowed
    async fn decide_local_only(
        &self,
        context: &FallbackContext,
        local_health: &[(String, ProviderHealthStatus)],
    ) -> FallbackDecision {
        if let Some((name, _status)) = self.find_healthy_local_provider(context, local_health) {
            FallbackDecision::UseLocal {
                provider_name: name.clone(),
                reason: "Local provider available and healthy".to_string(),
            }
        } else {
            FallbackDecision::NoProvider {
                reason: "No local providers available and fallback disabled".to_string(),
                attempted_providers: local_health.iter().map(|(name, _)| name.clone()).collect(),
            }
        }
    }

    /// Decide provider with manual intervention required
    async fn decide_manual(
        &self,
        context: &FallbackContext,
        local_health: &[(String, ProviderHealthStatus)],
    ) -> FallbackDecision {
        if let Some((name, _)) = self.find_healthy_local_provider(context, local_health) {
            FallbackDecision::UseLocal {
                provider_name: name.clone(),
                reason: "Local provider available".to_string(),
            }
        } else {
            let mut options = Vec::new();

            // Add degraded local providers as options
            for (name, status) in local_health {
                if matches!(status, ProviderHealthStatus::Degraded { .. }) {
                    options.push(format!("local:{name}"));
                }
            }

            // Add cloud providers as options
            for provider in &self.config.cloud_providers {
                options.push(format!("cloud:{provider}"));
            }

            FallbackDecision::RequireManual {
                reason: "No healthy local providers, manual selection required".to_string(),
                available_options: options,
            }
        }
    }

    /// Decide provider with immediate fallback
    async fn decide_immediate(
        &self,
        context: &FallbackContext,
        local_health: &[(String, ProviderHealthStatus)],
    ) -> FallbackDecision {
        if let Some((name, _)) = self.find_healthy_local_provider(context, local_health) {
            FallbackDecision::UseLocal {
                provider_name: name.clone(),
                reason: "Local provider available and healthy".to_string(),
            }
        } else if let Some(cloud_provider) = self.select_cloud_provider(context) {
            let local_status = local_health.first().map(|(_, status)| status.clone());
            FallbackDecision::UseCloud {
                provider_name: cloud_provider,
                reason: "No healthy local providers, immediate fallback to cloud".to_string(),
                local_status,
            }
        } else {
            FallbackDecision::NoProvider {
                reason: "No local or cloud providers available".to_string(),
                attempted_providers: local_health.iter().map(|(name, _)| name.clone()).collect(),
            }
        }
    }

    /// Decide provider with graceful fallback
    async fn decide_graceful(
        &self,
        context: &FallbackContext,
        local_health: &[(String, ProviderHealthStatus)],
    ) -> FallbackDecision {
        // Check if we should retry local providers
        if context.consecutive_failures < self.config.max_retries {
            if let Some((name, status)) = self.find_usable_local_provider(context, local_health) {
                let reason = match status {
                    ProviderHealthStatus::Healthy { .. } => "Local provider healthy".to_string(),
                    ProviderHealthStatus::Degraded { .. } => {
                        format!(
                            "Local provider degraded, attempting retry {}/{}",
                            context.consecutive_failures + 1,
                            self.config.max_retries
                        )
                    }
                    _ => "Local provider status unknown".to_string(),
                };

                return FallbackDecision::UseLocal { provider_name: name.clone(), reason };
            }
        }

        // Fallback to cloud if retries exhausted
        if let Some(cloud_provider) = self.select_cloud_provider(context) {
            let local_status = local_health.first().map(|(_, status)| status.clone());
            FallbackDecision::UseCloud {
                provider_name: cloud_provider,
                reason: format!(
                    "Local providers failed after {} retries, falling back to cloud",
                    context.consecutive_failures
                ),
                local_status,
            }
        } else {
            FallbackDecision::NoProvider {
                reason: "No local or cloud providers available after retries".to_string(),
                attempted_providers: local_health.iter().map(|(name, _)| name.clone()).collect(),
            }
        }
    }

    /// Find a healthy local provider that supports the requested model
    fn find_healthy_local_provider<'a>(
        &self,
        context: &FallbackContext,
        local_health: &'a [(String, ProviderHealthStatus)],
    ) -> Option<&'a (String, ProviderHealthStatus)> {
        local_health.iter().find(|(name, status)| {
            matches!(status, ProviderHealthStatus::Healthy { .. })
                && self.provider_supports_model(name, &context.model_id)
        })
    }

    /// Find a usable local provider (healthy or degraded) that supports the
    /// requested model
    fn find_usable_local_provider<'a>(
        &self,
        context: &FallbackContext,
        local_health: &'a [(String, ProviderHealthStatus)],
    ) -> Option<&'a (String, ProviderHealthStatus)> {
        local_health.iter().find(|(name, status)| {
            status.is_usable() && self.provider_supports_model(name, &context.model_id)
        })
    }

    /// Check if a provider supports the requested model
    fn provider_supports_model(&self, provider_name: &str, model_id: &str) -> bool {
        if let Some(provider_config) = self.local_config.providers.get(provider_name) {
            if provider_config.preferred_models.is_empty() {
                // If no preferred models specified, assume all models are supported
                return true;
            }

            // Check if model is in preferred list or matches pattern
            provider_config.preferred_models.iter().any(|preferred| {
                preferred == model_id || model_id.starts_with(&preferred.replace(":latest", ""))
            })
        } else {
            false
        }
    }

    /// Select a cloud provider based on context and availability
    fn select_cloud_provider(&self, context: &FallbackContext) -> Option<String> {
        // For now, simple round-robin selection
        // In the future, this could be more sophisticated based on:
        // - Provider capabilities (streaming, tools, etc.)
        // - Model availability
        // - Performance metrics
        // - User preferences

        if self.config.cloud_providers.is_empty() {
            return None;
        }

        // Prefer providers that support the required features
        let suitable_providers: Vec<_> = self
            .config
            .cloud_providers
            .iter()
            .filter(|provider| self.cloud_provider_supports_features(provider, context))
            .collect();

        if !suitable_providers.is_empty() {
            Some(suitable_providers[0].clone())
        } else {
            // Fallback to first available provider
            Some(self.config.cloud_providers[0].clone())
        }
    }

    /// Check if a cloud provider supports the required features
    fn cloud_provider_supports_features(&self, provider: &str, context: &FallbackContext) -> bool {
        match provider {
            "openai" => {
                // OpenAI supports streaming and tools
                true
            }
            "anthropic" => {
                // Anthropic supports streaming and tools
                true
            }
            _ => {
                // Unknown provider, assume basic support
                !context.requires_tools // Only if tools not required
            }
        }
    }

    /// Check if we should return to local provider
    pub fn should_return_to_local(
        &self,
        current_provider: &str,
        local_health: &[(String, ProviderHealthStatus)],
        time_since_fallback: Duration,
    ) -> Option<String> {
        if !self.config.auto_return_to_local {
            return None;
        }

        if !current_provider.starts_with("cloud:") {
            return None;
        }

        if time_since_fallback < self.config.local_recovery_delay() {
            return None;
        }

        // Find a healthy local provider
        local_health
            .iter()
            .find(|(_, status)| matches!(status, ProviderHealthStatus::Healthy { .. }))
            .map(|(name, _)| name.clone())
    }
}

impl FallbackDecision {
    /// Check if this decision uses a local provider
    pub fn is_local(&self) -> bool {
        matches!(self, FallbackDecision::UseLocal { .. })
    }

    /// Check if this decision uses a cloud provider
    pub fn is_cloud(&self) -> bool {
        matches!(self, FallbackDecision::UseCloud { .. })
    }

    /// Check if this decision requires manual intervention
    pub fn requires_manual(&self) -> bool {
        matches!(self, FallbackDecision::RequireManual { .. })
    }

    /// Check if no provider is available
    pub fn no_provider(&self) -> bool {
        matches!(self, FallbackDecision::NoProvider { .. })
    }

    /// Get the provider name if available
    pub fn provider_name(&self) -> Option<&str> {
        match self {
            FallbackDecision::UseLocal { provider_name, .. }
            | FallbackDecision::UseCloud { provider_name, .. } => Some(provider_name),
            _ => None,
        }
    }

    /// Get the reason for this decision
    pub fn reason(&self) -> &str {
        match self {
            FallbackDecision::UseLocal { reason, .. }
            | FallbackDecision::UseCloud { reason, .. }
            | FallbackDecision::RequireManual { reason, .. }
            | FallbackDecision::NoProvider { reason, .. } => reason,
        }
    }
}

impl FallbackContext {
    /// Create a new fallback context
    pub fn new(model_id: String) -> Self {
        Self {
            model_id,
            is_streaming: false,
            requires_tools: false,
            previous_provider: None,
            consecutive_failures: 0,
            time_since_last_success: None,
        }
    }

    /// Set streaming requirement
    pub fn with_streaming(mut self, streaming: bool) -> Self {
        self.is_streaming = streaming;
        self
    }

    /// Set tools requirement
    pub fn with_tools(mut self, tools: bool) -> Self {
        self.requires_tools = tools;
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

    /// Set time since last success
    pub fn with_time_since_last_success(mut self, time: Duration) -> Self {
        self.time_since_last_success = Some(time);
        self
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use pretty_assertions::assert_eq;

    use super::*;
    use crate::config::local_ai::LocalAiConfig;

    fn create_test_local_config() -> LocalAiConfig {
        LocalAiConfig::with_default_ollama()
    }

    fn create_healthy_status() -> ProviderHealthStatus {
        ProviderHealthStatus::Healthy {
            response_time: Duration::from_millis(100),
            models_available: 5,
            additional_info: None,
        }
    }

    fn create_unhealthy_status() -> ProviderHealthStatus {
        ProviderHealthStatus::Unhealthy {
            reason: "Connection refused".to_string(),
            response_time: Duration::from_millis(0),
        }
    }

    #[test]
    fn test_fallback_config_validation_success() {
        let fixture = FallbackConfig::default();
        let actual = fixture.validate();
        assert!(actual.is_ok());
    }

    #[test]
    fn test_fallback_config_validation_zero_timeout() {
        let fixture = FallbackConfig::default().decision_timeout_seconds(0u64);
        let actual = fixture.validate();
        assert!(actual.is_err());
    }

    #[tokio::test]
    async fn test_fallback_engine_local_only_healthy() {
        let config = FallbackConfig::default().strategy(FallbackStrategy::None);
        let local_config = create_test_local_config();
        let engine = FallbackEngine::new(config, local_config);

        let context = FallbackContext::new("llama3.2:latest".to_string());
        let health = vec![("ollama".to_string(), create_healthy_status())];

        let actual = engine.decide_provider(&context, &health).await;
        assert!(actual.is_local());
        assert_eq!(actual.provider_name(), Some("ollama"));
    }

    #[tokio::test]
    async fn test_fallback_engine_local_only_unhealthy() {
        let config = FallbackConfig::default().strategy(FallbackStrategy::None);
        let local_config = create_test_local_config();
        let engine = FallbackEngine::new(config, local_config);

        let context = FallbackContext::new("llama3.2:latest".to_string());
        let health = vec![("ollama".to_string(), create_unhealthy_status())];

        let actual = engine.decide_provider(&context, &health).await;
        assert!(actual.no_provider());
    }

    #[tokio::test]
    async fn test_fallback_engine_immediate_fallback() {
        let config = FallbackConfig::default().strategy(FallbackStrategy::Immediate);
        let local_config = create_test_local_config();
        let engine = FallbackEngine::new(config, local_config);

        let context = FallbackContext::new("gpt-4".to_string());
        let health = vec![("ollama".to_string(), create_unhealthy_status())];

        let actual = engine.decide_provider(&context, &health).await;
        assert!(actual.is_cloud());
        assert_eq!(actual.provider_name(), Some("openai"));
    }

    #[tokio::test]
    async fn test_fallback_engine_graceful_with_retries() {
        let config = FallbackConfig::default()
            .strategy(FallbackStrategy::Graceful)
            .max_retries(3u32);
        let local_config = create_test_local_config();
        let engine = FallbackEngine::new(config, local_config);

        let context =
            FallbackContext::new("llama3.2:latest".to_string()).with_consecutive_failures(1);
        let health = vec![(
            "ollama".to_string(),
            ProviderHealthStatus::Degraded {
                reason: "Slow response".to_string(),
                response_time: Duration::from_millis(5000),
                models_available: 3,
            },
        )];

        let actual = engine.decide_provider(&context, &health).await;
        assert!(actual.is_local());
    }

    #[tokio::test]
    async fn test_fallback_engine_graceful_exhausted_retries() {
        let config = FallbackConfig::default()
            .strategy(FallbackStrategy::Graceful)
            .max_retries(2u32);
        let local_config = create_test_local_config();
        let engine = FallbackEngine::new(config, local_config);

        let context =
            FallbackContext::new("llama3.2:latest".to_string()).with_consecutive_failures(3);
        let health = vec![("ollama".to_string(), create_unhealthy_status())];

        let actual = engine.decide_provider(&context, &health).await;
        assert!(actual.is_cloud());
    }

    #[test]
    fn test_fallback_decision_properties() {
        let local_decision = FallbackDecision::UseLocal {
            provider_name: "ollama".to_string(),
            reason: "Healthy".to_string(),
        };
        assert!(local_decision.is_local());
        assert!(!local_decision.is_cloud());
        assert_eq!(local_decision.provider_name(), Some("ollama"));

        let cloud_decision = FallbackDecision::UseCloud {
            provider_name: "openai".to_string(),
            reason: "Fallback".to_string(),
            local_status: None,
        };
        assert!(cloud_decision.is_cloud());
        assert!(!cloud_decision.is_local());
        assert_eq!(cloud_decision.provider_name(), Some("openai"));
    }

    #[test]
    fn test_fallback_context_builder() {
        let fixture = FallbackContext::new("test-model".to_string())
            .with_streaming(true)
            .with_tools(true)
            .with_previous_provider("ollama".to_string())
            .with_consecutive_failures(2)
            .with_time_since_last_success(Duration::from_secs(30));

        assert_eq!(fixture.model_id, "test-model");
        assert_eq!(fixture.is_streaming, true);
        assert_eq!(fixture.requires_tools, true);
        assert_eq!(fixture.previous_provider, Some("ollama".to_string()));
        assert_eq!(fixture.consecutive_failures, 2);
        assert_eq!(
            fixture.time_since_last_success,
            Some(Duration::from_secs(30))
        );
    }

    #[test]
    fn test_should_return_to_local() {
        let config = FallbackConfig::default()
            .auto_return_to_local(true)
            .local_recovery_delay_seconds(60u64);
        let local_config = create_test_local_config();
        let engine = FallbackEngine::new(config, local_config);

        let health = vec![("ollama".to_string(), create_healthy_status())];

        // Should not return if not enough time passed
        let result =
            engine.should_return_to_local("cloud:openai", &health, Duration::from_secs(30));
        assert_eq!(result, None);

        // Should return if enough time passed and local is healthy
        let result =
            engine.should_return_to_local("cloud:openai", &health, Duration::from_secs(120));
        assert_eq!(result, Some("ollama".to_string()));
    }
}
