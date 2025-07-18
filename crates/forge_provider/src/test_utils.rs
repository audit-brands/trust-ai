//! Test utilities and mock services for local provider testing

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use forge_app::domain::{Model, ModelId};
use tokio::sync::RwLock;

use crate::config::local_ai::{LocalAiConfig, ProviderHealthChecker, ProviderHealthStatus};
use crate::discovery::{DiscoveredModel, DiscoveryStats, ModelDiscoveryResult};
use crate::health::{HealthCheckResult, ProviderHealthInfo};
use crate::selection::{ProviderMetrics, ProviderType};

/// Mock health checker for testing
pub struct MockHealthChecker {
    status: ProviderHealthStatus,
    should_fail: bool,
    response_delay: Duration,
}

impl MockHealthChecker {
    /// Create a mock health checker that returns healthy status
    pub fn healthy() -> Self {
        Self {
            status: ProviderHealthStatus::Healthy {
                response_time: Duration::from_millis(100),
                models_available: 3,
                additional_info: Some("Mock healthy provider".to_string()),
            },
            should_fail: false,
            response_delay: Duration::from_millis(100),
        }
    }

    /// Create a mock health checker that returns degraded status
    pub fn degraded() -> Self {
        Self {
            status: ProviderHealthStatus::Degraded {
                reason: "Mock degraded provider".to_string(),
                response_time: Duration::from_millis(2000),
                models_available: 2,
                additional_info: None,
            },
            should_fail: false,
            response_delay: Duration::from_millis(2000),
        }
    }

    /// Create a mock health checker that returns unhealthy status
    pub fn unhealthy() -> Self {
        Self {
            status: ProviderHealthStatus::Unhealthy {
                reason: "Mock unhealthy provider".to_string(),
                response_time: Duration::from_millis(5000),
            },
            should_fail: false,
            response_delay: Duration::from_millis(5000),
        }
    }

    /// Create a mock health checker that fails
    pub fn failing() -> Self {
        Self {
            status: ProviderHealthStatus::Unhealthy {
                reason: "Mock failing provider".to_string(),
                response_time: Duration::from_millis(0),
            },
            should_fail: true,
            response_delay: Duration::from_millis(100),
        }
    }

    /// Set custom response delay
    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.response_delay = delay;
        self
    }
}

#[async_trait::async_trait]
impl ProviderHealthChecker for MockHealthChecker {
    async fn check_health(&self) -> anyhow::Result<ProviderHealthStatus> {
        // Simulate response delay
        tokio::time::sleep(self.response_delay).await;

        if self.should_fail {
            anyhow::bail!("Mock health check failed");
        }

        Ok(self.status.clone())
    }
}

/// Mock Ollama service for testing
pub struct MockOllamaService {
    available_models: Vec<Model>,
    health_status: ProviderHealthStatus,
    response_delay: Duration,
    should_fail: bool,
}

impl MockOllamaService {
    /// Create a new mock Ollama service
    pub fn new() -> Self {
        Self {
            available_models: vec![
                create_test_model("llama3.2:latest", "Llama 3.2"),
                create_test_model("qwen2.5:latest", "Qwen 2.5"),
                create_test_model("deepseek-r1:latest", "DeepSeek R1"),
            ],
            health_status: ProviderHealthStatus::Healthy {
                response_time: Duration::from_millis(100),
                models_available: 3,
                additional_info: Some("Mock Ollama service".to_string()),
            },
            response_delay: Duration::from_millis(100),
            should_fail: false,
        }
    }

    /// Set the health status
    pub fn with_health_status(mut self, status: ProviderHealthStatus) -> Self {
        self.health_status = status;
        self
    }

    /// Set the available models
    pub fn with_models(mut self, models: Vec<Model>) -> Self {
        self.available_models = models;
        self
    }

    /// Set response delay
    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.response_delay = delay;
        self
    }

    /// Make the service fail
    pub fn failing(mut self) -> Self {
        self.should_fail = true;
        self
    }

    /// Get available models
    pub async fn list_models(&self) -> anyhow::Result<Vec<Model>> {
        tokio::time::sleep(self.response_delay).await;

        if self.should_fail {
            anyhow::bail!("Mock Ollama service unavailable");
        }

        Ok(self.available_models.clone())
    }

    /// Check health
    pub async fn health_check(&self) -> anyhow::Result<ProviderHealthStatus> {
        tokio::time::sleep(self.response_delay).await;

        if self.should_fail {
            anyhow::bail!("Mock Ollama service health check failed");
        }

        Ok(self.health_status.clone())
    }
}

/// Test fixtures for creating common test data
pub struct TestFixtures;

impl TestFixtures {
    /// Create a test local AI configuration
    pub fn local_config() -> LocalAiConfig {
        LocalAiConfig::with_default_ollama()
    }

    /// Create a test local AI configuration with multiple providers
    pub fn multi_provider_config() -> LocalAiConfig {
        let mut config = LocalAiConfig::new();

        // Add primary Ollama provider
        config.add_ollama_provider("ollama-primary", "http://localhost:11434");

        // Add backup Ollama provider
        config.add_ollama_provider("ollama-backup", "http://localhost:11435");

        config
    }

    /// Create test models
    pub fn test_models() -> Vec<Model> {
        vec![
            create_test_model("llama3.2:latest", "Llama 3.2"),
            create_test_model("qwen2.5:latest", "Qwen 2.5"),
            create_test_model("deepseek-r1:latest", "DeepSeek R1"),
            create_test_model("mistral:latest", "Mistral"),
            create_test_model("codellama:latest", "Code Llama"),
        ]
    }

    /// Create test discovered models with various health statuses
    pub fn discovered_models() -> Vec<DiscoveredModel> {
        let models = Self::test_models();
        vec![
            DiscoveredModel {
                model: models[0].clone(),
                provider: "ollama-primary".to_string(),
                provider_health: ProviderHealthStatus::Healthy {
                    response_time: Duration::from_millis(100),
                    models_available: 3,
                    additional_info: None,
                },
                available: true,
                last_checked: Instant::now(),
                response_time: Some(Duration::from_millis(100)),
            },
            DiscoveredModel {
                model: models[1].clone(),
                provider: "ollama-primary".to_string(),
                provider_health: ProviderHealthStatus::Degraded {
                    reason: "High response time".to_string(),
                    response_time: Duration::from_millis(2000),
                    models_available: 2,
                    additional_info: None,
                },
                available: true,
                last_checked: Instant::now(),
                response_time: Some(Duration::from_millis(2000)),
            },
            DiscoveredModel {
                model: models[2].clone(),
                provider: "ollama-backup".to_string(),
                provider_health: ProviderHealthStatus::Unhealthy {
                    reason: "Connection timeout".to_string(),
                    response_time: Duration::from_millis(5000),
                },
                available: false,
                last_checked: Instant::now(),
                response_time: None,
            },
        ]
    }

    /// Create test discovery result
    pub fn discovery_result() -> ModelDiscoveryResult {
        let discovered_models = Self::discovered_models();
        let available_count = discovered_models.iter().filter(|m| m.available).count();
        let providers: std::collections::HashSet<_> =
            discovered_models.iter().map(|m| &m.provider).collect();

        ModelDiscoveryResult {
            discovered_models,
            stats: DiscoveryStats {
                total_models: 3,
                available_models: available_count,
                total_providers: providers.len(),
                last_discovery: Some(Instant::now()),
            },
        }
    }

    /// Create test provider health info
    pub fn provider_health_info(status: ProviderHealthStatus) -> ProviderHealthInfo {
        let success = matches!(status, ProviderHealthStatus::Healthy { .. });

        ProviderHealthInfo {
            status,
            last_checked: Instant::now(),
            consecutive_failures: if success { 0 } else { 1 },
            consecutive_successes: if success { 1 } else { 0 },
            avg_response_time: Duration::from_millis(if success { 100 } else { 2000 }),
            check_history: vec![HealthCheckResult {
                timestamp: Instant::now(),
                success,
                response_time: Duration::from_millis(if success { 100 } else { 2000 }),
                error: if success {
                    None
                } else {
                    Some("Test error".to_string())
                },
            }],
        }
    }

    /// Create test provider metrics
    pub fn provider_metrics(provider_type: ProviderType) -> ProviderMetrics {
        let mut metrics = ProviderMetrics::new(provider_type);
        metrics.total_requests = 10;
        metrics.successful_requests = 8;
        metrics.avg_response_time = Duration::from_millis(150);
        metrics.last_request_time = Some(Instant::now());
        metrics
    }
}

/// Helper function to create test models
pub fn create_test_model(id: &str, name: &str) -> Model {
    Model {
        id: ModelId::new(id),
        name: Some(name.to_string()),
        description: Some(format!("Test model: {name}")),
        context_length: Some(4096),
        tools_supported: Some(true),
        supports_parallel_tool_calls: Some(false),
        supports_reasoning: Some(true),
    }
}

/// Helper function to create healthy provider health status
pub fn create_healthy_status() -> ProviderHealthStatus {
    ProviderHealthStatus::Healthy {
        response_time: Duration::from_millis(100),
        models_available: 3,
        additional_info: Some("All systems operational".to_string()),
    }
}

/// Helper function to create degraded provider health status
pub fn create_degraded_status() -> ProviderHealthStatus {
    ProviderHealthStatus::Degraded {
        reason: "High response time".to_string(),
        response_time: Duration::from_millis(2000),
        models_available: 2,
        additional_info: None,
    }
}

/// Helper function to create unhealthy provider health status
pub fn create_unhealthy_status() -> ProviderHealthStatus {
    ProviderHealthStatus::Unhealthy {
        reason: "Connection timeout".to_string(),
        response_time: Duration::from_millis(5000),
    }
}

/// Mock health monitor for testing
pub struct MockHealthMonitor {
    health_status: Arc<RwLock<HashMap<String, ProviderHealthInfo>>>,
}

impl MockHealthMonitor {
    /// Create a new mock health monitor
    pub fn new() -> Self {
        Self { health_status: Arc::new(RwLock::new(HashMap::new())) }
    }

    /// Add a provider with specific health status
    pub async fn add_provider(&self, name: String, status: ProviderHealthStatus) {
        let info = TestFixtures::provider_health_info(status);
        let mut health_status = self.health_status.write().await;
        health_status.insert(name, info);
    }

    /// Get health status for all providers
    pub async fn get_health_status(&self) -> HashMap<String, ProviderHealthStatus> {
        let health_status = self.health_status.read().await;
        health_status
            .iter()
            .map(|(name, info)| (name.clone(), info.status.clone()))
            .collect()
    }

    /// Get providers sorted by health
    pub async fn get_providers_by_health(&self) -> Vec<(String, ProviderHealthStatus)> {
        let health_status = self.health_status.read().await;
        let mut providers: Vec<_> = health_status
            .iter()
            .map(|(name, info)| (name.clone(), info.status.clone()))
            .collect();

        // Sort by health status priority
        providers.sort_by(|(_, a), (_, b)| {
            let priority_a = match a {
                ProviderHealthStatus::Healthy { .. } => 0,
                ProviderHealthStatus::Degraded { .. } => 1,
                ProviderHealthStatus::Unhealthy { .. } => 2,
            };
            let priority_b = match b {
                ProviderHealthStatus::Healthy { .. } => 0,
                ProviderHealthStatus::Degraded { .. } => 1,
                ProviderHealthStatus::Unhealthy { .. } => 2,
            };
            priority_a.cmp(&priority_b)
        });

        providers
    }

    /// Check if a provider is usable
    pub async fn is_provider_usable(&self, provider_name: &str) -> bool {
        let health_status = self.health_status.read().await;
        if let Some(info) = health_status.get(provider_name) {
            info.status.is_usable()
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_create_test_model() {
        let fixture = create_test_model("llama3.2:latest", "Llama 3.2");

        assert_eq!(fixture.id.as_str(), "llama3.2:latest");
        assert_eq!(fixture.name.unwrap(), "Llama 3.2");
        assert!(fixture.description.unwrap().contains("Test model"));
        assert_eq!(fixture.context_length.unwrap(), 4096);
        assert!(fixture.tools_supported.unwrap());
    }

    #[test]
    fn test_mock_health_checker_healthy() {
        let fixture = MockHealthChecker::healthy();

        assert!(!fixture.should_fail);
        assert_eq!(fixture.response_delay, Duration::from_millis(100));
        assert!(matches!(
            fixture.status,
            ProviderHealthStatus::Healthy { .. }
        ));
    }

    #[test]
    fn test_mock_health_checker_degraded() {
        let fixture = MockHealthChecker::degraded();

        assert!(!fixture.should_fail);
        assert_eq!(fixture.response_delay, Duration::from_millis(2000));
        assert!(matches!(
            fixture.status,
            ProviderHealthStatus::Degraded { .. }
        ));
    }

    #[test]
    fn test_mock_health_checker_unhealthy() {
        let fixture = MockHealthChecker::unhealthy();

        assert!(!fixture.should_fail);
        assert_eq!(fixture.response_delay, Duration::from_millis(5000));
        assert!(matches!(
            fixture.status,
            ProviderHealthStatus::Unhealthy { .. }
        ));
    }

    #[test]
    fn test_mock_health_checker_failing() {
        let fixture = MockHealthChecker::failing();

        assert!(fixture.should_fail);
        assert_eq!(fixture.response_delay, Duration::from_millis(100));
    }

    #[test]
    fn test_mock_ollama_service_creation() {
        let fixture = MockOllamaService::new();

        assert_eq!(fixture.available_models.len(), 3);
        assert!(!fixture.should_fail);
        assert_eq!(fixture.response_delay, Duration::from_millis(100));
        assert!(matches!(
            fixture.health_status,
            ProviderHealthStatus::Healthy { .. }
        ));
    }

    #[test]
    fn test_test_fixtures_local_config() {
        let fixture = TestFixtures::local_config();
        assert!(!fixture.providers.is_empty());
    }

    #[test]
    fn test_test_fixtures_multi_provider_config() {
        let fixture = TestFixtures::multi_provider_config();
        assert!(fixture.providers.len() >= 2);
    }

    #[test]
    fn test_test_fixtures_test_models() {
        let fixture = TestFixtures::test_models();
        assert_eq!(fixture.len(), 5);

        // Verify all models have required fields
        for model in fixture {
            assert!(model.name.is_some());
            assert!(model.description.is_some());
            assert!(model.context_length.is_some());
        }
    }

    #[test]
    fn test_test_fixtures_discovered_models() {
        let fixture = TestFixtures::discovered_models();
        assert_eq!(fixture.len(), 3);

        // Verify different health statuses
        let health_statuses: Vec<_> = fixture.iter().map(|m| &m.provider_health).collect();
        assert!(health_statuses
            .iter()
            .any(|s| matches!(s, ProviderHealthStatus::Healthy { .. })));
        assert!(health_statuses
            .iter()
            .any(|s| matches!(s, ProviderHealthStatus::Degraded { .. })));
        assert!(health_statuses
            .iter()
            .any(|s| matches!(s, ProviderHealthStatus::Unhealthy { .. })));
    }

    #[test]
    fn test_test_fixtures_discovery_result() {
        let fixture = TestFixtures::discovery_result();

        assert_eq!(fixture.stats.total_models, 3);
        assert!(fixture.stats.available_models <= fixture.stats.total_models);
        assert!(fixture.stats.total_providers > 0);
        assert!(fixture.stats.last_discovery.is_some());
    }

    #[test]
    fn test_test_fixtures_provider_health_info() {
        let healthy_status = create_healthy_status();
        let fixture = TestFixtures::provider_health_info(healthy_status.clone());

        assert!(matches!(
            fixture.status,
            ProviderHealthStatus::Healthy { .. }
        ));
        assert_eq!(fixture.consecutive_successes, 1);
        assert_eq!(fixture.consecutive_failures, 0);
        assert_eq!(fixture.check_history.len(), 1);
        assert!(fixture.check_history[0].success);
    }

    #[test]
    fn test_test_fixtures_provider_metrics() {
        let fixture = TestFixtures::provider_metrics(ProviderType::Local);

        assert_eq!(fixture.provider_type, ProviderType::Local);
        assert_eq!(fixture.total_requests, 10);
        assert_eq!(fixture.successful_requests, 8);
        assert_eq!(fixture.success_rate(), 0.8);
        assert!(fixture.last_request_time.is_some());
    }

    #[tokio::test]
    async fn test_mock_health_monitor() {
        let fixture = MockHealthMonitor::new();

        // Add providers with different health statuses
        fixture
            .add_provider("healthy".to_string(), create_healthy_status())
            .await;
        fixture
            .add_provider("degraded".to_string(), create_degraded_status())
            .await;
        fixture
            .add_provider("unhealthy".to_string(), create_unhealthy_status())
            .await;

        let health_status = fixture.get_health_status().await;
        assert_eq!(health_status.len(), 3);

        // Test provider usability
        assert!(fixture.is_provider_usable("healthy").await);
        assert!(fixture.is_provider_usable("degraded").await); // Degraded is still usable
        assert!(!fixture.is_provider_usable("unhealthy").await);

        // Test provider sorting by health
        let sorted_providers = fixture.get_providers_by_health().await;
        assert_eq!(sorted_providers.len(), 3);

        // First should be healthy, last should be unhealthy
        assert!(matches!(
            sorted_providers[0].1,
            ProviderHealthStatus::Healthy { .. }
        ));
        assert!(matches!(
            sorted_providers[2].1,
            ProviderHealthStatus::Unhealthy { .. }
        ));
    }
}
