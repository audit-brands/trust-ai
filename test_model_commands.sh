#!/bin/bash

echo "=========================================="
echo "Trust-AI Model Command Investigation"
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

echo "Testing individual model commands with timeouts..."

# Test each model command individually with short timeouts
echo "1. Testing /model list"
echo "/model list" | timeout 5 trust-ai > model_list_test.log 2>&1
TEST1=$?
print_status $TEST1 "Model list command"

echo "2. Testing /model status"  
echo "/model status" | timeout 5 trust-ai > model_status_test.log 2>&1
TEST2=$?
print_status $TEST2 "Model status command"

echo "3. Testing /model health"
echo "/model health" | timeout 5 trust-ai > model_health_test.log 2>&1  
TEST3=$?
print_status $TEST3 "Model health command"

echo "4. Testing /model discover"
echo "/model discover" | timeout 5 trust-ai > model_discover_test.log 2>&1
TEST4=$?
print_status $TEST4 "Model discover command"

echo ""
echo "Results summary:"
echo "=================="

if [ $TEST1 -eq 124 ]; then
    echo -e "${RED}❌ /model list - TIMEOUT${NC}"
elif [ $TEST1 -eq 0 ]; then
    echo -e "${GREEN}✅ /model list - SUCCESS${NC}"
else
    echo -e "${YELLOW}⚠️  /model list - ERROR (exit code: $TEST1)${NC}"
fi

if [ $TEST2 -eq 124 ]; then
    echo -e "${RED}❌ /model status - TIMEOUT${NC}"
elif [ $TEST2 -eq 0 ]; then
    echo -e "${GREEN}✅ /model status - SUCCESS${NC}"
else
    echo -e "${YELLOW}⚠️  /model status - ERROR (exit code: $TEST2)${NC}"
fi

if [ $TEST3 -eq 124 ]; then
    echo -e "${RED}❌ /model health - TIMEOUT${NC}"
elif [ $TEST3 -eq 0 ]; then
    echo -e "${GREEN}✅ /model health - SUCCESS${NC}"
else
    echo -e "${YELLOW}⚠️  /model health - ERROR (exit code: $TEST3)${NC}"
fi

if [ $TEST4 -eq 124 ]; then
    echo -e "${RED}❌ /model discover - TIMEOUT${NC}"
elif [ $TEST4 -eq 0 ]; then
    echo -e "${GREEN}✅ /model discover - SUCCESS${NC}"
else
    echo -e "${YELLOW}⚠️  /model discover - ERROR (exit code: $TEST4)${NC}"
fi

echo ""
echo "Log files created:"
echo "- model_list_test.log"
echo "- model_status_test.log"
echo "- model_health_test.log"
echo "- model_discover_test.log"

echo ""
echo "Checking log contents for clues..."
echo "=================================="

for log in model_list_test.log model_status_test.log model_health_test.log model_discover_test.log; do
    if [ -f "$log" ]; then
        echo ""
        echo "--- $log ---"
        tail -10 "$log"
    fi
done