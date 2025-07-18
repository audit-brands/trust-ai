# Code Generation Testing Documentation

This document describes the testing methodology for validating code generation capabilities of the trust-ai CLI tool.

## Overview

The code generation tests validate that the trust-ai CLI can effectively generate, modify, and debug code using its available tools. These tests are designed to be run in an environment where trust-ai has access to a working AI model and the necessary file operation tools.

## Test Structure

### Test Files Created

1. **`tests/test_simple_code_generation.rs`** - Core code generation functionality tests
2. **`tests/test_code_generation_tools.rs`** - Comprehensive tool usage tests

### Test Categories

#### 1. Basic File Creation Tests
- **Purpose**: Verify that the AI can create new files with correct content
- **Tools Tested**: `forge_tool_fs_create`
- **Expected Outcomes**: 
  - Files are created in the correct location
  - Files contain syntactically correct code
  - Files include requested functionality

#### 2. File Modification Tests  
- **Purpose**: Verify that the AI can read and modify existing files
- **Tools Tested**: `forge_tool_fs_read`, `forge_tool_fs_patch`
- **Expected Outcomes**:
  - Existing files are read correctly
  - Modifications are applied precisely 
  - Original content is preserved where appropriate

#### 3. Shell Command Execution Tests
- **Purpose**: Verify that the AI can use shell commands for compilation and testing
- **Tools Tested**: `forge_tool_process_shell`
- **Expected Outcomes**:
  - Code compilation commands are executed
  - Test execution commands work correctly
  - Results are interpreted accurately

#### 4. Search and Refactor Tests
- **Purpose**: Verify that the AI can search for patterns and perform systematic refactoring
- **Tools Tested**: `forge_tool_fs_search`, `forge_tool_fs_patch`
- **Expected Outcomes**:
  - Search patterns find relevant code
  - Refactoring changes are applied consistently
  - Code structure is maintained

#### 5. Debug and Fix Tests
- **Purpose**: Verify that the AI can identify and fix problems in code
- **Tools Tested**: All tools in debugging context
- **Expected Outcomes**:
  - Failing tests are identified
  - Root causes are diagnosed correctly
  - Fixes resolve the issues

## Running the Tests

### Prerequisites

1. **Environment Setup**:
   ```bash
   export RUN_CODE_GENERATION_TESTS=1
   ```

2. **Dependencies**: Ensure all workspace dependencies are available:
   - `forge_api` - API interface for AI interactions
   - `forge_domain` - Domain models and types
   - `forge_tracker` - Logging and tracking
   - `tokio` - Async runtime
   - `pretty_assertions` - Enhanced assertion output

### Test Execution

#### Unit Tests (Always Run)
```bash
cargo test --test test_simple_code_generation unit_tests
```

#### Integration Tests (Conditional)
```bash
# Enable code generation tests
export RUN_CODE_GENERATION_TESTS=1

# Run basic tests
cargo test --test test_simple_code_generation conditional_tests

# Run comprehensive tests
cargo test --test test_code_generation_tools
```

### Test Configuration

Tests are configured to:
- Create isolated test directories under `target/test_*`
- Clean up previous test runs automatically
- Use the anthropic/claude-3.5-sonnet model by default
- Include comprehensive error reporting

## Expected Test Behaviors

### File Creation Test
```rust
// The AI should create a file like this:
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

fn main() {
    println!("{}", greet("World"));
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_greet() {
        assert_eq!(greet("Alice"), "Hello, Alice!");
    }
}
```

### File Modification Test
The AI should:
1. Read the existing file content
2. Identify the appropriate insertion point
3. Add new functionality without breaking existing code
4. Update tests to cover new functionality

### Shell Command Test
The AI should:
1. Create a simple Rust program
2. Use `rustc` to compile it
3. Execute the compiled program
4. Report success/failure status

## Test Validation Criteria

### Compilation Success
- All generated code must compile without errors
- Dependencies must be correctly specified
- Module structure must be valid

### Test Coverage
- Generated code must include appropriate tests
- Tests must use `pretty_assertions` for better error output
- Test coverage should be comprehensive

### Code Quality
- Generated code should follow Rust idioms
- Error handling should be appropriate
- Documentation should be included where relevant

### Tool Usage Effectiveness
- Tools should be used in the correct sequence
- Error conditions should be handled gracefully
- Results should be verified programmatically

## Troubleshooting

### Common Issues

1. **Tests Skipped**: Ensure `RUN_CODE_GENERATION_TESTS=1` is set
2. **API Errors**: Verify model availability and authentication
3. **File System Errors**: Check permissions in target directory
4. **Compilation Errors**: Ensure Rust toolchain is available

### Debug Mode

To debug test failures:
```bash
# Enable verbose logging
RUST_LOG=debug cargo test --test test_simple_code_generation

# Run single test with output
cargo test --test test_simple_code_generation test_basic_file_creation -- --nocapture
```

## Extending the Tests

### Adding New Test Cases

1. Create test fixture with descriptive name
2. Define clear task requirements
3. Execute task and capture response
4. Validate specific outcomes
5. Include both positive and negative test cases

### Test Categories to Consider

- **Multi-file Projects**: Creating projects with multiple interconnected files
- **Error Handling**: Testing how AI handles and recovates from errors
- **Performance**: Testing with larger codebases and complex refactoring
- **Language-specific**: Testing language-specific patterns and idioms
- **Integration**: Testing with external dependencies and complex build setups

## Best Practices

1. **Isolation**: Each test should be independent and not affect others
2. **Cleanup**: Test directories should be cleaned up automatically
3. **Assertions**: Use specific assertions rather than general "success" checks
4. **Documentation**: Document expected behaviors clearly
5. **Timeouts**: Include reasonable timeouts for AI operations
6. **Retries**: Consider retry mechanisms for transient failures

## Metrics and Success Criteria

### Success Metrics
- **Compilation Rate**: Percentage of generated code that compiles
- **Test Pass Rate**: Percentage of generated tests that pass
- **Task Completion Rate**: Percentage of tasks completed successfully
- **Code Quality Score**: Subjective assessment of generated code quality

### Benchmark Tasks
- Create a simple calculator library
- Refactor a multi-module project
- Debug and fix failing tests
- Add new feature to existing codebase

These tests provide a comprehensive framework for validating the code generation capabilities of the trust-ai CLI tool and ensure that it can effectively assist with real-world software development tasks.