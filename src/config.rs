use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::CrabError;

/// Configuration loaded from ~/.config/crabai/config.toml.
/// All fields are optional; missing values fall through to internal defaults.
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Config {
    pub default_provider: Option<String>,
    pub default_model: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub prompts_dir: Option<String>,
    pub model_cache: Option<bool>,
    pub model_cache_ttl_hours: Option<u64>,

    /// Advanced configuration for provider-specific settings.
    pub advanced: Option<AdvancedConfig>,
}

/// Advanced configuration section for provider-specific overrides.
/// This section is optional and allows fine-grained control over provider behavior.
#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct AdvancedConfig {
    /// Custom environment variable names for provider API keys.
    /// Maps provider name (lowercase) to the environment variable name to read.
    ///
    /// Example: { "anthropic": "MY_ANTHROPIC_KEY", "openai": "OPENAI_TOKEN" }
    ///
    /// If a provider is not listed here, the default environment variable name
    /// for that provider is used (e.g., "OPENAI_API_KEY" for OpenAI).
    pub api_key_vars: Option<HashMap<String, String>>,

    /// Provider-specific advanced settings for OpenAI.
    pub openai: Option<OpenAIAdvancedConfig>,
}

/// Advanced settings specific to the OpenAI provider.
#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct OpenAIAdvancedConfig {
    /// The parameter name to use for max_tokens in OpenAI API requests.
    /// Defaults to "max_tokens". Can be set to "max_completion_tokens"
    /// for compatibility with certain proxy or non-standard model providers.
    pub max_tokens_param: Option<String>,
}

impl Config {
    /// Load configuration from a TOML file.
    ///
    /// If `path` is Some, loads from that path. Otherwise, loads from the
    /// default location (~/.config/crabai/config.toml). If the file does
    /// not exist, returns a Config with all default values.
    pub fn load(path: Option<&str>) -> Result<Self, CrabError> {
        let config_path = match path {
            Some(p) => PathBuf::from(p),
            None => Self::default_config_path(),
        };

        if !config_path.exists() {
            return Ok(Config::default());
        }

        let contents = std::fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Returns the platform-specific config directory for CrabAI.
    ///
    /// On Linux: ~/.config/crabai
    /// On macOS: ~/Library/Application Support/crabai
    /// On Windows: %APPDATA%\crabai
    ///
    /// Falls back to ~/.config/crabai if the platform config directory
    /// cannot be determined.
    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("crabai")
    }

    pub fn default_config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    /// Returns the directory containing prompt template files.
    /// Expands tilde (~/) in the configured path if present.
    /// Defaults to ~/.config/crabai/prompts if not configured.
    pub fn prompts_dir(&self) -> PathBuf {
        match &self.prompts_dir {
            Some(dir) => {
                let expanded = shellexpand(dir);
                PathBuf::from(expanded)
            }
            None => Self::config_dir().join("prompts"),
        }
    }

    pub fn model_cache_enabled(&self) -> bool {
        self.model_cache.unwrap_or(true)
    }

    pub fn cache_ttl_hours(&self) -> u64 {
        self.model_cache_ttl_hours.unwrap_or(24)
    }

    pub fn resolve_temperature(&self) -> f32 {
        self.temperature.unwrap_or(0.2)
    }

    pub fn resolve_max_tokens(&self) -> u32 {
        self.max_tokens.unwrap_or(4096)
    }

    /// Returns the environment variable name to use for a provider's API key.
    ///
    /// Checks advanced.api_key_vars first. If not found there, returns the
    /// default environment variable name for the provider (e.g., "OPENAI_API_KEY").
    pub fn api_key_var(&self, provider: &str) -> String {
        if let Some(advanced) = &self.advanced {
            if let Some(vars) = &advanced.api_key_vars {
                if let Some(var_name) = vars.get(provider) {
                    return var_name.clone();
                }
            }
        }
        Self::default_api_key_var(provider)
    }

    /// Returns the standard default environment variable name for a provider.
    /// This is the built-in convention used when no custom mapping is configured.
    pub fn default_api_key_var(provider: &str) -> String {
        match provider {
            "openai" => "OPENAI_API_KEY",
            "anthropic" => "ANTHROPIC_API_KEY",
            "google" => "GEMINI_API_KEY",
            "openrouter" => "OPENROUTER_API_KEY",
            "groq" => "GROQ_API_KEY",
            "together" => "TOGETHER_API_KEY",
            "mistral" => "MISTRAL_API_KEY",
            "deepseek" => "DEEPSEEK_API_KEY",
            _ => return format!("{}_API_KEY", provider.to_uppercase()),
        }
        .to_string()
    }

    /// Serializes and writes the config to a TOML file.
    /// Creates parent directories if they don't exist.
    pub fn save(&self, path: &PathBuf) -> Result<(), CrabError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let contents = toml::to_string_pretty(self)?;
        std::fs::write(path, contents)?;
        Ok(())
    }
}

/// Expands tilde (~/) at the start of a path to the user's home directory.
/// Does not support ~username syntax or $VAR interpolation.
/// Returns the path unchanged if tilde expansion is not applicable.
fn shellexpand(path: &str) -> String {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest).to_string_lossy().into_owned();
        }
    }
    path.to_string()
}
