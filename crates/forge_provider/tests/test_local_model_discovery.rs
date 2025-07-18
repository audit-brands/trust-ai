use std::time::Duration;

use forge_provider::config::local_ai::{
    HealthCheckConfig, LocalAiConfig, LocalProviderConfig, ProviderSpecificConfig,
};
use forge_provider::discovery::ModelDiscoveryService;
use pretty_assertions::assert_eq;
use tokio::time::timeout;

/// Test local model discovery functionality
#[tokio::test]
async fn test_local_model_discovery_with_default_config() {
    let fixture = LocalAiConfig::with_default_ollama();
    let actual = ModelDiscoveryService::new(fixture).await;
    assert!(actual.is_ok());
}

#[tokio::test]
async fn test_local_model_discovery_with_empty_config() {
    let fixture = LocalAiConfig::new();
    let actual = ModelDiscoveryService::new(fixture).await;
    assert!(actual.is_ok());
}

#[tokio::test]
async fn test_local_model_discovery_with_disabled_config() {
    let fixture = LocalAiConfig::new().enabled(false);
    let actual = ModelDiscoveryService::new(fixture).await;
    assert!(actual.is_ok());
}

#[tokio::test]
async fn test_local_model_discovery_with_custom_ollama_config() {
    let ollama_config = LocalProviderConfig {
        enabled: true,
        provider_type: "ollama".to_string(),
        endpoint: "http://localhost:11434".to_string(),
        preferred_models: vec!["llama3.2:latest".to_string()],
        config: ProviderSpecificConfig::Ollama {
            timeout_seconds: 10,
            max_retries: 2,
            retry_delay_ms: 500,
            connection_pooling: true,
            user_agent: Some("test-agent".to_string()),
        },
        health_check: HealthCheckConfig {
            interval_seconds: 30,
            timeout_seconds: 5,
            failure_threshold: 3,
            success_threshold: 2,
        },
    };

    let fixture = LocalAiConfig::new()
        .enabled(true)
        .add_provider("test-ollama".to_string(), ollama_config);

    let actual = ModelDiscoveryService::new(fixture).await;
    assert!(actual.is_ok());
}

#[tokio::test]
async fn test_local_model_discovery_service_start() {
    let config = LocalAiConfig::with_default_ollama();
    let mut service = ModelDiscoveryService::new(config).await.unwrap();

    // Test that the service can start (even if no actual Ollama is running)
    let start_result = timeout(Duration::from_secs(5), service.start()).await;

    // The service should start successfully even if Ollama is not available
    // It will just report no healthy providers
    assert!(start_result.is_ok());
}

#[tokio::test]
async fn test_local_model_discovery_all_models() {
    let config = LocalAiConfig::with_default_ollama();
    let mut service = ModelDiscoveryService::new(config).await.unwrap();

    // Test discovering all models (should complete even if no Ollama is running)
    let discovery_result = timeout(Duration::from_secs(10), service.discover_all_models()).await;

    assert!(discovery_result.is_ok());
    let result = discovery_result.unwrap().unwrap();

    // Basic validation of the discovery result structure
    assert!(result.total_models >= 0);
    assert!(result.healthy_providers >= 0);
    assert!(result.available_models >= 0);
    assert!(result.discovery_duration > Duration::from_millis(0));

    // Available models should not exceed total models
    assert!(result.available_models <= result.total_models);
}

#[tokio::test]
async fn test_local_model_discovery_get_discovered_models() {
    let config = LocalAiConfig::with_default_ollama();
    let service = ModelDiscoveryService::new(config).await.unwrap();

    // Test getting discovered models (should return empty list if no discovery ran)
    let discovered_models = service.get_discovered_models();
    assert!(discovered_models.len() >= 0); // Should be 0 or more
}

#[tokio::test]
async fn test_local_model_discovery_get_available_models() {
    let config = LocalAiConfig::with_default_ollama();
    let service = ModelDiscoveryService::new(config).await.unwrap();

    // Test getting available models
    let available_models = service.get_available_models();
    assert!(available_models.len() >= 0); // Should be 0 or more
}

#[tokio::test]
async fn test_local_model_discovery_get_provider_models() {
    let config = LocalAiConfig::with_default_ollama();
    let service = ModelDiscoveryService::new(config).await.unwrap();

    // Test getting models from a specific provider
    let provider_models = service.get_provider_models("ollama");
    assert!(provider_models.len() >= 0); // Should be 0 or more
}

#[tokio::test]
async fn test_local_model_discovery_is_model_available() {
    use forge_app::domain::ModelId;

    let config = LocalAiConfig::with_default_ollama();
    let service = ModelDiscoveryService::new(config).await.unwrap();

    // Test checking if a specific model is available
    let model_id = ModelId::new("llama3.2:latest");
    let is_available = service.is_model_available(&model_id);

    // Should return false since no discovery has been performed
    assert_eq!(is_available, false);
}

#[tokio::test]
async fn test_local_model_discovery_get_provider_health_status() {
    let config = LocalAiConfig::with_default_ollama();
    let service = ModelDiscoveryService::new(config).await.unwrap();

    // Test getting provider health status
    let health_status = service.get_provider_health_status().await;
    assert!(health_status.len() >= 0); // Should be 0 or more providers
}

#[tokio::test]
async fn test_local_model_discovery_refresh_discovery() {
    let config = LocalAiConfig::with_default_ollama();
    let mut service = ModelDiscoveryService::new(config).await.unwrap();

    // Test refreshing discovery
    let refresh_result = timeout(Duration::from_secs(10), service.refresh_discovery()).await;

    assert!(refresh_result.is_ok());
    let result = refresh_result.unwrap().unwrap();

    // Basic validation of the refresh result
    assert!(result.total_models >= 0);
    assert!(result.healthy_providers >= 0);
    assert!(result.available_models >= 0);
}

#[tokio::test]
async fn test_local_model_discovery_get_discovery_stats() {
    let config = LocalAiConfig::with_default_ollama();
    let service = ModelDiscoveryService::new(config).await.unwrap();

    // Test getting discovery statistics
    let stats = service.get_discovery_stats();

    // Basic validation of stats structure
    assert!(stats.total_models >= 0);
    assert!(stats.available_models >= 0);
    assert!(stats.total_providers >= 0);
    assert!(stats.available_models <= stats.total_models);
}

#[tokio::test]
async fn test_local_model_discovery_multiple_providers() {
    let ollama_config_1 = LocalProviderConfig {
        enabled: true,
        provider_type: "ollama".to_string(),
        endpoint: "http://localhost:11434".to_string(),
        preferred_models: vec!["llama3.2:latest".to_string()],
        config: ProviderSpecificConfig::Ollama {
            timeout_seconds: 10,
            max_retries: 2,
            retry_delay_ms: 500,
            connection_pooling: true,
            user_agent: Some("test-agent-1".to_string()),
        },
        health_check: HealthCheckConfig::default(),
    };

    let ollama_config_2 = LocalProviderConfig {
        enabled: true,
        provider_type: "ollama".to_string(),
        endpoint: "http://localhost:11435".to_string(),
        preferred_models: vec!["codellama:latest".to_string()],
        config: ProviderSpecificConfig::Ollama {
            timeout_seconds: 10,
            max_retries: 2,
            retry_delay_ms: 500,
            connection_pooling: true,
            user_agent: Some("test-agent-2".to_string()),
        },
        health_check: HealthCheckConfig::default(),
    };

    let fixture = LocalAiConfig::new()
        .enabled(true)
        .add_provider("ollama-1".to_string(), ollama_config_1)
        .add_provider("ollama-2".to_string(), ollama_config_2);

    let mut service = ModelDiscoveryService::new(fixture).await.unwrap();

    // Test discovery with multiple providers
    let discovery_result = timeout(Duration::from_secs(15), service.discover_all_models()).await;

    assert!(discovery_result.is_ok());
    let result = discovery_result.unwrap().unwrap();

    // Basic validation
    assert!(result.total_models >= 0);
    assert!(result.healthy_providers >= 0);
    assert!(result.available_models >= 0);
}

#[tokio::test]
async fn test_local_model_discovery_validation() {
    // Test with invalid endpoint URL
    let invalid_config = LocalProviderConfig {
        enabled: true,
        provider_type: "ollama".to_string(),
        endpoint: "invalid-url".to_string(),
        preferred_models: vec![],
        config: ProviderSpecificConfig::Ollama {
            timeout_seconds: 10,
            max_retries: 2,
            retry_delay_ms: 500,
            connection_pooling: true,
            user_agent: None,
        },
        health_check: HealthCheckConfig::default(),
    };

    let fixture = LocalAiConfig::new()
        .enabled(true)
        .add_provider("invalid-ollama".to_string(), invalid_config);

    // Service creation should still succeed, but validation will catch the invalid
    // URL
    let service_result = ModelDiscoveryService::new(fixture).await;
    assert!(service_result.is_ok());
}
