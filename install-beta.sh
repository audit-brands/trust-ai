#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}Trust AI Beta Installation${NC}"
echo -e "${YELLOW}Building from source for beta release${NC}"
echo ""

# Check if curl is available
if ! command -v curl >/dev/null 2>&1; then
    echo -e "${RED}Error: curl is required but not installed.${NC}"
    exit 1
fi

# Check if git is available
if ! command -v git >/dev/null 2>&1; then
    echo -e "${RED}Error: git is required but not installed.${NC}"
    exit 1
fi

# Check if Rust is installed
if ! command -v cargo >/dev/null 2>&1; then
    echo -e "${YELLOW}Rust not found. Installing Rust...${NC}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
    
    if ! command -v cargo >/dev/null 2>&1; then
        echo -e "${RED}Failed to install Rust. Please install manually: https://rustup.rs/${NC}"
        exit 1
    fi
    echo -e "${GREEN}Rust installed successfully!${NC}"
fi

# Create temporary directory
TMP_DIR=$(mktemp -d)
cd "$TMP_DIR"

echo -e "${BLUE}Downloading Trust AI source code...${NC}"

# Clone the repository
git clone https://github.com/audit-brands/trust-ai.git
cd trust-ai

echo -e "${BLUE}Building Trust AI (this may take a few minutes)...${NC}"

# Build the project in release mode
cargo build --release

# Check if build was successful
if [ ! -f "target/release/trust" ]; then
    echo -e "${RED}Build failed. Binary not found at target/release/trust${NC}"
    exit 1
fi

echo -e "${BLUE}Installing Trust AI...${NC}"

# Install the binary
INSTALL_DIR="/usr/local/bin"
if [ -w "$INSTALL_DIR" ]; then
    cp target/release/trust "$INSTALL_DIR/trust-ai"
    chmod +x "$INSTALL_DIR/trust-ai"
else
    echo -e "${YELLOW}Installing to $INSTALL_DIR requires sudo...${NC}"
    sudo cp target/release/trust "$INSTALL_DIR/trust-ai"
    sudo chmod +x "$INSTALL_DIR/trust-ai"
fi

# Create symlink for 'trust' command
if [ -w "$INSTALL_DIR" ]; then
    ln -sf "$INSTALL_DIR/trust-ai" "$INSTALL_DIR/trust"
else
    sudo ln -sf "$INSTALL_DIR/trust-ai" "$INSTALL_DIR/trust"
fi

# Clean up
cd /
rm -rf "$TMP_DIR"

# Verify installation
if command -v trust-ai >/dev/null 2>&1; then
    echo -e "${GREEN}Trust AI has been successfully installed!${NC}"
    echo ""
    echo -e "${BLUE}Quick Start:${NC}"
    echo -e "  trust-ai init          # Initialize configuration"
    echo -e "  trust-ai config set provider openai"
    echo -e "  trust-ai config set api_key YOUR_API_KEY"
    echo -e "  trust-ai chat \"Hello!\"   # Start chatting"
    echo ""
    echo -e "${BLUE}Available commands:${NC}"
    echo -e "  trust-ai --help        # Show all commands"
    echo -e "  trust --help           # Same as above (shorter alias)"
    echo ""
    trust-ai --version
else
    echo -e "${RED}Installation failed. Please check your PATH and try again.${NC}"
    exit 1
fi