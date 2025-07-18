# Ollama Local AI Testing Plan - Corrected CLI Commands

## Objective
Provide corrected CLI commands and practical testing exercises for the user's Ollama integration after identifying that the trust-ai CLI doesn't have a 'config' subcommand.

## CLI Command Corrections

### Model Configuration Commands
The trust-ai CLI uses the `/model` command within the interactive session, not as a subcommand:

**Incorrect (from previous plan):**
```bash
trust-ai config model set ollama/qwen2.5-coder:7b
```

**Correct approach:**
1. Start trust-ai in interactive mode:
   ```bash
   trust-ai
   ```

2. Use the model commands within the session:
   ```
   /model list
   /model config
   /model select ollama/qwen2.5-coder:7b
   /model status
   ```

### Available Model Commands
Based on the CLI implementation, these commands are available within the interactive session:
- `/model` - Opens model selection interface
- `/model list` - Lists available models
- `/model status` - Shows current model status
- `/model config` - Shows model configuration
- `/model discover` - Discovers available models from all providers
- `/model health` - Shows health status of all providers
- `/model refresh` - Refreshes model discovery and health checks
- `/model select <model_id>` - Selects a specific model

## Corrected Testing Exercises

### Exercise 1: Model Discovery and Selection
**Objective:** Verify Ollama models are detected and can be selected

**Steps:**
1. Start trust-ai:
   ```bash
   trust-ai
   ```

2. Discover available models:
   ```
   /model discover
   ```

3. List all models to verify Ollama models appear:
   ```
   /model list
   ```

4. Select the downloaded model:
   ```
   /model select ollama/qwen2.5-coder:7b
   ```

5. Verify selection:
   ```
   /model status
   ```

### Exercise 2: Simple Python File Creation
**Objective:** Test basic code generation with file creation

**Prompt:**
```
Create a simple Python calculator that can perform basic arithmetic operations (add, subtract, multiply, divide). Save it as calculator.py in the current directory.
```

**Expected Outcome:** A functional calculator.py file should be created

### Exercise 3: Code Analysis and Improvement
**Objective:** Test code understanding and enhancement capabilities

**Steps:**
1. First, create a basic file:
   ```
   Create a simple Python function that calculates factorial. Save it as factorial.py.
   ```

2. Then ask for improvements:
   ```
   Analyze the factorial.py file and add error handling, type hints, and documentation. Update the file with these improvements.
   ```

### Exercise 4: Multi-file Project Creation
**Objective:** Test ability to create structured projects

**Prompt:**
```
Create a simple Python package for a todo list application with the following structure:
- todo_app/
  - __init__.py
  - todo.py (main Todo class)
  - cli.py (command line interface)
  - requirements.txt

The application should support adding, listing, and marking tasks as complete.
```

### Exercise 5: Code Debugging Exercise
**Objective:** Test debugging and problem-solving capabilities

**Steps:**
1. Create a file with intentional bugs:
   ```
   Create a Python script called buggy_sort.py that implements bubble sort but has 2-3 intentional bugs in it.
   ```

2. Ask for debugging:
   ```
   Analyze buggy_sort.py, identify the bugs, and create a fixed version called fixed_sort.py.
   ```

## Model Testing Sequence

### Test with qwen2.5-coder:7b (Already Downloaded)
1. Start with Exercise 1 to verify model selection
2. Progress through Exercises 2-5 to test capabilities
3. Note response quality, speed, and accuracy

### Download and Test Additional Models
After testing the first model, download others:

1. **deepseek-coder:6.7b** (Good for code generation):
   ```bash
   ollama pull deepseek-coder:6.7b
   ```

2. **codellama:7b-instruct** (Meta's code model):
   ```bash
   ollama pull codellama:7b-instruct
   ```

3. **mistral:7b-instruct** (General purpose with good coding):
   ```bash
   ollama pull mistral:7b-instruct
   ```

## Verification Criteria
- [ ] Models are properly discovered by `/model discover`
- [ ] Models can be selected via `/model select`
- [ ] File creation works correctly (files are actually created)
- [ ] Code quality is appropriate for each model
- [ ] Response times are acceptable for local execution
- [ ] Error handling works when models are unavailable

## Additional Notes
- Use `/tools` command to see available file operations
- Use `/help` for general assistance
- Use `/info` to see system information
- The trust-ai CLI is primarily interactive - most configuration happens within the session
- File operations should create actual files in the filesystem, not just display code

## Troubleshooting
If models don't appear in `/model list`:
1. Verify Ollama is running: `ollama list`
2. Check trust-ai can connect to Ollama
3. Use `/model health` to check provider status
4. Try `/model refresh` to refresh discovery

This corrected plan addresses the CLI command structure issue and provides a clear path for testing the Ollama integration with actual file creation exercises.