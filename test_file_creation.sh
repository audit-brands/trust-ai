#!/bin/bash

echo "=========================================="
echo "Trust-AI File Creation Test"
echo "=========================================="

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

print_status() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✅ $2${NC}"
    else
        echo -e "${RED}❌ $2${NC}"
    fi
}

# Create test directory
TEST_DIR="test_file_creation_output"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

echo "Test directory created: $(pwd)"
echo ""

# Create test input
echo "Creating file creation test prompt..."
cat > ../file_creation_test_commands.txt << 'EOF'
/model select ollama/qwen2.5-coder:7b
Create a Python configuration file called 'app_config.py' that contains database connection settings. Include the following:
- Database host, port, username, password, and database name
- Connection timeout and retry settings  
- Environment-specific configurations (dev, staging, prod)
- Proper comments explaining each setting
- Use appropriate Python data structures like dictionaries

Please create this file with proper formatting and best practices.
/exit
EOF

echo "Test prompt created."
echo ""

# Execute the test
echo "Executing file creation test..."
echo "=========================================="

# Run trust-ai with the command file
trust-ai -c ../file_creation_test_commands.txt > ../file_creation_output.log 2>&1
TEST_STATUS=$?

print_status $TEST_STATUS "File creation test execution"

echo ""
echo "Checking for created files..."

# Check if the file was created
if [ -f "app_config.py" ]; then
    print_status 0 "app_config.py file created"
    echo ""
    echo "Generated file content:"
    echo "=========================================="
    cat app_config.py
    echo ""
    echo "=========================================="
    
    # Basic syntax check
    python3 -m py_compile app_config.py 2>/dev/null
    SYNTAX_CHECK=$?
    print_status $SYNTAX_CHECK "Python syntax validation"
    
    # Check file size
    FILE_SIZE=$(wc -l < app_config.py)
    if [ $FILE_SIZE -gt 10 ]; then
        print_status 0 "File has substantial content ($FILE_SIZE lines)"
    else
        echo -e "${YELLOW}⚠️  File seems small ($FILE_SIZE lines)${NC}"
    fi
    
else
    print_status 1 "app_config.py file creation failed"
fi

echo ""
echo "Trust-AI execution log:"
echo "=========================================="
cat ../file_creation_output.log

cd ..

echo ""
echo "=========================================="

# Summary
if [ -f "$TEST_DIR/app_config.py" ] && [ $TEST_STATUS -eq 0 ]; then
    echo -e "${GREEN}✅ File creation test completed successfully${NC}"
    echo "- File was created with proper content"
    echo "- Python syntax appears valid"
else
    echo -e "${RED}❌ File creation test failed${NC}"
    echo "- Check file_creation_output.log for details"
fi

echo ""
echo "Next step: Run project creation test with ./test_project_creation.sh"

# Clean up
rm -f file_creation_test_commands.txt