use derive_more::From;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitAuth {
    pub session_id: String,
    pub auth_url: String,
    pub token: String,
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub key_info: Option<LoginInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_ai: Option<LocalAiAppConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback: Option<FallbackAppConfig>,
}

#[derive(Clone, Serialize, Deserialize, From)]
#[serde(rename_all = "camelCase")]
pub struct LoginInfo {
    pub api_key: String,
    pub api_key_name: String,
    pub api_key_masked: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_provider_id: Option<String>,
}

/// Local AI configuration for app config
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalAiAppConfig {
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preferred_provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_used_model: Option<String>,
    #[serde(default)]
    pub user_preferences: UserPreferencesConfig,
}

/// Fallback configuration for app config
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FallbackAppConfig {
    pub enabled: bool,
    pub strategy: String,
    #[serde(default)]
    pub notify_user: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_fallback_reason: Option<String>,
}

/// User preferences for local AI
#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserPreferencesConfig {
    #[serde(default)]
    pub prefer_local: bool,
    #[serde(default)]
    pub allow_fallback: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_response_time_seconds: Option<u64>,
    #[serde(default)]
    pub preferred_providers: Vec<String>,
}

impl Default for LocalAiAppConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            preferred_provider: None,
            last_used_model: None,
            user_preferences: UserPreferencesConfig::default(),
        }
    }
}

impl Default for FallbackAppConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            strategy: "graceful".to_string(),
            notify_user: true,
            last_fallback_reason: None,
        }
    }
}
