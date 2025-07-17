//! Integration tests for performance monitoring and optimization

use std::time::Duration;

use forge_provider::performance::{
    parse_performance_command, ModelLoadingOptimizer, OptimizationConfig, PerformanceCli,
    PerformanceCommand, PerformanceConfig, PerformanceMeasurement, PerformanceMonitor, RequestType,
    ResourceMonitor,
};
use pretty_assertions::assert_eq;

#[tokio::test]
async fn test_performance_monitoring_integration() {
    // Create performance monitor with test configuration
    let config = PerformanceConfig::default();
    let monitor = PerformanceMonitor::new(config);

    // Start monitoring
    let start_result = monitor.start().await;
    assert!(start_result.is_ok());

    // Record some test measurements
    let measurement1 = PerformanceMeasurement::new("ollama".to_string(), RequestType::Inference)
        .complete_success()
        .with_model("llama2".to_string())
        .with_response_size(1024);

    let measurement2 = PerformanceMeasurement::new("ollama".to_string(), RequestType::HealthCheck)
        .complete_success();

    let measurement3 = PerformanceMeasurement::new("anthropic".to_string(), RequestType::Inference)
        .complete_failure();

    // Record measurements
    monitor.record_measurement(measurement1).await;
    monitor.record_measurement(measurement2).await;
    monitor.record_measurement(measurement3).await;

    // Verify metrics were recorded
    let all_metrics = monitor.get_all_metrics().await;
    assert_eq!(all_metrics.len(), 2); // ollama and anthropic

    let ollama_metrics = monitor.get_provider_metrics("ollama").await;
    assert!(ollama_metrics.is_some());
    let ollama_metrics = ollama_metrics.unwrap();
    assert_eq!(ollama_metrics.total_requests, 2);
    assert_eq!(ollama_metrics.successful_requests, 2);
    assert_eq!(ollama_metrics.failed_requests, 0);

    let anthropic_metrics = monitor.get_provider_metrics("anthropic").await;
    assert!(anthropic_metrics.is_some());
    let anthropic_metrics = anthropic_metrics.unwrap();
    assert_eq!(anthropic_metrics.total_requests, 1);
    assert_eq!(anthropic_metrics.successful_requests, 0);
    assert_eq!(anthropic_metrics.failed_requests, 1);

    // Test performance summary
    let summary = monitor.get_performance_summary().await;
    assert_eq!(summary.total_providers, 2);
    assert_eq!(summary.total_requests, 3);
    assert_eq!(summary.active_providers, 2);
    assert_eq!(summary.measurements_count, 3);
    assert_eq!(summary.overall_success_rate, 2.0 / 3.0); // 2 successful out of
                                                         // 3 total
}

#[tokio::test]
async fn test_optimization_integration() {
    // Create optimizer with test configuration
    let config = OptimizationConfig::default();
    let optimizer = ModelLoadingOptimizer::new(config);

    // Test model loading optimization
    let result = optimizer.optimize_model_loading("ollama", "llama2").await;
    assert!(result.is_ok());

    let optimization_result = result.unwrap();
    assert!(optimization_result.success);
    assert!(optimization_result.improvement.response_time_improvement > Duration::from_millis(0));

    // Test cache statistics
    let stats = optimizer.get_cache_stats().await;
    assert_eq!(stats.total_models, 1); // Should have cached the model
    assert!(stats.total_size_mb > 0);
    assert!(stats.cache_utilization > 0.0);

    // Test second optimization (should hit cache)
    let result2 = optimizer.optimize_model_loading("ollama", "llama2").await;
    assert!(result2.is_ok());

    let stats2 = optimizer.get_cache_stats().await;
    assert_eq!(stats2.total_models, 1); // Same model, should still be 1
    assert_eq!(stats2.total_accesses, 2); // Should have 2 accesses now
}

#[tokio::test]
async fn test_resource_monitoring_integration() {
    // Create resource monitor with test configuration
    let config = OptimizationConfig::default();
    let monitor = ResourceMonitor::new(config);

    // Test resource usage
    let usage = monitor.get_resource_usage().await;
    assert!(usage.memory_usage_percent >= 0.0);
    assert!(usage.cpu_usage_percent >= 0.0);
    assert!(usage.available_memory_mb > 0);
    assert!(usage.disk_usage_percent >= 0.0);
    assert!(usage.network_bandwidth_mbps >= 0.0);

    // Test pressure detection
    let is_under_pressure = monitor.is_under_pressure().await;
    // With default mock values, should not be under pressure
    assert!(!is_under_pressure);

    // Test recommendations
    let recommendations = monitor.get_resource_recommendations().await;
    // With default mock values, should not have critical recommendations
    let critical_count = recommendations
        .iter()
        .filter(|r| {
            r.severity
                == forge_provider::performance::optimization::RecommendationSeverity::Critical
        })
        .count();
    assert_eq!(critical_count, 0);
}

#[tokio::test]
async fn test_benchmark_integration() {
    // Create performance monitor and record some measurements
    let config = PerformanceConfig::default();
    let monitor = PerformanceMonitor::new(config);

    // Record measurements for benchmark
    let fast_measurement =
        PerformanceMeasurement::new("fast-provider".to_string(), RequestType::Inference)
            .complete_success();

    let slow_measurement = PerformanceMeasurement {
        provider_name: "slow-provider".to_string(),
        start_time: std::time::Instant::now(),
        end_time: std::time::Instant::now() + Duration::from_millis(1000),
        success: true,
        response_size_bytes: None,
        model_name: None,
        request_type: RequestType::Inference,
        metadata: std::collections::HashMap::new(),
    };

    monitor.record_measurement(fast_measurement).await;
    monitor.record_measurement(slow_measurement).await;

    // Run benchmark
    let report = monitor.benchmark_against_targets().await;
    assert_eq!(report.provider_comparisons.len(), 2);
    assert!(report.overall_performance_score >= 0.0);

    // Check individual provider comparisons
    let fast_comparison = report.provider_comparisons.get("fast-provider");
    assert!(fast_comparison.is_some());

    let slow_comparison = report.provider_comparisons.get("slow-provider");
    assert!(slow_comparison.is_some());
    let slow_comparison = slow_comparison.unwrap();
    // Slow provider should have lower response time ratio
    assert!(slow_comparison.response_time_vs_target < 1.0);
}

#[tokio::test]
async fn test_optimization_recommendations_integration() {
    // Create performance monitor with strict thresholds
    let mut config = PerformanceConfig::default();
    config.alert_thresholds.max_response_time = Duration::from_millis(100);
    config.alert_thresholds.min_success_rate = 0.95;

    let monitor = PerformanceMonitor::new(config);

    // Record a slow, failing measurement
    let bad_measurement = PerformanceMeasurement {
        provider_name: "problematic-provider".to_string(),
        start_time: std::time::Instant::now(),
        end_time: std::time::Instant::now() + Duration::from_millis(500), // Slow
        success: false,                                                   // Failed
        response_size_bytes: None,
        model_name: None,
        request_type: RequestType::Inference,
        metadata: std::collections::HashMap::new(),
    };

    monitor.record_measurement(bad_measurement).await;

    // Generate recommendations
    let recommendations = monitor.generate_recommendations().await;
    assert!(!recommendations.is_empty());

    // Should have recommendations for both slow response and low success rate
    let has_network_rec = recommendations.iter().any(|r| {
        matches!(
            r.recommendation_type,
            forge_provider::performance::optimization::RecommendationType::Network
        )
    });
    let has_provider_rec = recommendations.iter().any(|r| {
        matches!(
            r.recommendation_type,
            forge_provider::performance::optimization::RecommendationType::ProviderSelection
        )
    });

    assert!(has_network_rec);
    assert!(has_provider_rec);

    // Check recommendation priorities
    let critical_recommendations: Vec<_> = recommendations
        .iter()
        .filter(|r| r.priority == forge_provider::performance::optimization::Priority::Critical)
        .collect();
    assert!(!critical_recommendations.is_empty());
}

#[tokio::test]
async fn test_cli_integration() {
    // Create CLI handler
    let cli = PerformanceCli::new();
    assert!(cli.is_ok());
    let cli = cli.unwrap();

    // Test status command
    let status_result = cli.execute_command(PerformanceCommand::Status).await;
    assert!(status_result.is_ok());
    let status_output = status_result.unwrap();
    assert!(status_output.success);
    assert!(status_output.message.contains("Performance Status"));

    // Test metrics command
    let metrics_result = cli
        .execute_command(PerformanceCommand::Metrics { provider_name: None })
        .await;
    assert!(metrics_result.is_ok());
    let metrics_output = metrics_result.unwrap();
    assert!(metrics_output.success);

    // Test cache command
    let cache_result = cli.execute_command(PerformanceCommand::Cache).await;
    assert!(cache_result.is_ok());
    let cache_output = cache_result.unwrap();
    assert!(cache_output.success);
    assert!(cache_output.message.contains("Model Cache Statistics"));

    // Test resources command
    let resources_result = cli.execute_command(PerformanceCommand::Resources).await;
    assert!(resources_result.is_ok());
    let resources_output = resources_result.unwrap();
    assert!(resources_output.success);
    assert!(resources_output.message.contains("System Resource Usage"));

    // Test benchmark command
    let benchmark_result = cli.execute_command(PerformanceCommand::Benchmark).await;
    assert!(benchmark_result.is_ok());
    let benchmark_output = benchmark_result.unwrap();
    assert!(benchmark_output.success);
    assert!(benchmark_output
        .message
        .contains("Performance Benchmark Results"));

    // Test start command
    let start_result = cli.execute_command(PerformanceCommand::Start).await;
    assert!(start_result.is_ok());
    let start_output = start_result.unwrap();
    assert!(start_output.success);
    assert!(start_output.message.contains("started successfully"));
}

#[test]
fn test_cli_command_parsing() {
    // Test valid commands
    let status_cmd = parse_performance_command("status");
    assert!(status_cmd.is_ok());
    assert!(matches!(status_cmd.unwrap(), PerformanceCommand::Status));

    let metrics_cmd = parse_performance_command("metrics");
    assert!(metrics_cmd.is_ok());
    if let PerformanceCommand::Metrics { provider_name } = metrics_cmd.unwrap() {
        assert_eq!(provider_name, None);
    } else {
        panic!("Expected Metrics command");
    }

    let metrics_provider_cmd = parse_performance_command("metrics ollama");
    assert!(metrics_provider_cmd.is_ok());
    if let PerformanceCommand::Metrics { provider_name } = metrics_provider_cmd.unwrap() {
        assert_eq!(provider_name, Some("ollama".to_string()));
    } else {
        panic!("Expected Metrics command with provider");
    }

    let benchmark_cmd = parse_performance_command("benchmark");
    assert!(benchmark_cmd.is_ok());
    assert!(matches!(
        benchmark_cmd.unwrap(),
        PerformanceCommand::Benchmark
    ));

    let optimize_cmd = parse_performance_command("optimize");
    assert!(optimize_cmd.is_ok());
    if let PerformanceCommand::Optimize { provider_name } = optimize_cmd.unwrap() {
        assert_eq!(provider_name, None);
    } else {
        panic!("Expected Optimize command");
    }

    let optimize_provider_cmd = parse_performance_command("optimize ollama");
    assert!(optimize_provider_cmd.is_ok());
    if let PerformanceCommand::Optimize { provider_name } = optimize_provider_cmd.unwrap() {
        assert_eq!(provider_name, Some("ollama".to_string()));
    } else {
        panic!("Expected Optimize command with provider");
    }

    let cache_cmd = parse_performance_command("cache");
    assert!(cache_cmd.is_ok());
    assert!(matches!(cache_cmd.unwrap(), PerformanceCommand::Cache));

    let resources_cmd = parse_performance_command("resources");
    assert!(resources_cmd.is_ok());
    assert!(matches!(
        resources_cmd.unwrap(),
        PerformanceCommand::Resources
    ));

    let start_cmd = parse_performance_command("start");
    assert!(start_cmd.is_ok());
    assert!(matches!(start_cmd.unwrap(), PerformanceCommand::Start));

    let stop_cmd = parse_performance_command("stop");
    assert!(stop_cmd.is_ok());
    assert!(matches!(stop_cmd.unwrap(), PerformanceCommand::Stop));

    // Test invalid command
    let invalid_cmd = parse_performance_command("invalid");
    assert!(invalid_cmd.is_err());

    // Test empty command (should default to status)
    let empty_cmd = parse_performance_command("");
    assert!(empty_cmd.is_ok());
    assert!(matches!(empty_cmd.unwrap(), PerformanceCommand::Status));
}

#[tokio::test]
async fn test_end_to_end_performance_workflow() {
    // This test simulates a complete performance monitoring workflow

    // 1. Create CLI and start monitoring
    let cli = PerformanceCli::new().unwrap();
    let start_result = cli.execute_command(PerformanceCommand::Start).await;
    assert!(start_result.is_ok());

    // 2. Simulate some provider activity by creating measurements
    let config = PerformanceConfig::default();
    let monitor = PerformanceMonitor::new(config);

    // Simulate successful Ollama requests
    for i in 0..10 {
        let measurement = PerformanceMeasurement::new("ollama".to_string(), RequestType::Inference)
            .complete_success()
            .with_model(format!("model-{}", i % 3))
            .with_response_size(1024 + i * 100);

        monitor.record_measurement(measurement).await;
    }

    // Simulate some failed Anthropic requests
    for i in 0..3 {
        let measurement =
            PerformanceMeasurement::new("anthropic".to_string(), RequestType::Inference)
                .complete_failure()
                .with_model("claude-3".to_string());

        monitor.record_measurement(measurement).await;
    }

    // 3. Check overall status
    let status_result = cli
        .execute_command(PerformanceCommand::Status)
        .await
        .unwrap();
    assert!(status_result.success);
    assert!(status_result.message.contains("Total Providers"));

    // 4. Get detailed metrics
    let metrics_result = cli
        .execute_command(PerformanceCommand::Metrics { provider_name: None })
        .await
        .unwrap();
    assert!(metrics_result.success);

    // 5. Run optimization
    let optimize_result = cli
        .execute_command(PerformanceCommand::Optimize { provider_name: Some("ollama".to_string()) })
        .await
        .unwrap();
    assert!(optimize_result.success);

    // 6. Check cache after optimization
    let cache_result = cli
        .execute_command(PerformanceCommand::Cache)
        .await
        .unwrap();
    assert!(cache_result.success);
    assert!(cache_result.message.contains("Model Cache Statistics"));

    // 7. Run benchmark
    let benchmark_result = cli
        .execute_command(PerformanceCommand::Benchmark)
        .await
        .unwrap();
    assert!(benchmark_result.success);
    assert!(benchmark_result
        .message
        .contains("Performance Benchmark Results"));

    // 8. Check system resources
    let resources_result = cli
        .execute_command(PerformanceCommand::Resources)
        .await
        .unwrap();
    assert!(resources_result.success);
    assert!(resources_result.message.contains("System Resource Usage"));

    // 9. Generate optimization recommendations
    let optimize_all_result = cli
        .execute_command(PerformanceCommand::Optimize { provider_name: None })
        .await
        .unwrap();
    assert!(optimize_all_result.success);
}
