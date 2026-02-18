use async_trait::async_trait;
use reqwest::Client;

use super::openai_compat;
use super::r#trait::Provider;
use crate::error::CrabError;

/// DeepSeek chat API (OpenAI-compatible). Model listing falls back to a
/// static list if no API key is set or if the models endpoint fails.
pub struct DeepSeekProvider {
    client: Client,
    api_key: Option<String>,
}

impl DeepSeekProvider {
    const BASE_URL: &'static str = "https://api.deepseek.com";

    /// Creates a new provider instance with a custom environment variable name.
    pub fn new_with_env(env_var: &str) -> Result<Self, CrabError> {
        Ok(Self {
            client: Client::new(),
            api_key: std::env::var(env_var).ok(),
        })
    }

    fn require_key(&self) -> Result<&str, CrabError> {
        self.api_key
            .as_deref()
            .ok_or_else(|| CrabError::MissingApiKey("deepseek".to_string()))
    }

    fn static_models() -> Vec<String> {
        vec!["deepseek-chat".to_string(), "deepseek-reasoner".to_string()]
    }
}

#[async_trait]
impl Provider for DeepSeekProvider {
    async fn send(
        &self,
        model: &str,
        prompt: &str,
        temperature: f32,
        max_tokens: u32,
    ) -> Result<String, CrabError> {
        openai_compat::send_chat_request(
            &self.client,
            Self::BASE_URL,
            self.require_key()?,
            model,
            prompt,
            temperature,
            max_tokens,
        )
        .await
    }

    async fn list_models(&self) -> Result<Vec<String>, CrabError> {
        let api_key = match self.require_key() {
            Ok(k) => k,
            Err(_) => return Ok(Self::static_models()),
        };
        match openai_compat::list_models_api(&self.client, Self::BASE_URL, api_key).await {
            Ok(models) => Ok(models),
            Err(_) => Ok(Self::static_models()),
        }
    }

    fn name(&self) -> &str {
        "deepseek"
    }

    fn get_max_tokens(&self, model: &str) -> Option<u32> {
        match model {
            // Sourced from DeepSeek's model documentation.
            "deepseek-chat" => Some(8192),
            "deepseek-coder" => Some(16384),
            _ => None,
        }
    }
}
