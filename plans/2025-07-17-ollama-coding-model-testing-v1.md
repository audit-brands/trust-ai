# Ollama Coding Model Testing with File Creation

## Objective
Test specific Ollama coding models with Trust AI through practical coding exercises that create actual files, moving beyond high-level planning to hands-on validation of file creation capabilities.

## Implementation Plan

### 1. **Download and Setup Specific Coding Models**
- Dependencies: None
- Notes: Focus on proven coding models with good file generation capabilities
- Files: N/A (Ollama model downloads)
- Status: Not Started

**Recommended Models:**
- `qwen2.5-coder:7b` - Excellent for Python and general coding tasks
- `deepseek-coder:6.7b` - Strong code generation and explanation
- `codellama:7b-instruct` - Meta's specialized coding model
- `mistral:7b-instruct` - Good general purpose with coding capabilities

**Download Commands:**
```bash
ollama pull qwen2.5-coder:7b
ollama pull deepseek-coder:6.7b
ollama pull codellama:7b-instruct
ollama pull mistral:7b-instruct
```

### 2. **Refresh Trust AI Configuration**
- Dependencies: Task 1
- Notes: Ensure Trust AI detects new models and provider is set correctly
- Files: `forge.yaml`, Trust AI configuration
- Status: Not Started

**Configuration Steps:**
- Set provider to Ollama: `trust-ai config set provider ollama`
- Verify model detection: `trust-ai models`
- Test basic connectivity: `trust-ai chat "Hello, can you see me?"`

### 3. **Python Function Creation Tests**
- Dependencies: Task 2
- Notes: Test file creation with increasing complexity, verify actual .py files are generated
- Files: Generated Python files in test directory
- Status: Not Started

**Test Scenarios:**
- Simple function: "Write a Python function that calculates fibonacci numbers and save it to fibonacci.py"
- Class with methods: "Create a Python class for a basic calculator with add, subtract, multiply, divide methods. Save to calculator.py"
- Module with imports: "Write a Python script that uses requests to fetch data from an API and save to api_client.py"

### 4. **Multi-File Project Generation**
- Dependencies: Task 3
- Notes: Test Trust AI's ability to create multiple related files in a project structure
- Files: Generated project directories and files
- Status: Not Started

**Test Scenarios:**
- "Create a simple Flask web application with app.py, requirements.txt, and a templates folder with index.html"
- "Generate a Python package structure with __init__.py, main.py, utils.py, and setup.py for a command-line tool"
- "Create a data analysis project with main.py, data_loader.py, analyzer.py, and requirements.txt"

### 5. **Model Comparison Testing**
- Dependencies: Task 4
- Notes: Compare file generation quality across different models for same prompts
- Files: Comparison results and generated files from each model
- Status: Not Started

**Comparison Metrics:**
- File creation success rate
- Code quality and correctness
- Response time per model
- File structure organization
- Code comments and documentation quality

### 6. **Advanced Coding Scenarios**
- Dependencies: Task 5
- Notes: Test complex scenarios that require multiple tool calls and file operations
- Files: Complex generated projects and code
- Status: Not Started

**Advanced Tests:**
- "Read an existing Python file, refactor it to use classes, and save the improved version"
- "Create a test suite for an existing Python module with proper imports and assertions"
- "Generate a complete REST API with multiple endpoints, error handling, and documentation"

## Verification Criteria
- All recommended models download successfully through Ollama
- Trust AI detects and can switch between different Ollama models
- Python files are created with valid syntax and executable code
- Multi-file projects have proper structure and file relationships
- Generated code includes appropriate imports, error handling, and documentation
- File creation works consistently across different model selections
- Response times are reasonable for coding tasks (under 30 seconds for simple functions)

## Potential Risks and Mitigations

### 1. **Model Download Size and Time**
**Risk**: Large model downloads (7B models are 4-8GB each) may take significant time
**Mitigation**: Start with one model (qwen2.5-coder:7b recommended), test thoroughly, then download others; provide clear time expectations

### 2. **File Creation Path Issues**
**Risk**: Trust AI may create files in unexpected locations or fail due to permission issues
**Mitigation**: Test in a dedicated test directory with proper permissions; verify file paths in prompts are explicit and accessible

### 3. **Model Performance Variation**
**Risk**: Different models may have varying code quality or file creation reliability
**Mitigation**: Use standardized test prompts across models; document performance differences; focus on most reliable model for primary testing

### 4. **Tool Integration Complexity**
**Risk**: Trust AI's file creation tool may not work seamlessly with all Ollama models
**Mitigation**: Test basic file creation first before complex scenarios; verify tool calling works with each model; have fallback manual verification process

## Alternative Approaches

### 1. **Single Model Deep Testing**
Focus intensively on one proven model (qwen2.5-coder:7b) with extensive file creation scenarios before expanding to other models

### 2. **Incremental Complexity**
Start with simple text file creation, progress to Python scripts, then advance to multi-file projects rather than testing all scenarios simultaneously

### 3. **Interactive Testing Session**
Use Trust AI's interactive chat mode for real-time testing and immediate feedback rather than scripted test scenarios