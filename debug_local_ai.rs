use std::process::Command;

fn main() {
    println!("=== Debugging Local AI Model Discovery ===\n");
    
    // Test 1: Check if Ollama service is running
    println!("1. Testing Ollama service connectivity...");
    let output = Command::new("curl")
        .args(&["-s", "http://localhost:11434/api/tags"])
        .output();
    
    match output {
        Ok(result) => {
            if result.status.success() {
                println!("✅ Ollama service is responding");
                let response = String::from_utf8_lossy(&result.stdout);
                println!("   Response: {}", response);
            } else {
                println!("❌ Ollama service request failed");
                println!("   Error: {}", String::from_utf8_lossy(&result.stderr));
            }
        }
        Err(e) => {
            println!("❌ Failed to execute curl: {}", e);
        }
    }
    
    // Test 2: Check current forge.yaml configuration
    println!("\n2. Checking forge.yaml configuration...");
    match std::fs::read_to_string("forge.yaml") {
        Ok(content) => {
            if content.contains("local") || content.contains("ollama") {
                println!("✅ Found local AI configuration in forge.yaml");
            } else {
                println!("⚠️  No local AI configuration found in forge.yaml");
                println!("   This means trust-ai will use default configuration");
            }
        }
        Err(e) => {
            println!("❌ Failed to read forge.yaml: {}", e);
        }
    }
    
    // Test 3: Check if trust-ai can discover models
    println!("\n3. Testing trust-ai model discovery...");
    let output = Command::new("./target/debug/forge")
        .args(&["/model", "discover"])
        .output();
    
    match output {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            let stderr = String::from_utf8_lossy(&result.stderr);
            
            if stdout.contains("ollama") || stdout.contains("qwen") {
                println!("✅ trust-ai discovered local models");
            } else {
                println!("❌ trust-ai did not discover local models");
            }
            
            println!("   Output: {}", stdout);
            if !stderr.is_empty() {
                println!("   Errors: {}", stderr);
            }
        }
        Err(e) => {
            println!("❌ Failed to run trust-ai: {}", e);
            println!("   Note: Make sure trust-ai is built first with 'cargo build'");
        }
    }
}