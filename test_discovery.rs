use anyhow::Result;
use forge_provider::config::local_ai::LocalAiConfig;
use forge_provider::discovery::ModelDiscoveryService;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("debug")
        .init();

    println!("Testing ModelDiscoveryService initialization...");

    // Create a default Ollama config
    let local_config = LocalAiConfig::with_default_ollama();
    println!("Created LocalAiConfig: {:?}", local_config);

    // Try to create the discovery service
    match ModelDiscoveryService::new(local_config).await {
        Ok(mut discovery) => {
            println!("✅ ModelDiscoveryService created successfully!");
            
            // Try to discover models
            match discovery.discover_all_models().await {
                Ok(result) => {
                    println!("✅ Model discovery completed successfully!");
                    println!("Total models: {}", result.total_models);
                    println!("Available models: {}", result.available_models);
                    println!("Healthy providers: {}", result.healthy_providers);
                    println!("Discovery duration: {:?}", result.discovery_duration);
                    
                    if !result.warnings.is_empty() {
                        println!("Warnings:");
                        for warning in &result.warnings {
                            println!("  - {}", warning);
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Model discovery failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to create ModelDiscoveryService: {}", e);
            println!("Error details: {:?}", e);
            
            // Log the error chain
            let mut current_error = e.source();
            let mut error_level = 1;
            while let Some(err) = current_error {
                println!("  Error level {}: {}", error_level, err);
                current_error = err.source();
                error_level += 1;
            }
        }
    }

    Ok(())
}