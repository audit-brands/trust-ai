#!/bin/bash
# Test script for Trust AI local model integration

echo "Testing Trust AI local model integration..."
echo ""

# Test model list command with full output
echo "=== Testing /model list (with stderr) ==="
echo "/model list" | timeout 30s ./target/release/forge

echo ""
echo "=== Testing /model discover (with stderr) ==="
echo "/model discover" | timeout 30s ./target/release/forge

echo ""
echo "Test completed."