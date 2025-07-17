# API Overview

## Introduction
Trust AI provides a comprehensive Rust API for integrating AI capabilities into your applications. The API is designed with performance, safety, and ease of use in mind.

## Core Architecture

### Main Components

```rust
use trust_ai::{
    TrustAI,
    Config,
    Provider,
    Model,
    ChatRequest,
    ChatResponse,
    PerformanceMonitor,
    Cache,
};
```

### Key Traits

#### `Provider` Trait
The `Provider` trait defines the interface for AI service providers:

```rust
#[async_trait]
pub trait Provider: Send + Sync {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse>;
    async fn models(&self) -> Result<Vec<Model>>;
    async fn health_check(&self) -> Result<ProviderHealth>;
}
```

#### `Cache` Trait
The `Cache` trait provides caching functionality:

```rust
#[async_trait]
pub trait Cache: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<CacheEntry>>;
    async fn set(&self, key: &str, value: CacheEntry, ttl: Duration) -> Result<()>;
    async fn invalidate(&self, key: &str) -> Result<()>;
    async fn clear(&self) -> Result<()>;
    async fn stats(&self) -> Result<CacheStats>;
}
```

## Core Types

### Configuration
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub provider: ProviderConfig,
    pub cache: CacheConfig,
    pub monitoring: MonitoringConfig,
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub provider_type: ProviderType,
    pub model: String,
    pub api_key: Option<String>,
    pub endpoint: Option<String>,
    pub timeout: Duration,
}
```

### Chat Types
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<Message>,
    pub model: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stop_sequences: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub message: Message,
    pub usage: TokenUsage,
    pub model: String,
    pub finish_reason: FinishReason,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}
```

### Performance Types
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub response_time: Duration,
    pub token_usage: TokenUsage,
    pub cache_hit_rate: f64,
    pub error_rate: f64,
    pub throughput: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub hit_rate: f64,
    pub miss_rate: f64,
    pub size: usize,
    pub max_size: usize,
    pub evictions: u64,
}
```

## Main API Interface

### TrustAI Client
The main entry point for the API:

```rust
pub struct TrustAI {
    provider: Box<dyn Provider>,
    cache: Box<dyn Cache>,
    monitor: PerformanceMonitor,
    config: Config,
}

impl TrustAI {
    /// Create a new TrustAI client with default configuration
    pub fn new() -> Result<Self>;
    
    /// Create a new TrustAI client with custom configuration
    pub fn with_config(config: Config) -> Result<Self>;
    
    /// Send a chat request
    pub async fn chat(&self, request: ChatRequest) -> Result<ChatResponse>;
    
    /// Get available models
    pub async fn models(&self) -> Result<Vec<Model>>;
    
    /// Get performance metrics
    pub async fn performance_metrics(&self) -> Result<PerformanceMetrics>;
    
    /// Get cache statistics
    pub async fn cache_stats(&self) -> Result<CacheStats>;
    
    /// Clear cache
    pub async fn clear_cache(&self) -> Result<()>;
    
    /// Health check
    pub async fn health_check(&self) -> Result<HealthStatus>;
}
```

## Provider Implementations

### OpenAI Provider
```rust
use trust_ai::providers::OpenAIProvider;

let provider = OpenAIProvider::new("sk-your-api-key")?;
let client = TrustAI::with_provider(provider)?;
```

### Ollama Provider
```rust
use trust_ai::providers::OllamaProvider;

let provider = OllamaProvider::new("http://localhost:11434")?;
let client = TrustAI::with_provider(provider)?;
```

### Anthropic Provider
```rust
use trust_ai::providers::AnthropicProvider;

let provider = AnthropicProvider::new("sk-ant-your-key")?;
let client = TrustAI::with_provider(provider)?;
```

## Error Handling

### Error Types
```rust
#[derive(Debug, thiserror::Error)]
pub enum TrustAIError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Provider error: {0}")]
    Provider(#[from] ProviderError),
    
    #[error("Cache error: {0}")]
    Cache(#[from] CacheError),
    
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Rate limit exceeded")]
    RateLimit,
    
    #[error("Authentication failed")]
    Authentication,
}

pub type Result<T> = std::result::Result<T, TrustAIError>;
```

## Performance Monitoring

### Performance Monitor
```rust
use trust_ai::performance::PerformanceMonitor;

let monitor = PerformanceMonitor::new();

// Record metrics
monitor.record_request_start();
let response = client.chat(request).await?;
monitor.record_request_end(response.usage.clone());

// Get metrics
let metrics = monitor.get_metrics().await?;
println!("Average response time: {:?}", metrics.avg_response_time);
```

### Optimization Engine
```rust
use trust_ai::performance::OptimizationEngine;

let engine = OptimizationEngine::new();
let suggestions = engine.analyze(&metrics).await?;

for suggestion in suggestions {
    println!("Optimization: {}", suggestion.description);
    if suggestion.auto_apply {
        engine.apply_optimization(suggestion).await?;
    }
}
```

## Caching

### LRU Cache Implementation
```rust
use trust_ai::cache::LruCache;

let cache = LruCache::new(1000, Duration::from_secs(3600))?;
let client = TrustAI::with_cache(cache)?;

// Cache is automatically used for chat requests
let response = client.chat(request).await?;
```

### Custom Cache Implementation
```rust
use trust_ai::cache::Cache;

#[derive(Debug)]
pub struct CustomCache {
    // Your implementation
}

#[async_trait]
impl Cache for CustomCache {
    async fn get(&self, key: &str) -> Result<Option<CacheEntry>> {
        // Your implementation
    }
    
    async fn set(&self, key: &str, value: CacheEntry, ttl: Duration) -> Result<()> {
        // Your implementation
    }
    
    // ... other methods
}
```

## Advanced Features

### Conversation Management
```rust
use trust_ai::conversation::{Conversation, ConversationManager};

let mut conversation = Conversation::new();
conversation.add_message(Message::user("Hello!"));

let response = client.chat_with_conversation(&mut conversation).await?;
conversation.add_message(response.message);

// Save conversation
let manager = ConversationManager::new();
manager.save("my-chat", &conversation).await?;

// Load conversation
let loaded = manager.load("my-chat").await?;
```

### Streaming Responses
```rust
use futures::StreamExt;

let stream = client.chat_stream(request).await?;
tokio::pin!(stream);

while let Some(chunk) = stream.next().await {
    match chunk? {
        ChatChunk::Content(text) => print!("{}", text),
        ChatChunk::Done(response) => {
            println!("\nResponse complete: {:?}", response.usage);
            break;
        }
    }
}
```

### Batch Processing
```rust
use trust_ai::batch::BatchProcessor;

let processor = BatchProcessor::new(client);
let requests = vec![request1, request2, request3];

let responses = processor
    .process_batch(requests)
    .concurrency(3)
    .timeout(Duration::from_secs(30))
    .await?;

for (i, response) in responses.iter().enumerate() {
    println!("Response {}: {:?}", i, response);
}
```

## Testing Support

### Mock Provider
```rust
use trust_ai::testing::MockProvider;

let mut mock = MockProvider::new();
mock.expect_chat()
    .returning(|_| Ok(ChatResponse {
        message: Message::assistant("Mocked response"),
        usage: TokenUsage::default(),
        model: "mock-model".to_string(),
        finish_reason: FinishReason::Stop,
    }));

let client = TrustAI::with_provider(mock)?;
```

### Test Utilities
```rust
use trust_ai::testing::{TestConfig, TestData};

#[tokio::test]
async fn test_chat_functionality() {
    let config = TestConfig::default();
    let client = TrustAI::with_config(config.into())?;
    
    let request = TestData::sample_chat_request();
    let response = client.chat(request).await?;
    
    assert_eq!(response.message.role, Role::Assistant);
    assert!(!response.message.content.is_empty());
}
```

## Configuration Builder

### Fluent Configuration API
```rust
use trust_ai::config::ConfigBuilder;

let config = ConfigBuilder::new()
    .provider(ProviderType::OpenAI)
    .model("gpt-4")
    .api_key("sk-your-key")
    .temperature(0.7)
    .max_tokens(2048)
    .cache_enabled(true)
    .cache_max_size(1000)
    .cache_ttl(Duration::from_secs(3600))
    .monitoring_enabled(true)
    .build()?;

let client = TrustAI::with_config(config)?;
```

## Integration Examples

See the [examples directory](examples/) for complete integration examples:

- [Basic Chat](examples/basic_chat.md)
- [Performance Monitoring](examples/performance_monitoring.md)
- [Custom Provider](examples/custom_provider.md)
- [Batch Processing](examples/batch_processing.md)
- [Web Service Integration](examples/web_service.md)

## API Reference

For complete API documentation, run:

```bash
cargo doc --open
```

This will generate and open the full API documentation in your browser.