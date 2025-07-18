use std::path::PathBuf;
use std::time::Duration;

use anyhow::Context;
use forge_api::{ForgeAPI, API};
use forge_domain::{Agent, AgentId, ChatRequest, ChatResponse, Event, EventContext, ModelId, SystemContext, Template, ToolName, Workflow};
use forge_tracker::Tracker;
use tokio_stream::StreamExt;
use tokio::fs;
use pretty_assertions::assert_eq;

/// Test fixture for code generation testing
struct CodeGenerationFixture {
    api: ForgeAPI,
    test_dir: PathBuf,
    _guard: forge_tracker::Guard,
}

lazy_static::lazy_static! {
    static ref tracker: Tracker = Tracker::default();
}

impl CodeGenerationFixture {
    /// Create a new test fixture
    async fn new() -> anyhow::Result<Self> {
        let test_dir = PathBuf::from("target/test_code_generation");
        
        // Clean up previous test directory
        if test_dir.exists() {
            fs::remove_dir_all(&test_dir).await?;
        }
        
        // Create test directory
        fs::create_dir_all(&test_dir).await?;
        
        let api = ForgeAPI::init(true);
        let _guard = forge_tracker::init_tracing(PathBuf::from("."), tracker.clone())?;
        
        Ok(Self {
            api,
            test_dir,
            _guard,
        })
    }

    /// Create a code generation agent
    fn create_code_agent(&self) -> Agent {
        let system_prompt = r#"
You are a skilled software engineer assistant. Your task is to write clean, well-tested Rust code.

Follow these guidelines:
1. Write idiomatic Rust code with proper error handling
2. Include comprehensive tests using pretty_assertions
3. Use appropriate data structures and patterns
4. Follow the project's coding standards
5. Ensure code compiles and tests pass

Available tools for file operations:
- forge_tool_fs_create: Create new files
- forge_tool_fs_read: Read existing files
- forge_tool_fs_patch: Modify existing files
- forge_tool_fs_search: Search for patterns in files
- forge_tool_process_shell: Run shell commands

First analyze the task, then implement the solution step by step.
"#;

        let user_prompt = r#"{{event.value}}"#;

        Agent::new(AgentId::new("code_generator"))
            .model(ModelId::new("anthropic/claude-3.5-sonnet"))
            .tool_supported(true)
            .tools(vec![
                ToolName::new("forge_tool_fs_create"),
                ToolName::new("forge_tool_fs_read"),
                ToolName::new("forge_tool_fs_patch"),
                ToolName::new("forge_tool_fs_search"),
                ToolName::new("forge_tool_process_shell"),
            ])
            .subscribe(vec!["code_generation_task".to_string()])
            .system_prompt(Template::<SystemContext>::new(system_prompt.trim()))
            .user_prompt(Template::<EventContext>::new(user_prompt.trim()))
    }

    /// Execute a code generation task
    async fn execute_task(&self, task: &str) -> anyhow::Result<String> {
        let workflow = Workflow::new().agents(vec![self.create_code_agent()]);
        
        let conversation_id = self.api.init_conversation(workflow).await?.id;
        let request = ChatRequest::new(
            Event::new("code_generation_task", Some(task)),
            conversation_id,
        );

        let response = self.api.chat(request)
            .await
            .with_context(|| "Failed to execute code generation task")?
            .filter_map(|message| match message.unwrap() {
                ChatResponse::Text { text, .. } => Some(text),
                _ => None,
            })
            .collect::<Vec<_>>()
            .await
            .join("")
            .trim()
            .to_string();

        Ok(response)
    }

    /// Test that a file was created and contains expected content
    async fn assert_file_contains(&self, path: &str, expected_content: &str) -> anyhow::Result<()> {
        let file_path = self.test_dir.join(path);
        let content = fs::read_to_string(&file_path).await
            .with_context(|| format!("Failed to read file: {}", file_path.display()))?;
        
        assert!(
            content.contains(expected_content),
            "File {} does not contain expected content: {}",
            path,
            expected_content
        );
        
        Ok(())
    }

    /// Test that code compiles successfully
    async fn assert_code_compiles(&self, cargo_toml_path: &str) -> anyhow::Result<()> {
        let cargo_dir = self.test_dir.join(cargo_toml_path).parent().unwrap().to_path_buf();
        
        let output = tokio::process::Command::new("cargo")
            .arg("check")
            .current_dir(&cargo_dir)
            .output()
            .await?;
        
        assert!(
            output.status.success(),
            "Code compilation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        
        Ok(())
    }

    /// Test that tests pass successfully
    async fn assert_tests_pass(&self, cargo_toml_path: &str) -> anyhow::Result<()> {
        let cargo_dir = self.test_dir.join(cargo_toml_path).parent().unwrap().to_path_buf();
        
        let output = tokio::process::Command::new("cargo")
            .arg("test")
            .current_dir(&cargo_dir)
            .output()
            .await?;
        
        assert!(
            output.status.success(),
            "Tests failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        
        Ok(())
    }
}

#[tokio::test]
async fn test_create_simple_rust_library() -> anyhow::Result<()> {
    let fixture = CodeGenerationFixture::new().await?;
    
    let task = format!(
        r#"Create a simple Rust library project in the directory `{}` with the following requirements:

1. Create a Cargo.toml file for a library called "math_utils"
2. Create src/lib.rs with a Calculator struct that has methods for basic arithmetic operations (add, subtract, multiply, divide)
3. Include proper error handling for division by zero
4. Write comprehensive tests for all methods
5. Use pretty_assertions in tests
6. Ensure the code compiles and tests pass

The library should be well-documented and follow Rust best practices."#,
        fixture.test_dir.display()
    );

    let _response = fixture.execute_task(&task).await?;

    // Verify the files were created
    fixture.assert_file_contains("Cargo.toml", "math_utils").await?;
    fixture.assert_file_contains("src/lib.rs", "struct Calculator").await?;
    fixture.assert_file_contains("src/lib.rs", "fn add").await?;
    fixture.assert_file_contains("src/lib.rs", "fn divide").await?;
    fixture.assert_file_contains("src/lib.rs", "pretty_assertions").await?;
    fixture.assert_file_contains("src/lib.rs", "#[cfg(test)]").await?;

    // Verify the code compiles
    fixture.assert_code_compiles("Cargo.toml").await?;

    // Verify tests pass
    fixture.assert_tests_pass("Cargo.toml").await?;

    Ok(())
}

#[tokio::test]
async fn test_modify_existing_code() -> anyhow::Result<()> {
    let fixture = CodeGenerationFixture::new().await?;
    
    // First, create initial code
    let initial_task = format!(
        r#"Create a basic Rust library in `{}` with:
1. Cargo.toml for "counter_lib"
2. src/lib.rs with a Counter struct that has increment() and get_value() methods
3. Basic tests

Keep it simple for now."#,
        fixture.test_dir.display()
    );

    let _initial_response = fixture.execute_task(&initial_task).await?;

    // Then modify the code to add new functionality
    let modification_task = format!(
        r#"Modify the existing counter library in `{}` to add the following features:

1. Add a decrement() method to the Counter struct
2. Add a reset() method that sets the counter back to 0
3. Add a step parameter to increment/decrement methods (default to 1)
4. Update all tests to cover the new functionality
5. Ensure everything still compiles and passes tests

Use the fs_patch tool to modify the existing files rather than recreating them."#,
        fixture.test_dir.display()
    );

    let _modification_response = fixture.execute_task(&modification_task).await?;

    // Verify the modifications were made
    fixture.assert_file_contains("src/lib.rs", "fn decrement").await?;
    fixture.assert_file_contains("src/lib.rs", "fn reset").await?;
    fixture.assert_file_contains("src/lib.rs", "step").await?;

    // Verify the code still compiles and tests pass
    fixture.assert_code_compiles("Cargo.toml").await?;
    fixture.assert_tests_pass("Cargo.toml").await?;

    Ok(())
}

#[tokio::test] 
async fn test_debug_failing_tests() -> anyhow::Result<()> {
    let fixture = CodeGenerationFixture::new().await?;
    
    // Create code with intentionally failing tests
    let buggy_task = format!(
        r#"Create a Rust library in `{}` that contains:

1. Cargo.toml for "buggy_math"
2. src/lib.rs with a function `multiply_by_two(x: i32) -> i32` that returns `x * 3` (intentionally wrong)
3. Tests that expect the function to return `x * 2`

This should create failing tests that need to be debugged."#,
        fixture.test_dir.display()
    );

    let _buggy_response = fixture.execute_task(&buggy_task).await?;

    // Verify tests fail (expect this to fail)
    let cargo_dir = fixture.test_dir.join("Cargo.toml").parent().unwrap().to_path_buf();
    let test_output = tokio::process::Command::new("cargo")
        .arg("test")
        .current_dir(&cargo_dir)
        .output()
        .await?;
    
    assert!(!test_output.status.success(), "Tests should be failing but they passed");

    // Now ask to debug and fix the failing tests
    let debug_task = format!(
        r#"The tests in `{}` are failing. Please:

1. Use forge_tool_process_shell to run the tests and see the failure output
2. Identify what's wrong with the code
3. Fix the multiply_by_two function to make tests pass
4. Verify the fix by running tests again

Debug the issue step by step and provide a clear explanation of what was wrong."#,
        fixture.test_dir.display()
    );

    let _debug_response = fixture.execute_task(&debug_task).await?;

    // Verify the code now compiles and tests pass
    fixture.assert_code_compiles("Cargo.toml").await?;
    fixture.assert_tests_pass("Cargo.toml").await?;

    Ok(())
}

#[tokio::test]
async fn test_search_and_refactor() -> anyhow::Result<()> {
    let fixture = CodeGenerationFixture::new().await?;
    
    // Create a project with multiple files
    let setup_task = format!(
        r#"Create a Rust project in `{}` with:

1. Cargo.toml for "string_utils"
2. src/lib.rs with modules: `formatter` and `validator`
3. src/formatter.rs with functions to format strings (uppercase, lowercase, title_case)
4. src/validator.rs with functions to validate strings (is_email, is_phone)
5. Tests for all functions

Create a somewhat complex project structure."#,
        fixture.test_dir.display()
    );

    let _setup_response = fixture.execute_task(&setup_task).await?;

    // Now ask to search and refactor
    let refactor_task = format!(
        r#"In the string_utils project at `{}`, perform the following refactoring:

1. Use forge_tool_fs_search to find all functions that return `bool`
2. Add a new `Result<bool, ValidationError>` return type to validator functions
3. Create a custom error enum `ValidationError` in src/validator.rs  
4. Update all validator functions to use the new error type
5. Update tests to handle the new return types
6. Ensure everything compiles and tests pass

Use the search tool to systematically find what needs to be changed."#,
        fixture.test_dir.display()
    );

    let _refactor_response = fixture.execute_task(&refactor_task).await?;

    // Verify the refactoring was successful
    fixture.assert_file_contains("src/validator.rs", "ValidationError").await?;
    fixture.assert_file_contains("src/validator.rs", "Result<bool, ValidationError>").await?;

    // Verify the code compiles and tests pass
    fixture.assert_code_compiles("Cargo.toml").await?;
    fixture.assert_tests_pass("Cargo.toml").await?;

    Ok(())
}

/// Test runner for environment variable gating
fn should_run_code_generation_tests() -> bool {
    std::env::var("RUN_CODE_GENERATION_TESTS").is_ok()
}

/// Conditional test runner macro
macro_rules! conditional_test {
    ($test_name:ident) => {
        #[tokio::test]
        async fn $test_name() -> anyhow::Result<()> {
            if !should_run_code_generation_tests() {
                eprintln!(
                    "Skipping code generation test {} as RUN_CODE_GENERATION_TESTS is not set",
                    stringify!($test_name)
                );
                return Ok(());
            }
            
            super::$test_name().await
        }
    };
}

mod gated_tests {
    use super::*;
    
    conditional_test!(test_create_simple_rust_library);
    conditional_test!(test_modify_existing_code);
    conditional_test!(test_debug_failing_tests);
    conditional_test!(test_search_and_refactor);
}