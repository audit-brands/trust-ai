use thiserror::Error;

/// Comprehensive error types for Ollama provider operations
#[derive(Debug, Error)]
pub enum OllamaError {
    /// Connection-related errors
    #[error("Failed to connect to Ollama service at {url}: {source}")]
    ConnectionFailed {
        url: String,
        #[source]
        source: reqwest::Error,
    },

    /// Service unavailable errors
    #[error(
        "Ollama service is unavailable. Please ensure Ollama is running and accessible at {url}"
    )]
    ServiceUnavailable { url: String },

    /// Authentication and permission errors
    #[error("Authentication failed for Ollama service: {message}")]
    AuthenticationFailed { message: String },

    /// Model-related errors
    #[error("Model '{model}' not found. Available models can be listed using the models command")]
    ModelNotFound { model: String },

    #[error("Model '{model}' is currently loading. Please wait and try again")]
    ModelLoading { model: String },

    #[error("Model '{model}' failed to load: {reason}")]
    ModelLoadFailed { model: String, reason: String },

    /// Request validation errors
    #[error("Invalid request: {message}")]
    InvalidRequest { message: String },

    #[error("Request payload too large: {size} bytes exceeds maximum allowed")]
    PayloadTooLarge { size: usize },

    #[error("Request timeout after {timeout_seconds} seconds")]
    RequestTimeout { timeout_seconds: u64 },

    /// Response parsing errors
    #[error("Failed to parse response from Ollama: {message}")]
    ResponseParsingFailed { message: String },

    #[error("Received malformed response from Ollama service")]
    MalformedResponse,

    #[error("Unexpected response format from Ollama service")]
    UnexpectedResponseFormat,

    /// Streaming errors
    #[error("Stream interrupted unexpectedly: {reason}")]
    StreamInterrupted { reason: String },

    #[error("Stream parsing failed: {message}")]
    StreamParsingFailed { message: String },

    /// Resource and system errors
    #[error("Insufficient system resources: {message}")]
    InsufficientResources { message: String },

    #[error("Service rate limit exceeded. Please wait before making more requests")]
    RateLimitExceeded,

    /// Configuration errors
    #[error("Invalid Ollama configuration: {message}")]
    InvalidConfiguration { message: String },

    #[error("Ollama base URL is invalid: {url}")]
    InvalidBaseUrl { url: String },

    /// Generic HTTP errors with context
    #[error("HTTP error {status}: {message}")]
    HttpError { status: u16, message: String },

    /// Unknown errors
    #[error("Unknown Ollama error: {message}")]
    Unknown { message: String },
}

impl OllamaError {
    /// Create a connection failed error
    pub fn connection_failed(url: String, source: reqwest::Error) -> Self {
        Self::ConnectionFailed { url, source }
    }

    /// Create a service unavailable error
    pub fn service_unavailable(url: String) -> Self {
        Self::ServiceUnavailable { url }
    }

    /// Create a model not found error
    pub fn model_not_found(model: String) -> Self {
        Self::ModelNotFound { model }
    }

    /// Create a model loading error
    pub fn model_loading(model: String) -> Self {
        Self::ModelLoading { model }
    }

    /// Create a model load failed error
    pub fn model_load_failed(model: String, reason: String) -> Self {
        Self::ModelLoadFailed { model, reason }
    }

    /// Create an invalid request error
    pub fn invalid_request(message: String) -> Self {
        Self::InvalidRequest { message }
    }

    /// Create a response parsing failed error
    pub fn response_parsing_failed(message: String) -> Self {
        Self::ResponseParsingFailed { message }
    }

    /// Create a stream interrupted error
    pub fn stream_interrupted(reason: String) -> Self {
        Self::StreamInterrupted { reason }
    }

    /// Create an HTTP error with status and message
    pub fn http_error(status: u16, message: String) -> Self {
        Self::HttpError { status, message }
    }

    /// Check if this error indicates the service is unavailable
    pub fn is_service_unavailable(&self) -> bool {
        matches!(
            self,
            OllamaError::ServiceUnavailable { .. }
                | OllamaError::ConnectionFailed { .. }
                | OllamaError::HttpError { status: 503, .. }
        )
    }

    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            OllamaError::ServiceUnavailable { .. }
                | OllamaError::ConnectionFailed { .. }
                | OllamaError::RequestTimeout { .. }
                | OllamaError::ModelLoading { .. }
                | OllamaError::RateLimitExceeded
                | OllamaError::HttpError { status: 429 | 502 | 503 | 504, .. }
        )
    }

    /// Check if this error indicates a client-side issue
    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            OllamaError::InvalidRequest { .. }
                | OllamaError::PayloadTooLarge { .. }
                | OllamaError::ModelNotFound { .. }
                | OllamaError::InvalidConfiguration { .. }
                | OllamaError::InvalidBaseUrl { .. }
                | OllamaError::HttpError { status: 400..=499, .. }
        )
    }

    /// Get user-friendly error message with actionable guidance
    pub fn user_message(&self) -> String {
        match self {
            OllamaError::ServiceUnavailable { url } => {
                format!(
                    "Ollama service is not running. Please start Ollama and ensure it's accessible at {url}"
                )
            }
            OllamaError::ConnectionFailed { url, .. } => {
                format!(
                    "Cannot connect to Ollama service at {url}. Please check that Ollama is running and the URL is correct"
                )
            }
            OllamaError::ModelNotFound { model } => {
                format!(
                    "Model '{model}' is not available. Use 'ollama list' to see available models or 'ollama pull {model}' to download it"
                )
            }
            OllamaError::ModelLoading { model } => {
                format!("Model '{model}' is currently loading. Please wait a moment and try again")
            }
            OllamaError::AuthenticationFailed { .. } => {
                "Authentication failed. Please check your Ollama configuration and permissions"
                    .to_string()
            }
            OllamaError::RateLimitExceeded => {
                "Too many requests. Please wait a moment before trying again".to_string()
            }
            OllamaError::RequestTimeout { timeout_seconds } => {
                format!(
                    "Request timed out after {timeout_seconds} seconds. The model might be too large or the system is under heavy load"
                )
            }
            OllamaError::InvalidConfiguration { message } => {
                format!("Configuration error: {message}. Please check your Ollama settings")
            }
            _ => self.to_string(),
        }
    }
}

/// Convert reqwest errors to OllamaError with context
impl From<reqwest::Error> for OllamaError {
    fn from(error: reqwest::Error) -> Self {
        if error.is_timeout() {
            OllamaError::RequestTimeout {
                timeout_seconds: 30, // Default timeout assumption
            }
        } else if error.is_connect() {
            OllamaError::ConnectionFailed {
                url: error.url().map(|u| u.to_string()).unwrap_or_default(),
                source: error,
            }
        } else {
            OllamaError::Unknown { message: error.to_string() }
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_service_unavailable_detection() {
        let fixture = OllamaError::service_unavailable("http://localhost:11434".to_string());
        let actual = fixture.is_service_unavailable();
        let expected = true;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_retryable_error_detection() {
        let fixture = OllamaError::model_loading("llama3.2".to_string());
        let actual = fixture.is_retryable();
        let expected = true;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_client_error_detection() {
        let fixture = OllamaError::model_not_found("nonexistent".to_string());
        let actual = fixture.is_client_error();
        let expected = true;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_user_message_for_service_unavailable() {
        let fixture = OllamaError::service_unavailable("http://localhost:11434".to_string());
        let actual = fixture.user_message();
        let expected = "Ollama service is not running. Please start Ollama and ensure it's accessible at http://localhost:11434";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_user_message_for_model_not_found() {
        let fixture = OllamaError::model_not_found("llama3.2".to_string());
        let actual = fixture.user_message();
        let expected = "Model 'llama3.2' is not available. Use 'ollama list' to see available models or 'ollama pull llama3.2' to download it";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_http_error_status_categorization() {
        let fixture = OllamaError::http_error(404, "Not Found".to_string());
        let actual = fixture.is_client_error();
        let expected = true;
        assert_eq!(actual, expected);

        let fixture = OllamaError::http_error(503, "Service Unavailable".to_string());
        let actual = fixture.is_service_unavailable();
        let expected = true;
        assert_eq!(actual, expected);
    }
}
