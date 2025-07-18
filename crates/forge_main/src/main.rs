use anyhow::Result;
use clap::Parser;
use forge_api::ForgeAPI;
use forge_display::TitleFormat;
use forge_main::{tracker, Cli, UI};

// Commands that can run without authentication
const OFFLINE_COMMANDS: &[&str] = &["help", "info", "models", "local", "ollama"];

#[tokio::main]
async fn main() -> Result<()> {
    // Set up panic hook for better error display
    panic::set_hook(Box::new(|panic_info| {
        let message = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "Unexpected error occurred".to_string()
        };

        eprintln!("{}", TitleFormat::error(message.to_string()));
        tracker::error_blocking(message);
        std::process::exit(1);
    }));

    // Initialize and run the UI
    let cli = Cli::parse();
    
    // Determine if we should run in offline mode
    let offline_mode = cli.offline || is_offline_command(&cli);
    
    // Initialize the ForgeAPI with the restricted mode if specified
    let restricted = cli.restricted;
    let mut ui = UI::init(cli, move || ForgeAPI::init(restricted))?;
    ui.run_with_offline_mode(offline_mode).await;

    Ok(())
}

// Check if the command being run can work offline
fn is_offline_command(cli: &Cli) -> bool {
    // If it's a prompt command, check if it's an offline command
    if let Some(prompt) = &cli.prompt {
        let command = prompt.trim().to_lowercase();
        return OFFLINE_COMMANDS.iter().any(|&offline_cmd| {
            command == format!("/{}", offline_cmd) || command == offline_cmd
        });
    }
    
    // Check for commands that include "models" or "local" which should work without cloud authentication
    if let Some(prompt) = &cli.prompt {
        let command = prompt.trim().to_lowercase();
        return command.contains("models") || command.contains("local") || command.contains("ollama");
    }
    
    false
}
