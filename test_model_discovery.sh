#!/bin/bash

echo "=== Trust-AI Model Discovery Test ==="

# Create test input file for model discovery
cat > test_commands.txt << 'EOF'
/model discover
/model list
/model health
/model select ollama/qwen2.5-coder:7b
/model status
/exit
EOF

echo "Testing model discovery and selection..."
echo "Input commands:"
cat test_commands.txt
echo ""
echo "Executing test with local trust binary..."

# Execute test with local binary
./target/release/trust -c test_commands.txt

echo ""
echo "Model discovery test completed."