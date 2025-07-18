use forge_provider::config::local_ai::LocalAiConfig;

fn main() {
    let config = LocalAiConfig::with_default_ollama();
    println!("LocalAiConfig: {:#?}", config);
    
    println!("\nEnabled providers:");
    for (name, provider_config) in config.enabled_providers() {
        println!("  - {}: {:?}", name, provider_config);
        
        match provider_config.create_health_checker() {
            Ok(_) => println!("    ✅ Health checker created successfully"),
            Err(e) => {
                println!("    ❌ Health checker creation failed: {}", e);
                println!("    Error details: {:?}", e);
            }
        }
    }
}