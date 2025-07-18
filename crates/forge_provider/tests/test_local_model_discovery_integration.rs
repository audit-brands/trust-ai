use std::time::Duration;

use forge_provider::config::local_ai::{
    HealthCheckConfig, LocalAiConfig, LocalProviderConfig, ProviderSpecificConfig,
};
use forge_provider::discovery::ModelDiscoveryService;
use pretty_assertions::assert_eq;
use tokio::time::timeout;

/// Advanced test for local model discovery with real network calls
/// This test will attempt to connect to a real Ollama instance if available
#[tokio::test]
async fn test_local_model_discovery_with_real_ollama() {
    // Create a configuration that tries to connect to a real Ollama instance
    let ollama_config = LocalProviderConfig {
        enabled: true,
        provider_type: "ollama".to_string(),
        endpoint: "http://localhost:11434".to_string(),
        preferred_models: vec![
            "llama3.2:latest".to_string(),
            "codellama:latest".to_string(),
        ],
        config: ProviderSpecificConfig::Ollama {
            timeout_seconds: 5, // Short timeout for testing
            max_retries: 1,     // Single retry for testing
            retry_delay_ms: 100,
            connection_pooling: true,
            user_agent: Some("forge-test-agent".to_string()),
        },
        health_check: HealthCheckConfig {
            interval_seconds: 30,
            timeout_seconds: 3, // Short timeout for testing
            failure_threshold: 1,
            success_threshold: 1,
        },
    };

    let config = LocalAiConfig::new()
        .enabled(true)
        .add_provider("test-ollama".to_string(), ollama_config);

    let mut service = ModelDiscoveryService::new(config).await.unwrap();

    // Attempt to discover models with a reasonable timeout
    let discovery_result = timeout(Duration::from_secs(10), service.discover_all_models()).await;

    // The test should complete regardless of whether Ollama is running
    assert!(discovery_result.is_ok());
    let result = discovery_result.unwrap().unwrap();

    // Log the results for debugging
    println!("Discovery completed:");
    println!("  Total models: {}", result.total_models);
    println!("  Healthy providers: {}", result.healthy_providers);
    println!("  Available models: {}", result.available_models);
    println!("  Discovery duration: {:?}", result.discovery_duration);
    println!("  Warnings: {:?}", result.warnings);

    // Basic validation - the discovery should complete successfully
    assert!(result.discovery_duration > Duration::from_millis(0));
    assert!(result.available_models <= result.total_models);

    // If Ollama is running and has models, we should see some results
    if result.total_models > 0 {
        println!(
            "Ollama appears to be running with {} models discovered",
            result.total_models
        );

        // Get the discovered models
        let discovered_models = service.get_discovered_models();
        println!("Discovered {} models", discovered_models.len());

        for model in discovered_models {
            println!(
                "  Model: {} from provider: {}",
                model.model.id.as_str(),
                model.provider
            );
            println!("    Available: {}", model.available);
            println!("    Response time: {:?}", model.response_time);
        }

        // Get provider health status
        let health_status = service.get_provider_health_status().await;
        for (provider, status) in health_status {
            println!("Provider '{}' health: {:?}", provider, status);
        }

        // If models were discovered, the discovery was successful
        assert!(result.total_models > 0);
    } else {
        println!("No models found - Ollama may not be running or accessible");
        // This is fine for testing - we just want to ensure the discovery process works
        assert_eq!(result.total_models, 0);
        assert_eq!(result.available_models, 0);
    }
}

/// Test automatic Ollama discovery functionality
#[tokio::test]
async fn test_automatic_ollama_discovery() {
    // Create a configuration without explicit Ollama providers
    // This should trigger automatic discovery
    let config = LocalAiConfig::new().enabled(true);

    let mut service = ModelDiscoveryService::new(config).await.unwrap();

    // Attempt discovery which should include automatic Ollama detection
    let discovery_result = timeout(Duration::from_secs(15), service.discover_all_models()).await;

    assert!(discovery_result.is_ok());
    let result = discovery_result.unwrap().unwrap();

    println!("Automatic discovery completed:");
    println!("  Total models: {}", result.total_models);
    println!("  Healthy providers: {}", result.healthy_providers);
    println!("  Available models: {}", result.available_models);
    println!("  Discovery duration: {:?}", result.discovery_duration);

    // The discovery should complete successfully
    assert!(result.discovery_duration > Duration::from_millis(0));

    // Check if any models were automatically discovered
    let discovered_models = service.get_discovered_models();
    if !discovered_models.is_empty() {
        println!(
            "Automatically discovered {} models",
            discovered_models.len()
        );
        for model in discovered_models {
            println!(
                "  Auto-discovered model: {} from provider: {}",
                model.model.id.as_str(),
                model.provider
            );
        }
    } else {
        println!("No models automatically discovered - this is normal if Ollama is not running");
    }
}

/// Test discovery with multiple Ollama instances on different ports
#[tokio::test]
async fn test_discovery_multiple_ollama_instances() {
    let ollama_config_1 = LocalProviderConfig {
        enabled: true,
        provider_type: "ollama".to_string(),
        endpoint: "http://localhost:11434".to_string(),
        preferred_models: vec!["llama3.2:latest".to_string()],
        config: ProviderSpecificConfig::Ollama {
            timeout_seconds: 3,
            max_retries: 1,
            retry_delay_ms: 100,
            connection_pooling: true,
            user_agent: Some("forge-test-1".to_string()),
        },
        health_check: HealthCheckConfig {
            interval_seconds: 30,
            timeout_seconds: 2,
            failure_threshold: 1,
            success_threshold: 1,
        },
    };

    let ollama_config_2 = LocalProviderConfig {
        enabled: true,
        provider_type: "ollama".to_string(),
        endpoint: "http://localhost:11435".to_string(), // Different port
        preferred_models: vec!["codellama:latest".to_string()],
        config: ProviderSpecificConfig::Ollama {
            timeout_seconds: 3,
            max_retries: 1,
            retry_delay_ms: 100,
            connection_pooling: true,
            user_agent: Some("forge-test-2".to_string()),
        },
        health_check: HealthCheckConfig {
            interval_seconds: 30,
            timeout_seconds: 2,
            failure_threshold: 1,
            success_threshold: 1,
        },
    };

    let config = LocalAiConfig::new()
        .enabled(true)
        .add_provider("ollama-primary".to_string(), ollama_config_1)
        .add_provider("ollama-secondary".to_string(), ollama_config_2);

    let mut service = ModelDiscoveryService::new(config).await.unwrap();

    // Test discovery with multiple providers
    let discovery_result = timeout(Duration::from_secs(20), service.discover_all_models()).await;

    assert!(discovery_result.is_ok());
    let result = discovery_result.unwrap().unwrap();

    println!("Multi-provider discovery completed:");
    println!("  Total models: {}", result.total_models);
    println!("  Healthy providers: {}", result.healthy_providers);
    println!("  Available models: {}", result.available_models);
    println!("  Warnings: {:?}", result.warnings);

    // Test provider-specific model retrieval
    let primary_models = service.get_provider_models("ollama-primary");
    let secondary_models = service.get_provider_models("ollama-secondary");

    println!("Primary provider models: {}", primary_models.len());
    println!("Secondary provider models: {}", secondary_models.len());

    // Get health status for both providers
    let health_status = service.get_provider_health_status().await;
    for (provider, status) in health_status {
        println!("Provider '{}' status: {:?}", provider, status);
    }

    // The discovery should complete successfully
    assert!(result.discovery_duration > Duration::from_millis(0));
    assert!(result.available_models <= result.total_models);
}

/// Test discovery refresh functionality
#[tokio::test]
async fn test_discovery_refresh_functionality() {
    let config = LocalAiConfig::with_default_ollama();
    let mut service = ModelDiscoveryService::new(config).await.unwrap();

    // Perform initial discovery
    let initial_result = timeout(Duration::from_secs(10), service.discover_all_models()).await;
    assert!(initial_result.is_ok());
    let initial = initial_result.unwrap().unwrap();

    println!("Initial discovery:");
    println!("  Total models: {}", initial.total_models);
    println!("  Healthy providers: {}", initial.healthy_providers);

    // Wait a short time
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Refresh discovery
    let refresh_result = timeout(Duration::from_secs(10), service.refresh_discovery()).await;
    assert!(refresh_result.is_ok());
    let refreshed = refresh_result.unwrap().unwrap();

    println!("Refreshed discovery:");
    println!("  Total models: {}", refreshed.total_models);
    println!("  Healthy providers: {}", refreshed.healthy_providers);

    // Both discoveries should complete successfully
    assert!(initial.discovery_duration > Duration::from_millis(0));
    assert!(refreshed.discovery_duration > Duration::from_millis(0));

    // The results might vary between calls due to network timing or Ollama state
    // changes What's important is that both complete successfully
    println!("Initial vs Refreshed comparison:");
    println!(
        "  Models: {} -> {}",
        initial.total_models, refreshed.total_models
    );
    println!(
        "  Providers: {} -> {}",
        initial.healthy_providers, refreshed.healthy_providers
    );
    println!(
        "  Available: {} -> {}",
        initial.available_models, refreshed.available_models
    );

    // Both should have consistent internal state
    assert!(initial.available_models <= initial.total_models);
    assert!(refreshed.available_models <= refreshed.total_models);
}

/// Test discovery statistics functionality
#[tokio::test]
async fn test_discovery_statistics() {
    let config = LocalAiConfig::with_default_ollama();
    let mut service = ModelDiscoveryService::new(config).await.unwrap();

    // Get initial stats (should be empty)
    let initial_stats = service.get_discovery_stats();
    assert_eq!(initial_stats.total_models, 0);
    assert_eq!(initial_stats.available_models, 0);
    assert_eq!(initial_stats.total_providers, 0);
    assert!(initial_stats.last_discovery.is_none());

    // Perform discovery
    let discovery_result = timeout(Duration::from_secs(10), service.discover_all_models()).await;
    assert!(discovery_result.is_ok());

    // Get updated stats
    let updated_stats = service.get_discovery_stats();

    println!("Discovery statistics:");
    println!("  Total models: {}", updated_stats.total_models);
    println!("  Available models: {}", updated_stats.available_models);
    println!("  Total providers: {}", updated_stats.total_providers);
    println!("  Last discovery: {:?}", updated_stats.last_discovery);

    // Stats should be updated after discovery
    assert!(updated_stats.available_models <= updated_stats.total_models);

    // If discovery found anything, last_discovery should be set
    if updated_stats.total_models > 0 || updated_stats.total_providers > 0 {
        assert!(updated_stats.last_discovery.is_some());
    }
}
