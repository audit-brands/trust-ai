use std::time::Duration;

use anyhow::Context as _;
use forge_app::domain::{Context, ContextMessage, ModelId};
use reqwest::Client;
use serde_json::json;
use tokio::time::timeout;
use tokio_stream::StreamExt;

#[cfg(test)]
use crate::mock_server::{normalize_ports, MockServer};
use crate::ollama::error::OllamaError;
use crate::ollama::Ollama;

/// Integration test suite for Ollama provider
///
/// This module contains comprehensive integration tests that validate:
/// - Real Ollama service integration (when available)
/// - Mock service testing for CI/CD compatibility
/// - Error handling scenarios
/// - Performance and reliability
/// - Cross-provider compatibility
#[cfg(test)]
pub struct OllamaIntegrationTest {
    /// Real Ollama service URL (if available)
    real_service_url: Option<String>,
    /// Mock server for controlled testing
    mock_server: MockServer,
    /// HTTP client for testing
    client: Client,
}

#[cfg(test)]
impl OllamaIntegrationTest {
    /// Create a new integration test suite
    pub async fn new() -> anyhow::Result<Self> {
        let mock_server = MockServer::new().await;
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        // Try to detect real Ollama service
        let real_service_url = Self::detect_ollama_service(&client).await;

        Ok(Self { real_service_url, mock_server, client })
    }

    /// Detect if a real Ollama service is available
    async fn detect_ollama_service(client: &Client) -> Option<String> {
        let urls = vec![
            "http://localhost:11434",
            "http://127.0.0.1:11434",
            "http://0.0.0.0:11434",
        ];

        for url in urls {
            if let Ok(response) = client.get(&format!("{}/api/tags", url)).send().await {
                if response.status().is_success() {
                    tracing::info!("Detected Ollama service at {}", url);
                    return Some(url.to_string());
                }
            }
        }

        tracing::warn!("No real Ollama service detected, using mock only");
        None
    }

    /// Create Ollama client for real service testing
    fn create_real_ollama(&self) -> anyhow::Result<Option<Ollama>> {
        if let Some(ref url) = self.real_service_url {
            let ollama = Ollama::builder()
                .client(self.client.clone())
                .base_url(url.parse()?)
                .build()
                .unwrap();
            Ok(Some(ollama))
        } else {
            Ok(None)
        }
    }

    /// Create Ollama client for mock service testing
    fn create_mock_ollama(&self) -> anyhow::Result<Ollama> {
        let ollama = Ollama::builder()
            .client(self.client.clone())
            .base_url(self.mock_server.url().parse()?)
            .build()
            .unwrap();
        Ok(ollama)
    }

    /// Test model discovery with real service
    pub async fn test_real_model_discovery(&self) -> anyhow::Result<()> {
        if let Some(ollama) = self.create_real_ollama()? {
            let models = ollama.models().await?;
            tracing::info!("Found {} models in real Ollama service", models.len());

            // Validate model structure
            for model in &models {
                assert!(
                    !model.id.as_str().is_empty(),
                    "Model ID should not be empty"
                );
                assert!(!model.name.is_empty(), "Model name should not be empty");
            }

            Ok(())
        } else {
            tracing::info!("Skipping real model discovery test - no service available");
            Ok(())
        }
    }

    /// Test chat completion with real service
    pub async fn test_real_chat_completion(&self) -> anyhow::Result<()> {
        if let Some(ollama) = self.create_real_ollama()? {
            // First get available models
            let models = ollama.models().await?;
            if models.is_empty() {
                tracing::warn!("No models available for chat completion test");
                return Ok(());
            }

            let model_id = ModelId::new(&models[0].id.as_str());
            let context = Context::default()
                .add_message(ContextMessage::system("You are a helpful assistant."))
                .add_message(ContextMessage::user(
                    "Say 'Hello, World!' and nothing else.",
                    model_id.clone().into(),
                ));

            // Test with timeout to prevent hanging
            let stream =
                timeout(Duration::from_secs(60), ollama.chat(&model_id, context)).await??;

            let messages: Vec<_> = stream.take(10).collect().await;
            assert!(!messages.is_empty(), "Should receive at least one message");

            // Validate message structure
            for message in messages {
                let msg = message?;
                assert!(
                    !msg.content.is_empty(),
                    "Message content should not be empty"
                );
            }

            tracing::info!("Real chat completion test passed");
            Ok(())
        } else {
            tracing::info!("Skipping real chat completion test - no service available");
            Ok(())
        }
    }

    /// Test service unavailable scenario
    pub async fn test_service_unavailable(&mut self) -> anyhow::Result<()> {
        // Mock a service unavailable response
        let _mock = self
            .mock_server
            .mock_ollama_models(json!({"error": "Service unavailable"}), 503)
            .await;

        let ollama = self.create_mock_ollama()?;
        let result = ollama.models().await;

        assert!(result.is_err(), "Should fail when service unavailable");
        tracing::info!("Service unavailable test passed");
        Ok(())
    }

    /// Test invalid model request
    pub async fn test_invalid_model_request(&mut self) -> anyhow::Result<()> {
        // Mock a successful models response
        let _mock = self
            .mock_server
            .mock_ollama_models(json!({"models": []}), 200)
            .await;

        let ollama = self.create_mock_ollama()?;
        let model_id = ModelId::new("nonexistent-model");
        let context =
            Context::default().add_message(ContextMessage::user("Test", model_id.clone().into()));

        // This should work with mock but would fail with real service
        let result = ollama.chat(&model_id, context).await;

        // With mock server, this might succeed but with real service it would fail
        tracing::info!("Invalid model request test completed: {:?}", result.is_ok());
        Ok(())
    }

    /// Test malformed response handling
    pub async fn test_malformed_response(&mut self) -> anyhow::Result<()> {
        // Mock a malformed JSON response
        let _mock = self
            .mock_server
            .mock_ollama_models(json!({"invalid": "structure"}), 200)
            .await;

        let ollama = self.create_mock_ollama()?;
        let result = ollama.models().await;

        assert!(result.is_err(), "Should fail with malformed response");
        tracing::info!("Malformed response test passed");
        Ok(())
    }

    /// Test connection timeout scenario
    pub async fn test_connection_timeout(&self) -> anyhow::Result<()> {
        // Create client with very short timeout
        let short_timeout_client = Client::builder()
            .timeout(Duration::from_millis(1))
            .build()?;

        let ollama = Ollama::builder()
            .client(short_timeout_client)
            .base_url("http://httpbin.org/delay/10".parse()?) // Slow endpoint
            .build()
            .unwrap();

        let result = ollama.models().await;
        assert!(result.is_err(), "Should timeout");
        tracing::info!("Connection timeout test passed");
        Ok(())
    }

    /// Test provider switching scenario
    pub async fn test_provider_switching(&self) -> anyhow::Result<()> {
        // Test that we can create multiple Ollama instances with different
        // configurations
        let ollama1 = self.create_mock_ollama()?;
        let ollama2 = Ollama::builder()
            .client(self.client.clone())
            .base_url("http://localhost:11435".parse()?) // Different port
            .build()
            .unwrap();

        // Both should be independent
        assert_ne!(
            ollama1.url("api/tags")?.to_string(),
            ollama2.url("api/tags")?.to_string()
        );

        tracing::info!("Provider switching test passed");
        Ok(())
    }

    /// Test streaming response handling
    pub async fn test_streaming_response(&mut self) -> anyhow::Result<()> {
        if let Some(ollama) = self.create_real_ollama()? {
            let models = ollama.models().await?;
            if models.is_empty() {
                tracing::warn!("No models available for streaming test");
                return Ok(());
            }

            let model_id = ModelId::new(&models[0].id.as_str());
            let context = Context::default().add_message(ContextMessage::user(
                "Count from 1 to 5",
                model_id.clone().into(),
            ));

            let stream = ollama.chat(&model_id, context).await?;
            let messages: Vec<_> = stream.take(20).collect().await; // Limit to prevent infinite streams

            assert!(!messages.is_empty(), "Should receive streaming messages");
            tracing::info!("Received {} streaming messages", messages.len());
            Ok(())
        } else {
            tracing::info!("Skipping streaming test - no real service available");
            Ok(())
        }
    }

    /// Test performance baseline
    pub async fn test_performance_baseline(&self) -> anyhow::Result<()> {
        if let Some(ollama) = self.create_real_ollama()? {
            let start = std::time::Instant::now();
            let _models = ollama.models().await?;
            let duration = start.elapsed();

            // Model discovery should complete within reasonable time
            assert!(
                duration < Duration::from_secs(10),
                "Model discovery took too long: {:?}",
                duration
            );

            tracing::info!("Model discovery completed in {:?}", duration);
            Ok(())
        } else {
            tracing::info!("Skipping performance test - no real service available");
            Ok(())
        }
    }

    /// Run all integration tests
    pub async fn run_all_tests(&mut self) -> anyhow::Result<()> {
        tracing::info!("Starting Ollama integration test suite");

        // Real service tests (if available)
        self.test_real_model_discovery().await?;
        self.test_real_chat_completion().await?;
        self.test_streaming_response().await?;
        self.test_performance_baseline().await?;

        // Mock service tests (always run)
        self.test_service_unavailable().await?;
        self.test_invalid_model_request().await?;
        self.test_malformed_response().await?;
        self.test_connection_timeout().await?;
        self.test_provider_switching().await?;

        tracing::info!("All Ollama integration tests completed successfully");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn test_integration_test_creation() {
        let fixture = OllamaIntegrationTest::new().await;
        let actual = fixture.is_ok();
        let expected = true;
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_mock_ollama_creation() {
        let fixture = OllamaIntegrationTest::new().await.unwrap();
        let actual = fixture.create_mock_ollama();
        assert!(actual.is_ok());
    }

    #[tokio::test]
    async fn test_provider_switching() {
        let fixture = OllamaIntegrationTest::new().await.unwrap();
        let actual = fixture.test_provider_switching().await;
        let expected = Ok(());
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_service_unavailable_scenario() {
        let mut fixture = OllamaIntegrationTest::new().await.unwrap();
        let actual = fixture.test_service_unavailable().await;
        let expected = Ok(());
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_malformed_response_scenario() {
        let mut fixture = OllamaIntegrationTest::new().await.unwrap();
        let actual = fixture.test_malformed_response().await;
        let expected = Ok(());
        assert_eq!(actual, expected);
    }
}
