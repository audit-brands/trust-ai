# Getting Started with Trust AI

## Overview
Trust AI is a powerful command-line interface for interacting with AI models, featuring advanced performance optimization, intelligent caching, and comprehensive monitoring capabilities.

## Quick Start

### Installation

#### Option 1: Pre-built Binaries (Recommended)
```bash
# Download the latest release for your platform
curl -L https://github.com/audit-brands/trust-ai/releases/latest/download/trust-ai-linux-x86_64.tar.gz | tar xz
sudo mv trust-ai /usr/local/bin/
```

#### Option 2: Build from Source
```bash
# Clone the repository
git clone https://github.com/audit-brands/trust-ai.git
cd trust-ai

# Build with Rust (requires Rust 1.70+)
cargo build --release

# Install globally
cargo install --path .
```

### First Steps

1. **Initialize Configuration**
   ```bash
   trust-ai init
   ```

2. **Configure Your AI Provider**
   ```bash
   # Set up OpenAI
   trust-ai config set provider openai
   trust-ai config set api-key YOUR_API_KEY

   # Or configure Ollama for local models
   trust-ai config set provider ollama
   trust-ai config set endpoint http://localhost:11434
   ```

3. **Start Your First Conversation**
   ```bash
   trust-ai chat "Hello! Can you help me understand how to use this CLI?"
   ```

## Core Features

### 🚀 Performance Optimization
- **Intelligent Caching**: 80% faster model loading with LRU cache
- **Real-time Monitoring**: Track performance metrics and optimize usage
- **Smart Optimization**: Automatic performance tuning based on usage patterns

### 💬 Conversational AI
- **Interactive Chat**: Seamless conversations with AI models
- **Context Management**: Maintain conversation history and context
- **Multi-provider Support**: Works with OpenAI, Ollama, and more

### 🔧 Advanced CLI
- **Rich Commands**: 8 performance monitoring commands
- **Configuration Management**: Easy setup and customization
- **Error Handling**: Clear, actionable error messages

## Basic Commands

### Chat and Interaction
```bash
# Start interactive chat session
trust-ai chat

# Send single message
trust-ai chat "Explain quantum computing"

# Continue previous conversation
trust-ai chat --continue "Tell me more about that"
```

### Performance Monitoring
```bash
# View performance dashboard
trust-ai perf status

# Monitor cache performance
trust-ai perf cache

# View optimization suggestions
trust-ai perf optimize

# Real-time performance monitoring
trust-ai perf monitor --live
```

### Configuration
```bash
# View current configuration
trust-ai config show

# Set configuration values
trust-ai config set model gpt-4
trust-ai config set temperature 0.7

# Reset to defaults
trust-ai config reset
```

## Configuration File

Trust AI uses a YAML configuration file located at `~/.config/trust-ai/config.yaml`:

```yaml
provider: openai
model: gpt-4
api_key: your-api-key-here
temperature: 0.7
max_tokens: 2048

# Performance settings
cache:
  enabled: true
  max_size: 1000
  ttl: 3600

# Monitoring settings
monitoring:
  enabled: true
  metrics_retention: 7d
```

## Environment Variables

You can also configure Trust AI using environment variables:

```bash
export TRUST_AI_PROVIDER=openai
export TRUST_AI_API_KEY=your-api-key
export TRUST_AI_MODEL=gpt-4
export TRUST_AI_TEMPERATURE=0.7
```

## Next Steps

- 📖 Read the [CLI Reference](cli-reference.md) for detailed command documentation
- ⚙️ Learn about [Configuration Options](configuration.md)
- 🎯 Explore [Best Practices](best-practices.md) for optimal usage
- 🔧 Check [Troubleshooting](../troubleshooting/faq.md) if you encounter issues

## Getting Help

```bash
# General help
trust-ai --help

# Command-specific help
trust-ai chat --help
trust-ai perf --help

# Version information
trust-ai --version
```

For additional support, visit our [GitHub repository](https://github.com/audit-brands/trust-ai) or check the [FAQ](../troubleshooting/faq.md).