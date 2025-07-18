#!/bin/bash

# Test script for Trust AI beta installation
# This script tests the installation without actually installing

set -e

echo "🧪 Trust AI Beta Installation Test"
echo "=================================="
echo ""

# Test 1: Check required tools
echo "✅ Testing prerequisites..."

if command -v curl >/dev/null 2>&1; then
    echo "  ✓ curl is available"
else
    echo "  ❌ curl is required but not found"
    exit 1
fi

if command -v git >/dev/null 2>&1; then
    echo "  ✓ git is available"
else
    echo "  ❌ git is required but not found"
    exit 1
fi

# Test 2: Check if Rust is available (optional)
if command -v cargo >/dev/null 2>&1; then
    echo "  ✓ Rust/cargo is already installed"
    RUST_VERSION=$(cargo --version)
    echo "    $RUST_VERSION"
else
    echo "  ⚠️  Rust not found (will be installed during setup)"
fi

echo ""

# Test 3: Verify installation scripts exist
echo "✅ Testing installation scripts..."

if [ -f "install-beta.sh" ]; then
    echo "  ✓ install-beta.sh exists"
    if [ -x "install-beta.sh" ]; then
        echo "  ✓ install-beta.sh is executable"
    else
        echo "  ❌ install-beta.sh is not executable"
        exit 1
    fi
else
    echo "  ❌ install-beta.sh not found"
    exit 1
fi

if [ -f "curl-install.sh" ]; then
    echo "  ✓ curl-install.sh exists"
    if [ -x "curl-install.sh" ]; then
        echo "  ✓ curl-install.sh is executable"
    else
        echo "  ❌ curl-install.sh is not executable"
        exit 1
    fi
else
    echo "  ❌ curl-install.sh not found"
    exit 1
fi

echo ""

# Test 4: Check project structure
echo "✅ Testing project structure..."

if [ -f "Cargo.toml" ]; then
    echo "  ✓ Root Cargo.toml exists"
else
    echo "  ❌ Root Cargo.toml not found"
    exit 1
fi

if [ -d "crates/forge_main" ]; then
    echo "  ✓ Main crate directory exists"
else
    echo "  ❌ Main crate directory not found"
    exit 1
fi

if [ -f "crates/forge_main/Cargo.toml" ]; then
    echo "  ✓ Main crate Cargo.toml exists"
else
    echo "  ❌ Main crate Cargo.toml not found"
    exit 1
fi

echo ""

# Test 5: Validate installation script syntax
echo "✅ Testing script syntax..."

if bash -n install-beta.sh; then
    echo "  ✓ install-beta.sh syntax is valid"
else
    echo "  ❌ install-beta.sh has syntax errors"
    exit 1
fi

if bash -n curl-install.sh; then
    echo "  ✓ curl-install.sh syntax is valid"
else
    echo "  ❌ curl-install.sh has syntax errors"
    exit 1
fi

echo ""

# Test 6: Check documentation
echo "✅ Testing documentation..."

if [ -f "BETA_INSTALL.md" ]; then
    echo "  ✓ BETA_INSTALL.md exists"
else
    echo "  ❌ BETA_INSTALL.md not found"
    exit 1
fi

if [ -f "README.md" ]; then
    echo "  ✓ README.md exists"
    if grep -q "Beta Release" README.md; then
        echo "  ✓ README.md contains beta installation info"
    else
        echo "  ❌ README.md missing beta installation section"
        exit 1
    fi
else
    echo "  ❌ README.md not found"
    exit 1
fi

echo ""
echo "🎉 All tests passed!"
echo ""
echo "Trust AI beta installation is ready. Users can install with:"
echo "  curl -sSL https://raw.githubusercontent.com/audit-brands/trust-ai/main/curl-install.sh | bash"
echo ""
echo "Installation will:"
echo "  • Install Rust if needed (1-2 minutes)"
echo "  • Download and build Trust AI (2-3 minutes)"
echo "  • Install to /usr/local/bin/trust-ai"
echo "  • Create 'trust' command alias"
echo ""
echo "Total installation time: 3-5 minutes"