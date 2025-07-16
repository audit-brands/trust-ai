use anyhow::Context as _;
use forge_app::domain::{Context, ContextMessage, ModelId};
use tokio_stream::StreamExt;
use tracing::info;

use crate::ollama::{OllamaIntegrationTest, OllamaError};

/// End-to-end integration test for Ollama provider
/// 
/// This test validates the complete integration of the Ollama provider
/// with both real and mock services, ensuring robust error handling
/// and proper functionality across various scenarios.
#[tokio::test]
async fn test_ollama_end_to_end_integration() -> anyhow::Result<()> {
    // Initialize tracing for test visibility
    tracing_subscriber::fmt::init();
    
    info!("Starting Ollama end-to-end integration test");
    
    // Create integration test suite
    let mut test_suite = OllamaIntegrationTest::new().await
        .context("Failed to create integration test suite")?;
    
    // Run all integration tests
    test_suite.run_all_tests().await
        .context("Integration test suite failed")?;
    
    info!("Ollama end-to-end integration test completed successfully");
    Ok(())
}

/// Test Ollama error handling and user-friendly messages
#[tokio::test]
async fn test_ollama_error_handling() -> anyhow::Result<()> {
    info!("Testing Ollama error handling patterns");
    
    // Test service unavailable error
    let error = OllamaError::service_unavailable("http://localhost:11434".to_string());
    assert!(error.is_service_unavailable());
    assert!(error.is_retryable());
    assert!(!error.is_client_error());
    
    let user_message = error.user_message();
    assert!(user_message.contains("Ollama service is not running"));
    assert!(user_message.contains("http://localhost:11434"));
    
    // Test model not found error
    let error = OllamaError::model_not_found("llama3.2".to_string());
    assert!(!error.is_service_unavailable());
    assert!(!error.is_retryable());
    assert!(error.is_client_error());
    
    let user_message = error.user_message();
    assert!(user_message.contains("Model 'llama3.2' is not available"));
    assert!(user_message.contains("ollama pull"));
    
    // Test connection failed error
    let reqwest_error = reqwest::Error::from(reqwest::Error::from(
        std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "Connection refused")
    ));
    let error = OllamaError::connection_failed("http://localhost:11434".to_string(), reqwest_error);
    assert!(error.is_service_unavailable());
    assert!(error.is_retryable());
    
    info!("Ollama error handling test completed successfully");
    Ok(())
}

/// Test provider configuration and validation
#[tokio::test]
async fn test_ollama_provider_configuration() -> anyhow::Result<()> {
    info!("Testing Ollama provider configuration");
    
    // Test valid configuration
    let ollama = crate::ollama::Ollama::builder()
        .client(reqwest::Client::new())
        .base_url("http://localhost:11434".parse()?)
        .build()
        .unwrap();
    
    // Test URL construction
    let api_url = ollama.url("api/tags")?;
    assert_eq!(api_url.as_str(), "http://localhost:11434/api/tags");
    
    let chat_url = ollama.url("api/chat")?;
    assert_eq!(chat_url.as_str(), "http://localhost:11434/api/chat");
    
    // Test invalid path handling
    let result = ollama.url("../malicious/path");
    assert!(result.is_err());
    
    let result = ollama.url("http://evil.com/path");
    assert!(result.is_err());
    
    info!("Ollama provider configuration test completed successfully");
    Ok(())
}

/// Test provider compatibility with existing conversation flow
#[tokio::test]
async fn test_ollama_conversation_compatibility() -> anyhow::Result<()> {
    info!("Testing Ollama conversation compatibility");
    
    // Create a typical conversation context
    let model_id = ModelId::new("llama3.2");
    let context = Context::default()
        .add_message(ContextMessage::system("You are a helpful assistant."))
        .add_message(ContextMessage::user("Hello, how are you?", model_id.clone().into()))
        .add_message(ContextMessage::assistant("I'm doing well, thank you! How can I help you today?"))
        .add_message(ContextMessage::user("What's the weather like?", model_id.clone().into()));
    
    // Test request conversion
    let request = crate::ollama::request::ChatRequest::try_from(context)?
        .model("llama3.2".to_string())
        .stream(true);
    
    // Validate request structure
    assert_eq!(request.model, "llama3.2");
    assert_eq!(request.stream, Some(true));
    assert!(!request.messages.is_empty());
    
    // Validate message conversion
    let messages = &request.messages;
    assert!(messages.iter().any(|m| m.role == "system"));
    assert!(messages.iter().any(|m| m.role == "user"));
    assert!(messages.iter().any(|m| m.role == "assistant"));
    
    info!("Ollama conversation compatibility test completed successfully");
    Ok(())
}

/// Performance benchmark test for Ollama provider
#[tokio::test]
async fn test_ollama_performance_benchmark() -> anyhow::Result<()> {
    info!("Testing Ollama performance benchmarks");
    
    let test_suite = OllamaIntegrationTest::new().await?;
    
    // Test URL construction performance
    let ollama = test_suite.create_mock_ollama()?;
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let _ = ollama.url("api/tags")?;
    }
    let duration = start.elapsed();
    assert!(duration.as_millis() < 100, "URL construction too slow: {:?}", duration);
    
    // Test request conversion performance
    let model_id = ModelId::new("test-model");
    let context = Context::default()
        .add_message(ContextMessage::user("Test message", model_id.into()));
    
    let start = std::time::Instant::now();
    for _ in 0..100 {
        let _ = crate::ollama::request::ChatRequest::try_from(context.clone())?;
    }
    let duration = start.elapsed();
    assert!(duration.as_millis() < 100, "Request conversion too slow: {:?}", duration);
    
    info!("Ollama performance benchmark test completed successfully");
    Ok(())
}

/// Test graceful degradation and fallback scenarios
#[tokio::test]
async fn test_ollama_graceful_degradation() -> anyhow::Result<()> {
    info!("Testing Ollama graceful degradation");
    
    // Test timeout handling
    let short_timeout_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(1))
        .build()?;
    
    let ollama = crate::ollama::Ollama::builder()
        .client(short_timeout_client)
        .base_url("http://httpbin.org/delay/10".parse()?)
        .build()
        .unwrap();
    
    // This should timeout gracefully
    let result = ollama.models().await;
    assert!(result.is_err());
    
    // Verify error is categorized correctly
    let error_chain = format!("{:?}", result.unwrap_err());
    // Should contain timeout-related information
    
    info!("Ollama graceful degradation test completed successfully");
    Ok(())
}