# Configuration Guide

## Overview
Trust AI provides flexible configuration options through configuration files, environment variables, and command-line arguments. This guide covers all configuration aspects in detail.

## Configuration Hierarchy

Trust AI uses the following configuration precedence (highest to lowest):

1. **Command-line arguments** (highest priority)
2. **Environment variables**
3. **Configuration file**
4. **Default values** (lowest priority)

## Configuration File

### Location
The configuration file is located at:
- **Linux/macOS**: `~/.config/trust-ai/config.yaml`
- **Windows**: `%APPDATA%\trust-ai\config.yaml`

You can specify a custom location with the `--config` flag or `TRUST_AI_CONFIG` environment variable.

### Format
The configuration file uses YAML format:

```yaml
# Provider Configuration
provider: openai
model: gpt-4
api_key: sk-your-api-key-here
endpoint: https://api.openai.com/v1
timeout: 30

# Response Settings
temperature: 0.7
max_tokens: 2048
system_prompt: "You are a helpful AI assistant."

# Cache Configuration
cache:
  enabled: true
  max_size: 1000
  ttl: 3600
  storage_path: ~/.cache/trust-ai

# Performance Monitoring
monitoring:
  enabled: true
  metrics_retention: 7d
  auto_optimize: false
  report_interval: 1h

# UI Settings
ui:
  color: auto
  progress: true
  timestamps: false
  theme: default

# Logging
logging:
  level: info
  file: ~/.local/share/trust-ai/logs/trust-ai.log
  max_size: 10MB
  max_files: 5
```

## Configuration Sections

### Provider Configuration

#### OpenAI
```yaml
provider: openai
api_key: sk-your-api-key
model: gpt-4
endpoint: https://api.openai.com/v1  # Optional, uses default if not specified
organization: org-your-org-id        # Optional
```

**Supported Models:**
- `gpt-4`: Most capable model
- `gpt-4-turbo`: Faster GPT-4 variant
- `gpt-3.5-turbo`: Fast and cost-effective
- `gpt-3.5-turbo-16k`: Extended context version

#### Ollama (Local Models)
```yaml
provider: ollama
endpoint: http://localhost:11434
model: llama2
timeout: 60  # Longer timeout for local models
```

**Popular Ollama Models:**
- `llama2`: Meta's Llama 2 model
- `codellama`: Code-specialized Llama model
- `mistral`: Mistral 7B model
- `neural-chat`: Intel's neural chat model

#### Anthropic
```yaml
provider: anthropic
api_key: sk-ant-your-key
model: claude-3-opus-20240229
endpoint: https://api.anthropic.com/v1
```

**Supported Models:**
- `claude-3-opus-20240229`: Most capable Claude model
- `claude-3-sonnet-20240229`: Balanced performance
- `claude-3-haiku-20240307`: Fast and efficient

### Response Settings

```yaml
# Creativity and randomness (0.0 = deterministic, 2.0 = very creative)
temperature: 0.7

# Maximum response length in tokens
max_tokens: 2048

# Default system prompt for all conversations
system_prompt: "You are a helpful AI assistant specialized in software development."

# Stop sequences to end generation
stop_sequences:
  - "Human:"
  - "Assistant:"

# Frequency penalty to reduce repetition (-2.0 to 2.0)
frequency_penalty: 0.0

# Presence penalty to encourage topic diversity (-2.0 to 2.0)
presence_penalty: 0.0
```

### Cache Configuration

```yaml
cache:
  # Enable/disable caching
  enabled: true
  
  # Maximum number of cached entries
  max_size: 1000
  
  # Time-to-live for cache entries (in seconds)
  ttl: 3600
  
  # Cache storage location
  storage_path: ~/.cache/trust-ai
  
  # Cache strategy: lru, lfu, fifo
  strategy: lru
  
  # Compress cached data
  compression: true
  
  # Cache hit threshold for optimization suggestions
  hit_threshold: 0.8
```

### Performance Monitoring

```yaml
monitoring:
  # Enable performance monitoring
  enabled: true
  
  # How long to retain metrics
  metrics_retention: 7d
  
  # Automatically apply optimizations
  auto_optimize: false
  
  # Interval for generating reports
  report_interval: 1h
  
  # Metrics to collect
  metrics:
    - response_time
    - cache_hit_rate
    - token_usage
    - error_rate
    - throughput
  
  # Performance thresholds
  thresholds:
    response_time_warning: 5s
    response_time_critical: 10s
    cache_hit_rate_warning: 0.6
    error_rate_critical: 0.1
```

### UI Settings

```yaml
ui:
  # Color output: auto, always, never
  color: auto
  
  # Show progress indicators
  progress: true
  
  # Show timestamps in output
  timestamps: false
  
  # UI theme: default, dark, light, minimal
  theme: default
  
  # Output format for structured data
  default_format: table
  
  # Paging for long output
  pager: auto
  
  # Editor for multi-line input
  editor: $EDITOR
```

### Logging Configuration

```yaml
logging:
  # Log level: trace, debug, info, warn, error
  level: info
  
  # Log file location
  file: ~/.local/share/trust-ai/logs/trust-ai.log
  
  # Maximum log file size
  max_size: 10MB
  
  # Number of log files to keep
  max_files: 5
  
  # Log format: json, text
  format: text
  
  # Include timestamps
  timestamps: true
  
  # Log to stderr in addition to file
  stderr: false
```

## Environment Variables

All configuration options can be set via environment variables using the `TRUST_AI_` prefix:

### Provider Settings
```bash
export TRUST_AI_PROVIDER=openai
export TRUST_AI_MODEL=gpt-4
export TRUST_AI_API_KEY=sk-your-key
export TRUST_AI_ENDPOINT=https://api.openai.com/v1
export TRUST_AI_TIMEOUT=30
```

### Response Settings
```bash
export TRUST_AI_TEMPERATURE=0.7
export TRUST_AI_MAX_TOKENS=2048
export TRUST_AI_SYSTEM_PROMPT="You are a helpful assistant"
export TRUST_AI_FREQUENCY_PENALTY=0.0
export TRUST_AI_PRESENCE_PENALTY=0.0
```

### Cache Settings
```bash
export TRUST_AI_CACHE_ENABLED=true
export TRUST_AI_CACHE_MAX_SIZE=1000
export TRUST_AI_CACHE_TTL=3600
export TRUST_AI_CACHE_STORAGE_PATH=~/.cache/trust-ai
export TRUST_AI_CACHE_STRATEGY=lru
```

### Monitoring Settings
```bash
export TRUST_AI_MONITORING_ENABLED=true
export TRUST_AI_METRICS_RETENTION=7d
export TRUST_AI_AUTO_OPTIMIZE=false
export TRUST_AI_REPORT_INTERVAL=1h
```

### UI Settings
```bash
export TRUST_AI_COLOR=auto
export TRUST_AI_PROGRESS=true
export TRUST_AI_TIMESTAMPS=false
export TRUST_AI_THEME=default
```

## Configuration Management Commands

### View Configuration
```bash
# Show all configuration
trust-ai config show

# Show in JSON format
trust-ai config show --format json

# Show specific section
trust-ai config show --section cache
```

### Set Configuration
```bash
# Set single value
trust-ai config set provider openai
trust-ai config set api_key sk-your-key

# Set nested value
trust-ai config set cache.enabled true
trust-ai config set monitoring.metrics_retention 14d
```

### Get Configuration
```bash
# Get single value
trust-ai config get model

# Get nested value
trust-ai config get cache.max_size
```

### Reset Configuration
```bash
# Reset all to defaults
trust-ai config reset --confirm

# Reset specific section
trust-ai config reset --section cache
```

## Configuration Profiles

You can maintain multiple configuration profiles for different use cases:

### Creating Profiles
```bash
# Create development profile
trust-ai config profile create development
trust-ai config set --profile development provider ollama
trust-ai config set --profile development model codellama

# Create production profile
trust-ai config profile create production
trust-ai config set --profile production provider openai
trust-ai config set --profile production model gpt-4
```

### Using Profiles
```bash
# Use specific profile
trust-ai --profile development chat "Debug this code"

# Set default profile
trust-ai config profile set-default production

# List profiles
trust-ai config profile list
```

## Security Considerations

### API Key Management
- **Never commit API keys to version control**
- Use environment variables in CI/CD environments
- Consider using secret management tools in production
- Rotate API keys regularly

### File Permissions
```bash
# Secure configuration file
chmod 600 ~/.config/trust-ai/config.yaml

# Secure cache directory
chmod 700 ~/.cache/trust-ai
```

### Network Security
- Use HTTPS endpoints only
- Verify SSL certificates
- Consider using proxy settings if required

## Validation and Troubleshooting

### Validate Configuration
```bash
# Check configuration validity
trust-ai config validate

# Test provider connection
trust-ai config test-connection
```

### Common Issues

#### Invalid API Key
```bash
# Error: Authentication failed
# Solution: Check API key
trust-ai config get api_key
trust-ai config set api_key sk-correct-key
```

#### Cache Issues
```bash
# Error: Cache permission denied
# Solution: Fix permissions
chmod 700 ~/.cache/trust-ai

# Clear corrupted cache
trust-ai perf cache --clear
```

#### Network Issues
```bash
# Error: Connection timeout
# Solution: Increase timeout
trust-ai config set timeout 60

# Check endpoint
trust-ai config get endpoint
```

## Best Practices

### Development Environment
```yaml
provider: ollama
model: codellama
temperature: 0.3
cache:
  enabled: true
  max_size: 500
monitoring:
  enabled: true
  auto_optimize: true
```

### Production Environment
```yaml
provider: openai
model: gpt-4
temperature: 0.7
timeout: 30
cache:
  enabled: true
  max_size: 2000
  ttl: 7200
monitoring:
  enabled: true
  metrics_retention: 30d
logging:
  level: warn
  max_files: 10
```

### Performance-Focused Setup
```yaml
cache:
  enabled: true
  max_size: 5000
  strategy: lru
  compression: true
monitoring:
  enabled: true
  auto_optimize: true
  thresholds:
    response_time_warning: 3s
    cache_hit_rate_warning: 0.8
```

For more information, see the [CLI Reference](cli-reference.md) and [Best Practices](best-practices.md) guides.