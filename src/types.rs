use std::fmt;
use std::str::FromStr;

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
