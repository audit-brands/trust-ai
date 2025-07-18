# Ollama Model Detection Fix

## Objective
Investigate and fix the issue where trust-ai cannot detect successfully downloaded Ollama models, preventing local AI testing implementation. The user has confirmed that `qwen2.5-coder:7b` was downloaded successfully via `ollama pull` but trust-ai's `/model discover` shows only cloud models and `/model select ollama/qwen2.5-coder:7b` fails with "Model not found" error.

## Implementation Plan

1. **Verify Ollama Service Status and Connectivity**
   - Dependencies: None
   - Notes: ✅ CONFIRMED - Ollama service is running and has 3 models: qwen2.5-coder:7b, qwen2.5:1.5b, mistral:latest
   - Files: API response shows models are available at `http://localhost:11434/api/tags`
   - Status: Completed

2. **Examine Local AI Configuration Structure** [DELEGATE TO TRUST AGENT]
   - Dependencies: Task 1
   - Notes: ✅ COMPLETED - Configuration analysis shows no local AI config in forge.yaml files, but default LocalAiConfig::with_default_ollama() should work
   - Files: `forge.yaml`, `forge.default.yaml`, `crates/forge_provider/src/config/local_ai.rs:1`
   - Status: Completed

3. **Trace Model Discovery Service Integration** [DELEGATE TO TRUST AGENT]
   - Dependencies: Task 2
   - Notes: ✅ COMPLETED - Traced complete flow: UI -> API -> ProviderService -> ModelDiscoveryService. Issue likely in discovery service initialization or health check
   - Files: `crates/forge_main/src/ui.rs:701`, `crates/forge_api/src/forge_api.rs:49`, `crates/forge_services/src/provider.rs:98`
   - Status: Completed

4. **Investigate Discovery Service Ollama Implementation**
   - Dependencies: Task 3
   - Notes: Check automatic Ollama discovery and model fetching logic
   - Files: `crates/forge_provider/src/discovery.rs:206`, `crates/forge_provider/src/discovery.rs:241`
   - Status: Not Started

5. **Verify Health Monitoring and Provider Detection**
   - Dependencies: Task 4
   - Notes: Ensure health monitor properly detects and validates Ollama service
   - Files: `crates/forge_provider/src/ollama/config.rs:1`, `crates/forge_provider/src/health/mod.rs:1`
   - Status: Not Started

6. **Check Provider Registry Local Provider Integration**
   - Dependencies: Task 5
   - Notes: Verify provider registry includes local providers in model listing
   - Files: `crates/forge_services/src/provider_registry.rs:55`, `crates/forge_services/src/provider.rs:175`
   - Status: Not Started

7. **Test Ollama Provider Direct Model Fetching**
   - Dependencies: Task 6
   - Notes: Verify Ollama provider can directly fetch models from service
   - Files: `crates/forge_provider/src/ollama/provider.rs:137`, `crates/forge_provider/src/client.rs:100`
   - Status: Not Started

8. **Implement Configuration or Code Fix**
   - Dependencies: Tasks 1-7
   - Notes: Based on root cause analysis, implement necessary fixes
   - Files: Determined by investigation findings
   - Status: Not Started

9. **Test and Validate Solution**
   - Dependencies: Task 8
   - Notes: Verify fix works with real Ollama installation and model selection
   - Files: Integration tests, manual CLI testing
   - Status: Not Started

## Verification Criteria
- `/model discover` command shows both cloud and local Ollama models
- Downloaded Ollama model `qwen2.5-coder:7b` appears in model list
- `/model select ollama/qwen2.5-coder:7b` command succeeds
- Local AI integration works end-to-end for chat functionality
- No regression in cloud provider model discovery

## Potential Risks and Mitigations

1. **Ollama Service Connection Failure**
   Mitigation: Verify service is running on expected port and implement proper error handling with user-friendly messages

2. **Configuration Mismatch Between Trust-ai and Ollama**
   Mitigation: Check default configuration values and ensure they match standard Ollama installation

3. **Health Check Timeout or Failure**
   Mitigation: Investigate health check implementation and adjust timeouts or retry logic if needed

4. **Model Discovery Service Not Properly Initialized**
   Mitigation: Trace service initialization in provider service and ensure discovery service is created and started

5. **Provider Registry Missing Local Provider Integration**
   Mitigation: Verify local models are properly merged with cloud models in the final model list

## Alternative Approaches

1. **Configuration-First Approach**: Focus on enabling local AI in user configuration before investigating code issues
2. **Direct Provider Testing**: Test Ollama provider independently before investigating service integration
3. **Health Check Bypass**: Temporarily bypass health checks to isolate discovery issues from connectivity issues

## Summary

With the Ollama service confirmed working and API response showing 3 available models (including the target `qwen2.5-coder:7b`), the issue is clearly in trust-ai's integration layer. Tasks 2 and 3 have been delegated to the trust agent to investigate configuration and service integration gaps. The remaining tasks focus on implementing the fixes once the root cause is identified.

**Key Finding**: Ollama service is fully functional - the problem lies in trust-ai's discovery service integration, not the Ollama provider implementation.

## Investigation Results (Tasks 2 & 3)

**Configuration Analysis (Task 2):**
- ✅ No local AI configuration found in forge.yaml or forge.default.yaml
- ✅ AppConfig defaults to empty, which triggers LocalAiConfig::with_default_ollama()
- ✅ Default configuration should work with localhost:11434 endpoint
- ✅ LocalAiConfig structure is comprehensive and properly implemented

**Service Integration Tracing (Task 3):**
- ✅ UI `/model discover` calls `get_models()` which calls `api.models()`
- ✅ API layer calls `services.models(provider, app_config)`
- ✅ ProviderService calls `discover_local_models(app_config)`
- ✅ Discovery flow: `ensure_local_discovery()` -> `ModelDiscoveryService::new()` -> `HealthMonitor::new()`
- ⚠️  Potential issue: ModelDiscoveryService initialization may be failing silently
- ⚠️  Warning logged but operation continues: "Failed to initialize local AI discovery service"

**Root Cause Hypothesis:**
The ModelDiscoveryService or HealthMonitor initialization is failing during the `ensure_local_discovery()` call, causing the discovery service to remain `None`. This results in no local models being discovered despite Ollama being fully functional.