use std::collections::HashMap;
use std::time::Duration;

use anyhow::Context as _;

use derive_setters::Setters;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use crate::ollama::{OllamaConfig, OllamaHealthCheck, HealthStatus};

/// Configuration for local AI providers
#[derive(Debug, Clone, Serialize, Deserialize, Setters)]
#[setters(strip_option, into)]
pub struct LocalAiConfig {
    /// Whether local AI is enabled
    pub enabled: bool,
    /// Configuration for individual providers
    pub providers: HashMap<String, LocalProviderConfig>,
    /// Global settings for local AI
    pub settings: LocalAiSettings,
}

/// Configuration for a specific local provider
#[derive(Debug, Clone, Serialize, Deserialize, Setters)]
#[setters(strip_option, into)]
pub struct LocalProviderConfig {
    /// Whether this provider is enabled
    pub enabled: bool,
    /// Provider type (ollama, etc.)
    pub provider_type: String,
    /// Endpoint URL for the provider
    pub endpoint: String,
    /// Preferred models for this provider
    pub preferred_models: Vec<String>,
    /// Provider-specific configuration
    pub config: ProviderSpecificConfig,
    /// Health check settings
    pub health_check: HealthCheckConfig,
}

/// Provider-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ProviderSpecificConfig {
    #[serde(rename = "ollama")]
    Ollama {
        timeout_seconds: u64,
        max_retries: u32,
        retry_delay_ms: u64,
        connection_pooling: bool,
        user_agent: Option<String>,
    },
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize, Setters)]
#[setters(strip_option, into)]
pub struct HealthCheckConfig {
    /// Health check interval in seconds
    pub interval_seconds: u64,
    /// Timeout for health checks in seconds
    pub timeout_seconds: u64,
    /// Number of consecutive failures before marking unhealthy
    pub failure_threshold: u32,
    /// Number of consecutive successes before marking healthy
    pub success_threshold: u32,
}

/// Global settings for local AI
#[derive(Debug, Clone, Serialize, Deserialize, Setters)]
#[setters(strip_option, into)]
pub struct LocalAiSettings {
    /// Discovery settings
    pub discovery: DiscoveryConfig,
    /// Performance monitoring settings
    pub monitoring: MonitoringConfig,
}

/// Service discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize, Setters)]
#[setters(strip_option, into)]
pub struct DiscoveryConfig {
    /// Whether to enable automatic service discovery
    pub enabled: bool,
    /// Ports to scan for services
    pub scan_ports: Vec<u16>,
    /// Hosts to scan
    pub scan_hosts: Vec<String>,
    /// Discovery interval in seconds
    pub interval_seconds: u64,
}

/// Performance monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize, Setters)]
#[setters(strip_option, into)]
pub struct MonitoringConfig {
    /// Whether to enable performance monitoring
    pub enabled: bool,
    /// Metrics collection interval in seconds
    pub interval_seconds: u64,
    /// Maximum response time threshold in milliseconds
    pub max_response_time_ms: u64,
}

impl Default for LocalAiConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            providers: HashMap::new(),
            settings: LocalAiSettings::default(),
        }
    }
}

impl Default for LocalProviderConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            provider_type: "ollama".to_string(),
            endpoint: "http://localhost:11434".to_string(),
            preferred_models: vec![
                "llama3.2:latest".to_string(),
                "codellama:latest".to_string(),
            ],
            config: ProviderSpecificConfig::Ollama {
                timeout_seconds: 30,
                max_retries: 3,
                retry_delay_ms: 1000,
                connection_pooling: true,
                user_agent: Some("forge-ai/1.0".to_string()),
            },
            health_check: HealthCheckConfig::default(),
        }
    }
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            interval_seconds: 30,
            timeout_seconds: 5,
            failure_threshold: 3,
            success_threshold: 2,
        }
    }
}

impl Default for LocalAiSettings {
    fn default() -> Self {
        Self {
            discovery: DiscoveryConfig::default(),
            monitoring: MonitoringConfig::default(),
        }
    }
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            scan_ports: vec![11434, 11435, 11436],
            scan_hosts: vec!["localhost".to_string(), "127.0.0.1".to_string()],
            interval_seconds: 300, // 5 minutes
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_seconds: 60,
            max_response_time_ms: 5000,
        }
    }
}

impl LocalAiConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a provider configuration
    pub fn add_provider(mut self, name: String, config: LocalProviderConfig) -> Self {
        self.providers.insert(name, config);
        self
    }

    /// Get enabled providers
    pub fn enabled_providers(&self) -> impl Iterator<Item = (&String, &LocalProviderConfig)> {
        self.providers.iter().filter(|(_, config)| config.enabled)
    }

    /// Validate the configuration
    pub fn validate(&self) -> anyhow::Result<()> {
        if !self.enabled {
            debug!("Local AI is disabled");
            return Ok(());
        }

        if self.providers.is_empty() {
            warn!("Local AI is enabled but no providers are configured");
        }

        for (name, provider) in &self.providers {
            provider.validate()
                .with_context(|| format!("Invalid configuration for provider '{}'", name))?;
        }

        Ok(())
    }

    /// Create a default configuration with Ollama
    pub fn with_default_ollama() -> Self {
        let mut config = Self::new();
        config.providers.insert(
            "ollama".to_string(),
            LocalProviderConfig::default(),
        );
        config
    }
}

impl LocalProviderConfig {
    /// Validate the provider configuration
    pub fn validate(&self) -> anyhow::Result<()> {
        if !self.enabled {
            debug!("Provider {} is disabled", self.provider_type);
            return Ok(());
        }

        // Validate endpoint URL
        reqwest::Url::parse(&self.endpoint)
            .with_context(|| format!("Invalid endpoint URL: {}", self.endpoint))?;

        // Validate health check configuration
        self.health_check.validate()?;

        // Validate provider-specific configuration
        match &self.config {
            ProviderSpecificConfig::Ollama { timeout_seconds, max_retries, .. } => {
                if *timeout_seconds == 0 {
                    anyhow::bail!("Timeout cannot be zero");
                }
                if *timeout_seconds > 300 {
                    warn!("Timeout of {} seconds is very high", timeout_seconds);
                }
                if *max_retries > 10 {
                    warn!("Max retries of {} is very high", max_retries);
                }
            }
        }

        debug!("Provider configuration validated successfully: {}", self.provider_type);
        Ok(())
    }

    /// Convert to OllamaConfig if this is an Ollama provider
    pub fn to_ollama_config(&self) -> anyhow::Result<OllamaConfig> {
        match &self.config {
            ProviderSpecificConfig::Ollama {
                timeout_seconds,
                max_retries,
                retry_delay_ms,
                connection_pooling,
                user_agent,
            } => {
                let mut config = OllamaConfig::new()
                    .with_base_url(self.endpoint.clone())
                    .with_timeout(*timeout_seconds)
                    .with_max_retries(*max_retries)
                    .with_retry_delay(*retry_delay_ms)
                    .with_connection_pooling(*connection_pooling);

                if let Some(ref ua) = user_agent {
                    config = config.with_user_agent(ua.clone());
                }

                Ok(config)
            }
        }
    }

    /// Create a health checker for this provider
    pub fn create_health_checker(&self) -> anyhow::Result<Box<dyn ProviderHealthChecker>> {
        match &self.config {
            ProviderSpecificConfig::Ollama { .. } => {
                let ollama_config = self.to_ollama_config()?;
                Ok(Box::new(OllamaProviderHealthChecker::new(ollama_config)))
            }
        }
    }
}

impl HealthCheckConfig {
    /// Validate the health check configuration
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.interval_seconds == 0 {
            anyhow::bail!("Health check interval cannot be zero");
        }
        if self.timeout_seconds == 0 {
            anyhow::bail!("Health check timeout cannot be zero");
        }
        if self.timeout_seconds >= self.interval_seconds {
            warn!("Health check timeout ({}) is >= interval ({})", 
                  self.timeout_seconds, self.interval_seconds);
        }
        if self.failure_threshold == 0 {
            anyhow::bail!("Failure threshold cannot be zero");
        }
        if self.success_threshold == 0 {
            anyhow::bail!("Success threshold cannot be zero");
        }
        Ok(())
    }

    /// Get the health check timeout as Duration
    pub fn timeout_duration(&self) -> Duration {
        Duration::from_secs(self.timeout_seconds)
    }

    /// Get the health check interval as Duration
    pub fn interval_duration(&self) -> Duration {
        Duration::from_secs(self.interval_seconds)
    }
}

/// Trait for provider-specific health checking
#[async_trait::async_trait]
pub trait ProviderHealthChecker: Send + Sync {
    /// Check the health of the provider
    async fn check_health(&self) -> anyhow::Result<ProviderHealthStatus>;
    
    /// Get the provider type
    fn provider_type(&self) -> &str;
}

/// Health status of a provider
#[derive(Debug, Clone)]
pub enum ProviderHealthStatus {
    /// Provider is healthy and responsive
    Healthy {
        response_time: Duration,
        models_available: usize,
        additional_info: Option<String>,
    },
    /// Provider is responding but with issues
    Degraded {
        reason: String,
        response_time: Duration,
        models_available: usize,
    },
    /// Provider is not responding or has errors
    Unhealthy {
        reason: String,
        response_time: Duration,
    },
}

impl ProviderHealthStatus {
    /// Check if the provider is usable
    pub fn is_usable(&self) -> bool {
        matches!(self, ProviderHealthStatus::Healthy { .. } | ProviderHealthStatus::Degraded { .. })
    }

    /// Get response time
    pub fn response_time(&self) -> Duration {
        match self {
            ProviderHealthStatus::Healthy { response_time, .. }
            | ProviderHealthStatus::Degraded { response_time, .. }
            | ProviderHealthStatus::Unhealthy { response_time, .. } => *response_time,
        }
    }

    /// Get number of available models
    pub fn models_available(&self) -> usize {
        match self {
            ProviderHealthStatus::Healthy { models_available, .. }
            | ProviderHealthStatus::Degraded { models_available, .. } => *models_available,
            ProviderHealthStatus::Unhealthy { .. } => 0,
        }
    }
}

/// Ollama-specific health checker implementation
pub struct OllamaProviderHealthChecker {
    health_check: OllamaHealthCheck,
}

impl OllamaProviderHealthChecker {
    pub fn new(config: OllamaConfig) -> Self {
        Self {
            health_check: OllamaHealthCheck::new(config),
        }
    }
}

#[async_trait::async_trait]
impl ProviderHealthChecker for OllamaProviderHealthChecker {
    async fn check_health(&self) -> anyhow::Result<ProviderHealthStatus> {
        let status = self.health_check.check_health().await?;
        
        let provider_status = match status {
            HealthStatus::Healthy { response_time, models_available } => {
                ProviderHealthStatus::Healthy {
                    response_time,
                    models_available,
                    additional_info: None,
                }
            }
            HealthStatus::Degraded { reason, response_time } => {
                ProviderHealthStatus::Degraded {
                    reason,
                    response_time,
                    models_available: 0, // Unknown in degraded state
                }
            }
            HealthStatus::Unhealthy { reason, response_time } => {
                ProviderHealthStatus::Unhealthy {
                    reason,
                    response_time,
                }
            }
        };

        Ok(provider_status)
    }

    fn provider_type(&self) -> &str {
        "ollama"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_default_local_ai_config() {
        let fixture = LocalAiConfig::default();
        assert_eq!(fixture.enabled, true);
        assert_eq!(fixture.providers.len(), 0);
    }

    #[test]
    fn test_local_ai_config_with_default_ollama() {
        let fixture = LocalAiConfig::with_default_ollama();
        assert_eq!(fixture.enabled, true);
        assert_eq!(fixture.providers.len(), 1);
        assert!(fixture.providers.contains_key("ollama"));
    }

    #[test]
    fn test_local_provider_config_validation_success() {
        let fixture = LocalProviderConfig::default();
        let actual = fixture.validate();
        assert!(actual.is_ok());
    }

    #[test]
    fn test_local_provider_config_validation_invalid_url() {
        let fixture = LocalProviderConfig::default()
            .endpoint("invalid-url".to_string());
        let actual = fixture.validate();
        assert!(actual.is_err());
    }

    #[test]
    fn test_health_check_config_validation_success() {
        let fixture = HealthCheckConfig::default();
        let actual = fixture.validate();
        assert!(actual.is_ok());
    }

    #[test]
    fn test_health_check_config_validation_zero_interval() {
        let fixture = HealthCheckConfig::default()
            .interval_seconds(0u64);
        let actual = fixture.validate();
        assert!(actual.is_err());
    }

    #[test]
    fn test_health_check_config_validation_zero_timeout() {
        let fixture = HealthCheckConfig::default()
            .timeout_seconds(0u64);
        let actual = fixture.validate();
        assert!(actual.is_err());
    }

    #[test]
    fn test_provider_health_status_usability() {
        let healthy = ProviderHealthStatus::Healthy {
            response_time: Duration::from_millis(100),
            models_available: 5,
            additional_info: None,
        };
        assert!(healthy.is_usable());

        let degraded = ProviderHealthStatus::Degraded {
            reason: "Slow response".to_string(),
            response_time: Duration::from_millis(5000),
            models_available: 3,
        };
        assert!(degraded.is_usable());

        let unhealthy = ProviderHealthStatus::Unhealthy {
            reason: "Connection refused".to_string(),
            response_time: Duration::from_millis(0),
        };
        assert!(!unhealthy.is_usable());
    }

    #[test]
    fn test_ollama_config_conversion() {
        let fixture = LocalProviderConfig::default();
        let actual = fixture.to_ollama_config();
        assert!(actual.is_ok());
        
        let ollama_config = actual.unwrap();
        assert_eq!(ollama_config.base_url, "http://localhost:11434");
        assert_eq!(ollama_config.timeout_seconds, 30);
    }

    #[test]
    fn test_enabled_providers_filter() {
        let mut fixture = LocalAiConfig::new();
        fixture.providers.insert(
            "enabled".to_string(),
            LocalProviderConfig::default().enabled(true),
        );
        fixture.providers.insert(
            "disabled".to_string(),
            LocalProviderConfig::default().enabled(false),
        );

        let enabled: Vec<_> = fixture.enabled_providers().collect();
        assert_eq!(enabled.len(), 1);
        assert_eq!(enabled[0].0, "enabled");
    }
}