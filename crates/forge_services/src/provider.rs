use std::sync::Arc;

use anyhow::{Context, Result};
use forge_app::domain::{
    ChatCompletionMessage, Context as ChatContext, HttpConfig, Model, ModelId, Provider,
    ResultStream, RetryConfig,
};
use forge_app::{AppConfig, ProviderService};
use forge_provider::config::local_ai::LocalAiConfig;
use forge_provider::discovery::ModelDiscoveryService;
use forge_provider::Client;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

use crate::EnvironmentInfra;

#[derive(Clone)]
pub struct ForgeProviderService {
    retry_config: Arc<RetryConfig>,
    cached_client: Arc<Mutex<Option<Client>>>,
    cached_models: Arc<Mutex<Option<Vec<Model>>>>,
    cached_local_models: Arc<Mutex<Option<Vec<Model>>>>,
    local_discovery: Arc<Mutex<Option<ModelDiscoveryService>>>,
    version: String,
    timeout_config: HttpConfig,
}

impl ForgeProviderService {
    pub fn new<I: EnvironmentInfra>(infra: Arc<I>) -> Self {
        let env = infra.get_environment();
        let version = env.version();
        let retry_config = Arc::new(env.retry_config);
        Self {
            retry_config,
            cached_client: Arc::new(Mutex::new(None)),
            cached_models: Arc::new(Mutex::new(None)),
            cached_local_models: Arc::new(Mutex::new(None)),
            local_discovery: Arc::new(Mutex::new(None)),
            version,
            timeout_config: env.http,
        }
    }

    async fn client(&self, provider: Provider) -> Result<Client> {
        let mut client_guard = self.cached_client.lock().await;

        match client_guard.as_ref() {
            Some(client) => Ok(client.clone()),
            None => {
                // Client doesn't exist, create new one
                let client = Client::new(
                    provider,
                    self.retry_config.clone(),
                    &self.version,
                    &self.timeout_config,
                )?;

                // Cache the new client
                *client_guard = Some(client.clone());
                Ok(client)
            }
        }
    }

    async fn get_local_ai_config(app_config: &AppConfig) -> LocalAiConfig {
        // Check if local AI is enabled in app config
        if let Some(local_ai_config) = &app_config.local_ai {
            if local_ai_config.enabled {
                debug!("Local AI is enabled in app config");
                return LocalAiConfig::with_default_ollama();
            }
        }

        // Default to enabled with Ollama for now
        debug!("Using default local AI config with Ollama");
        LocalAiConfig::with_default_ollama()
    }

    async fn ensure_local_discovery(&self, app_config: &AppConfig) -> Result<()> {
        let mut discovery_guard = self.local_discovery.lock().await;

        if discovery_guard.is_none() {
            let local_config = Self::get_local_ai_config(app_config).await;

            debug!(
                "Attempting to initialize ModelDiscoveryService with config: {:?}",
                local_config
            );

            match ModelDiscoveryService::new(local_config).await {
                Ok(discovery) => {
                    info!("Local AI model discovery service initialized successfully");
                    *discovery_guard = Some(discovery);
                }
                Err(e) => {
                    error!("Failed to initialize local AI discovery service: {}", e);
                    error!("Error details: {:?}", e);

                    // Log the error chain for better debugging
                    let mut current_error = e.source();
                    let mut error_level = 1;
                    while let Some(err) = current_error {
                        error!("  Error level {}: {}", error_level, err);
                        current_error = err.source();
                        error_level += 1;
                    }

                    // Don't fail the entire operation, just log the warning
                }
            }
        }

        Ok(())
    }

    async fn discover_local_models(&self, app_config: &AppConfig) -> Result<Vec<Model>> {
        // Check cache first
        {
            let local_models_guard = self.cached_local_models.lock().await;
            if let Some(cached_local_models) = local_models_guard.as_ref() {
                debug!(
                    "Returning {} cached local models",
                    cached_local_models.len()
                );
                return Ok(cached_local_models.clone());
            }
        }

        // Ensure discovery service is initialized
        self.ensure_local_discovery(app_config).await?;

        let mut discovery_guard = self.local_discovery.lock().await;
        if let Some(ref mut discovery) = *discovery_guard {
            match discovery.discover_all_models().await {
                Ok(_discovery_result) => {
                    // Get the discovered models from the service
                    let discovered_models = discovery.get_available_models();

                    let local_models: Vec<Model> = discovered_models
                        .into_iter()
                        .map(|discovered_model| Model {
                            id: discovered_model.model.id.clone(),
                            name: discovered_model.model.name.clone(),
                            context_length: discovered_model.model.context_length,
                            description: Some(format!(
                                "Local {} model ({}ms response time)",
                                discovered_model.provider,
                                discovered_model
                                    .response_time
                                    .map(|d| d.as_millis().to_string())
                                    .unwrap_or_else(|| "unknown".to_string())
                            )),
                            tools_supported: discovered_model.model.tools_supported,
                            supports_parallel_tool_calls: discovered_model
                                .model
                                .supports_parallel_tool_calls,
                            supports_reasoning: discovered_model.model.supports_reasoning,
                        })
                        .collect();

                    info!("Discovered {} local AI models", local_models.len());

                    // Cache the local models
                    {
                        let mut local_models_guard = self.cached_local_models.lock().await;
                        *local_models_guard = Some(local_models.clone());
                    }

                    Ok(local_models)
                }
                Err(e) => {
                    warn!("Failed to discover local models: {}", e);
                    Ok(vec![])
                }
            }
        } else {
            debug!("Local discovery service not available");
            Ok(vec![])
        }
    }
}

#[async_trait::async_trait]
impl ProviderService for ForgeProviderService {
    async fn chat(
        &self,
        model: &ModelId,
        request: ChatContext,
        provider: Provider,
    ) -> ResultStream<ChatCompletionMessage, anyhow::Error> {
        let client = self.client(provider).await?;

        client
            .chat(model, request)
            .await
            .with_context(|| format!("Failed to chat with model: {model}"))
    }

    async fn models(&self, provider: Provider, app_config: AppConfig) -> Result<Vec<Model>> {
        // Get cloud provider models
        let mut all_models = Vec::new();

        // Check cache first for cloud models
        {
            let models_guard = self.cached_models.lock().await;
            if let Some(cached_models) = models_guard.as_ref() {
                all_models.extend_from_slice(cached_models);
            } else {
                // Models not in cache, fetch from client
                let client = self.client(provider).await?;
                let cloud_models = client.models().await?;

                // Cache the cloud models
                {
                    let mut models_guard = self.cached_models.lock().await;
                    *models_guard = Some(cloud_models.clone());
                }

                all_models.extend(cloud_models);
            }
        }

        // Get local AI models if enabled
        let local_models = self.discover_local_models(&app_config).await?;
        all_models.extend(local_models);

        info!(
            "Total models available: {} (cloud + local)",
            all_models.len()
        );
        Ok(all_models)
    }
}
