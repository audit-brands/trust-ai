# Trust AI Beta Installation Guide

## Quick Install (Recommended for Beta)

Trust AI beta can be installed directly from source using curl:

```bash
# One-line installation
curl -sSL https://raw.githubusercontent.com/audit-brands/trust-ai/main/curl-install.sh | bash
```

This will:
1. Install Rust if not already present
2. Download Trust AI source code
3. Build from source (takes 2-5 minutes)
4. Install to `/usr/local/bin/trust-ai`
5. Create `trust` alias for convenience

## Manual Installation

If you prefer to install manually:

```bash
# 1. Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 2. Clone and build Trust AI
git clone https://github.com/audit-brands/trust-ai.git
cd trust-ai
cargo build --release

# 3. Install the binary
sudo cp target/release/trust /usr/local/bin/trust-ai
sudo chmod +x /usr/local/bin/trust-ai
sudo ln -sf /usr/local/bin/trust-ai /usr/local/bin/trust
```

## First Time Setup

After installation, configure Trust AI:

```bash
# Initialize configuration
trust-ai init

# Configure OpenAI (example)
trust-ai config set provider openai
trust-ai config set api_key sk-your-api-key-here
trust-ai config set model gpt-4

# Test the installation
trust-ai chat "Hello! Are you working correctly?"
```

## Alternative Providers

### Ollama (Local AI)
```bash
# Install Ollama first: https://ollama.ai
trust-ai config set provider ollama
trust-ai config set model llama2
trust-ai config set base_url http://localhost:11434
```

### Anthropic Claude
```bash
trust-ai config set provider anthropic
trust-ai config set api_key your-anthropic-key
trust-ai config set model claude-3-opus-20240229
```

## Verification

Verify your installation:

```bash
# Check version
trust-ai --version

# View configuration
trust-ai config show

# Test chat
trust-ai chat "What is 2+2?"

# Check performance
trust-ai perf status
```

## Troubleshooting

### Rust Installation Issues
If Rust installation fails:
```bash
# Manual Rust installation
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### Build Issues
If compilation fails:
```bash
# Update Rust
rustup update

# Clean build
cargo clean
cargo build --release
```

### Permission Issues
If you get permission errors:
```bash
# Install to user directory instead
cargo install --path . --root ~/.local
export PATH="$HOME/.local/bin:$PATH"
```

## Beta Limitations

This beta release:
- ✅ Builds from source (no pre-built binaries)
- ✅ Uses curl for easy installation
- ✅ Full feature set available
- ⚠️ Longer installation time (2-5 minutes)
- ⚠️ Requires Rust toolchain
- ⚠️ May have rough edges

## Getting Help

- View all commands: `trust-ai --help`
- Configuration help: `trust-ai config --help`
- Performance monitoring: `trust-ai perf --help`
- Report issues: https://github.com/audit-brands/trust-ai/issues

## Next Steps

Once installed, explore Trust AI's capabilities:

1. **Interactive Chat**: `trust-ai chat`
2. **Performance Monitoring**: `trust-ai perf status`
3. **Configuration**: `trust-ai config show`
4. **Help System**: `trust-ai --help`