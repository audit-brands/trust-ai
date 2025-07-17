# CLI Reference Guide

## Overview
This guide provides comprehensive documentation for all Trust AI CLI commands, options, and usage patterns.

## Global Options

All commands support these global options:

```bash
--config, -c <PATH>     Use custom configuration file
--verbose, -v           Enable verbose output
--quiet, -q            Suppress non-essential output
--help, -h             Show help information
--version, -V          Show version information
```

## Commands

### `trust-ai chat`
Start interactive conversations with AI models.

#### Usage
```bash
trust-ai chat [OPTIONS] [MESSAGE]
```

#### Options
```bash
--continue              Continue previous conversation
--model <MODEL>         Override default model
--temperature <FLOAT>   Set response creativity (0.0-2.0)
--max-tokens <INT>      Maximum response length
--system <TEXT>         Set system prompt
--save <NAME>           Save conversation with name
--load <NAME>           Load saved conversation
--export <FORMAT>       Export conversation (json, markdown, txt)
```

#### Examples
```bash
# Interactive chat session
trust-ai chat

# Single message
trust-ai chat "Explain machine learning"

# Continue previous conversation
trust-ai chat --continue "Can you elaborate on that?"

# Use specific model with custom temperature
trust-ai chat --model gpt-4 --temperature 0.3 "Write a formal email"

# Set system prompt
trust-ai chat --system "You are a helpful coding assistant" "Help me debug this code"

# Save conversation
trust-ai chat --save "ml-discussion" "Explain neural networks"

# Load and continue saved conversation
trust-ai chat --load "ml-discussion" --continue "What about deep learning?"
```

### `trust-ai config`
Manage configuration settings.

#### Subcommands

##### `config show`
Display current configuration.

```bash
trust-ai config show [--format json|yaml|table]
```

##### `config set`
Set configuration values.

```bash
trust-ai config set <KEY> <VALUE>
```

**Available Keys:**
- `provider`: AI provider (openai, ollama, anthropic)
- `model`: Default model name
- `api_key`: API key for the provider
- `endpoint`: Custom API endpoint
- `temperature`: Default temperature (0.0-2.0)
- `max_tokens`: Default maximum tokens
- `timeout`: Request timeout in seconds
- `cache.enabled`: Enable/disable caching
- `cache.max_size`: Maximum cache entries
- `cache.ttl`: Cache time-to-live in seconds
- `monitoring.enabled`: Enable performance monitoring
- `monitoring.metrics_retention`: Metrics retention period

##### `config get`
Get specific configuration value.

```bash
trust-ai config get <KEY>
```

##### `config reset`
Reset configuration to defaults.

```bash
trust-ai config reset [--confirm]
```

#### Examples
```bash
# View all configuration
trust-ai config show

# Set provider and API key
trust-ai config set provider openai
trust-ai config set api_key sk-your-key-here

# Configure Ollama
trust-ai config set provider ollama
trust-ai config set endpoint http://localhost:11434
trust-ai config set model llama2

# Enable caching with custom settings
trust-ai config set cache.enabled true
trust-ai config set cache.max_size 2000
trust-ai config set cache.ttl 7200

# Get specific value
trust-ai config get model

# Reset to defaults
trust-ai config reset --confirm
```

### `trust-ai perf`
Performance monitoring and optimization commands.

#### Subcommands

##### `perf status`
Display performance dashboard.

```bash
trust-ai perf status [--format table|json] [--period 1h|1d|1w]
```

##### `perf cache`
Show cache performance metrics.

```bash
trust-ai perf cache [--clear] [--stats]
```

##### `perf monitor`
Real-time performance monitoring.

```bash
trust-ai perf monitor [--live] [--interval <SECONDS>] [--duration <MINUTES>]
```

##### `perf optimize`
Get optimization suggestions.

```bash
trust-ai perf optimize [--apply] [--dry-run]
```

##### `perf metrics`
Export performance metrics.

```bash
trust-ai perf metrics [--format csv|json] [--output <FILE>] [--period <DURATION>]
```

##### `perf benchmark`
Run performance benchmarks.

```bash
trust-ai perf benchmark [--iterations <COUNT>] [--model <MODEL>]
```

##### `perf profile`
Profile system performance.

```bash
trust-ai perf profile [--duration <SECONDS>] [--output <FILE>]
```

##### `perf report`
Generate performance reports.

```bash
trust-ai perf report [--format html|pdf|markdown] [--output <FILE>]
```

#### Examples
```bash
# View performance dashboard
trust-ai perf status

# Monitor cache performance
trust-ai perf cache --stats

# Real-time monitoring for 5 minutes
trust-ai perf monitor --live --duration 5

# Get optimization suggestions
trust-ai perf optimize

# Apply optimizations automatically
trust-ai perf optimize --apply

# Export metrics to CSV
trust-ai perf metrics --format csv --output metrics.csv --period 7d

# Run benchmark with 10 iterations
trust-ai perf benchmark --iterations 10

# Profile for 30 seconds
trust-ai perf profile --duration 30 --output profile.json

# Generate HTML performance report
trust-ai perf report --format html --output report.html
```

### `trust-ai init`
Initialize Trust AI configuration and setup.

```bash
trust-ai init [--force] [--config-dir <PATH>]
```

#### Options
```bash
--force              Overwrite existing configuration
--config-dir <PATH>  Custom configuration directory
```

### `trust-ai version`
Display version and build information.

```bash
trust-ai version [--format table|json]
```

### `trust-ai completions`
Generate shell completions.

```bash
trust-ai completions <SHELL>
```

**Supported Shells:** bash, zsh, fish, powershell

#### Examples
```bash
# Generate bash completions
trust-ai completions bash > ~/.bash_completion.d/trust-ai

# Generate zsh completions
trust-ai completions zsh > ~/.zsh/completions/_trust-ai

# Generate fish completions
trust-ai completions fish > ~/.config/fish/completions/trust-ai.fish
```

## Configuration File Format

The configuration file (`~/.config/trust-ai/config.yaml`) supports these options:

```yaml
# Provider settings
provider: openai              # openai, ollama, anthropic
model: gpt-4                 # Default model
api_key: your-api-key        # API key
endpoint: https://api.openai.com/v1  # Custom endpoint
timeout: 30                  # Request timeout (seconds)

# Response settings
temperature: 0.7             # Creativity (0.0-2.0)
max_tokens: 2048            # Maximum response length
system_prompt: ""           # Default system prompt

# Cache settings
cache:
  enabled: true             # Enable caching
  max_size: 1000           # Maximum cache entries
  ttl: 3600               # Time-to-live (seconds)

# Monitoring settings
monitoring:
  enabled: true            # Enable performance monitoring
  metrics_retention: 7d   # How long to keep metrics
  auto_optimize: false    # Automatic optimization

# UI settings
ui:
  color: auto             # auto, always, never
  progress: true          # Show progress indicators
  timestamps: false       # Show timestamps in output
```

## Environment Variables

Configuration can also be set via environment variables:

```bash
# Provider settings
TRUST_AI_PROVIDER=openai
TRUST_AI_MODEL=gpt-4
TRUST_AI_API_KEY=your-key
TRUST_AI_ENDPOINT=https://api.openai.com/v1
TRUST_AI_TIMEOUT=30

# Response settings
TRUST_AI_TEMPERATURE=0.7
TRUST_AI_MAX_TOKENS=2048
TRUST_AI_SYSTEM_PROMPT="You are a helpful assistant"

# Cache settings
TRUST_AI_CACHE_ENABLED=true
TRUST_AI_CACHE_MAX_SIZE=1000
TRUST_AI_CACHE_TTL=3600

# Monitoring settings
TRUST_AI_MONITORING_ENABLED=true
TRUST_AI_METRICS_RETENTION=7d
```

## Exit Codes

Trust AI uses standard exit codes:

- `0`: Success
- `1`: General error
- `2`: Configuration error
- `3`: Network error
- `4`: Authentication error
- `5`: Rate limit exceeded
- `6`: Invalid input
- `7`: File system error

## Tips and Best Practices

### Performance Optimization
- Enable caching for frequently used prompts
- Use appropriate temperature settings for your use case
- Monitor performance with `trust-ai perf status`
- Run periodic optimizations with `trust-ai perf optimize`

### Configuration Management
- Use environment variables for CI/CD environments
- Keep sensitive data like API keys in environment variables
- Use different configuration files for different projects

### Troubleshooting
- Use `--verbose` flag for detailed output
- Check configuration with `trust-ai config show`
- Monitor performance with `trust-ai perf monitor`
- Clear cache if experiencing issues: `trust-ai perf cache --clear`

For more detailed information, see the [Configuration Guide](configuration.md) and [Best Practices](best-practices.md).