use thiserror::Error;

#[derive(Debug, Error)]
pub enum CrabError {
    #[error("Missing API key for provider: {0}")]
    MissingApiKey(String),

    #[error("Provider {provider} error: {message}")]
    ProviderError { provider: String, message: String },

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Prompt not found: {0}")]
    PromptNotFound(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Toml(#[from] toml::de::Error),
}
