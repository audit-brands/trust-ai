mod anthropic;
mod client;
mod error;
mod forge_provider;
#[cfg(test)]
mod mock_server;
mod ollama;
mod retry;

mod utils;

// Re-export from builder.rs
pub use client::Client;

pub mod config;
pub mod discovery;
pub mod health;
pub mod performance;
pub mod selection;
#[cfg(test)]
pub mod test_utils;
