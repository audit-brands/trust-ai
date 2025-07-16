# Ollama Provider Integration Testing and Error Handling

## Overview

This document describes the comprehensive integration testing and error handling implementation for the Ollama provider in Phase 5 of the local AI integration roadmap.

## Architecture

### Error Handling System

The Ollama provider implements a robust error handling system with the following components:

#### OllamaError Types

```rust
pub enum OllamaError {
    // Connection-related errors
    ConnectionFailed { url: String, source: reqwest::Error },
    ServiceUnavailable { url: String },
    
    // Authentication and permission errors
    AuthenticationFailed { message: String },
    
    // Model-related errors
    ModelNotFound { model: String },
    ModelLoading { model: String },
    ModelLoadFailed { model: String, reason: String },
    
    // Request validation errors
    InvalidRequest { message: String },
    PayloadTooLarge { size: usize },
    RequestTimeout { timeout_seconds: u64 },
    
    // Response parsing errors
    ResponseParsingFailed { message: String },
    MalformedResponse,
    UnexpectedResponseFormat,
    
    // Streaming errors
    StreamInterrupted { reason: String },
    StreamParsingFailed { message: String },
    
    // Resource and system errors
    InsufficientResources { message: String },
    RateLimitExceeded,
    
    // Configuration errors
    InvalidConfiguration { message: String },
    InvalidBaseUrl { url: String },
    
    // Generic HTTP errors with context
    HttpError { status: u16, message: String },
    
    // Unknown errors
    Unknown { message: String },
}
```

#### Error Categorization

Errors are categorized into three main types:

1. **Service Unavailable**: Connection issues, service down, network problems
2. **Client Errors**: Invalid requests, missing models, configuration issues
3. **Retryable Errors**: Temporary failures that can be retried

#### User-Friendly Error Messages

Each error type provides user-friendly messages with actionable guidance:

```rust
impl OllamaError {
    pub fn user_message(&self) -> String {
        match self {
            OllamaError::ServiceUnavailable { url } => {
                format!("Ollama service is not running. Please start Ollama and ensure it's accessible at {}", url)
            }
            OllamaError::ModelNotFound { model } => {
                format!("Model '{}' is not available. Use 'ollama list' to see available models or 'ollama pull {}' to download it", model, model)
            }
            // ... more user-friendly messages
        }
    }
}
```

### Integration Testing Framework

#### OllamaIntegrationTest

Comprehensive integration testing suite that supports both real and mock service testing:

```rust
pub struct OllamaIntegrationTest {
    real_service_url: Option<String>,
    mock_server: MockServer,
    client: Client,
}
```

#### Test Categories

1. **Real Service Tests**: When Ollama service is available
   - Model discovery validation
   - Chat completion functionality
   - Streaming response handling
   - Performance baseline establishment

2. **Mock Service Tests**: Always run for CI/CD compatibility
   - Service unavailable scenarios
   - Invalid model requests
   - Malformed response handling
   - Connection timeout scenarios

3. **Error Handling Tests**: Comprehensive error scenario coverage
   - Network failures
   - Authentication issues
   - Rate limiting
   - Resource constraints

#### End-to-End Testing

Complete validation of the Ollama provider integration:

```rust
#[tokio::test]
async fn test_ollama_end_to_end_integration() -> anyhow::Result<()> {
    let mut test_suite = OllamaIntegrationTest::new().await?;
    test_suite.run_all_tests().await?;
    Ok(())
}
```

### Configuration and Health Checking

#### OllamaConfig

Comprehensive configuration with validation:

```rust
pub struct OllamaConfig {
    pub base_url: String,
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
    pub connection_pooling: bool,
    pub user_agent: Option<String>,
}
```

#### Health Check System

Service discovery and health monitoring:

```rust
pub struct OllamaHealthCheck {
    config: OllamaConfig,
}

pub enum HealthStatus {
    Healthy { response_time: Duration, models_available: usize },
    Degraded { reason: String, response_time: Duration },
    Unhealthy { reason: String, response_time: Duration },
}
```

## Testing Strategy

### Mock Testing

For CI/CD environments where Ollama service is not available:

```rust
// Mock server provides controlled responses
let mut mock_server = MockServer::new().await;
let mock = mock_server.mock_ollama_models(response_json, status_code).await;
```

### Real Service Testing

For development environments with Ollama installed:

```rust
// Auto-detection of running Ollama service
let real_service_url = Self::detect_ollama_service(&client).await;
```

### Performance Testing

Baseline performance validation:

```rust
// Response time validation
let start = std::time::Instant::now();
let models = ollama.models().await?;
let duration = start.elapsed();
assert!(duration < Duration::from_secs(10));
```

## Error Recovery Patterns

### Retry Logic

Automatic retry for transient failures:

```rust
impl OllamaError {
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            OllamaError::ServiceUnavailable { .. }
                | OllamaError::ConnectionFailed { .. }
                | OllamaError::RequestTimeout { .. }
                | OllamaError::ModelLoading { .. }
                | OllamaError::RateLimitExceeded
        )
    }
}
```

### Graceful Degradation

Fallback strategies for service unavailability:

1. **Service Discovery**: Automatic detection of alternative Ollama instances
2. **Configuration Validation**: Early validation to prevent runtime errors
3. **Health Monitoring**: Continuous service health assessment

## Usage Examples

### Basic Provider Setup

```rust
use forge_provider::ollama::{OllamaConfig, OllamaHealthCheck};

// Create configuration
let config = OllamaConfig::new()
    .with_base_url("http://localhost:11434".to_string())
    .with_timeout(30)
    .with_max_retries(3);

// Validate configuration
config.validate()?;

// Create provider
let ollama = config.create_provider()?;
```

### Health Check

```rust
// Check service health
let health_check = OllamaHealthCheck::new(config);
let status = health_check.check_health().await?;

match status {
    HealthStatus::Healthy { models_available, .. } => {
        println!("Ollama is healthy with {} models", models_available);
    }
    HealthStatus::Degraded { reason, .. } => {
        println!("Ollama is degraded: {}", reason);
    }
    HealthStatus::Unhealthy { reason, .. } => {
        println!("Ollama is unhealthy: {}", reason);
    }
}
```

### Error Handling

```rust
match ollama.models().await {
    Ok(models) => {
        // Process models
    }
    Err(error) => {
        if let Some(ollama_error) = error.downcast_ref::<OllamaError>() {
            eprintln!("User message: {}", ollama_error.user_message());
            
            if ollama_error.is_retryable() {
                // Implement retry logic
            }
        }
    }
}
```

### Integration Testing

```rust
// Run comprehensive integration tests
let mut test_suite = OllamaIntegrationTest::new().await?;
test_suite.run_all_tests().await?;

// Run specific test scenarios
test_suite.test_real_model_discovery().await?;
test_suite.test_service_unavailable().await?;
test_suite.test_streaming_response().await?;
```

## Quality Assurance

### Test Coverage

- **Unit Tests**: Individual component testing
- **Integration Tests**: Real and mock service testing
- **End-to-End Tests**: Complete workflow validation
- **Performance Tests**: Response time and throughput validation
- **Error Scenario Tests**: Comprehensive failure mode coverage

### Success Criteria

1. **Technical Success**:
   - Complete integration test suite with >95% coverage
   - Robust error handling for all identified failure modes
   - Performance meets baseline requirements
   - Zero regression in existing functionality

2. **User Experience Success**:
   - Clear, actionable error messages for all failure scenarios
   - Seamless provider switching and fallback
   - Predictable behavior under various network conditions
   - Comprehensive troubleshooting documentation

## Troubleshooting Guide

### Common Issues

1. **Service Unavailable**
   - Check if Ollama is running: `ollama serve`
   - Verify service URL and port
   - Check firewall and network connectivity

2. **Model Not Found**
   - List available models: `ollama list`
   - Pull required model: `ollama pull <model-name>`
   - Verify model name spelling

3. **Connection Timeout**
   - Increase timeout configuration
   - Check system resources
   - Verify network stability

4. **Authentication Failed**
   - Check Ollama configuration
   - Verify permissions and access rights
   - Review security settings

### Debugging

Enable detailed logging for troubleshooting:

```rust
// Initialize tracing
tracing_subscriber::fmt::init();

// Error context provides detailed information
let result = ollama.models().await;
if let Err(error) = result {
    tracing::error!("Ollama error: {:#}", error);
}
```

## Future Enhancements

1. **Advanced Retry Logic**: Exponential backoff, circuit breaker patterns
2. **Metrics Collection**: Performance monitoring and alerting
3. **Load Balancing**: Multiple Ollama instance support
4. **Caching**: Response caching for improved performance
5. **Security**: Enhanced authentication and encryption support