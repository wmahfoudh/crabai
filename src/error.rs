use thiserror::Error;

/// Error types for CrabAI operations.
/// All errors implement Display and std::error::Error via thiserror.
#[derive(Debug, Error)]
pub enum CrabError {
    /// API key environment variable not set or empty for the specified provider.
    #[error("Missing API key for provider: {0}")]
    MissingApiKey(String),

    /// Provider API returned an error or unexpected response.
    #[error("Provider {provider} error: {message}")]
    ProviderError { provider: String, message: String },

    /// Configuration file parsing error or invalid configuration.
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Requested prompt template file not found in prompts directory.
    #[error("Prompt not found: {0}")]
    PromptNotFound(String),

    /// File I/O error (reading prompts, config files, cache).
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// HTTP request error from provider API calls.
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    /// JSON parsing error from provider API responses.
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    /// TOML deserialization error when loading config files.
    #[error(transparent)]
    TomlDe(#[from] toml::de::Error),

    /// TOML serialization error when saving config files.
    #[error(transparent)]
    TomlSer(#[from] toml::ser::Error),

    /// Interactive prompt error from dialoguer (config wizard).
    #[error(transparent)]
    Dialoguer(#[from] dialoguer::Error),
}
