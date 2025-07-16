mod config;
#[cfg(test)]
mod e2e_tests;
mod error;
#[cfg(test)]
mod integration_tests;
mod provider;
mod request;
mod response;

pub use config::{HealthStatus, OllamaConfig, OllamaHealthCheck};
pub use error::OllamaError;
#[cfg(test)]
pub use integration_tests::OllamaIntegrationTest;
pub use provider::Ollama;
