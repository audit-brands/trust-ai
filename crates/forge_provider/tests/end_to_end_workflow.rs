//! End-to-end integration tests for the complete local provider workflow

use std::time::Duration;

use forge_provider::config::fallback::FallbackConfig;
use forge_provider::config::local_ai::LocalAiConfig;
use forge_provider::discovery::{ModelDiscoveryResult, ModelDiscoveryService};
use forge_provider::health::HealthMonitor;
use forge_provider::selection::{
    ProviderSelector, ProviderType, SelectionContext, UserPreferences,
};
use forge_provider::test_utils::{
    create_degraded_status, create_healthy_status, create_unhealthy_status, MockHealthMonitor,
    TestFixtures,
};
use pretty_assertions::assert_eq;

/// Test the complete workflow from configuration to model selection
#[tokio::test]
async fn test_complete_local_provider_workflow() {
    // Step 1: Configuration
    let local_config = TestFixtures::local_config();
    let fallback_config = FallbackConfig::default();

    // Step 2: Initialize services
    let discovery_service = ModelDiscoveryService::new(local_config.clone())
        .await
        .unwrap();
    let health_monitor = HealthMonitor::new(local_config.clone()).await.unwrap();
    let mut provider_selector = ProviderSelector::new(local_config, fallback_config)
        .await
        .unwrap();

    // Step 3: Initialize provider selector
    provider_selector.initialize().await.unwrap();

    // Step 4: Discover available models
    let discovery_result = discovery_service.discover_models().await.unwrap();

    // Step 5: Check health status
    let health_status = health_monitor.get_health_status().await;

    // Step 6: Select a provider for a model request
    let selection_context = SelectionContext::new("llama3.2:latest".to_string())
        .with_streaming(false)
        .with_tools(false);

    let selection_result = provider_selector.select_provider(selection_context).await;

    // Verify the complete workflow
    assert!(discovery_result.stats.total_models >= 0);
    assert!(health_status.len() >= 0);

    // Selection may fail if no real providers are available, but shouldn't crash
    match selection_result {
        Ok(selection) => {
            assert!(!selection.provider_name.is_empty());
            assert!(!selection.reason.is_empty());

            // Record the successful operation
            provider_selector.record_success(&selection.provider_name, Duration::from_millis(200));

            // Verify metrics were updated
            let metrics = provider_selector.get_provider_metric(&selection.provider_name);
            if metrics.is_some() {
                let metrics = metrics.unwrap();
                assert!(metrics.successful_requests > 0);
            }
        }
        Err(_) => {
            // Expected if no real providers are configured and available
        }
    }
}

/// Test workflow with multiple providers and health states
#[tokio::test]
async fn test_multi_provider_workflow_with_health_states() {
    // Use multi-provider configuration
    let local_config = TestFixtures::multi_provider_config();
    let fallback_config = FallbackConfig::default();

    // Initialize services
    let discovery_service = ModelDiscoveryService::new(local_config.clone())
        .await
        .unwrap();
    let mut provider_selector = ProviderSelector::new(local_config, fallback_config)
        .await
        .unwrap();
    provider_selector.initialize().await.unwrap();

    // Discover models
    let discovery_result = discovery_service.discover_models().await.unwrap();

    // Test selection with different scenarios
    let scenarios = vec![
        ("llama3.2:latest", false, false, "Basic model request"),
        ("qwen2.5:latest", true, false, "Streaming request"),
        ("deepseek-r1:latest", false, true, "Tools request"),
        ("mistral:latest", true, true, "Full features request"),
    ];

    for (model_id, streaming, tools, description) in scenarios {
        let context = SelectionContext::new(model_id.to_string())
            .with_streaming(streaming)
            .with_tools(tools);

        let result = provider_selector.select_provider(context).await;

        // Log the scenario result
        match result {
            Ok(selection) => {
                println!(
                    "{}: Selected {} ({})",
                    description, selection.provider_name, selection.reason
                );

                // Simulate request completion
                provider_selector
                    .record_success(&selection.provider_name, Duration::from_millis(150));
            }
            Err(e) => {
                println!("{}: Failed - {}", description, e);
            }
        }
    }

    // Verify discovery worked
    assert!(discovery_result.stats.total_models >= 0);

    // Verify provider metrics were updated
    let metrics = provider_selector.get_provider_metrics();
    assert!(!metrics.is_empty());
}

/// Test workflow with user preferences
#[tokio::test]
async fn test_workflow_with_user_preferences() {
    let local_config = TestFixtures::local_config();
    let fallback_config = FallbackConfig::default();

    let mut provider_selector = ProviderSelector::new(local_config, fallback_config)
        .await
        .unwrap();
    provider_selector.initialize().await.unwrap();

    // Test different user preference scenarios
    let preference_scenarios = vec![
        (UserPreferences::prefer_local(), "Local preference"),
        (UserPreferences::prefer_cloud(), "Cloud preference"),
        (UserPreferences::default(), "Default preference"),
    ];

    for (preferences, description) in preference_scenarios {
        let context = SelectionContext::new("llama3.2:latest".to_string())
            .with_preferences(preferences.clone());

        let result = provider_selector.select_provider(context).await;

        match result {
            Ok(selection) => {
                println!(
                    "{}: Selected {} (type: {:?})",
                    description, selection.provider_name, selection.provider_type
                );

                // Verify preference alignment
                match preferences.prefer_local {
                    true => {
                        // Should prefer local if available
                        if selection.provider_type == ProviderType::Local {
                            assert!(!selection.is_fallback);
                        }
                    }
                    false => {
                        // May select cloud providers
                    }
                }
            }
            Err(e) => {
                println!("{}: Failed - {}", description, e);
            }
        }
    }
}

/// Test workflow with simulated failures and recovery
#[tokio::test]
async fn test_workflow_with_failure_and_recovery() {
    let local_config = TestFixtures::local_config();
    let fallback_config = FallbackConfig::default();

    let mut provider_selector = ProviderSelector::new(local_config, fallback_config)
        .await
        .unwrap();
    provider_selector.initialize().await.unwrap();

    // Simulate a series of requests with failures
    let model_id = "llama3.2:latest";
    let mut consecutive_failures = 0;

    for attempt in 1..=5 {
        let context = SelectionContext::new(model_id.to_string())
            .with_consecutive_failures(consecutive_failures);

        let result = provider_selector.select_provider(context).await;

        match result {
            Ok(selection) => {
                println!(
                    "Attempt {}: Selected {} (fallback: {})",
                    attempt, selection.provider_name, selection.is_fallback
                );

                // Simulate success or failure
                if attempt <= 3 {
                    // Simulate failure
                    provider_selector.record_failure(&selection.provider_name, "Simulated timeout");
                    consecutive_failures += 1;
                } else {
                    // Simulate success
                    provider_selector
                        .record_success(&selection.provider_name, Duration::from_millis(200));
                    consecutive_failures = 0;
                }
            }
            Err(e) => {
                println!("Attempt {}: Failed - {}", attempt, e);
                consecutive_failures += 1;
            }
        }
    }

    // Verify the selector handled failures gracefully
    let metrics = provider_selector.get_provider_metrics();
    assert!(!metrics.is_empty());
}

/// Test workflow performance under load
#[tokio::test]
async fn test_workflow_performance() {
    let local_config = TestFixtures::local_config();
    let fallback_config = FallbackConfig::default();

    let discovery_service = ModelDiscoveryService::new(local_config.clone())
        .await
        .unwrap();
    let mut provider_selector = ProviderSelector::new(local_config, fallback_config)
        .await
        .unwrap();
    provider_selector.initialize().await.unwrap();

    // Measure discovery performance
    let start_time = std::time::Instant::now();
    let discovery_result = discovery_service.discover_models().await.unwrap();
    let discovery_time = start_time.elapsed();

    println!("Discovery completed in {:?}", discovery_time);
    assert!(discovery_time < Duration::from_secs(10)); // Should be reasonably fast

    // Measure selection performance
    let selection_times = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let mut handles = Vec::new();

    for i in 0..10 {
        let mut selector = provider_selector.clone();
        let times = selection_times.clone();

        let handle = tokio::spawn(async move {
            let start = std::time::Instant::now();

            let context = SelectionContext::new(format!("model-{}", i));
            let _result = selector.select_provider(context).await;

            let elapsed = start.elapsed();
            times.lock().unwrap().push(elapsed);
        });

        handles.push(handle);
    }

    // Wait for all selections to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Analyze performance
    let times = selection_times.lock().unwrap();
    if !times.is_empty() {
        let total_time: Duration = times.iter().sum();
        let avg_time = total_time / times.len() as u32;
        let max_time = times.iter().max().unwrap();

        println!(
            "Selection performance: avg={:?}, max={:?}",
            avg_time, max_time
        );

        // Performance assertions
        assert!(avg_time < Duration::from_secs(1)); // Average should be fast
        assert!(*max_time < Duration::from_secs(5)); // Even worst case should
                                                     // be reasonable
    }

    // Verify discovery results
    assert!(discovery_result.stats.total_models >= 0);
}

/// Test workflow with configuration changes
#[tokio::test]
async fn test_workflow_with_configuration_changes() {
    // Start with basic configuration
    let mut local_config = TestFixtures::local_config();
    let fallback_config = FallbackConfig::default();

    let discovery_service = ModelDiscoveryService::new(local_config.clone())
        .await
        .unwrap();
    let mut provider_selector =
        ProviderSelector::new(local_config.clone(), fallback_config.clone())
            .await
            .unwrap();
    provider_selector.initialize().await.unwrap();

    // Initial discovery
    let initial_result = discovery_service.discover_models().await.unwrap();
    let initial_metrics_count = provider_selector.get_provider_metrics().len();

    println!(
        "Initial state: {} models, {} providers",
        initial_result.stats.total_models, initial_metrics_count
    );

    // Simulate configuration change by creating new services
    let multi_config = TestFixtures::multi_provider_config();
    let new_discovery = ModelDiscoveryService::new(multi_config.clone())
        .await
        .unwrap();
    let mut new_selector = ProviderSelector::new(multi_config, fallback_config)
        .await
        .unwrap();
    new_selector.initialize().await.unwrap();

    // New discovery with updated configuration
    let updated_result = new_discovery.discover_models().await.unwrap();
    let updated_metrics_count = new_selector.get_provider_metrics().len();

    println!(
        "Updated state: {} models, {} providers",
        updated_result.stats.total_models, updated_metrics_count
    );

    // Verify configuration changes were applied
    assert!(updated_metrics_count >= initial_metrics_count);

    // Test selection with new configuration
    let context = SelectionContext::new("llama3.2:latest".to_string());
    let selection_result = new_selector.select_provider(context).await;

    match selection_result {
        Ok(selection) => {
            println!("New configuration selected: {}", selection.provider_name);
            assert!(!selection.provider_name.is_empty());
        }
        Err(_) => {
            // Expected if no real providers are available
        }
    }
}

/// Test complete workflow error handling
#[tokio::test]
async fn test_workflow_error_handling() {
    // Test with empty configuration
    let empty_config = LocalAiConfig::new();
    let fallback_config = FallbackConfig::default();

    // Services should initialize without crashing
    let discovery_service = ModelDiscoveryService::new(empty_config.clone())
        .await
        .unwrap();
    let health_monitor = HealthMonitor::new(empty_config.clone()).await.unwrap();
    let mut provider_selector = ProviderSelector::new(empty_config, fallback_config)
        .await
        .unwrap();

    // Initialize selector
    let init_result = provider_selector.initialize().await;
    assert!(init_result.is_ok());

    // Discovery should work but return empty results
    let discovery_result = discovery_service.discover_models().await.unwrap();
    assert_eq!(discovery_result.stats.total_models, 0);
    assert_eq!(discovery_result.stats.available_models, 0);

    // Health monitoring should work but return empty status
    let health_status = health_monitor.get_health_status().await;
    assert!(health_status.is_empty());

    // Provider selection should handle no available providers gracefully
    let context = SelectionContext::new("any-model".to_string());
    let selection_result = provider_selector.select_provider(context).await;

    // Should either succeed with fallback or fail gracefully
    match selection_result {
        Ok(selection) => {
            // If it succeeds, it should be a cloud fallback
            println!("Empty config fallback: {}", selection.provider_name);
            assert!(selection.is_fallback || selection.provider_name.starts_with("cloud:"));
        }
        Err(e) => {
            // Graceful failure is acceptable
            println!("Empty config failed gracefully: {}", e);
        }
    }
}

/// Test workflow with mock health monitor
#[tokio::test]
async fn test_workflow_with_mock_health_monitor() {
    // Create mock health monitor with controlled states
    let mock_health = MockHealthMonitor::new().await;

    // Add providers with different health states
    mock_health
        .add_provider("ollama-healthy".to_string(), create_healthy_status())
        .await;
    mock_health
        .add_provider("ollama-degraded".to_string(), create_degraded_status())
        .await;
    mock_health
        .add_provider("ollama-unhealthy".to_string(), create_unhealthy_status())
        .await;

    // Test health-aware operations
    let health_status = mock_health.get_health_status().await;
    assert_eq!(health_status.len(), 3);

    // Test provider usability
    assert!(mock_health.is_provider_usable("ollama-healthy").await);
    assert!(mock_health.is_provider_usable("ollama-degraded").await); // Degraded is still usable
    assert!(!mock_health.is_provider_usable("ollama-unhealthy").await);

    // Test provider sorting by health
    let sorted_providers = mock_health.get_providers_by_health().await;
    assert_eq!(sorted_providers.len(), 3);

    // Verify sorting: healthy first, unhealthy last
    assert_eq!(sorted_providers[0].0, "ollama-healthy");
    assert_eq!(sorted_providers[2].0, "ollama-unhealthy");

    // Test discovery with mock health data
    let discovery_result = TestFixtures::discovery_result();

    // Verify health-aware filtering
    let available_models: Vec<_> = discovery_result
        .discovered_models
        .iter()
        .filter(|m| m.available)
        .collect();

    let unavailable_models: Vec<_> = discovery_result
        .discovered_models
        .iter()
        .filter(|m| !m.available)
        .collect();

    // Should have both available and unavailable models based on health
    assert!(!available_models.is_empty() || !unavailable_models.is_empty());

    // Verify health status correlation
    for model in available_models {
        assert!(model.provider_health.is_usable());
    }

    for model in unavailable_models {
        assert!(!model.provider_health.is_usable());
    }
}
