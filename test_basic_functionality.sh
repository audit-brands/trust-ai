#!/bin/bash

echo "=========================================="
echo "Trust-AI Basic Functionality Test"
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

# Test 1: Basic help command
echo "Test 1: Basic help command"
timeout 5 trust-ai --help > basic_help.log 2>&1
TEST1_STATUS=$?
print_status $TEST1_STATUS "Help command"

# Test 2: Version command
echo "Test 2: Version command"
timeout 5 trust-ai --version > version.log 2>&1
TEST2_STATUS=$?
print_status $TEST2_STATUS "Version command"

# Test 3: Simple prompt without model commands
echo "Test 3: Simple prompt test"
echo "hello" | timeout 10 trust-ai -p "What is 2+2?" > simple_prompt.log 2>&1
TEST3_STATUS=$?
print_status $TEST3_STATUS "Simple prompt"

# Test 4: Info command
echo "Test 4: Info command test"
cat > info_commands.txt << 'EOF'
/info
/exit
EOF

timeout 10 trust-ai -c info_commands.txt > info_output.log 2>&1
TEST4_STATUS=$?
print_status $TEST4_STATUS "Info command"

# Test 5: Help command in interactive mode
echo "Test 5: Help command in interactive mode"
cat > help_commands.txt << 'EOF'
/help
/exit
EOF

timeout 10 trust-ai -c help_commands.txt > help_output.log 2>&1
TEST5_STATUS=$?
print_status $TEST5_STATUS "Interactive help command"

echo ""
echo "=========================================="
echo "Test Results Summary:"
echo "=========================================="

if [ $TEST1_STATUS -eq 0 ]; then
    echo -e "${GREEN}✅ CLI help works${NC}"
else
    echo -e "${RED}❌ CLI help failed${NC}"
fi

if [ $TEST2_STATUS -eq 0 ]; then
    echo -e "${GREEN}✅ Version command works${NC}"
else
    echo -e "${RED}❌ Version command failed${NC}"
fi

if [ $TEST3_STATUS -eq 0 ]; then
    echo -e "${GREEN}✅ Simple prompt works${NC}"
else
    echo -e "${RED}❌ Simple prompt failed${NC}"
fi

if [ $TEST4_STATUS -eq 0 ]; then
    echo -e "${GREEN}✅ Info command works${NC}"
else
    echo -e "${RED}❌ Info command failed${NC}"
fi

if [ $TEST5_STATUS -eq 0 ]; then
    echo -e "${GREEN}✅ Interactive help works${NC}"
else
    echo -e "${RED}❌ Interactive help failed${NC}"
fi

echo ""
echo "Log files created:"
echo "- basic_help.log"
echo "- version.log" 
echo "- simple_prompt.log"
echo "- info_output.log"
echo "- help_output.log"

# Clean up command files
rm -f info_commands.txt help_commands.txt

echo ""
echo "Next: If basic functionality works, investigate model discovery issue"