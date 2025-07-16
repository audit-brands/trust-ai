use anyhow::Context as _;
use derive_builder::Builder;
use forge_app::domain::{ChatCompletionMessage, Context, Model, ModelId, ResultStream};
use reqwest::{Client, Url};
use reqwest_eventsource::{Event, RequestBuilderExt};
use tokio_stream::StreamExt;
use tracing::debug;

use super::error::OllamaError;
use super::request::ChatRequest;
use super::response::{ChatResponse, ListModelsResponse};

use crate::utils::format_http_context;

#[derive(Clone, Builder)]
pub struct Ollama {
    client: Client,
    base_url: Url,
}

impl Ollama {
    pub fn builder() -> OllamaBuilder {
        OllamaBuilder::default()
    }

    fn url(&self, path: &str) -> anyhow::Result<Url> {
        // Validate the path doesn't contain certain patterns
        if path.contains("://") || path.contains("..") {
            anyhow::bail!("Invalid path: Contains forbidden patterns");
        }

        // Remove leading slash to avoid double slashes
        let path = path.trim_start_matches('/');

        self.base_url
            .join(path)
            .with_context(|| format!("Failed to append {} to base URL: {}", path, self.base_url))
    }
}

impl Ollama {
    pub async fn chat(
        &self,
        model: ModelId,
        context: Context,
    ) -> ResultStream<ChatCompletionMessage, anyhow::Error> {
        // Convert context to Ollama chat request
        let request = ChatRequest::try_from(context)?
            .model(model.as_str().to_string())
            .stream(true);

        let url = self.url("api/chat")?;
        debug!(url = %url, model = %model, "Connecting to Ollama");

        let es = self
            .client
            .post(url.clone())
            .json(&request)
            .eventsource()
            .with_context(|| format_http_context(None, "POST", &url))?;

        let url_clone = url.clone();
        let url_clone2 = url.clone();
        let model_clone = model.clone();
        let stream = es
            .take_while(|message| !matches!(message, Err(reqwest_eventsource::Error::StreamEnded)))
            .then(move |event| {
                let url_inner = url_clone.clone();
                let model_inner = model_clone.clone();
                async move {
                match event {
                    Ok(event) => match event {
                        Event::Open => None,
                        Event::Message(event) if ["[DONE]", ""].contains(&event.data.as_str()) => {
                            debug!("Received completion from Ollama");
                            None
                        }
                        Event::Message(message) => Some(
                            serde_json::from_str::<ChatResponse>(&message.data)
                                .map_err(|e| OllamaError::StreamParsingFailed { message: e.to_string() })
                                .with_context(|| "Failed to parse Ollama event")
                                .and_then(|event| {
                                    ChatCompletionMessage::try_from(event).with_context(|| {
                                        format!(
                                            "Failed to create completion message: {}",
                                            message.data
                                        )
                                    })
                                }),
                        ),
                    },
                    Err(error) => match error {
                        reqwest_eventsource::Error::StreamEnded => None,
                        reqwest_eventsource::Error::InvalidStatusCode(_, response) => {
                            let status = response.status();
                            let body_text = response.text().await.ok();
                            
                            // Convert to appropriate OllamaError
                            let ollama_error = match status.as_u16() {
                                404 => OllamaError::model_not_found(model_inner.as_str().to_string()),
                                503 => OllamaError::service_unavailable(url_inner.to_string()),
                                _ => OllamaError::http_error(status.as_u16(), 
                                    body_text.clone().unwrap_or_else(|| "Unknown error".to_string())),
                            };
                            
                            Some(Err(anyhow::anyhow!(ollama_error)).with_context(
                                || match body_text {
                                    Some(body) => {
                                        format!("Invalid status code: {status} Reason: {body}")
                                    }
                                    None => {
                                        format!("Invalid status code: {status} Reason: [Unknown]")
                                    }
                                },
                            ))
                        }
                        reqwest_eventsource::Error::InvalidContentType(_, ref response) => {
                            let status_code = response.status();
                            debug!(response = ?response, "Invalid content type");
                            Some(Err(error).with_context(|| format!("Http Status: {status_code}")))
                        }
                        error => {
                            tracing::error!(error = ?error, "Failed to receive chat completion event");
                            Some(Err(error.into()))
                        }
                    },
                }
                }
            })
            .map(move |response| match response {
                Some(Err(err)) => {
                    Some(Err(err).with_context(|| format_http_context(None, "POST", &url_clone2)))
                }
                _ => response,
            });

        Ok(Box::pin(stream.filter_map(|x| x)))
    }

    pub async fn models(&self) -> anyhow::Result<Vec<Model>> {
        let url = self.url("api/tags")?;
        debug!(url = %url, "Fetching models from Ollama");

        let result = self.client.get(url.clone()).send().await;

        match result {
            Err(error) => {
                tracing::error!(error = ?error, "Failed to fetch models");
                
                // Get status before moving error
                let error_status = error.status();
                let error_msg = error.to_string();
                
                // Convert to OllamaError for better user experience
                let ollama_error = if error.is_timeout() {
                    OllamaError::RequestTimeout { timeout_seconds: 30 }
                } else if error.is_connect() {
                    OllamaError::connection_failed(url.to_string(), error)
                } else {
                    OllamaError::Unknown { message: error_msg }
                };
                
                let ctx_msg = format_http_context(error_status, "GET", &url);
                Err(anyhow::anyhow!(ollama_error))
                    .with_context(|| ctx_msg)
                    .with_context(|| "Failed to fetch models")
            }
            Ok(response) => {
                let status = response.status();
                let ctx_msg = format_http_context(Some(response.status()), "GET", &url);
                
                // Handle different status codes with appropriate errors
                if status == 503 {
                    return Err(anyhow::anyhow!(OllamaError::service_unavailable(url.to_string())))
                        .with_context(|| ctx_msg);
                }
                
                let text = response
                    .text()
                    .await
                    .with_context(|| ctx_msg.clone())
                    .with_context(|| "Failed to decode response into text")?;

                if status.is_success() {
                    let response: ListModelsResponse = serde_json::from_str(&text)
                        .map_err(|e| OllamaError::response_parsing_failed(e.to_string()))
                        .with_context(|| ctx_msg)
                        .with_context(|| "Failed to deserialize models response")?;
                    Ok(response.models.into_iter().map(Into::into).collect())
                } else {
                    // Treat non-200 response as error with appropriate categorization
                    let ollama_error = match status.as_u16() {
                        400..=499 => OllamaError::http_error(status.as_u16(), text),
                        500..=599 => OllamaError::service_unavailable(url.to_string()),
                        _ => OllamaError::http_error(status.as_u16(), text),
                    };
                    
                    Err(anyhow::anyhow!(ollama_error))
                        .with_context(|| ctx_msg)
                        .with_context(|| "Failed to fetch the models")
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use forge_app::domain::ContextMessage;

    use super::*;
    use crate::mock_server::{normalize_ports, MockServer};

    fn create_ollama(base_url: &str) -> anyhow::Result<Ollama> {
        Ok(Ollama::builder()
            .client(Client::new())
            .base_url(Url::parse(base_url)?)
            .build()
            .unwrap())
    }

    fn create_mock_models_response() -> serde_json::Value {
        serde_json::json!({
            "models": [
                {
                    "name": "llama3.2:latest",
                    "model": "llama3.2:latest",
                    "modified_at": "2025-05-04T17:37:44.706015396-07:00",
                    "size": 2019393189u64,
                    "digest": "a80c4f17acd55265feec403c7aef86be0c25983ab279d83f3bcd3abbcb5b8b72",
                    "details": {
                        "parent_model": "",
                        "format": "gguf",
                        "family": "llama",
                        "families": ["llama"],
                        "parameter_size": "3.2B",
                        "quantization_level": "Q4_K_M"
                    }
                },
                {
                    "name": "deepseek-r1:latest",
                    "model": "deepseek-r1:latest",
                    "modified_at": "2025-05-10T08:06:48.639712648-07:00",
                    "size": 4683075271u64,
                    "digest": "0a8c266910232fd3291e71e5ba1e058cc5af9d411192cf88b6d30e92b6e73163",
                    "details": {
                        "parent_model": "",
                        "format": "gguf",
                        "family": "qwen2",
                        "families": ["qwen2"],
                        "parameter_size": "7.6B",
                        "quantization_level": "Q4_K_M"
                    }
                }
            ]
        })
    }

    fn create_error_response(message: &str, code: u16) -> serde_json::Value {
        serde_json::json!({
            "error": {
                "code": code,
                "message": message
            }
        })
    }

    fn create_empty_response() -> serde_json::Value {
        serde_json::json!({
            "models": []
        })
    }

    #[tokio::test]
    async fn test_url_for_models() {
        let ollama = Ollama::builder()
            .client(Client::new())
            .base_url(Url::parse("http://localhost:11434/api/").unwrap())
            .build()
            .unwrap();
        assert_eq!(
            ollama.url("tags").unwrap().as_str(),
            "http://localhost:11434/api/tags"
        );
    }

    #[tokio::test]
    async fn test_request_conversion() {
        let model_id = ModelId::new("llama3.2");
        let context = Context::default()
            .add_message(ContextMessage::system("You're an expert assistant."))
            .add_message(ContextMessage::user(
                "Hello, how are you?",
                model_id.clone().into(),
            ));

        let request = ChatRequest::try_from(context)
            .unwrap()
            .model("llama3.2".to_string())
            .stream(true);

        insta::assert_snapshot!(serde_json::to_string_pretty(&request).unwrap());
    }

    #[tokio::test]
    async fn test_fetch_models_success() -> anyhow::Result<()> {
        let mut fixture = MockServer::new().await;
        let mock = fixture
            .mock_ollama_models(create_mock_models_response(), 200)
            .await;
        let ollama = create_ollama(&fixture.url())?;
        let actual = ollama.models().await?;

        mock.assert_async().await;

        // Verify we got the expected models
        assert_eq!(actual.len(), 2);
        insta::assert_json_snapshot!(actual);
        Ok(())
    }

    #[tokio::test]
    async fn test_fetch_models_http_error_status() -> anyhow::Result<()> {
        let mut fixture = MockServer::new().await;
        let mock = fixture
            .mock_ollama_models(create_error_response("Ollama not running", 503), 503)
            .await;

        let ollama = create_ollama(&fixture.url())?;
        let actual = ollama.models().await;

        mock.assert_async().await;

        // Verify that we got an error
        assert!(actual.is_err());
        insta::assert_snapshot!(normalize_ports(format!("{:#?}", actual.unwrap_err())));
        Ok(())
    }

    #[tokio::test]
    async fn test_fetch_models_empty_response() -> anyhow::Result<()> {
        let mut fixture = MockServer::new().await;
        let mock = fixture
            .mock_ollama_models(create_empty_response(), 200)
            .await;

        let ollama = create_ollama(&fixture.url())?;
        let actual = ollama.models().await?;

        mock.assert_async().await;
        assert!(actual.is_empty());
        Ok(())
    }
}
