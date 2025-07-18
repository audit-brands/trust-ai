use std::sync::Arc;

use anyhow::Context;
use forge_app::domain::{Provider, ProviderUrl};
use forge_app::{AppConfig, ProviderRegistry};
use forge_provider::config::fallback::FallbackConfig;
use forge_provider::config::local_ai::LocalAiConfig;
use forge_provider::selection::{ProviderSelector, ProviderType, SelectionContext};
use tokio::sync::RwLock;

use crate::EnvironmentInfra;

type ProviderSearch = (&'static str, Box<dyn FnOnce(&str) -> Provider>);

pub struct ForgeProviderRegistry<F> {
    infra: Arc<F>,
    // IMPORTANT: This cache is used to avoid logging out if the user has logged out from other
    // session. This helps to keep the user logged in for current session.
    cache: Arc<RwLock<Option<Provider>>>,
    provider_selector: Arc<RwLock<Option<ProviderSelector>>>,
}

impl<F: EnvironmentInfra> ForgeProviderRegistry<F> {
    pub fn new(infra: Arc<F>) -> Self {
        Self {
            infra,
            cache: Arc::new(Default::default()),
            provider_selector: Arc::new(Default::default()),
        }
    }

    fn provider_url(&self) -> Option<ProviderUrl> {
        if let Some(url) = self.infra.get_env_var("OPENAI_URL") {
            return Some(ProviderUrl::OpenAI(url));
        }

        // Check for Anthropic URL override
        if let Some(url) = self.infra.get_env_var("ANTHROPIC_URL") {
            return Some(ProviderUrl::Anthropic(url));
        }
        None
    }
    fn get_provider(&self, forge_config: AppConfig) -> Option<Provider> {
        if let Some(forge_key) = &forge_config.key_info {
            let provider = Provider::forge(forge_key.api_key.as_str());
            return Some(override_url(provider, self.provider_url()));
        }
        resolve_env_provider(self.provider_url(), self.infra.as_ref())
    }
    async fn ensure_provider_selector(&self, app_config: &AppConfig) -> anyhow::Result<()> {
        let mut selector_guard = self.provider_selector.write().await;

        if selector_guard.is_none() {
            // Create local AI config
            let local_config = if let Some(local_ai_config) = &app_config.local_ai {
                if local_ai_config.enabled {
                    LocalAiConfig::with_default_ollama()
                } else {
                    LocalAiConfig::new().enabled(false)
                }
            } else {
                LocalAiConfig::with_default_ollama()
            };

            // Create fallback config with environment-based cloud providers
            let mut cloud_providers = Vec::new();

            // Add cloud providers based on environment variables
            if let Some(forge_key) = &app_config.key_info {
                let provider = Provider::forge(forge_key.api_key.as_str());
                let provider = override_url(provider, self.provider_url());
                cloud_providers.push("forge".to_string());
            } else if let Some(_provider) =
                resolve_env_provider(self.provider_url(), self.infra.as_ref())
            {
                cloud_providers.push("openai".to_string()); // Default cloud
                                                            // provider
            }

            let fallback_config = FallbackConfig::default().cloud_providers(cloud_providers);

            // Create the enhanced provider selector
            let selector = ProviderSelector::new(local_config, fallback_config)
                .await
                .context("Failed to create provider selector")?;

            *selector_guard = Some(selector);
        }

        Ok(())
    }

    async fn get_provider_enhanced(
        &self,
        app_config: AppConfig,
    ) -> anyhow::Result<Option<Provider>> {
        // Ensure provider selector is initialized
        self.ensure_provider_selector(&app_config).await?;

        let mut selector_guard = self.provider_selector.write().await;
        if let Some(ref mut selector) = *selector_guard {
            // Create a default selection context
            let context = SelectionContext {
                model_id: "default".to_string(),
                requires_streaming: false,
                requires_tools: false,
                user_preferences: None,
                previous_provider: None,
                consecutive_failures: 0,
            };

            // Use enhanced provider selection
            match selector.select_provider(context).await {
                Ok(selection) => {
                    // Convert the selection to a Provider
                    match selection.provider_type {
                        ProviderType::Local => {
                            // For local providers, create an Ollama provider
                            Ok(Some(Provider::Ollama {
                                url: reqwest::Url::parse("http://localhost:11434").unwrap(),
                            }))
                        }
                        ProviderType::Cloud => {
                            // For cloud providers, use the existing logic
                            Ok(self.get_provider_fallback(app_config))
                        }
                    }
                }
                Err(_) => {
                    // Fall back to environment-based selection
                    Ok(self.get_provider_fallback(app_config))
                }
            }
        } else {
            Ok(self.get_provider_fallback(app_config))
        }
    }

    fn get_provider_fallback(&self, forge_config: AppConfig) -> Option<Provider> {
        if let Some(forge_key) = &forge_config.key_info {
            let provider = Provider::forge(forge_key.api_key.as_str());
            return Some(override_url(provider, self.provider_url()));
        }
        resolve_env_provider(self.provider_url(), self.infra.as_ref())
    }
}

#[async_trait::async_trait]
impl<F: EnvironmentInfra> ProviderRegistry for ForgeProviderRegistry<F> {
    async fn get_provider(&self, config: AppConfig) -> anyhow::Result<Provider> {
        if let Some(provider) = self.cache.read().await.as_ref() {
            return Ok(provider.clone());
        }

        // Try enhanced provider selection first
        let provider = match self.get_provider_enhanced(config.clone()).await? {
            Some(provider) => provider,
            None => {
                // Fall back to the old logic if enhanced selection fails
                self.get_provider_fallback(config)
                    .context("Failed to detect upstream provider")?
            }
        };

        self.cache.write().await.replace(provider.clone());
        Ok(provider)
    }
}

fn resolve_env_provider<F: EnvironmentInfra>(
    url: Option<ProviderUrl>,
    env: &F,
) -> Option<Provider> {
    let keys: [ProviderSearch; 6] = [
        ("FORGE_KEY", Box::new(Provider::forge)),
        ("OPENROUTER_API_KEY", Box::new(Provider::open_router)),
        ("REQUESTY_API_KEY", Box::new(Provider::requesty)),
        ("XAI_API_KEY", Box::new(Provider::xai)),
        ("OPENAI_API_KEY", Box::new(Provider::openai)),
        ("ANTHROPIC_API_KEY", Box::new(Provider::anthropic)),
    ];

    keys.into_iter().find_map(|(key, fun)| {
        env.get_env_var(key).map(|key| {
            let provider = fun(&key);
            override_url(provider, url.clone())
        })
    })
}

fn override_url(mut provider: Provider, url: Option<ProviderUrl>) -> Provider {
    if let Some(url) = url {
        provider.url(url);
    }
    provider
}
