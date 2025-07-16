use forge_app::domain::{ChatCompletionMessage, Content, Model, ModelId};
use serde::Deserialize;

// Response for /api/tags endpoint
#[derive(Deserialize, Debug)]
pub struct ListModelsResponse {
    pub models: Vec<OllamaModel>,
}

#[derive(Deserialize, Debug)]
pub struct OllamaModel {
    pub name: String,
    pub model: String,
    pub modified_at: String,
    pub size: u64,
    pub digest: String,
    pub details: ModelDetails,
}

#[derive(Deserialize, Debug)]
pub struct ModelDetails {
    pub parent_model: String,
    pub format: String,
    pub family: String,
    pub families: Vec<String>,
    pub parameter_size: String,
    pub quantization_level: String,
}

impl From<OllamaModel> for Model {
    fn from(value: OllamaModel) -> Self {
        Self {
            id: ModelId::new(value.name.clone()),
            name: Some(value.name),
            description: Some(format!("Ollama model - {}", value.details.family)),
            context_length: None, // Ollama doesn't provide this in the models response
            tools_supported: Some(true), // Most Ollama models support tools
            supports_parallel_tool_calls: None,
            supports_reasoning: None,
        }
    }
}

// Response for /api/chat endpoint (streaming)
#[derive(Deserialize, Debug)]
pub struct ChatResponse {
    pub model: String,
    pub created_at: String,
    pub message: ChatMessage,
    pub done: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_duration: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub load_duration: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_eval_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_eval_duration: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eval_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eval_duration: Option<u64>,
}

#[derive(Deserialize, Debug)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<String>>,
}

impl TryFrom<ChatResponse> for ChatCompletionMessage {
    type Error = anyhow::Error;

    fn try_from(response: ChatResponse) -> Result<Self, Self::Error> {
        Ok(ChatCompletionMessage {
            content: Some(Content::part(response.message.content)),
            reasoning: None, // Ollama doesn't provide reasoning separately
            reasoning_details: None,
            tool_calls: Vec::new(), // TODO: Handle tool calls when needed
            finish_reason: None,    // TODO: Map Ollama finish reasons
            usage: None,            // TODO: Map usage statistics
        })
    }
}
