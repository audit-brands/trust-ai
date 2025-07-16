# Phase 2 Technical Specifications: Rust-Native Local AI Architecture

## Overview
This document provides detailed technical specifications for integrating local AI capabilities into the trust-ai Rust workspace, focusing on Ollama HTTP integration and future HuggingFace native support.

## Architecture Design

### 1. Provider Abstraction Extension

#### Current Provider Enum (forge_domain/src/model.rs)
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Provider {
    OpenAi,
    Anthropic,
    OpenRouter,
}
```

#### Proposed Extension
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Provider {
    OpenAi,
    Anthropic,
    OpenRouter,
    Ollama,
    HuggingFace,
}
```

### 2. Ollama Integration Architecture

#### HTTP Client Implementation
- **Location**: `crates/forge_app/src/providers/ollama/`
- **Approach**: Leverage existing `reqwest` HTTP client infrastructure
- **API Compatibility**: Use Ollama's OpenAI-compatible endpoint (`/v1/chat/completions`)

#### Key Components:

##### OllamaClient Structure
```rust
pub struct OllamaClient {
    base_url: String,
    client: reqwest::Client,
    timeout: Duration,
}

impl OllamaClient {
    pub fn new(base_url: String) -> Self { ... }
    pub async fn list_models(&self) -> Result<Vec<OllamaModel>> { ... }
    pub async fn generate_completion(&self, request: ChatRequest) -> Result<ChatResponse> { ... }
    pub async fn health_check(&self) -> Result<bool> { ... }
}
```

##### Configuration Integration
```rust
// In forge_app/src/app_config.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub base_url: String,           // Default: "http://localhost:11434"
    pub timeout_seconds: u64,       // Default: 30
    pub preferred_models: Vec<String>, // User's preferred model order
    pub auto_discovery: bool,       // Auto-detect available models
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    // ... existing fields
    pub ollama: Option<OllamaConfig>,
}
```

### 3. HuggingFace Integration Strategy

#### Research Findings
1. **candle-core**: Pure Rust ML framework
   - Pros: No external dependencies, HuggingFace model compatibility
   - Cons: Limited model support, performance considerations
   - Use Case: Lightweight local inference

2. **llama-cpp-rs**: Rust bindings for llama.cpp
   - Pros: Excellent performance, broad model support
   - Cons: C++ dependency, compilation complexity
   - Use Case: High-performance local inference

3. **ort**: ONNX Runtime bindings
   - Pros: Broad model format support, optimized runtime
   - Cons: Additional runtime dependency
   - Use Case: Production-ready inference

#### Recommended Implementation Phases
1. **Phase 1**: Ollama HTTP integration (immediate)
2. **Phase 2**: candle-core proof-of-concept (research)
3. **Phase 3**: llama-cpp-rs evaluation (performance)
4. **Phase 4**: Production-ready native integration

### 4. Provider Selection and Fallback Logic

#### Intelligent Provider Selection
```rust
#[derive(Debug, Clone)]
pub struct ProviderSelector {
    preference_order: Vec<Provider>,
    fallback_enabled: bool,
    health_check_timeout: Duration,
}

impl ProviderSelector {
    pub async fn select_provider(&self, model_requirements: &ModelRequirements) -> Result<Provider> {
        // 1. Check user preferences
        // 2. Verify provider availability
        // 3. Match model capabilities
        // 4. Fallback to cloud providers if needed
    }
}
```

#### Fallback Strategy
1. **Local First**: Try Ollama/HuggingFace models first
2. **Capability Matching**: Ensure selected provider supports required features
3. **Health Checking**: Verify provider availability before selection
4. **Graceful Degradation**: Fall back to cloud providers if local unavailable

### 5. Model Discovery and Management

#### Ollama Model Discovery
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModel {
    pub name: String,
    pub size: u64,
    pub modified_at: String,
    pub digest: String,
    pub details: OllamaModelDetails,
}

pub struct ModelDiscovery {
    ollama_client: OllamaClient,
    cache_duration: Duration,
}

impl ModelDiscovery {
    pub async fn discover_available_models(&self) -> Result<Vec<LocalModel>> { ... }
    pub async fn check_model_health(&self, model_name: &str) -> Result<ModelHealth> { ... }
}
```

### 6. Configuration System Integration

#### YAML Configuration Extension
```yaml
# forge.yaml
providers:
  local:
    ollama:
      base_url: "http://localhost:11434"
      timeout_seconds: 30
      preferred_models:
        - "llama3.1:8b"
        - "codellama:7b"
        - "mistral:7b"
      auto_discovery: true
      
    huggingface:
      cache_dir: "~/.cache/huggingface"
      device: "auto"  # auto, cpu, cuda
      models:
        - name: "microsoft/DialoGPT-medium"
          local_path: null
          
  fallback:
    enabled: true
    order: ["ollama", "huggingface", "openai", "anthropic"]
    
model_preferences:
  default_provider: "ollama"
  local_first: true
```

### 7. Error Handling and Resilience

#### Error Types
```rust
#[derive(Debug, thiserror::Error)]
pub enum LocalAIError {
    #[error("Ollama service unavailable: {0}")]
    OllamaUnavailable(String),
    
    #[error("Model not found: {model_name}")]
    ModelNotFound { model_name: String },
    
    #[error("Local AI timeout after {timeout_seconds}s")]
    Timeout { timeout_seconds: u64 },
    
    #[error("Provider fallback failed: {reason}")]
    FallbackFailed { reason: String },
}
```

#### Resilience Patterns
1. **Circuit Breaker**: Temporarily disable failing providers
2. **Retry Logic**: Configurable retry attempts with exponential backoff
3. **Health Monitoring**: Periodic health checks for local services
4. **Graceful Degradation**: Automatic fallback to available providers

### 8. Performance Considerations

#### Optimization Strategies
1. **Connection Pooling**: Reuse HTTP connections to Ollama
2. **Response Streaming**: Support streaming responses for better UX
3. **Model Preloading**: Keep frequently used models warm
4. **Caching**: Cache model metadata and health status

#### Monitoring Points
1. **Response Times**: Track local vs cloud provider performance
2. **Success Rates**: Monitor provider reliability
3. **Resource Usage**: Track memory and CPU usage for local models
4. **User Experience**: Measure perceived performance

### 9. Testing Strategy

#### Unit Tests
- Provider enum extensions
- Configuration parsing
- Error handling scenarios
- Model discovery logic

#### Integration Tests
- Ollama HTTP client communication
- Provider fallback logic
- Configuration loading
- Health checking

#### Mock Services
- Mock Ollama server for testing
- Simulated network failures
- Model availability scenarios

### 10. Migration and Compatibility

#### Backward Compatibility
- Existing provider configurations remain unchanged
- New local AI features are opt-in
- Graceful handling of missing local services

#### Migration Path
1. **Phase 1**: Add local AI configuration options
2. **Phase 2**: Implement Ollama integration
3. **Phase 3**: Add provider selection logic
4. **Phase 4**: Enable local-first by default

## Implementation Timeline

### Week 1: Foundation
- Extend Provider enum
- Add Ollama configuration structure
- Create basic HTTP client

### Week 2: Core Integration
- Implement Ollama HTTP client
- Add model discovery
- Basic error handling

### Week 3: Provider Selection
- Implement fallback logic
- Add health checking
- Configuration integration

### Week 4: Testing and Polish
- Comprehensive testing
- Performance optimization
- Documentation

## Success Criteria

1. **Functional**: Ollama integration works end-to-end
2. **Reliable**: Graceful fallback to cloud providers
3. **Performant**: Local AI response times competitive with cloud
4. **Maintainable**: Clean architecture supporting future extensions
5. **User-Friendly**: Intuitive configuration and management

## Risk Mitigation

### Technical Risks
1. **Ollama Availability**: Implement robust health checking
2. **Performance**: Benchmark and optimize critical paths
3. **Compatibility**: Maintain backward compatibility throughout

### User Experience Risks
1. **Configuration Complexity**: Provide sensible defaults
2. **Setup Difficulty**: Create clear setup documentation
3. **Reliability**: Ensure fallback mechanisms work seamlessly

## Next Steps

1. Begin Phase 3: Extend Provider Domain Model
2. Create proof-of-concept Ollama client
3. Design configuration schema
4. Implement basic provider selection logic

---
*This specification serves as the technical foundation for local AI integration and will be updated as implementation progresses.*