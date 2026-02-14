mod openai_compat;
pub mod r#trait;

pub mod openai;
pub mod anthropic;
pub mod google;
pub mod openrouter;
pub mod groq;
pub mod together;
pub mod mistral;
pub mod deepseek;

pub use r#trait::Provider;

use crate::error::CrabError;
use crate::types::ProviderName;

/// Instantiate a provider by name. The API key is read from the
/// environment but not validated here; it is checked lazily when
/// send() or list_models() requires it.
pub fn get_provider(name: &str) -> Result<Box<dyn Provider>, CrabError> {
    let provider_name: ProviderName = name
        .parse()
        .map_err(|e: String| CrabError::ConfigError(e))?;

    match provider_name {
        ProviderName::OpenAI => Ok(Box::new(openai::OpenAIProvider::new()?)),
        ProviderName::Anthropic => Ok(Box::new(anthropic::AnthropicProvider::new()?)),
        ProviderName::Google => Ok(Box::new(google::GoogleProvider::new()?)),
        ProviderName::OpenRouter => Ok(Box::new(openrouter::OpenRouterProvider::new()?)),
        ProviderName::Groq => Ok(Box::new(groq::GroqProvider::new()?)),
        ProviderName::Together => Ok(Box::new(together::TogetherProvider::new()?)),
        ProviderName::Mistral => Ok(Box::new(mistral::MistralProvider::new()?)),
        ProviderName::DeepSeek => Ok(Box::new(deepseek::DeepSeekProvider::new()?)),
    }
}

pub fn list_provider_names() -> Vec<&'static str> {
    ProviderName::ALL.iter().map(|p| p.as_str()).collect()
}
