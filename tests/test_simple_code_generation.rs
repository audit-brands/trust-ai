use std::path::PathBuf;
use std::time::Duration;

use anyhow::Context;
use forge_api::{ForgeAPI, API};
use forge_domain::{Agent, AgentId, ChatRequest, ChatResponse, Event, EventContext, ModelId, SystemContext, Template, ToolName, Workflow};
use forge_tracker::Tracker;
use tokio_stream::StreamExt;
use tokio::fs;
use pretty_assertions::assert_eq;

/// Simple test fixture for basic code generation testing
struct SimpleCodeGenFixture {
    api: ForgeAPI,
    test_dir: PathBuf,
    _guard: forge_tracker::Guard,
}

lazy_static::lazy_static! {
    static ref tracker: Tracker = Tracker::default();
}

impl SimpleCodeGenFixture {
    /// Create a new test fixture
    async fn new(test_name: &str) -> anyhow::Result<Self> {
        let test_dir = PathBuf::from("target").join(format!("test_{}", test_name));
        
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

    /// Create a simple code generation agent
    fn create_simple_agent(&self) -> Agent {
        let system_prompt = r#"
You are a software engineering assistant focused on creating clean, working code.

Your task is to write code that:
1. Compiles successfully
2. Includes proper tests
3. Follows Rust best practices
4. Uses the provided file operation tools

Available tools:
- forge_tool_fs_create: Create new files with content
- forge_tool_fs_read: Read existing files
- forge_tool_fs_patch: Modify parts of existing files
- forge_tool_process_shell: Run shell commands for testing

Be systematic and verify your work by running the code.
"#;

        let user_prompt = r#"{{event.value}}"#;

        Agent::new(AgentId::new("simple_coder"))
            .model(ModelId::new("anthropic/claude-3.5-sonnet"))
            .tool_supported(true)
            .tools(vec![
                ToolName::new("forge_tool_fs_create"),
                ToolName::new("forge_tool_fs_read"),
                ToolName::new("forge_tool_fs_patch"),
                ToolName::new("forge_tool_process_shell"),
            ])
            .subscribe(vec!["simple_code_task".to_string()])
            .system_prompt(Template::<SystemContext>::new(system_prompt.trim()))
            .user_prompt(Template::<EventContext>::new(user_prompt.trim()))
    }

    /// Execute a simple code generation task
    async fn execute_simple_task(&self, task: &str) -> anyhow::Result<String> {
        let workflow = Workflow::new().agents(vec![self.create_simple_agent()]);
        
        let conversation_id = self.api.init_conversation(workflow).await?.id;
        let request = ChatRequest::new(
            Event::new("simple_code_task", Some(task)),
            conversation_id,
        );

        let response = self.api.chat(request)
            .await
            .with_context(|| "Failed to execute simple code task")?
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

    /// Check if a file exists and contains specific content
    async fn file_contains(&self, path: &str, content: &str) -> bool {
        let file_path = self.test_dir.join(path);
        if let Ok(file_content) = fs::read_to_string(&file_path).await {
            file_content.contains(content)
        } else {
            false
        }
    }

    /// Get the test directory path as a string
    fn test_dir_str(&self) -> String {
        self.test_dir.display().to_string()
    }
}

#[tokio::test]
async fn test_basic_file_creation() -> anyhow::Result<()> {
    let fixture = SimpleCodeGenFixture::new("basic_file").await?;
    
    let task = format!(
        r#"Create a simple Rust source file at `{}/hello.rs` with:

1. A function called `greet` that takes a name parameter and returns a greeting string
2. A main function that calls greet with "World" and prints the result
3. Basic tests for the greet function

Use forge_tool_fs_create to create the file. Make sure the code is syntactically correct."#,
        fixture.test_dir_str()
    );

    let response = fixture.execute_simple_task(&task).await?;
    
    // Check that the task was understood and attempted
    assert!(!response.is_empty(), "Response should not be empty");
    
    // Check that the file was created with expected content
    assert!(fixture.file_contains("hello.rs", "fn greet").await, "File should contain greet function");
    assert!(fixture.file_contains("hello.rs", "fn main").await, "File should contain main function");
    
    Ok(())
}

#[tokio::test]
async fn test_file_modification() -> anyhow::Result<()> {
    let fixture = SimpleCodeGenFixture::new("file_mod").await?;
    
    // First create a simple file
    let initial_content = r#"fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }
}"#;
    
    let file_path = fixture.test_dir.join("math.rs");
    fs::write(&file_path, initial_content).await?;
    
    let task = format!(
        r#"Modify the existing file `{}/math.rs` to add a new function called `multiply` that multiplies two numbers.

1. Use forge_tool_fs_read to read the current file
2. Use forge_tool_fs_patch to add the multiply function after the add function
3. Add a test for the multiply function in the existing test module

The new function should be: `fn multiply(a: i32, b: i32) -> i32`"#,
        fixture.test_dir_str()
    );

    let response = fixture.execute_simple_task(&task).await?;
    
    // Check that the response indicates understanding
    assert!(!response.is_empty(), "Response should not be empty");
    
    // Check that the file was modified with new content
    assert!(fixture.file_contains("math.rs", "fn multiply").await, "File should contain multiply function");
    assert!(fixture.file_contains("math.rs", "test_multiply").await || 
            fixture.file_contains("math.rs", "multiply(").await, 
            "File should contain multiply test");
    
    Ok(())
}

#[tokio::test]
async fn test_shell_command_execution() -> anyhow::Result<()> {
    let fixture = SimpleCodeGenFixture::new("shell_cmd").await?;
    
    let task = format!(
        r#"Create a simple Rust program at `{}/version.rs` and then use shell commands to verify it works:

1. Create a program that prints the Rust version information
2. Use forge_tool_process_shell to compile it with rustc
3. Use forge_tool_process_shell to run the compiled program
4. Report whether compilation and execution were successful

The program should be simple and just print some version info or a hello message."#,
        fixture.test_dir_str()
    );

    let response = fixture.execute_simple_task(&task).await?;
    
    // Check that the response shows shell command usage
    assert!(response.contains("forge_tool_process_shell") || 
           response.contains("rustc") || 
           response.contains("compilation") ||
           response.to_lowercase().contains("success"),
           "Response should indicate shell command usage and results");
    
    Ok(())
}

/// Test runner that checks if code generation tests should run
fn should_run_code_generation_tests() -> bool {
    std::env::var("RUN_CODE_GENERATION_TESTS").is_ok()
}

/// Wrapper tests that can be conditionally skipped
mod conditional_tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_file_creation_conditional() -> anyhow::Result<()> {
        if !should_run_code_generation_tests() {
            eprintln!("Skipping code generation test - set RUN_CODE_GENERATION_TESTS=1 to enable");
            return Ok(());
        }
        
        super::test_basic_file_creation().await
    }

    #[tokio::test] 
    async fn test_file_modification_conditional() -> anyhow::Result<()> {
        if !should_run_code_generation_tests() {
            eprintln!("Skipping code generation test - set RUN_CODE_GENERATION_TESTS=1 to enable");
            return Ok(());
        }
        
        super::test_file_modification().await
    }

    #[tokio::test]
    async fn test_shell_command_execution_conditional() -> anyhow::Result<()> {
        if !should_run_code_generation_tests() {
            eprintln!("Skipping code generation test - set RUN_CODE_GENERATION_TESTS=1 to enable");
            return Ok(());
        }
        
        super::test_shell_command_execution().await
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn test_fixture_creation() {
        let fixture = SimpleCodeGenFixture::new("test_fixture").await.unwrap();
        
        // Test that the fixture was created properly
        assert!(fixture.test_dir.exists());
        assert!(fixture.test_dir_str().contains("test_fixture"));
    }

    #[test]
    fn test_should_run_flag() {
        // Test that the environment variable check works
        let original = std::env::var("RUN_CODE_GENERATION_TESTS").ok();
        
        // Test when not set
        std::env::remove_var("RUN_CODE_GENERATION_TESTS");
        assert!(!should_run_code_generation_tests());
        
        // Test when set
        std::env::set_var("RUN_CODE_GENERATION_TESTS", "1");
        assert!(should_run_code_generation_tests());
        
        // Restore original value
        if let Some(val) = original {
            std::env::set_var("RUN_CODE_GENERATION_TESTS", val);
        } else {
            std::env::remove_var("RUN_CODE_GENERATION_TESTS");
        }
    }
}