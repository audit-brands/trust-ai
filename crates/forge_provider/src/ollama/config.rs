use std::time::Duration;

use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use super::error::OllamaError;
use super::Ollama;

/// Configuration for Ollama provider with validation and defaults
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    /// Base URL for Ollama service
    pub base_url: String,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Retry delay in milliseconds
    pub retry_delay_ms: u64,
    /// Enable connection pooling
    pub connection_pooling: bool,
    /// User agent string
    pub user_agent: Option<String>,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:11434".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
            retry_delay_ms: 1000,
            connection_pooling: true,
            user_agent: Some("forge-ai/1.0".to_string()),
        }
    }
}

impl OllamaConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the base URL for Ollama service
    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }

    /// Set the request timeout
    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = timeout_seconds;
        self
    }

    /// Set the maximum retry attempts
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Set the retry delay
    pub fn with_retry_delay(mut self, retry_delay_ms: u64) -> Self {
        self.retry_delay_ms = retry_delay_ms;
        self
    }

    /// Enable or disable connection pooling
    pub fn with_connection_pooling(mut self, enabled: bool) -> Self {
        self.connection_pooling = enabled;
        self
    }

    /// Set custom user agent
    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), OllamaError> {
        // Validate base URL
        let url = Url::parse(&self.base_url)
            .map_err(|_| OllamaError::InvalidBaseUrl { url: self.base_url.clone() })?;

        // Check for reasonable values
        if self.timeout_seconds == 0 {
            return Err(OllamaError::InvalidConfiguration {
                message: "Timeout cannot be zero".to_string(),
            });
        }

        if self.timeout_seconds > 300 {
            warn!("Timeout of {} seconds is very high", self.timeout_seconds);
        }

        if self.max_retries > 10 {
            warn!("Max retries of {} is very high", self.max_retries);
        }

        // Validate URL scheme
        if !["http", "https"].contains(&url.scheme()) {
            return Err(OllamaError::InvalidBaseUrl { url: self.base_url.clone() });
        }

        debug!("Ollama configuration validated successfully");
        Ok(())
    }

    /// Create an HTTP client based on this configuration
    pub fn create_client(&self) -> Result<Client, OllamaError> {
        let mut builder = Client::builder()
            .timeout(Duration::from_secs(self.timeout_seconds))
            .connect_timeout(Duration::from_secs(5)) // Add connection timeout
            .pool_idle_timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(if self.connection_pooling { 10 } else { 0 });

        if let Some(ref user_agent) = self.user_agent {
            builder = builder.user_agent(user_agent);
        }

        builder
            .build()
            .map_err(|e| OllamaError::InvalidConfiguration {
                message: format!("Failed to create HTTP client: {e}"),
            })
    }

    /// Create an Ollama provider instance from this configuration
    pub fn create_provider(&self) -> Result<Ollama, OllamaError> {
        self.validate()?;

        let client = self.create_client()?;
        let base_url = Url::parse(&self.base_url)
            .map_err(|_| OllamaError::InvalidBaseUrl { url: self.base_url.clone() })?;

        Ok(Ollama::builder()
            .client(client)
            .base_url(base_url)
            .build()
            .unwrap())
    }
}

/// Health check and service discovery utilities
pub struct OllamaHealthCheck {
    config: OllamaConfig,
}

impl OllamaHealthCheck {
    /// Create a new health check instance
    pub fn new(config: OllamaConfig) -> Self {
        Self { config }
    }

    /// Check if Ollama service is available and healthy
    pub async fn check_health(&self) -> Result<HealthStatus, OllamaError> {
        let client = self.config.create_client()?;
        let base_url = Url::parse(&self.config.base_url)
            .map_err(|_| OllamaError::InvalidBaseUrl { url: self.config.base_url.clone() })?;

        info!("Checking Ollama service health at {}", base_url);

        // Try to fetch models as a health check
        let models_url = base_url
            .join("api/tags")
            .map_err(|_| OllamaError::InvalidBaseUrl { url: self.config.base_url.clone() })?;

        let start = std::time::Instant::now();
        let response = client.get(models_url).send().await?;
        let duration = start.elapsed();

        let status = if response.status().is_success() {
            let body = response.text().await.unwrap_or_default();
            match serde_json::from_str::<serde_json::Value>(&body) {
                Ok(json) => {
                    if let Some(models) = json.get("models").and_then(|m| m.as_array()) {
                        HealthStatus::Healthy {
                            response_time: duration,
                            models_available: models.len(),
                        }
                    } else {
                        HealthStatus::Degraded {
                            reason: "Invalid models response format".to_string(),
                            response_time: duration,
                        }
                    }
                }
                Err(_) => HealthStatus::Degraded {
                    reason: "Failed to parse models response".to_string(),
                    response_time: duration,
                },
            }
        } else {
            HealthStatus::Unhealthy {
                reason: format!(
                    "HTTP {}: {}",
                    response.status(),
                    response.status().canonical_reason().unwrap_or("Unknown")
                ),
                response_time: duration,
            }
        };

        info!("Ollama health check completed: {:?}", status);
        Ok(status)
    }

    /// Discover available Ollama services on common ports
    pub async fn discover_services(&self) -> Vec<String> {
        let ports = vec![11434, 11435, 11436]; // Common Ollama ports
        let hosts = vec!["localhost", "127.0.0.1"];
        let mut discovered = Vec::new();

        for host in hosts {
            for port in &ports {
                let url = format!("http://{host}:{port}");
                let config = OllamaConfig::new()
                    .with_base_url(url.clone())
                    .with_timeout(3);
                let health_check = OllamaHealthCheck::new(config);

                if let Ok(HealthStatus::Healthy { .. }) = health_check.check_health().await {
                    discovered.push(url);
                }
            }
        }

        discovered
    }
}

/// Health status of Ollama service
#[derive(Debug, Clone)]
pub enum HealthStatus {
    /// Service is healthy and responsive
    Healthy {
        response_time: Duration,
        models_available: usize,
    },
    /// Service is responding but with issues
    Degraded {
        reason: String,
        response_time: Duration,
    },
    /// Service is not responding or has errors
    Unhealthy {
        reason: String,
        response_time: Duration,
    },
}

impl HealthStatus {
    /// Check if the service is usable
    pub fn is_usable(&self) -> bool {
        matches!(
            self,
            HealthStatus::Healthy { .. } | HealthStatus::Degraded { .. }
        )
    }

    /// Get response time
    pub fn response_time(&self) -> Duration {
        match self {
            HealthStatus::Healthy { response_time, .. }
            | HealthStatus::Degraded { response_time, .. }
            | HealthStatus::Unhealthy { response_time, .. } => *response_time,
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_default_config() {
        let fixture = OllamaConfig::default();
        let actual = fixture.base_url;
        let expected = "http://localhost:11434";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_config_validation_success() {
        let fixture = OllamaConfig::new().with_base_url("http://localhost:11434".to_string());
        let actual = fixture.validate();
        assert!(actual.is_ok());
    }

    #[test]
    fn test_config_validation_invalid_url() {
        let fixture = OllamaConfig::new().with_base_url("invalid-url".to_string());
        let actual = fixture.validate();
        assert!(actual.is_err());
    }

    #[test]
    fn test_config_validation_zero_timeout() {
        let fixture = OllamaConfig::new().with_timeout(0);
        let actual = fixture.validate();
        assert!(actual.is_err());
    }

    #[test]
    fn test_config_builder_pattern() {
        let fixture = OllamaConfig::new()
            .with_base_url("http://example.com:8080".to_string())
            .with_timeout(60)
            .with_max_retries(5)
            .with_retry_delay(2000)
            .with_connection_pooling(false)
            .with_user_agent("test-agent".to_string());

        assert_eq!(fixture.base_url, "http://example.com:8080");
        assert_eq!(fixture.timeout_seconds, 60);
        assert_eq!(fixture.max_retries, 5);
        assert_eq!(fixture.retry_delay_ms, 2000);
        assert_eq!(fixture.connection_pooling, false);
        assert_eq!(fixture.user_agent, Some("test-agent".to_string()));
    }

    #[test]
    fn test_health_status_usability() {
        let healthy = HealthStatus::Healthy {
            response_time: Duration::from_millis(100),
            models_available: 5,
        };
        assert!(healthy.is_usable());

        let degraded = HealthStatus::Degraded {
            reason: "Slow response".to_string(),
            response_time: Duration::from_millis(5000),
        };
        assert!(degraded.is_usable());

        let unhealthy = HealthStatus::Unhealthy {
            reason: "Connection refused".to_string(),
            response_time: Duration::from_millis(0),
        };
        assert!(!unhealthy.is_usable());
    }

    #[tokio::test]
    async fn test_health_check_creation() {
        let config = OllamaConfig::default();
        let fixture = OllamaHealthCheck::new(config);
        // Just test that it can be created
        assert!(true);
    }
}
