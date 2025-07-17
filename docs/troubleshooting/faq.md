# Frequently Asked Questions (FAQ)

## General Questions

### What is Trust AI?
Trust AI is a high-performance command-line interface for interacting with AI models. It features advanced performance optimization, intelligent caching, comprehensive monitoring, and support for multiple AI providers including OpenAI, Ollama, and Anthropic.

### What makes Trust AI different from other AI CLIs?
- **Performance Optimization**: 80% faster model loading with intelligent caching
- **Real-time Monitoring**: Comprehensive performance metrics and optimization suggestions
- **Multi-provider Support**: Works with OpenAI, Ollama, Anthropic, and more
- **Advanced Caching**: LRU cache with compression and smart eviction
- **Production Ready**: Built with Rust for reliability and performance

### Is Trust AI free to use?
Yes, Trust AI is open-source and free to use. However, you'll need API keys for cloud providers like OpenAI or Anthropic, which may have their own pricing.

## Installation and Setup

### How do I install Trust AI?
See our comprehensive [Installation Guide](../deployment/installation.md) for platform-specific instructions. The quickest method is using pre-built binaries:

```bash
curl -L https://github.com/audit-brands/trust-ai/releases/latest/download/trust-ai-linux-x86_64.tar.gz | tar xz
sudo mv trust-ai /usr/local/bin/
```

### Where is the configuration file located?
- **Linux/macOS**: `~/.config/trust-ai/config.yaml`
- **Windows**: `%APPDATA%\trust-ai\config.yaml`

### How do I get started after installation?
1. Initialize Trust AI: `trust-ai init`
2. Configure your provider: `trust-ai config set provider openai`
3. Set your API key: `trust-ai config set api_key sk-your-key`
4. Test it: `trust-ai chat "Hello!"`

### Can I use Trust AI without an internet connection?
Yes, if you use Ollama with local models. Install Ollama, pull a model like `llama2`, and configure Trust AI to use it:

```bash
trust-ai config set provider ollama
trust-ai config set endpoint http://localhost:11434
trust-ai config set model llama2
```

## Configuration

### How do I switch between different AI providers?
```bash
# Switch to OpenAI
trust-ai config set provider openai
trust-ai config set api_key sk-your-openai-key

# Switch to Ollama
trust-ai config set provider ollama
trust-ai config set endpoint http://localhost:11434

# Switch to Anthropic
trust-ai config set provider anthropic
trust-ai config set api_key sk-ant-your-key
```

### Can I use different models for different tasks?
Yes, you can override the model for specific commands:

```bash
# Use GPT-4 for complex analysis
trust-ai chat --model gpt-4 "Analyze this complex problem..."

# Use GPT-3.5 for simple tasks
trust-ai chat --model gpt-3.5-turbo "Summarize this text..."

# Use local model for privacy-sensitive tasks
trust-ai chat --model codellama "Review this code..."
```

### How do I set up multiple configurations?
Use configuration profiles:

```bash
# Create profiles
trust-ai config profile create work
trust-ai config profile create personal

# Configure work profile
trust-ai config set --profile work provider openai
trust-ai config set --profile work model gpt-4

# Use specific profile
trust-ai --profile work chat "Work-related question"
```

### Can I use environment variables instead of the config file?
Yes, all configuration options can be set via environment variables:

```bash
export TRUST_AI_PROVIDER=openai
export TRUST_AI_API_KEY=sk-your-key
export TRUST_AI_MODEL=gpt-4
export TRUST_AI_TEMPERATURE=0.7
```

## Performance and Caching

### How does the caching system work?
Trust AI uses an intelligent LRU (Least Recently Used) cache that:
- Stores responses to avoid repeated API calls
- Compresses data to save space
- Automatically evicts old entries
- Provides 80% faster response times for cached queries

### How do I monitor cache performance?
```bash
# View cache statistics
trust-ai perf cache --stats

# Monitor real-time performance
trust-ai perf monitor --live

# View performance dashboard
trust-ai perf status
```

### How do I clear the cache?
```bash
# Clear all cache
trust-ai perf cache --clear

# Or manually delete cache directory
rm -rf ~/.cache/trust-ai
```

### Why is my first request slow but subsequent ones fast?
This is normal behavior. The first request:
1. Loads the model (if using local providers)
2. Establishes connections
3. Initializes caching

Subsequent requests benefit from:
- Cached responses
- Persistent connections
- Optimized model loading

### How do I optimize performance?
```bash
# Get optimization suggestions
trust-ai perf optimize

# Apply optimizations automatically
trust-ai perf optimize --apply

# Enable auto-optimization
trust-ai config set monitoring.auto_optimize true
```

## Troubleshooting

### I'm getting "Authentication failed" errors
1. Check your API key: `trust-ai config get api_key`
2. Verify the key is valid for your provider
3. Ensure the key has necessary permissions
4. Try setting the key again: `trust-ai config set api_key sk-new-key`

### The command "trust-ai" is not found
1. Check if it's in your PATH: `which trust-ai`
2. Add to PATH if needed:
   ```bash
   echo 'export PATH="/usr/local/bin:$PATH"' >> ~/.bashrc
   source ~/.bashrc
   ```
3. Verify installation: `ls -la /usr/local/bin/trust-ai`

### I'm getting network/connection errors
1. Test your internet connection
2. Check if you're behind a proxy:
   ```bash
   export HTTPS_PROXY=https://proxy.company.com:8080
   ```
3. Verify the endpoint: `trust-ai config get endpoint`
4. Test connection: `trust-ai config test-connection`

### Ollama models are not working
1. Ensure Ollama is running: `ollama serve`
2. Check if the model is installed: `ollama list`
3. Pull the model if needed: `ollama pull llama2`
4. Verify endpoint: `trust-ai config get endpoint`
5. Test with curl:
   ```bash
   curl http://localhost:11434/api/generate -d '{"model":"llama2","prompt":"test"}'
   ```

### Cache is not working or causing issues
1. Check cache status: `trust-ai perf cache --stats`
2. Clear corrupted cache: `trust-ai perf cache --clear`
3. Check permissions: `ls -la ~/.cache/trust-ai`
4. Disable cache temporarily: `trust-ai config set cache.enabled false`

### High memory usage
1. Reduce cache size: `trust-ai config set cache.max_size 500`
2. Enable compression: `trust-ai config set cache.compression true`
3. Reduce TTL: `trust-ai config set cache.ttl 1800`
4. Monitor usage: `trust-ai perf monitor --live`

### Slow response times
1. Check performance metrics: `trust-ai perf status`
2. Optimize settings: `trust-ai perf optimize --apply`
3. Use faster models: `trust-ai config set model gpt-3.5-turbo`
4. Increase timeout: `trust-ai config set timeout 60`

## Usage Questions

### How do I have a conversation with context?
```bash
# Start interactive mode
trust-ai chat

# Or use conversation continuity
trust-ai chat "What is machine learning?"
trust-ai chat --continue "Can you give me examples?"
trust-ai chat --continue "What about deep learning?"
```

### Can I save and load conversations?
```bash
# Save a conversation
trust-ai chat --save "ml-discussion" "Explain neural networks"

# Load and continue
trust-ai chat --load "ml-discussion" --continue "What about CNNs?"

# Export conversation
trust-ai chat --load "ml-discussion" --export json > conversation.json
```

### How do I use Trust AI for code generation?
```bash
# Use lower temperature for code
trust-ai chat --temperature 0.2 "Write a Python function to sort a list"

# Use code-specific models
trust-ai chat --model codellama "Debug this Python code: [paste code]"

# Set system prompt for coding
trust-ai chat --system "You are an expert programmer" "Help me with this algorithm"
```

### Can I use Trust AI in scripts?
```bash
#!/bin/bash
# Example script
RESPONSE=$(trust-ai chat "Generate a random password" --format plain)
echo "Generated password: $RESPONSE"

# Or process files
for file in *.py; do
    trust-ai chat "Review this code: $(cat $file)" > "review-$file.md"
done
```

### How do I integrate Trust AI with my development workflow?
See our [Best Practices Guide](../user-guide/best-practices.md) for:
- CI/CD integration
- Pre-commit hooks
- IDE integration
- Team collaboration

## API and Development

### Can I use Trust AI as a library in my Rust project?
Yes! Add to your `Cargo.toml`:

```toml
[dependencies]
trust-ai = "0.1.0"
```

See our [API Documentation](../api/overview.md) for usage examples.

### How do I create a custom provider?
Implement the `Provider` trait:

```rust
use trust_ai::Provider;

#[derive(Debug)]
pub struct MyCustomProvider;

#[async_trait]
impl Provider for MyCustomProvider {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        // Your implementation
    }
    // ... other methods
}
```

### Can I extend Trust AI with plugins?
Currently, Trust AI doesn't have a plugin system, but you can:
1. Fork the repository and add features
2. Create wrapper scripts
3. Use the Rust API to build custom applications

## Security and Privacy

### Is my data secure?
- API keys are stored locally in configuration files
- Conversations are cached locally (can be disabled)
- No data is sent to Trust AI servers
- All communication is with your chosen AI provider

### How do I ensure my API keys are secure?
1. Use environment variables: `export TRUST_AI_API_KEY=sk-key`
2. Set proper file permissions: `chmod 600 ~/.config/trust-ai/config.yaml`
3. Don't commit config files to version control
4. Rotate keys regularly

### Can I use Trust AI in air-gapped environments?
Yes, with Ollama and local models:
1. Install Ollama offline
2. Transfer model files manually
3. Configure Trust AI to use local endpoint
4. Disable monitoring/telemetry if needed

## Performance Benchmarks

### What performance improvements can I expect?
- **80% faster model loading** with caching enabled
- **50%+ reduction in API calls** with intelligent caching
- **Real-time optimization** suggestions
- **Sub-second responses** for cached queries

### How does Trust AI compare to other tools?
Trust AI is optimized for:
- **Speed**: Advanced caching and optimization
- **Reliability**: Built with Rust for stability
- **Monitoring**: Comprehensive performance metrics
- **Flexibility**: Multiple providers and models

## Contributing and Support

### How can I contribute to Trust AI?
1. Report bugs on [GitHub Issues](https://github.com/audit-brands/trust-ai/issues)
2. Submit feature requests
3. Contribute code via pull requests
4. Improve documentation
5. Share usage examples

### Where can I get help?
1. Check this FAQ
2. Read the [documentation](../user-guide/getting-started.md)
3. Search [GitHub Issues](https://github.com/audit-brands/trust-ai/issues)
4. Create a new issue if needed

### How do I report a bug?
1. Check if it's already reported
2. Gather information:
   ```bash
   trust-ai --version
   trust-ai config show
   trust-ai --verbose [command that fails]
   ```
3. Create a GitHub issue with details

### Can I request new features?
Yes! Create a feature request on GitHub Issues with:
- Use case description
- Expected behavior
- Current workarounds (if any)
- Willingness to contribute

## Roadmap and Future

### What's planned for future versions?
- Plugin system
- More AI providers
- Enhanced monitoring
- Web interface
- Team collaboration features

### How often is Trust AI updated?
- Bug fixes: As needed
- Minor releases: Monthly
- Major releases: Quarterly

### Is Trust AI production-ready?
Yes! Trust AI is built with production use in mind:
- Comprehensive error handling
- Performance monitoring
- Extensive testing
- Stable API
- Active maintenance

---

**Still have questions?** Check our [GitHub repository](https://github.com/audit-brands/trust-ai) or create an issue for support.