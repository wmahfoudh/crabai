use std::path::PathBuf;

use serde::Deserialize;

use crate::error::CrabError;

/// Deserialized from ~/.config/crabai/config.toml. All fields are optional;
/// missing values fall through to internal defaults.
#[derive(Debug, Deserialize, Default)]
pub struct Config {
    pub default_provider: Option<String>,
    pub default_model: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub prompts_dir: Option<String>,
    pub model_cache: Option<bool>,
    pub model_cache_ttl_hours: Option<u64>,
}

impl Config {
    /// Load config from an explicit path or the default location.
    /// Returns default Config if the file does not exist.
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

    /// Platform-appropriate config directory (e.g. ~/.config/crabai on Linux).
    /// Falls back to a literal "~/.config" path if the platform has no
    /// standard config directory; tilde is not expanded in that case.
    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("crabai")
    }

    pub fn default_config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    /// Resolve prompts directory. Supports ~/  expansion in config values.
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
}

/// Minimal tilde expansion. Only handles the "~/" prefix; does not
/// support ~user syntax or environment variable interpolation.
fn shellexpand(path: &str) -> String {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest).to_string_lossy().into_owned();
        }
    }
    path.to_string()
}
