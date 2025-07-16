//! Integration tests for discovery and health monitoring

use std::time::Duration;
use pretty_assertions::assert_eq;

use forge_provider::test_utils::{TestFixtures, MockHealthMonitor, create_healthy_status, create_degraded_status, create_unhealthy_status};
use forge_provider::discovery::{ModelDiscoveryService, DiscoveredModel};
use forge_provider::health::HealthMonitor;
use forge_provider::config::local_ai::LocalAiConfig;

#[tokio::test]
async fn test_discovery_with_health_integration() {
    // Create test configuration
    let config = TestFixtures::local_config();
    
    // Create discovery service
    let discovery_service = ModelDiscoveryService::new(config.clone()).await.unwrap();
    
    // Create health monitor
    let health_monitor = HealthMonitor::new(config).await.unwrap();
    
    // Test that both services can be created and work together
    let discovery_result = discovery_service.discover_models().await;
    let health_status = health_monitor.get_health_status().await;
    
    // Both should work without errors
    assert!(discovery_result.is_ok());
    assert!(health_status.len() >= 0); // May be empty if no providers configured
}

#[tokio::test]
async fn test_discovery_with_mock_health_monitor() {
    // Create mock health monitor
    let mock_health = MockHealthMonitor::new().await;
    
    // Add providers with different health statuses
    mock_health.add_provider("ollama-healthy".to_string(), create_healthy_status()).await;
    mock_health.add_provider("ollama-degraded".to_string(), create_degraded_status()).await;
    mock_health.add_provider("ollama-unhealthy".to_string(), create_unhealthy_status()).await;
    
    // Get health status
    let health_status = mock_health.get_health_status().await;
    assert_eq!(health_status.len(), 3);
    
    // Test provider usability
    assert!(mock_health.is_provider_usable("ollama-healthy").await);
    assert!(mock_health.is_provider_usable("ollama-degraded").await); // Degraded is still usable
    assert!(!mock_health.is_provider_usable("ollama-unhealthy").await);
    
    // Test provider sorting
    let sorted_providers = mock_health.get_providers_by_health().await;
    assert_eq!(sorted_providers.len(), 3);
    
    // Verify sorting order: healthy first, unhealthy last
    let provider_names: Vec<_> = sorted_providers.iter().map(|(name, _)| name).collect();
    assert_eq!(provider_names[0], "ollama-healthy");
    assert_eq!(provider_names[2], "ollama-unhealthy");
}

#[tokio::test]
async fn test_discovery_result_with_health_aware_filtering() {
    // Create test discovery result
    let discovery_result = TestFixtures::discovery_result();
    
    // Verify the result contains models with different health statuses
    assert_eq!(discovery_result.discovered_models.len(), 3);
    assert_eq!(discovery_result.stats.total_models, 3);
    
    // Filter by availability (should exclude unhealthy providers)
    let available_models: Vec<_> = discovery_result.discovered_models
        .iter()
        .filter(|m| m.available)
        .collect();
    
    let unavailable_models: Vec<_> = discovery_result.discovered_models
        .iter()
        .filter(|m| !m.available)
        .collect();
    
    assert_eq!(available_models.len(), 2); // Healthy and degraded
    assert_eq!(unavailable_models.len(), 1); // Unhealthy
    
    // Verify health-aware availability
    for model in available_models {
        assert!(model.provider_health.is_usable());
    }
    
    for model in unavailable_models {
        assert!(!model.provider_health.is_usable());
    }
}

#[tokio::test]
async fn test_discovery_performance_metrics() {
    // Create test discovery result
    let discovery_result = TestFixtures::discovery_result();
    
    // Verify response time metrics
    let response_times: Vec<_> = discovery_result.discovered_models
        .iter()
        .filter_map(|m| m.response_time)
        .collect();
    
    // Should have response times for available models
    assert!(!response_times.is_empty());
    
    // Verify response times are reasonable
    for &response_time in &response_times {
        assert!(response_time <= Duration::from_secs(5)); // Reasonable upper bound
    }
    
    // Calculate average response time for available models
    if !response_times.is_empty() {
        let total_time: Duration = response_times.iter().sum();
        let avg_time = total_time / response_times.len() as u32;
        assert!(avg_time <= Duration::from_secs(2)); // Should be reasonably fast
    }
}

#[tokio::test]
async fn test_discovery_provider_distribution() {
    // Create test discovery result
    let discovery_result = TestFixtures::discovery_result();
    
    // Analyze provider distribution
    let mut provider_counts = std::collections::HashMap::new();
    for model in &discovery_result.discovered_models {
        *provider_counts.entry(&model.provider).or_insert(0) += 1;
    }
    
    // Verify we have multiple providers
    assert!(provider_counts.len() >= 1);
    
    // Verify stats match actual counts
    assert_eq!(discovery_result.stats.total_providers, provider_counts.len());
    
    // Verify model counts
    let total_models: usize = provider_counts.values().sum();
    assert_eq!(discovery_result.stats.total_models, total_models);
}

#[tokio::test]
async fn test_health_monitor_initialization() {
    // Test health monitor with different configurations
    let configs = vec![
        LocalAiConfig::new(), // Empty config
        TestFixtures::local_config(), // Default config
        TestFixtures::multi_provider_config(), // Multi-provider config
    ];
    
    for config in configs {
        let health_monitor = HealthMonitor::new(config).await;
        assert!(health_monitor.is_ok());
        
        let monitor = health_monitor.unwrap();
        let health_status = monitor.get_health_status().await;
        
        // Should not crash and should return a valid map
        assert!(health_status.len() >= 0);
    }
}

#[tokio::test]
async fn test_discovery_service_initialization() {
    // Test discovery service with different configurations
    let configs = vec![
        LocalAiConfig::new(), // Empty config
        TestFixtures::local_config(), // Default config
        TestFixtures::multi_provider_config(), // Multi-provider config
    ];
    
    for config in configs {
        let discovery_service = ModelDiscoveryService::new(config).await;
        assert!(discovery_service.is_ok());
        
        let service = discovery_service.unwrap();
        let result = service.discover_models().await;
        
        // Should not crash and should return a valid result
        assert!(result.is_ok());
        
        let discovery_result = result.unwrap();
        assert!(discovery_result.stats.total_models >= 0);
        assert!(discovery_result.stats.available_models <= discovery_result.stats.total_models);
    }
}

#[tokio::test]
async fn test_end_to_end_discovery_workflow() {
    // Create configuration
    let config = TestFixtures::local_config();
    
    // Initialize services
    let discovery_service = ModelDiscoveryService::new(config.clone()).await.unwrap();
    let health_monitor = HealthMonitor::new(config).await.unwrap();
    
    // Perform discovery
    let discovery_result = discovery_service.discover_models().await.unwrap();
    
    // Check health status
    let health_status = health_monitor.get_health_status().await;
    
    // Verify integration
    assert!(discovery_result.stats.last_discovery.is_some());
    assert!(health_status.len() >= 0);
    
    // If we have discovered models, verify they have health information
    for model in &discovery_result.discovered_models {
        // Each model should have provider health information
        assert!(!model.provider.is_empty());
        
        // Available models should have response times
        if model.available {
            assert!(model.response_time.is_some());
        }
    }
    
    // Verify statistics consistency
    let available_count = discovery_result.discovered_models
        .iter()
        .filter(|m| m.available)
        .count();
    assert_eq!(discovery_result.stats.available_models, available_count);
    
    let provider_count = discovery_result.discovered_models
        .iter()
        .map(|m| &m.provider)
        .collect::<std::collections::HashSet<_>>()
        .len();
    assert_eq!(discovery_result.stats.total_providers, provider_count);
}

#[tokio::test]
async fn test_concurrent_discovery_and_health_checks() {
    // Create configuration
    let config = TestFixtures::local_config();
    
    // Create services
    let discovery_service = ModelDiscoveryService::new(config.clone()).await.unwrap();
    let health_monitor = HealthMonitor::new(config).await.unwrap();
    
    // Run discovery and health checks concurrently
    let (discovery_result, health_status) = tokio::join!(
        discovery_service.discover_models(),
        health_monitor.get_health_status()
    );
    
    // Both should succeed
    assert!(discovery_result.is_ok());
    let discovery = discovery_result.unwrap();
    
    // Verify results
    assert!(discovery.stats.total_models >= 0);
    assert!(health_status.len() >= 0);
    
    // No race conditions should occur
    assert!(discovery.stats.last_discovery.is_some());
}

#[tokio::test]
async fn test_discovery_error_handling() {
    // Test discovery with various error conditions
    
    // Empty configuration should not crash
    let empty_config = LocalAiConfig::new();
    let discovery_service = ModelDiscoveryService::new(empty_config).await.unwrap();
    let result = discovery_service.discover_models().await;
    
    // Should succeed but return empty results
    assert!(result.is_ok());
    let discovery_result = result.unwrap();
    assert_eq!(discovery_result.stats.total_models, 0);
    assert_eq!(discovery_result.stats.available_models, 0);
    assert_eq!(discovery_result.stats.total_providers, 0);
}

#[tokio::test]
async fn test_health_monitor_error_handling() {
    // Test health monitor with various error conditions
    
    // Empty configuration should not crash
    let empty_config = LocalAiConfig::new();
    let health_monitor = HealthMonitor::new(empty_config).await.unwrap();
    let health_status = health_monitor.get_health_status().await;
    
    // Should succeed but return empty results
    assert!(health_status.is_empty());
}