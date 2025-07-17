# Installation Guide

## Overview
Trust AI can be installed through multiple methods depending on your platform and preferences. This guide covers all installation options.

## System Requirements

### Minimum Requirements
- **Operating System**: Linux, macOS, or Windows
- **Architecture**: x86_64 or ARM64
- **Memory**: 256 MB RAM
- **Storage**: 50 MB free space

### Recommended Requirements
- **Memory**: 1 GB RAM (for optimal caching)
- **Storage**: 500 MB free space (for logs and cache)
- **Network**: Stable internet connection for AI providers

### Dependencies
- **For building from source**: Rust 1.70+ and Cargo

## Installation Methods

### Method 1: Pre-built Binaries (Recommended)

#### Linux (x86_64)
```bash
# Download and install
curl -L https://github.com/audit-brands/trust-ai/releases/latest/download/trust-ai-linux-x86_64.tar.gz | tar xz
sudo mv trust-ai /usr/local/bin/
chmod +x /usr/local/bin/trust-ai

# Verify installation
trust-ai --version
```

#### Linux (ARM64)
```bash
# Download and install
curl -L https://github.com/audit-brands/trust-ai/releases/latest/download/trust-ai-linux-arm64.tar.gz | tar xz
sudo mv trust-ai /usr/local/bin/
chmod +x /usr/local/bin/trust-ai

# Verify installation
trust-ai --version
```

#### macOS (Intel)
```bash
# Download and install
curl -L https://github.com/audit-brands/trust-ai/releases/latest/download/trust-ai-macos-x86_64.tar.gz | tar xz
sudo mv trust-ai /usr/local/bin/
chmod +x /usr/local/bin/trust-ai

# Verify installation
trust-ai --version
```

#### macOS (Apple Silicon)
```bash
# Download and install
curl -L https://github.com/audit-brands/trust-ai/releases/latest/download/trust-ai-macos-arm64.tar.gz | tar xz
sudo mv trust-ai /usr/local/bin/
chmod +x /usr/local/bin/trust-ai

# Verify installation
trust-ai --version
```

#### Windows
```powershell
# Download from GitHub releases
Invoke-WebRequest -Uri "https://github.com/audit-brands/trust-ai/releases/latest/download/trust-ai-windows-x86_64.zip" -OutFile "trust-ai.zip"

# Extract
Expand-Archive -Path "trust-ai.zip" -DestinationPath "C:\Program Files\TrustAI"

# Add to PATH (run as Administrator)
$env:PATH += ";C:\Program Files\TrustAI"
[Environment]::SetEnvironmentVariable("PATH", $env:PATH, [EnvironmentVariableTarget]::Machine)

# Verify installation
trust-ai --version
```

### Method 2: Package Managers

#### Homebrew (macOS/Linux)
```bash
# Add tap
brew tap audit-brands/trust-ai

# Install
brew install trust-ai

# Verify installation
trust-ai --version
```

#### Chocolatey (Windows)
```powershell
# Install Chocolatey if not already installed
# Then install Trust AI
choco install trust-ai

# Verify installation
trust-ai --version
```

#### Snap (Linux)
```bash
# Install from Snap Store
sudo snap install trust-ai

# Verify installation
trust-ai --version
```

#### APT (Debian/Ubuntu)
```bash
# Add repository
curl -fsSL https://packages.audit-brands.com/gpg | sudo apt-key add -
echo "deb https://packages.audit-brands.com/apt stable main" | sudo tee /etc/apt/sources.list.d/trust-ai.list

# Update and install
sudo apt update
sudo apt install trust-ai

# Verify installation
trust-ai --version
```

#### YUM/DNF (RHEL/Fedora)
```bash
# Add repository
sudo tee /etc/yum.repos.d/trust-ai.repo <<EOF
[trust-ai]
name=Trust AI Repository
baseurl=https://packages.audit-brands.com/rpm
enabled=1
gpgcheck=1
gpgkey=https://packages.audit-brands.com/gpg
EOF

# Install
sudo dnf install trust-ai  # Fedora
# or
sudo yum install trust-ai  # RHEL/CentOS

# Verify installation
trust-ai --version
```

### Method 3: Build from Source

#### Prerequisites
```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Verify Rust installation
rustc --version
cargo --version
```

#### Build and Install
```bash
# Clone repository
git clone https://github.com/audit-brands/trust-ai.git
cd trust-ai

# Build release version
cargo build --release

# Install globally
cargo install --path .

# Or install to custom location
cargo install --path . --root /opt/trust-ai

# Verify installation
trust-ai --version
```

#### Development Build
```bash
# Clone repository
git clone https://github.com/audit-brands/trust-ai.git
cd trust-ai

# Build development version
cargo build

# Run directly
./target/debug/trust-ai --version

# Or install development version
cargo install --path . --debug
```

### Method 4: Docker

#### Official Docker Image
```bash
# Pull the latest image
docker pull auditbrands/trust-ai:latest

# Run Trust AI in container
docker run --rm -it \
  -e TRUST_AI_API_KEY=your-api-key \
  -v $(pwd):/workspace \
  auditbrands/trust-ai:latest chat "Hello, World!"

# Create alias for easier usage
echo 'alias trust-ai="docker run --rm -it -e TRUST_AI_API_KEY=\$TRUST_AI_API_KEY -v \$(pwd):/workspace auditbrands/trust-ai:latest"' >> ~/.bashrc
source ~/.bashrc
```

#### Build Custom Docker Image
```dockerfile
# Dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/trust-ai /usr/local/bin/trust-ai
ENTRYPOINT ["trust-ai"]
```

```bash
# Build image
docker build -t my-trust-ai .

# Run
docker run --rm -it my-trust-ai --version
```

## Post-Installation Setup

### Initialize Configuration
```bash
# Initialize Trust AI
trust-ai init

# This creates:
# - ~/.config/trust-ai/config.yaml
# - ~/.cache/trust-ai/
# - ~/.local/share/trust-ai/logs/
```

### Configure Your Provider

#### OpenAI Setup
```bash
trust-ai config set provider openai
trust-ai config set api_key sk-your-api-key
trust-ai config set model gpt-4
```

#### Ollama Setup
```bash
# First, install and start Ollama
curl -fsSL https://ollama.ai/install.sh | sh
ollama serve &

# Pull a model
ollama pull llama2

# Configure Trust AI
trust-ai config set provider ollama
trust-ai config set endpoint http://localhost:11434
trust-ai config set model llama2
```

#### Anthropic Setup
```bash
trust-ai config set provider anthropic
trust-ai config set api_key sk-ant-your-key
trust-ai config set model claude-3-opus-20240229
```

### Test Installation
```bash
# Test basic functionality
trust-ai chat "Hello! Are you working correctly?"

# Test configuration
trust-ai config show

# Test performance monitoring
trust-ai perf status
```

## Platform-Specific Instructions

### Linux

#### Ubuntu/Debian
```bash
# Install dependencies
sudo apt update
sudo apt install curl ca-certificates

# Install Trust AI (choose one method above)
# Then test
trust-ai --version
```

#### CentOS/RHEL/Fedora
```bash
# Install dependencies
sudo dnf install curl ca-certificates  # Fedora
# or
sudo yum install curl ca-certificates  # CentOS/RHEL

# Install Trust AI (choose one method above)
# Then test
trust-ai --version
```

#### Arch Linux
```bash
# Install from AUR
yay -S trust-ai

# Or build from source
git clone https://aur.archlinux.org/trust-ai.git
cd trust-ai
makepkg -si
```

### macOS

#### Using Homebrew (Recommended)
```bash
# Install Homebrew if not already installed
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install Trust AI
brew tap audit-brands/trust-ai
brew install trust-ai
```

#### Manual Installation
```bash
# Download and install
curl -L https://github.com/audit-brands/trust-ai/releases/latest/download/trust-ai-macos-$(uname -m).tar.gz | tar xz
sudo mv trust-ai /usr/local/bin/
```

### Windows

#### Using Chocolatey (Recommended)
```powershell
# Install Chocolatey if not already installed
Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))

# Install Trust AI
choco install trust-ai
```

#### Using Scoop
```powershell
# Install Scoop if not already installed
iwr -useb get.scoop.sh | iex

# Add bucket and install
scoop bucket add audit-brands https://github.com/audit-brands/scoop-bucket.git
scoop install trust-ai
```

#### Manual Installation
1. Download the Windows binary from [GitHub Releases](https://github.com/audit-brands/trust-ai/releases)
2. Extract to `C:\Program Files\TrustAI\`
3. Add to PATH environment variable
4. Open new Command Prompt or PowerShell
5. Test with `trust-ai --version`

## Troubleshooting

### Common Issues

#### Permission Denied
```bash
# Linux/macOS: Fix permissions
sudo chmod +x /usr/local/bin/trust-ai

# Or install to user directory
mkdir -p ~/.local/bin
mv trust-ai ~/.local/bin/
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

#### Command Not Found
```bash
# Check if binary is in PATH
which trust-ai

# Add to PATH if needed
echo 'export PATH="/usr/local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

#### SSL Certificate Issues
```bash
# Update certificates
sudo apt update && sudo apt install ca-certificates  # Ubuntu/Debian
sudo dnf update ca-certificates                      # Fedora
brew install ca-certificates                         # macOS
```

#### Rust Compilation Errors
```bash
# Update Rust toolchain
rustup update

# Clean and rebuild
cargo clean
cargo build --release
```

### Getting Help

#### Check Version and Build Info
```bash
trust-ai --version
trust-ai version --format json
```

#### Validate Installation
```bash
trust-ai config validate
trust-ai config test-connection
```

#### Enable Debug Logging
```bash
trust-ai --verbose chat "test message"
```

#### Check System Requirements
```bash
# Check available memory
free -h  # Linux
vm_stat  # macOS
systeminfo | findstr Memory  # Windows

# Check disk space
df -h    # Linux/macOS
dir      # Windows
```

## Updating Trust AI

### Update via Package Manager
```bash
# Homebrew
brew update && brew upgrade trust-ai

# Chocolatey
choco upgrade trust-ai

# APT
sudo apt update && sudo apt upgrade trust-ai

# DNF/YUM
sudo dnf update trust-ai
```

### Manual Update
```bash
# Download latest version
curl -L https://github.com/audit-brands/trust-ai/releases/latest/download/trust-ai-$(uname -s | tr '[:upper:]' '[:lower:]')-$(uname -m).tar.gz | tar xz

# Replace existing binary
sudo mv trust-ai /usr/local/bin/trust-ai

# Verify update
trust-ai --version
```

### Update from Source
```bash
cd trust-ai
git pull origin main
cargo install --path . --force
```

## Uninstallation

### Remove Binary
```bash
# Remove from standard location
sudo rm /usr/local/bin/trust-ai

# Remove via package manager
brew uninstall trust-ai          # Homebrew
choco uninstall trust-ai         # Chocolatey
sudo apt remove trust-ai         # APT
sudo dnf remove trust-ai         # DNF
```

### Remove Configuration and Data
```bash
# Remove configuration
rm -rf ~/.config/trust-ai

# Remove cache
rm -rf ~/.cache/trust-ai

# Remove logs
rm -rf ~/.local/share/trust-ai
```

For additional support, visit our [GitHub repository](https://github.com/audit-brands/trust-ai) or check the [FAQ](../troubleshooting/faq.md).