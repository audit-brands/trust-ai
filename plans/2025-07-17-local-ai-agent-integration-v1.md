# Local AI Model Selection Agent Integration

## Objective
Fix the "Agent not found in the arena: trust" error that occurs when selecting local AI models (qwen2.5:1.5b, mistral:latest) discovered through Ollama integration. The local model discovery is working correctly, but model selection fails due to missing agent configuration for the default "trust" agent.

## Implementation Plan

1. **Analyze Agent Configuration Architecture**
   - Dependencies: None
   - Notes: Understand how agents are defined and loaded from workflow configuration
   - Files: `forge.default.yaml`, `crates/forge_domain/src/agent.rs`, `crates/forge_domain/src/workflow.rs`
   - Status: Not Started

2. **Identify Agent-Model Relationship**
   - Dependencies: Task 1
   - Notes: Determine how model selection triggers agent lookup and why "trust" is expected
   - Files: `crates/forge_main/src/ui.rs:654`, `crates/forge_domain/src/conversation.rs:202`
   - Status: Not Started

3. **Define Trust Agent Configuration**
   - Dependencies: Task 1, 2
   - Notes: Create appropriate agent definition for the missing "trust" agent
   - Files: `forge.default.yaml`
   - Status: Not Started

4. **Implement Agent Definition**
   - Dependencies: Task 3
   - Notes: Add trust agent to default workflow with proper model, tools, and prompt configuration
   - Files: `forge.default.yaml`
   - Status: Not Started

5. **Test Local Model Selection**
   - Dependencies: Task 4
   - Notes: Verify that local AI models can be selected without agent lookup errors
   - Files: All modified components
   - Status: Not Started

6. **Validate End-to-End Chat Functionality**
   - Dependencies: Task 5
   - Notes: Ensure selected local models work for actual conversations
   - Files: Conversation and orchestration components
   - Status: Not Started

## Verification Criteria
- Local AI models (qwen2.5:1.5b, mistral:latest) appear in `/model list` output
- Model selection via `/model select qwen2.5:1.5b` completes without "Agent not found" error
- Selected local model shows as active in conversation
- Chat functionality works with selected local AI model
- No regression in existing cloud model selection functionality

## Potential Risks and Mitigations

1. **Agent Configuration Conflicts**
   Mitigation: Ensure trust agent configuration is compatible with existing forge/muse agents and doesn't conflict with their tool definitions or system prompts

2. **Model-Agent Capability Mismatch**
   Mitigation: Configure trust agent with appropriate tools and capabilities that work well with local AI models' limitations compared to cloud models

3. **Default Agent Selection Logic**
   Mitigation: Verify that adding trust agent doesn't disrupt existing agent selection logic or default behaviors in conversation initialization

## Alternative Approaches

1. **Modify Default Agent**: Change the default agent from "trust" to "forge" throughout the codebase to use existing agent definitions
2. **Dynamic Agent Creation**: Create agents dynamically based on selected model type rather than requiring pre-defined agent configurations
3. **Agent-Model Decoupling**: Redesign the architecture to separate model selection from agent configuration, allowing any model to work with any agent