use derive_setters::Setters;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Default, Setters)]
#[setters(into, strip_option)]
pub struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    keep_alive: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<String>>,
}

impl TryFrom<forge_app::domain::Context> for ChatRequest {
    type Error = anyhow::Error;

    fn try_from(context: forge_app::domain::Context) -> Result<Self, Self::Error> {
        let messages = context
            .messages
            .into_iter()
            .filter_map(|msg| {
                if let forge_app::domain::ContextMessage::Text(text_msg) = msg {
                    Some(Message {
                        role: match text_msg.role {
                            forge_app::domain::Role::System => "system".to_string(),
                            forge_app::domain::Role::User => "user".to_string(),
                            forge_app::domain::Role::Assistant => "assistant".to_string(),
                        },
                        content: text_msg.content,
                        images: None, // TODO: Handle images when needed
                    })
                } else {
                    None // Skip non-text messages for now
                }
            })
            .collect();

        Ok(ChatRequest {
            model: String::new(), // Will be set by the provider
            messages,
            stream: Some(true), // Default to streaming
            format: None,
            options: None,
            keep_alive: None,
        })
    }
}
