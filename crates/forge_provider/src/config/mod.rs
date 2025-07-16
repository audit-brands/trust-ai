//! Configuration system for local AI providers and fallback logic

pub mod enhanced;
pub mod fallback;
pub mod local_ai;

pub use enhanced::{EnhancedFallbackConfig, EnhancedFallbackEngine};
pub use fallback::{FallbackConfig, FallbackStrategy};
pub use local_ai::{LocalAiConfig, LocalProviderConfig};