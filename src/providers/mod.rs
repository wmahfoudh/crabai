mod openai_compat;
pub mod r#trait;

pub mod anthropic;
pub mod deepseek;
pub mod google;
pub mod groq;
pub mod mistral;
pub mod openai;
pub mod openrouter;
pub mod together;

pub use r#trait::Provider;

use crate::config::Config;
use crate::error::CrabError;
use crate::types::ProviderName;

/// Creates a provider instance by name using default API key environment variables.
///
/// This is a convenience wrapper around get_provider_with_config that uses
/// Config::default(), which means all providers will use their standard
/// environment variable names (e.g., OPENAI_API_KEY, ANTHROPIC_API_KEY).
///
/// The API key is not validated during construction; validation happens lazily
/// when send() or list_models() is called.
#[allow(dead_code)]
pub fn get_provider(name: &str) -> Result<Box<dyn Provider>, CrabError> {
    get_provider_with_config(name, &Config::default())
}

/// Creates a provider instance with custom API key environment variable lookup.
///
/// Uses the provided config to determine which environment variable to read for
/// the provider's API key. This allows users to customize env var names via the
/// advanced.api_key_vars config section.
///
/// The API key is not validated during construction; validation happens lazily
/// when send() or list_models() is called.
pub fn get_provider_with_config(
    name: &str,
    config: &Config,
) -> Result<Box<dyn Provider>, CrabError> {
    let provider_name: ProviderName = name
        .parse()
        .map_err(|e: String| CrabError::ConfigError(e))?;

    match provider_name {
        ProviderName::OpenAI => Ok(Box::new(openai::OpenAIProvider::new(config)?)),
        ProviderName::Anthropic => {
            let api_key_var = config.api_key_var(name);
            Ok(Box::new(anthropic::AnthropicProvider::new_with_env(
                &api_key_var,
            )?))
        }
        ProviderName::Google => {
            let api_key_var = config.api_key_var(name);
            Ok(Box::new(google::GoogleProvider::new_with_env(
                &api_key_var,
            )?))
        }
        ProviderName::OpenRouter => {
            let api_key_var = config.api_key_var(name);
            Ok(Box::new(openrouter::OpenRouterProvider::new_with_env(
                &api_key_var,
            )?))
        }
        ProviderName::Groq => {
            let api_key_var = config.api_key_var(name);
            Ok(Box::new(groq::GroqProvider::new_with_env(&api_key_var)?))
        }
        ProviderName::Together => {
            let api_key_var = config.api_key_var(name);
            Ok(Box::new(together::TogetherProvider::new_with_env(
                &api_key_var,
            )?))
        }
        ProviderName::Mistral => {
            let api_key_var = config.api_key_var(name);
            Ok(Box::new(mistral::MistralProvider::new_with_env(
                &api_key_var,
            )?))
        }
        ProviderName::DeepSeek => {
            let api_key_var = config.api_key_var(name);
            Ok(Box::new(deepseek::DeepSeekProvider::new_with_env(
                &api_key_var,
            )?))
        }
    }
}

pub fn list_provider_names() -> Vec<&'static str> {
    ProviderName::ALL.iter().map(|p| p.as_str()).collect()
}
