# Model List Provider Distinction Enhancement

## Objective
Enhance the `/model list` command to clearly distinguish between private (local) and cloud-based models, improving user understanding of model availability and source.

## Implementation Plan

1. **Enhance Domain Model Structure**
   - Dependencies: None
   - Notes: Add optional provider type field to Model struct using enum for Local/Cloud categorization
   - Files: `crates/forge_domain/src/model.rs`
   - Status: Not Started

2. **Update Provider Service Model Retrieval**
   - Dependencies: Task 1
   - Notes: Modify models method to set provider type during model discovery and cloud model retrieval
   - Files: `crates/forge_services/src/provider.rs`
   - Status: Not Started

3. **Enhance Local Model Discovery**
   - Dependencies: Task 1
   - Notes: Ensure local model discovery properly sets provider type as Local
   - Files: `crates/forge_services/src/provider.rs` (discover_local_models method)
   - Status: Not Started

4. **Add Cloud Model Provider Detection**
   - Dependencies: Task 1
   - Notes: Implement logic to detect and set provider type as Cloud for models from provider clients
   - Files: `crates/forge_services/src/provider.rs` (models method)
   - Status: Not Started

5. **Update UI Display Logic**
   - Dependencies: Tasks 1-4
   - Notes: Enhance model list display to show provider type distinction with visual indicators
   - Files: `crates/forge_main/src/ui.rs` (on_model_list method)
   - Status: Not Started

6. **Update Model Discovery Display**
   - Dependencies: Task 5
   - Notes: Ensure model discovery command also shows enhanced provider information
   - Files: `crates/forge_main/src/ui.rs` (on_model_discover method)
   - Status: Not Started

7. **Add Comprehensive Tests**
   - Dependencies: Tasks 1-6
   - Notes: Create tests for provider type detection and display formatting
   - Files: Test files in relevant crates
   - Status: Not Started

8. **Compilation and Integration Verification**
   - Dependencies: Tasks 1-7
   - Notes: Verify all changes compile correctly and integrate properly
   - Files: All modified files
   - Status: Not Started

## Verification Criteria
- `/model list` command displays clear distinction between local and cloud models
- Provider type information is accurately detected and stored
- Display format is consistent and user-friendly
- All existing functionality remains intact
- Comprehensive test coverage for new features
- No compilation errors or warnings

## Potential Risks and Mitigations

1. **Breaking Changes to Domain Model**
   Mitigation: Use optional fields and derive_setters to maintain backward compatibility

2. **Performance Impact from Additional Provider Detection**
   Mitigation: Leverage existing caching mechanisms and minimize additional processing overhead

3. **Inconsistent Provider Information**
   Mitigation: Implement fallback logic to handle cases where provider type cannot be determined

4. **UI Display Complexity**
   Mitigation: Use simple, clear visual indicators and maintain existing display structure

## Alternative Approaches

1. **Display-Only Enhancement**: Modify only UI layer to infer provider type from model ID patterns without changing domain model
2. **Full Provider Metadata**: Add comprehensive provider information including specific provider names, versions, and capabilities
3. **Configuration-Based Categorization**: Allow users to configure custom model categorization rules