# Basic Chat Example

This example demonstrates the basic usage of Trust AI's chat functionality.

## Simple Chat

```rust
use trust_ai::{TrustAI, ChatRequest, Message, Role};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client with default configuration
    let client = TrustAI::new()?;
    
    // Create a simple chat request
    let request = ChatRequest {
        messages: vec![
            Message {
                role: Role::User,
                content: "Hello! Can you explain what Rust is?".to_string(),
                timestamp: chrono::Utc::now(),
            }
        ],
        model: None, // Use default model
        temperature: Some(0.7),
        max_tokens: Some(1000),
        stop_sequences: None,
    };
    
    // Send the request
    let response = client.chat(request).await?;
    
    // Print the response
    println!("AI Response: {}", response.message.content);
    println!("Tokens used: {}", response.usage.total_tokens);
    println!("Model: {}", response.model);
    
    Ok(())
}
```

## Chat with Configuration

```rust
use trust_ai::{TrustAI, Config, ProviderConfig, ProviderType, ChatRequest, Message, Role};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create custom configuration
    let config = Config {
        provider: ProviderConfig {
            provider_type: ProviderType::OpenAI,
            model: "gpt-4".to_string(),
            api_key: Some("sk-your-api-key".to_string()),
            endpoint: None, // Use default
            timeout: Duration::from_secs(30),
        },
        cache: Default::default(),
        monitoring: Default::default(),
        ui: Default::default(),
    };
    
    // Create client with custom configuration
    let client = TrustAI::with_config(config)?;
    
    // Create chat request
    let request = ChatRequest {
        messages: vec![
            Message {
                role: Role::System,
                content: "You are a helpful programming assistant.".to_string(),
                timestamp: chrono::Utc::now(),
            },
            Message {
                role: Role::User,
                content: "Write a simple 'Hello, World!' program in Rust.".to_string(),
                timestamp: chrono::Utc::now(),
            }
        ],
        model: Some("gpt-4".to_string()),
        temperature: Some(0.3), // Lower temperature for code generation
        max_tokens: Some(500),
        stop_sequences: None,
    };
    
    // Send the request
    let response = client.chat(request).await?;
    
    println!("Code generated:");
    println!("{}", response.message.content);
    
    Ok(())
}
```

## Multi-turn Conversation

```rust
use trust_ai::{TrustAI, ChatRequest, Message, Role};
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = TrustAI::new()?;
    let mut conversation_history = Vec::new();
    
    // Add system message
    conversation_history.push(Message {
        role: Role::System,
        content: "You are a helpful assistant. Keep responses concise.".to_string(),
        timestamp: chrono::Utc::now(),
    });
    
    loop {
        // Get user input
        print!("You: ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        if input.is_empty() || input == "quit" {
            break;
        }
        
        // Add user message to history
        conversation_history.push(Message {
            role: Role::User,
            content: input.to_string(),
            timestamp: chrono::Utc::now(),
        });
        
        // Create request with full conversation history
        let request = ChatRequest {
            messages: conversation_history.clone(),
            model: None,
            temperature: Some(0.7),
            max_tokens: Some(1000),
            stop_sequences: None,
        };
        
        // Send request
        match client.chat(request).await {
            Ok(response) => {
                println!("AI: {}", response.message.content);
                
                // Add AI response to history
                conversation_history.push(response.message);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
    
    println!("Goodbye!");
    Ok(())
}
```

## Error Handling

```rust
use trust_ai::{TrustAI, ChatRequest, Message, Role, TrustAIError};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = TrustAI::new()?;
    
    let request = ChatRequest {
        messages: vec![
            Message {
                role: Role::User,
                content: "Hello!".to_string(),
                timestamp: chrono::Utc::now(),
            }
        ],
        model: None,
        temperature: Some(0.7),
        max_tokens: Some(100),
        stop_sequences: None,
    };
    
    match client.chat(request).await {
        Ok(response) => {
            println!("Success: {}", response.message.content);
        }
        Err(TrustAIError::Authentication) => {
            eprintln!("Authentication failed. Please check your API key.");
        }
        Err(TrustAIError::RateLimit) => {
            eprintln!("Rate limit exceeded. Please wait before trying again.");
        }
        Err(TrustAIError::Network(e)) => {
            eprintln!("Network error: {}. Please check your connection.", e);
        }
        Err(e) => {
            eprintln!("Unexpected error: {}", e);
        }
    }
    
    Ok(())
}
```

## Using Environment Variables

```rust
use trust_ai::{TrustAI, Config, ProviderConfig, ProviderType, ChatRequest, Message, Role};
use std::env;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration from environment variables
    let config = Config {
        provider: ProviderConfig {
            provider_type: match env::var("TRUST_AI_PROVIDER")?.as_str() {
                "openai" => ProviderType::OpenAI,
                "ollama" => ProviderType::Ollama,
                "anthropic" => ProviderType::Anthropic,
                _ => ProviderType::OpenAI, // Default
            },
            model: env::var("TRUST_AI_MODEL").unwrap_or_else(|_| "gpt-3.5-turbo".to_string()),
            api_key: env::var("TRUST_AI_API_KEY").ok(),
            endpoint: env::var("TRUST_AI_ENDPOINT").ok(),
            timeout: Duration::from_secs(
                env::var("TRUST_AI_TIMEOUT")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()
                    .unwrap_or(30)
            ),
        },
        cache: Default::default(),
        monitoring: Default::default(),
        ui: Default::default(),
    };
    
    let client = TrustAI::with_config(config)?;
    
    let request = ChatRequest {
        messages: vec![
            Message {
                role: Role::User,
                content: "What's the weather like?".to_string(),
                timestamp: chrono::Utc::now(),
            }
        ],
        model: None,
        temperature: Some(0.7),
        max_tokens: Some(200),
        stop_sequences: None,
    };
    
    let response = client.chat(request).await?;
    println!("Response: {}", response.message.content);
    
    Ok(())
}
```

## Async with Tokio Runtime

```rust
use trust_ai::{TrustAI, ChatRequest, Message, Role};
use tokio::time::{timeout, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = TrustAI::new()?;
    
    let request = ChatRequest {
        messages: vec![
            Message {
                role: Role::User,
                content: "Tell me a short joke".to_string(),
                timestamp: chrono::Utc::now(),
            }
        ],
        model: None,
        temperature: Some(0.9), // Higher temperature for creativity
        max_tokens: Some(100),
        stop_sequences: None,
    };
    
    // Set a timeout for the request
    match timeout(Duration::from_secs(10), client.chat(request)).await {
        Ok(Ok(response)) => {
            println!("Joke: {}", response.message.content);
        }
        Ok(Err(e)) => {
            eprintln!("Chat error: {}", e);
        }
        Err(_) => {
            eprintln!("Request timed out after 10 seconds");
        }
    }
    
    Ok(())
}
```

## Building and Running

Add this to your `Cargo.toml`:

```toml
[dependencies]
trust-ai = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
chrono = { version = "0.4", features = ["serde"] }
```

Set your environment variables:

```bash
export TRUST_AI_PROVIDER=openai
export TRUST_AI_API_KEY=sk-your-api-key-here
export TRUST_AI_MODEL=gpt-3.5-turbo
```

Run the example:

```bash
cargo run --example basic_chat
```

## Next Steps

- Check out the [Performance Monitoring Example](performance_monitoring.md)
- Learn about [Custom Providers](custom_provider.md)
- Explore [Batch Processing](batch_processing.md)
- See [Web Service Integration](web_service.md)