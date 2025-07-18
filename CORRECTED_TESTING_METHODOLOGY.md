# Corrected Trust-AI CLI Testing Methodology

## Critical Error Identified

The previous testing methodology contained a **fundamental flaw**: it used direct `ollama run MODEL "prompt"` commands instead of the proper trust-ai CLI interface. This is incorrect because:

1. **Wrong Interface**: Direct ollama commands bypass trust-ai's tool integration
2. **No Tool Access**: Ollama CLI doesn't have access to forge_tool_fs_* functions
3. **Missing Context**: trust-ai provides conversation context and state management
4. **Incorrect Workflow**: Real usage involves trust-ai interactive mode, not direct ollama calls

## Correct Trust-AI CLI Usage

### Proper Command Structure

**WRONG (Previous Approach):**
```bash
ollama run qwen2.5-coder:7b "Create a file using forge_tool_fs_create..."
```

**CORRECT (Trust-AI Approach):**
```bash
# Start trust-ai in interactive mode
trust-ai

# Within the interactive session:
/model discover
/model select ollama/qwen2.5-coder:7b
Create a Python file using the available file tools...
```

### Available Trust-AI Commands

Within the trust-ai interactive session:
- `/model` - Opens model selection interface
- `/model list` - Lists available models
- `/model status` - Shows current model status
- `/model config` - Shows model configuration
- `/model discover` - Discovers available models from all providers
- `/model health` - Shows health status of all providers
- `/model refresh` - Refreshes model discovery and health checks
- `/model select <model_id>` - Selects a specific model
- `/tools` - Shows available tools
- `/help` - Shows help information
- `/info` - Shows system information

## Corrected Test Execution Plan

### Phase 1: Environment Verification

1. **Verify Ollama Service**
   ```bash
   # Check Ollama is running
   ollama list
   
   # Ensure models are available
   ollama list | grep qwen2.5-coder
   ```

2. **Verify Trust-AI Installation**
   ```bash
   # Check trust-ai is installed and working
   trust-ai --version
   
   # Test basic startup
   echo "/info" | trust-ai -p "/info"
   ```

### Phase 2: Model Discovery and Selection Testing

**Test Script: `test_model_discovery.sh`**
```bash
#!/bin/bash

echo "=== Trust-AI Model Discovery Test ==="

# Create test input file
cat > test_commands.txt << 'EOF'
/model discover
/model list
/model health
/model select ollama/qwen2.5-coder:7b
/model status
/exit
EOF

# Execute test
echo "Testing model discovery and selection..."
trust-ai -c test_commands.txt

echo "Model discovery test completed."
```

### Phase 3: File Creation Testing

**Test Script: `test_file_creation.sh`**
```bash
#!/bin/bash

echo "=== Trust-AI File Creation Test ==="

# Create test directory
mkdir -p test_output
cd test_output

# Create test input
cat > ../file_creation_test.txt << 'EOF'
/model select ollama/qwen2.5-coder:7b
Create a Python configuration file called 'app_config.py' that contains database connection settings including host, port, username, password, and database name. Use appropriate data structures and include comments explaining each setting.
/exit
EOF

# Execute test
echo "Testing file creation capabilities..."
trust-ai -c ../file_creation_test.txt

# Verify file was created
if [ -f "app_config.py" ]; then
    echo "✅ File creation successful"
    echo "Generated file content:"
    cat app_config.py
else
    echo "❌ File creation failed"
fi

cd ..
```

### Phase 4: Multi-File Project Testing

**Test Script: `test_project_creation.sh`**
```bash
#!/bin/bash

echo "=== Trust-AI Multi-File Project Test ==="

# Create test directory
mkdir -p test_project
cd test_project

# Create comprehensive test prompt
cat > ../project_test.txt << 'EOF'
/model select ollama/qwen2.5-coder:7b
Create a simple Python task management application with the following structure:
- main.py (entry point with CLI interface)
- task_manager.py (core task management logic)
- storage.py (file-based task storage)
- requirements.txt (dependencies)

The application should support:
- Adding new tasks
- Listing all tasks
- Marking tasks as complete
- Saving/loading tasks from a JSON file

Please create all files with proper imports and functionality.
/exit
EOF

# Execute test
echo "Testing multi-file project creation..."
trust-ai -c ../project_test.txt

# Verify files were created
echo "Checking created files:"
for file in main.py task_manager.py storage.py requirements.txt; do
    if [ -f "$file" ]; then
        echo "✅ $file created"
    else
        echo "❌ $file missing"
    fi
done

cd ..
```

### Phase 5: Tool Integration Testing

**Test Script: `test_tool_integration.sh`**
```bash
#!/bin/bash

echo "=== Trust-AI Tool Integration Test ==="

# Create test directory with sample files
mkdir -p test_tools
cd test_tools

# Create sample files for testing
echo "sample content for testing" > sample.txt
echo "DATABASE_CONFIG = 'localhost:5432'" > config.py

# Create tool integration test
cat > ../tool_test.txt << 'EOF'
/model select ollama/qwen2.5-coder:7b
Please perform the following tasks:
1. Search for files containing "DATABASE_CONFIG" in the current directory
2. Read the content of any configuration files found
3. Create a summary report called 'analysis_report.md' with your findings
/exit
EOF

# Execute test
echo "Testing tool integration..."
trust-ai -c ../tool_test.txt

# Verify results
if [ -f "analysis_report.md" ]; then
    echo "✅ Tool integration successful"
    echo "Generated report:"
    cat analysis_report.md
else
    echo "❌ Tool integration failed"
fi

cd ..
```

## Interactive Testing Session

For manual validation, use this interactive approach:

```bash
# Start trust-ai
trust-ai

# Test sequence:
/model discover
/model list
/model select ollama/qwen2.5-coder:7b
/model status
/tools

# Test file creation
Create a simple Python calculator script called calculator.py with functions for add, subtract, multiply, and divide operations.

# Test file reading
Read the calculator.py file and suggest improvements.

# Test file modification
Update the calculator.py file to include error handling for division by zero.

# Exit
/exit
```

## Validation Criteria

### Success Indicators
- [ ] trust-ai starts without errors
- [ ] `/model discover` finds Ollama models
- [ ] `/model select ollama/qwen2.5-coder:7b` succeeds
- [ ] Model generates proper tool calls (JSON format)
- [ ] Files are actually created in the filesystem
- [ ] Generated code is syntactically correct
- [ ] Tool integration works seamlessly

### Failure Indicators
- [ ] trust-ai fails to start
- [ ] Models not discovered by `/model discover`
- [ ] Model selection fails
- [ ] No files created despite requests
- [ ] Malformed tool calls
- [ ] Syntax errors in generated code

## Performance Metrics to Collect

1. **Response Time**: Time from prompt to completion
2. **Tool Call Accuracy**: Percentage of correct tool usage
3. **File Creation Success**: Percentage of successful file operations
4. **Code Quality**: Syntax correctness and functionality
5. **Resource Usage**: Memory and CPU during operation

## Common Issues and Solutions

### Issue: Models Not Found
```bash
# Solution: Refresh discovery
/model refresh
/model discover
```

### Issue: Tool Calls Not Working
- Verify trust-ai has proper permissions
- Check file system access
- Ensure working directory is writable

### Issue: Poor Model Performance
- Try different models: `/model list`
- Check system resources
- Verify model is properly loaded

## Next Steps for Execution

1. **Run Environment Verification** (Phase 1)
2. **Execute Model Discovery Tests** (Phase 2)
3. **Test File Creation Capabilities** (Phase 3)
4. **Validate Multi-File Projects** (Phase 4)
5. **Verify Tool Integration** (Phase 5)
6. **Collect Performance Metrics**
7. **Document Results and Recommendations**

This corrected methodology ensures proper testing of the trust-ai CLI system with actual model interaction and tool integration, providing accurate assessment of local AI capabilities.