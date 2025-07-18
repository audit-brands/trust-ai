#!/bin/bash

echo "=========================================="
echo "Trust-AI Environment Verification Script"
echo "=========================================="

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print status
print_status() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✅ $2${NC}"
    else
        echo -e "${RED}❌ $2${NC}"
    fi
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# Test 1: Check Ollama Service
echo "1. Checking Ollama Service..."
ollama list > /dev/null 2>&1
OLLAMA_STATUS=$?
print_status $OLLAMA_STATUS "Ollama service is running"

if [ $OLLAMA_STATUS -eq 0 ]; then
    echo "   Available Ollama models:"
    ollama list | grep -E "(NAME|qwen2.5-coder|mistral|llama)" | head -10
    
    # Check for qwen2.5-coder specifically
    if ollama list | grep -q "qwen2.5-coder:7b"; then
        echo -e "   ${GREEN}✅ qwen2.5-coder:7b is available${NC}"
    else
        echo -e "   ${YELLOW}⚠️  qwen2.5-coder:7b not found${NC}"
        echo "   To download: ollama pull qwen2.5-coder:7b"
    fi
else
    echo -e "   ${RED}❌ Ollama service not running${NC}"
    echo "   Start with: ollama serve"
fi

echo ""

# Test 2: Check Trust-AI Installation
echo "2. Checking Trust-AI Installation..."
trust-ai --version > /dev/null 2>&1
TRUSTAI_STATUS=$?
print_status $TRUSTAI_STATUS "trust-ai is installed and accessible"

if [ $TRUSTAI_STATUS -eq 0 ]; then
    echo "   Version: $(trust-ai --version)"
else
    echo -e "   ${RED}❌ trust-ai not found in PATH${NC}"
    echo "   Install or check PATH configuration"
fi

echo ""

# Test 3: Check Trust-AI Basic Functionality
echo "3. Testing Trust-AI Basic Functionality..."
if [ $TRUSTAI_STATUS -eq 0 ]; then
    # Test basic command execution
    echo "/info" | timeout 10 trust-ai -p "/info" > /tmp/trustai_test.log 2>&1
    BASIC_TEST=$?
    
    if [ $BASIC_TEST -eq 0 ]; then
        print_status 0 "trust-ai basic functionality works"
        echo "   Basic info command executed successfully"
    else
        print_status 1 "trust-ai basic functionality failed"
        echo "   Check /tmp/trustai_test.log for details"
    fi
else
    print_warning "Skipping basic functionality test (trust-ai not available)"
fi

echo ""

# Test 4: Check File System Permissions
echo "4. Checking File System Permissions..."
TEST_DIR="./test_permissions"
mkdir -p "$TEST_DIR" 2>/dev/null
MKDIR_STATUS=$?
print_status $MKDIR_STATUS "Can create directories"

if [ $MKDIR_STATUS -eq 0 ]; then
    echo "test content" > "$TEST_DIR/test_file.txt" 2>/dev/null
    WRITE_STATUS=$?
    print_status $WRITE_STATUS "Can write files"
    
    if [ -f "$TEST_DIR/test_file.txt" ]; then
        cat "$TEST_DIR/test_file.txt" > /dev/null 2>&1
        READ_STATUS=$?
        print_status $READ_STATUS "Can read files"
    fi
    
    # Clean up
    rm -rf "$TEST_DIR" 2>/dev/null
fi

echo ""

# Test 5: Check Required Dependencies
echo "5. Checking System Dependencies..."

# Check for common tools
for tool in curl wget git; do
    if command -v $tool > /dev/null 2>&1; then
        print_status 0 "$tool is available"
    else
        print_status 1 "$tool is missing"
    fi
done

echo ""

# Summary
echo "=========================================="
echo "Environment Verification Summary"
echo "=========================================="

if [ $OLLAMA_STATUS -eq 0 ] && [ $TRUSTAI_STATUS -eq 0 ]; then
    echo -e "${GREEN}✅ Environment is ready for testing${NC}"
    echo ""
    echo "Next steps:"
    echo "1. Run model discovery test: ./test_model_discovery.sh"
    echo "2. Run file creation test: ./test_file_creation.sh"
    echo "3. Run project creation test: ./test_project_creation.sh"
    echo "4. Run tool integration test: ./test_tool_integration.sh"
else
    echo -e "${RED}❌ Environment setup incomplete${NC}"
    echo ""
    echo "Required actions:"
    if [ $OLLAMA_STATUS -ne 0 ]; then
        echo "- Start Ollama service: ollama serve"
        echo "- Download models: ollama pull qwen2.5-coder:7b"
    fi
    if [ $TRUSTAI_STATUS -ne 0 ]; then
        echo "- Install or configure trust-ai"
    fi
fi

echo ""
echo "Log files:"
echo "- Trust-AI test log: /tmp/trustai_test.log"
echo "- This script output can be saved for reference"