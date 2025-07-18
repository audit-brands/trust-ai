# Trust AI Ollama Model Detection Analysis

## Objective
Analyze Trust AI's current Ollama model detection mechanism and provide recommendations for automatic detection of newly downloaded models in interactive mode.

## Current Implementation Analysis

### 1. Model Discovery Architecture

Trust AI implements a sophisticated model discovery system with the following components:

**ModelDiscoveryService** (`crates/forge_provider/src/discovery.rs:244-320`)
- Automatically discovers Ollama installations on common ports (11434, 11435, 11436)
- Performs health checks to verify service availability
- Creates automatic configurations if no explicit Ollama setup exists
- Supports both default and custom Ollama instances

**Provider Service Integration** (`crates/forge_services/src/provider.rs:98-140`)
- Implements two-level caching: cloud models and local models
- Lazy initialization of ModelDiscoveryService
- Combines cloud and local models into unified list
- Cache invalidation only on explicit refresh

### 2. Interactive Mode Integration

**Model Commands** (`crates/forge_main/src/model.rs:183-206`)
- `/model list` - Shows all available models
- `/model refresh` - Forces discovery refresh and cache invalidation
- `/model discover` - Triggers comprehensive model discovery
- `/model status` - Shows current model information

**UI Implementation** (`crates/forge_main/src/ui.rs:567-888`)
- Models fetched via `self.api.models().await?`
- Spinner feedback during model loading
- Provider-specific categorization in refresh output
- Manual refresh requirement for new model detection

### 3. Current Behavior Summary

**Automatic Detection**: ✅ **YES** - Trust AI automatically detects Ollama models, but with limitations:

1. **Initial Discovery**: Automatically discovers Ollama on startup if not configured
2. **Caching**: Models are cached after first discovery for performance
3. **Manual Refresh**: New models require `/model refresh` command to be detected
4. **Health Monitoring**: Continuous health checks but no background model scanning

## Implementation Plan

### 1. **Enhance Cache Invalidation Strategy**
   - Dependencies: None
   - Notes: Implement time-based cache expiration for local models
   - Files: `crates/forge_services/src/provider.rs`
   - Status: Not Started

### 2. **Add Background Model Monitoring**
   - Dependencies: Task 1
   - Notes: Optional background service to detect new models
   - Files: `crates/forge_provider/src/discovery.rs`
   - Status: Not Started

### 3. **Improve Discovery Feedback**
   - Dependencies: Task 1
   - Notes: Better user notifications for model changes
   - Files: `crates/forge_main/src/ui.rs`
   - Status: Not Started

### 4. **Configuration Options**
   - Dependencies: Task 2
   - Notes: Allow users to control automatic detection behavior
   - Files: Configuration files and app config
   - Status: Not Started

### 5. **Performance Optimization**
   - Dependencies: Task 2, 3
   - Notes: Optimize discovery frequency and resource usage
   - Files: `crates/forge_provider/src/discovery.rs`
   - Status: Not Started

### 6. **Testing and Validation**
   - Dependencies: Task 1, 2, 3, 4, 5
   - Notes: Comprehensive testing of new detection mechanisms
   - Files: Test files across all modified crates
   - Status: Not Started

## Verification Criteria

- New Ollama models are detected within configurable time window
- Interactive mode shows updated model list without manual refresh
- Performance impact is minimal (< 100ms additional latency)
- User can control automatic detection behavior
- Cache invalidation works correctly across all scenarios
- Background monitoring can be disabled for resource-constrained environments

## Potential Risks and Mitigations

1. **Performance Impact from Background Monitoring**
   Mitigation: Implement configurable polling intervals and efficient change detection

2. **Cache Inconsistency During Concurrent Access**
   Mitigation: Use proper locking mechanisms and atomic cache updates

3. **Resource Usage from Continuous Monitoring**
   Mitigation: Implement smart polling that adapts to usage patterns

4. **Network Overhead from Frequent Health Checks**
   Mitigation: Optimize health check frequency and use lightweight ping methods

5. **User Experience Disruption from Automatic Updates**
   Mitigation: Provide configuration options and non-intrusive notifications

## Alternative Approaches

1. **File System Watching**: Monitor Ollama's model directory for changes instead of API polling
2. **Event-Driven Updates**: Integrate with Ollama's event system if available
3. **Hybrid Approach**: Combine time-based and event-driven detection methods
4. **User-Controlled Intervals**: Allow users to set custom refresh intervals
5. **Smart Caching**: Implement cache with TTL based on usage patterns

## Current Answer to Your Question

**Does Trust AI automatically detect new Ollama models?**

**Partially YES** - Trust AI has automatic Ollama detection capabilities, but with important limitations:

✅ **What Works**:
- Automatically discovers Ollama installations on startup
- Detects all models available at discovery time
- Provides manual refresh via `/model refresh` command
- Caches models for performance

❌ **What Doesn't Work Automatically**:
- New models downloaded after initial discovery are not detected until manual refresh
- No background monitoring for model changes
- Cache doesn't expire automatically to check for new models

**Recommendation**: Use `/model refresh` command in interactive mode whenever you download new Ollama models to update the available model list.

## Enhancement Recommendations

1. **Short-term**: Implement time-based cache expiration (5-10 minutes) for local models
2. **Medium-term**: Add optional background monitoring with configurable intervals
3. **Long-term**: Implement file system watching for real-time model detection
4. **User Experience**: Add notifications when new models are detected automatically