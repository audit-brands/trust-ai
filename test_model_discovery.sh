#!/bin/bash

echo "=========================================="
echo "Trust-AI Model Discovery Test"
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

# Create test commands file
echo "Creating model discovery test commands..."
cat > model_discovery_commands.txt << 'EOF'
/model discover
/model list
/model health
/model select ollama/qwen2.5-coder:7b
/model status
/exit
EOF

echo "Test commands created:"
cat model_discovery_commands.txt
echo ""

# Execute the test
echo "Executing model discovery test..."
echo "=========================================="

# Run trust-ai with the command file
trust-ai -c model_discovery_commands.txt > model_discovery_output.log 2>&1
TEST_STATUS=$?

print_status $TEST_STATUS "Model discovery test execution"

echo ""
echo "Test output:"
echo "=========================================="
cat model_discovery_output.log

echo ""
echo "=========================================="

# Analyze results
if [ $TEST_STATUS -eq 0 ]; then
    echo -e "${GREEN}✅ Model discovery test completed successfully${NC}"
    
    # Check for specific success indicators
    if grep -q "qwen2.5-coder:7b" model_discovery_output.log; then
        echo -e "${GREEN}✅ qwen2.5-coder:7b model found${NC}"
    else
        echo -e "${YELLOW}⚠️  qwen2.5-coder:7b model not found in output${NC}"
    fi
    
    if grep -q "Selected model" model_discovery_output.log; then
        echo -e "${GREEN}✅ Model selection appears successful${NC}"
    else
        echo -e "${YELLOW}⚠️  Model selection status unclear${NC}"
    fi
else
    echo -e "${RED}❌ Model discovery test failed${NC}"
    echo "Check model_discovery_output.log for details"
fi

echo ""
echo "Next step: Run file creation test with ./test_file_creation.sh"

# Clean up
rm -f model_discovery_commands.txt