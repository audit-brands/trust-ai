//! Configuration system for local AI providers and fallback logic

pub mod local_ai;
pub mod fallback;

pub use local_ai::{LocalAiConfig, LocalProviderConfig};
pub use fallback::{FallbackConfig, FallbackStrategy};