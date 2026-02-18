use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Information about a model's capabilities and constraints.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelInfo {
    pub id: String,
    pub max_output_tokens: Option<u32>,
    pub supports_temperature: bool,
    /// Optional: The name of the parameter used for max tokens (e.g. "max_completion_tokens")
    pub max_tokens_param: Option<String>,
}

impl ModelInfo {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            max_output_tokens: None,
            supports_temperature: true,
            max_tokens_param: None,
        }
    }
}

/// Enumeration of all supported LLM providers.
/// Used for compile-time validation and case-insensitive string parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderName {
    OpenAI,
    Anthropic,
    Google,
    OpenRouter,
    Groq,
    Together,
    Mistral,
    DeepSeek,
}

impl ProviderName {
    /// Complete list of all supported providers.
    /// Used for iteration in --list-providers and --list-models -a.
    pub const ALL: &'static [ProviderName] = &[
        ProviderName::OpenAI,
        ProviderName::Anthropic,
        ProviderName::Google,
        ProviderName::OpenRouter,
        ProviderName::Groq,
        ProviderName::Together,
        ProviderName::Mistral,
        ProviderName::DeepSeek,
    ];

    /// Returns the lowercase string identifier for this provider.
    pub fn as_str(&self) -> &'static str {
        match self {
            ProviderName::OpenAI => "openai",
            ProviderName::Anthropic => "anthropic",
            ProviderName::Google => "google",
            ProviderName::OpenRouter => "openrouter",
            ProviderName::Groq => "groq",
            ProviderName::Together => "together",
            ProviderName::Mistral => "mistral",
            ProviderName::DeepSeek => "deepseek",
        }
    }
}

impl fmt::Display for ProviderName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ProviderName {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "openai" => Ok(ProviderName::OpenAI),
            "anthropic" => Ok(ProviderName::Anthropic),
            "google" => Ok(ProviderName::Google),
            "openrouter" => Ok(ProviderName::OpenRouter),
            "groq" => Ok(ProviderName::Groq),
            "together" => Ok(ProviderName::Together),
            "mistral" => Ok(ProviderName::Mistral),
            "deepseek" => Ok(ProviderName::DeepSeek),
            _ => Err(format!("Unknown provider: {s}")),
        }
    }
}
