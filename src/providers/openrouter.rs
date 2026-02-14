use async_trait::async_trait;
use reqwest::Client;

use crate::error::CrabError;
use super::openai_compat;
use super::r#trait::Provider;

/// OpenRouter aggregator. OpenAI-compatible API.
pub struct OpenRouterProvider {
    client: Client,
    api_key: Option<String>,
}

impl OpenRouterProvider {
    const BASE_URL: &'static str = "https://openrouter.ai/api/v1";

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
            .ok_or_else(|| CrabError::MissingApiKey("openrouter".to_string()))
    }

    fn static_models() -> Vec<String> {
        vec![
            "anthropic/claude-sonnet-4-20250514".to_string(),
            "openai/gpt-4o".to_string(),
            "google/gemini-2.0-flash-exp".to_string(),
            "meta-llama/llama-3.3-70b-instruct".to_string(),
        ]
    }
}

#[async_trait]
impl Provider for OpenRouterProvider {
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
        "openrouter"
    }
}
