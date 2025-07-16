//! Model discovery service for automatic detection and health monitoring
//! 
//! This module provides enhanced model discovery capabilities that go beyond
//! simple model listing to include automatic detection, health monitoring,
//! and availability reporting for local AI services.

use std::collections::HashMap;
use std::time::Duration;

use anyhow::{Context, Result};
use forge_app::domain::{Model, ModelId};
use tracing::{debug, info, warn};

use crate::config::local_ai::{LocalAiConfig, LocalProviderConfig, ProviderHealthStatus, ProviderSpecificConfig};
use crate::health::HealthMonitor;
use crate::ollama::{OllamaConfig, OllamaHealthCheck};

/// Enhanced model discovery service with automatic detection and health monitoring
pub struct ModelDiscoveryService {
    /// Health monitor for tracking provider status
    health_monitor: HealthMonitor,
    /// Local AI configuration
    local_config: LocalAiConfig,
    /// Cached discovered models with their health status
    discovered_models: HashMap<String, DiscoveredModel>,
}

/// Information about a discovered model including its health and availability
#[derive(Debug, Clone)]
pub struct DiscoveredModel {
    /// The model information
    pub model: Model,
    /// Which provider this model is available from
    pub provider: String,
    /// Current health status of the provider serving this model
    pub provider_health: ProviderHealthStatus,
    /// Whether the model is currently available for use
    pub available: bool,
    /// Last time this model was checked
    pub last_checked: std::time::Instant,
    /// Response time for the last health check
    pub response_time: Option<Duration>,
}

/// Result of model discovery operation
#[derive(Debug)]
pub struct ModelDiscoveryResult {
    /// Total number of models discovered
    pub total_models: usize,
    /// Number of healthy providers
    pub healthy_providers: usize,
    /// Number of available models
    pub available_models: usize,
    /// Discovery duration
    pub discovery_duration: Duration,
    /// Any warnings or issues encountered
    pub warnings: Vec<String>,
}

impl ModelDiscoveryService {
    /// Create a new model discovery service
    pub async fn new(local_config: LocalAiConfig) -> Result<Self> {
        let health_monitor = HealthMonitor::new(local_config.clone()).await?;
        
        Ok(Self {
            health_monitor,
            local_config,
            discovered_models: HashMap::new(),
        })
    }

    /// Start the discovery service with automatic monitoring
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting model discovery service");
        
        // Start health monitoring
        self.health_monitor.start().await?;
        
        // Perform initial discovery
        self.discover_all_models().await?;
        
        info!("Model discovery service started successfully");
        Ok(())
    }

    /// Discover all available models from all configured providers
    pub async fn discover_all_models(&mut self) -> Result<ModelDiscoveryResult> {
        let start_time = std::time::Instant::now();
        let mut warnings = Vec::new();
        
        info!("Starting comprehensive model discovery");
        
        // Clear previous discoveries
        self.discovered_models.clear();
        
        // Discover from each provider
        let providers = self.local_config.providers.clone();
        for (provider_name, provider_config) in providers {
            match self.discover_provider_models(&provider_name, &provider_config).await {
                Ok(count) => {
                    info!("Discovered {} models from provider '{}'", count, provider_name);
                }
                Err(e) => {
                    let warning = format!("Failed to discover models from '{}': {}", provider_name, e);
                    warn!("{}", warning);
                    warnings.push(warning);
                }
            }
        }
        
        // Automatic Ollama discovery if not explicitly configured
        if !self.local_config.providers.contains_key("ollama") {
            match self.discover_ollama_automatically().await {
                Ok(count) => {
                    if count > 0 {
                        info!("Automatically discovered {} Ollama models", count);
                    }
                }
                Err(e) => {
                    let warning = format!("Automatic Ollama discovery failed: {}", e);
                    debug!("{}", warning);
                    warnings.push(warning);
                }
            }
        }
        
        let discovery_duration = start_time.elapsed();
        
        // Get health status
        let health_status = self.health_monitor.get_health_status().await;
        let healthy_providers = health_status
            .values()
            .filter(|status| matches!(status, ProviderHealthStatus::Healthy { .. }))
            .count();
        
        let available_models = self.discovered_models
            .values()
            .filter(|model| model.available)
            .count();
        
        let result = ModelDiscoveryResult {
            total_models: self.discovered_models.len(),
            healthy_providers,
            available_models,
            discovery_duration,
            warnings,
        };
        
        info!(
            "Model discovery completed: {} models from {} healthy providers ({} available) in {:?}",
            result.total_models,
            result.healthy_providers, 
            result.available_models,
            result.discovery_duration
        );
        
        Ok(result)
    }

    /// Discover models from a specific provider
    async fn discover_provider_models(
        &mut self,
        provider_name: &str,
        provider_config: &LocalProviderConfig,
    ) -> Result<usize> {
        debug!("Discovering models from provider: {}", provider_name);
        
        // Check provider health first
        let provider_health = self.health_monitor
            .get_provider_health(provider_name)
            .await
            .unwrap_or(ProviderHealthStatus::Unhealthy {
                reason: "Provider not monitored".to_string(),
                response_time: Duration::from_secs(0),
            });
        
        // Only discover from healthy or degraded providers
        if !matches!(provider_health, ProviderHealthStatus::Healthy { .. } | ProviderHealthStatus::Degraded { .. }) {
            return Ok(0);
        }
        
        match &provider_config.config {
            ProviderSpecificConfig::Ollama { .. } => {
                let ollama_config = provider_config.to_ollama_config()?;
                self.discover_ollama_models(provider_name, &ollama_config, provider_health).await
            }
        }
    }

    /// Discover models from Ollama provider
    async fn discover_ollama_models(
        &mut self,
        provider_name: &str,
        config: &OllamaConfig,
        provider_health: ProviderHealthStatus,
    ) -> Result<usize> {
        let ollama = config.create_provider()
            .with_context(|| format!("Failed to create Ollama provider for '{}'", provider_name))?;
        
        let models = ollama.models().await
            .with_context(|| format!("Failed to fetch models from Ollama provider '{}'", provider_name))?;
        
        let now = std::time::Instant::now();
        let response_time = Some(provider_health.response_time());
        
        let available = matches!(provider_health, ProviderHealthStatus::Healthy { .. });
        
        for model in &models {
            let discovered_model = DiscoveredModel {
                model: model.clone(),
                provider: provider_name.to_string(),
                provider_health: provider_health.clone(),
                available,
                last_checked: now,
                response_time,
            };
            
            // Use model ID as key to avoid duplicates
            self.discovered_models.insert(model.id.as_str().to_string(), discovered_model);
        }
        
        Ok(models.len())
    }

    /// Automatically discover Ollama installations on common ports
    async fn discover_ollama_automatically(&mut self) -> Result<usize> {
        debug!("Attempting automatic Ollama discovery");
        
        let default_config = OllamaConfig::default();
        let health_check = OllamaHealthCheck::new(default_config.clone());
        
        // Try the default configuration first
        match health_check.check_health().await {
            Ok(health_status) if health_status.is_usable() => {
                info!("Found Ollama service at default location: {}", default_config.base_url);
                
                let provider_health = match health_status {
                    crate::ollama::HealthStatus::Healthy { response_time, models_available } => {
                        ProviderHealthStatus::Healthy { 
                            response_time, 
                            models_available,
                            additional_info: None,
                        }
                    }
                    crate::ollama::HealthStatus::Degraded { reason, response_time } => {
                        ProviderHealthStatus::Degraded { 
                            reason, 
                            response_time,
                            models_available: 0, // Unknown in degraded state
                        }
                    }
                    crate::ollama::HealthStatus::Unhealthy { reason, response_time } => {
                        ProviderHealthStatus::Unhealthy { 
                            reason, 
                            response_time,
                        }
                    }
                };
                
                return self.discover_ollama_models("ollama-auto", &default_config, provider_health).await;
            }
            _ => {
                debug!("Default Ollama location not available, trying discovery");
            }
        }
        
        // If default doesn't work, try discovery
        let discovered_services = health_check.discover_services().await;
        
        for service_url in discovered_services {
            let config = OllamaConfig::new().with_base_url(service_url.clone());
            let health_check = OllamaHealthCheck::new(config.clone());
            
            if let Ok(health_status) = health_check.check_health().await {
                if health_status.is_usable() {
                    info!("Auto-discovered Ollama service at: {}", service_url);
                    
                    let provider_health = match health_status {
                        crate::ollama::HealthStatus::Healthy { response_time, models_available } => {
                            ProviderHealthStatus::Healthy { 
                                response_time, 
                                models_available,
                                additional_info: None,
                            }
                        }
                        crate::ollama::HealthStatus::Degraded { reason, response_time } => {
                            ProviderHealthStatus::Degraded { 
                                reason, 
                                response_time,
                                models_available: 0,
                            }
                        }
                        crate::ollama::HealthStatus::Unhealthy { reason, response_time } => {
                            ProviderHealthStatus::Unhealthy { 
                                reason, 
                                response_time,
                            }
                        }
                    };
                    
                    return self.discover_ollama_models("ollama-discovered", &config, provider_health).await;
                }
            }
        }
        
        Ok(0)
    }

    /// Get all discovered models
    pub fn get_discovered_models(&self) -> Vec<&DiscoveredModel> {
        self.discovered_models.values().collect()
    }

    /// Get available models only
    pub fn get_available_models(&self) -> Vec<&DiscoveredModel> {
        self.discovered_models
            .values()
            .filter(|model| model.available)
            .collect()
    }

    /// Get models from a specific provider
    pub fn get_provider_models(&self, provider_name: &str) -> Vec<&DiscoveredModel> {
        self.discovered_models
            .values()
            .filter(|model| model.provider == provider_name)
            .collect()
    }

    /// Check if a specific model is available
    pub fn is_model_available(&self, model_id: &ModelId) -> bool {
        self.discovered_models
            .get(model_id.as_str())
            .map(|model| model.available)
            .unwrap_or(false)
    }

    /// Get health status for all providers
    pub async fn get_provider_health_status(&self) -> HashMap<String, ProviderHealthStatus> {
        self.health_monitor.get_health_status().await
    }

    /// Force refresh of model discovery
    pub async fn refresh_discovery(&mut self) -> Result<ModelDiscoveryResult> {
        info!("Refreshing model discovery");
        
        // Force health check refresh
        let _ = self.health_monitor.force_check_all().await;
        
        // Rediscover all models
        self.discover_all_models().await
    }

    /// Get discovery statistics
    pub fn get_discovery_stats(&self) -> DiscoveryStats {
        let total_models = self.discovered_models.len();
        let available_models = self.discovered_models
            .values()
            .filter(|model| model.available)
            .count();
        
        let providers: std::collections::HashSet<_> = self.discovered_models
            .values()
            .map(|model| &model.provider)
            .collect();
        
        DiscoveryStats {
            total_models,
            available_models,
            total_providers: providers.len(),
            last_discovery: self.discovered_models
                .values()
                .map(|model| model.last_checked)
                .min(),
        }
    }
}

/// Statistics about model discovery
#[derive(Debug)]
pub struct DiscoveryStats {
    /// Total number of discovered models
    pub total_models: usize,
    /// Number of available models
    pub available_models: usize,
    /// Number of providers with models
    pub total_providers: usize,
    /// Time of last discovery
    pub last_discovery: Option<std::time::Instant>,
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use pretty_assertions::assert_eq;
    use super::*;

    fn create_test_model(id: &str, name: &str) -> Model {
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

    fn create_healthy_status() -> ProviderHealthStatus {
        ProviderHealthStatus::Healthy {
            response_time: Duration::from_millis(100),
            models_available: 3,
            additional_info: Some("All systems operational".to_string()),
        }
    }

    fn create_degraded_status() -> ProviderHealthStatus {
        ProviderHealthStatus::Degraded {
            reason: "High response time".to_string(),
            response_time: Duration::from_millis(2000),
            models_available: 2,
            additional_info: None,
        }
    }

    fn create_unhealthy_status() -> ProviderHealthStatus {
        ProviderHealthStatus::Unhealthy {
            reason: "Connection timeout".to_string(),
            response_time: Duration::from_millis(5000),
        }
    }

    #[tokio::test]
    async fn test_model_discovery_service_creation() {
        let config = LocalAiConfig::default();
        let actual = ModelDiscoveryService::new(config).await;
        assert!(actual.is_ok());
    }

    #[tokio::test]
    async fn test_model_discovery_service_with_ollama_config() {
        let config = LocalAiConfig::with_default_ollama();
        let actual = ModelDiscoveryService::new(config).await;
        assert!(actual.is_ok());
    }

    #[tokio::test]
    async fn test_model_discovery_service_empty_config() {
        let config = LocalAiConfig::new();
        let service = ModelDiscoveryService::new(config).await.unwrap();
        let result = service.discover_models().await.unwrap();
        
        assert_eq!(result.discovered_models.len(), 0);
        assert_eq!(result.stats.total_models, 0);
        assert_eq!(result.stats.available_models, 0);
        assert_eq!(result.stats.total_providers, 0);
    }

    #[test]
    fn test_discovered_model_healthy_provider() {
        let model = create_test_model("llama3.2:latest", "Llama 3.2");
        let health_status = create_healthy_status();
        
        let fixture = DiscoveredModel {
            model: model.clone(),
            provider: "ollama".to_string(),
            provider_health: health_status.clone(),
            available: true,
            last_checked: std::time::Instant::now(),
            response_time: Some(Duration::from_millis(100)),
        };
        
        assert_eq!(fixture.model.id, model.id);
        assert_eq!(fixture.provider, "ollama");
        assert!(fixture.available);
        assert!(matches!(fixture.provider_health, ProviderHealthStatus::Healthy { .. }));
        assert!(fixture.response_time.is_some());
    }

    #[test]
    fn test_discovered_model_degraded_provider() {
        let model = create_test_model("qwen2.5:latest", "Qwen 2.5");
        let health_status = create_degraded_status();
        
        let fixture = DiscoveredModel {
            model: model.clone(),
            provider: "ollama".to_string(),
            provider_health: health_status.clone(),
            available: true, // Still available but degraded
            last_checked: std::time::Instant::now(),
            response_time: Some(Duration::from_millis(2000)),
        };
        
        assert_eq!(fixture.model.id, model.id);
        assert_eq!(fixture.provider, "ollama");
        assert!(fixture.available);
        assert!(matches!(fixture.provider_health, ProviderHealthStatus::Degraded { .. }));
        assert_eq!(fixture.response_time.unwrap(), Duration::from_millis(2000));
    }

    #[test]
    fn test_discovered_model_unhealthy_provider() {
        let model = create_test_model("deepseek-r1:latest", "DeepSeek R1");
        let health_status = create_unhealthy_status();
        
        let fixture = DiscoveredModel {
            model: model.clone(),
            provider: "ollama".to_string(),
            provider_health: health_status.clone(),
            available: false, // Not available due to unhealthy provider
            last_checked: std::time::Instant::now(),
            response_time: None,
        };
        
        assert_eq!(fixture.model.id, model.id);
        assert_eq!(fixture.provider, "ollama");
        assert!(!fixture.available);
        assert!(matches!(fixture.provider_health, ProviderHealthStatus::Unhealthy { .. }));
        assert!(fixture.response_time.is_none());
    }

    #[test]
    fn test_discovery_stats_empty() {
        let fixture = DiscoveryStats {
            total_models: 0,
            available_models: 0,
            total_providers: 0,
            last_discovery: None,
        };
        
        assert_eq!(fixture.total_models, 0);
        assert_eq!(fixture.available_models, 0);
        assert_eq!(fixture.total_providers, 0);
        assert!(fixture.last_discovery.is_none());
    }

    #[test]
    fn test_discovery_stats_with_models() {
        let fixture = DiscoveryStats {
            total_models: 5,
            available_models: 3,
            total_providers: 2,
            last_discovery: Some(std::time::Instant::now()),
        };
        
        assert_eq!(fixture.total_models, 5);
        assert_eq!(fixture.available_models, 3);
        assert_eq!(fixture.total_providers, 2);
        assert!(fixture.last_discovery.is_some());
    }

    #[test]
    fn test_discovery_stats_availability_rate() {
        let fixture = DiscoveryStats {
            total_models: 10,
            available_models: 7,
            total_providers: 3,
            last_discovery: Some(std::time::Instant::now()),
        };
        
        let availability_rate = fixture.available_models as f64 / fixture.total_models as f64;
        assert_eq!(availability_rate, 0.7);
        assert!(availability_rate > 0.5); // More than half available
    }

    #[test]
    fn test_model_discovery_result_empty() {
        let fixture = ModelDiscoveryResult {
            discovered_models: vec![],
            stats: DiscoveryStats {
                total_models: 0,
                available_models: 0,
                total_providers: 0,
                last_discovery: Some(std::time::Instant::now()),
            },
        };
        
        assert!(fixture.discovered_models.is_empty());
        assert_eq!(fixture.stats.total_models, 0);
        assert_eq!(fixture.stats.available_models, 0);
    }

    #[test]
    fn test_model_discovery_result_with_models() {
        let model1 = create_test_model("llama3.2:latest", "Llama 3.2");
        let model2 = create_test_model("qwen2.5:latest", "Qwen 2.5");
        
        let discovered_model1 = DiscoveredModel {
            model: model1,
            provider: "ollama".to_string(),
            provider_health: create_healthy_status(),
            available: true,
            last_checked: std::time::Instant::now(),
            response_time: Some(Duration::from_millis(100)),
        };
        
        let discovered_model2 = DiscoveredModel {
            model: model2,
            provider: "ollama".to_string(),
            provider_health: create_degraded_status(),
            available: true,
            last_checked: std::time::Instant::now(),
            response_time: Some(Duration::from_millis(2000)),
        };
        
        let fixture = ModelDiscoveryResult {
            discovered_models: vec![discovered_model1, discovered_model2],
            stats: DiscoveryStats {
                total_models: 2,
                available_models: 2,
                total_providers: 1,
                last_discovery: Some(std::time::Instant::now()),
            },
        };
        
        assert_eq!(fixture.discovered_models.len(), 2);
        assert_eq!(fixture.stats.total_models, 2);
        assert_eq!(fixture.stats.available_models, 2);
        assert_eq!(fixture.stats.total_providers, 1);
        
        // Verify both models are available
        assert!(fixture.discovered_models.iter().all(|m| m.available));
        
        // Verify different health statuses
        let health_statuses: Vec<_> = fixture.discovered_models
            .iter()
            .map(|m| &m.provider_health)
            .collect();
        assert!(health_statuses.iter().any(|s| matches!(s, ProviderHealthStatus::Healthy { .. })));
        assert!(health_statuses.iter().any(|s| matches!(s, ProviderHealthStatus::Degraded { .. })));
    }

    #[test]
    fn test_model_discovery_result_mixed_availability() {
        let model1 = create_test_model("llama3.2:latest", "Llama 3.2");
        let model2 = create_test_model("qwen2.5:latest", "Qwen 2.5");
        let model3 = create_test_model("deepseek-r1:latest", "DeepSeek R1");
        
        let discovered_models = vec![
            DiscoveredModel {
                model: model1,
                provider: "ollama".to_string(),
                provider_health: create_healthy_status(),
                available: true,
                last_checked: std::time::Instant::now(),
                response_time: Some(Duration::from_millis(100)),
            },
            DiscoveredModel {
                model: model2,
                provider: "ollama".to_string(),
                provider_health: create_degraded_status(),
                available: true,
                last_checked: std::time::Instant::now(),
                response_time: Some(Duration::from_millis(2000)),
            },
            DiscoveredModel {
                model: model3,
                provider: "ollama-backup".to_string(),
                provider_health: create_unhealthy_status(),
                available: false,
                last_checked: std::time::Instant::now(),
                response_time: None,
            },
        ];
        
        let fixture = ModelDiscoveryResult {
            discovered_models: discovered_models.clone(),
            stats: DiscoveryStats {
                total_models: 3,
                available_models: 2,
                total_providers: 2,
                last_discovery: Some(std::time::Instant::now()),
            },
        };
        
        assert_eq!(fixture.discovered_models.len(), 3);
        assert_eq!(fixture.stats.total_models, 3);
        assert_eq!(fixture.stats.available_models, 2);
        assert_eq!(fixture.stats.total_providers, 2);
        
        // Verify availability counts
        let available_count = fixture.discovered_models.iter().filter(|m| m.available).count();
        let unavailable_count = fixture.discovered_models.iter().filter(|m| !m.available).count();
        assert_eq!(available_count, 2);
        assert_eq!(unavailable_count, 1);
        
        // Verify provider distribution
        let providers: std::collections::HashSet<_> = fixture.discovered_models
            .iter()
            .map(|m| &m.provider)
            .collect();
        assert_eq!(providers.len(), 2);
        assert!(providers.contains(&"ollama".to_string()));
        assert!(providers.contains(&"ollama-backup".to_string()));
    }
}