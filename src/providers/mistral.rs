use async_trait::async_trait;
use reqwest::Client;

use super::openai_compat;
use super::r#trait::Provider;
use crate::error::CrabError;

/// Mistral AI inference API. OpenAI-compatible.
pub struct MistralProvider {
    client: Client,
    api_key: Option<String>,
}

impl MistralProvider {
    const BASE_URL: &'static str = "https://api.mistral.ai/v1";

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
            .ok_or_else(|| CrabError::MissingApiKey("mistral".to_string()))
    }

    fn static_models() -> Vec<String> {
        vec![
            "mistral-large-latest".to_string(),
            "mistral-medium-latest".to_string(),
            "mistral-small-latest".to_string(),
            "open-mistral-7b".to_string(),
        ]
    }
}

#[async_trait]
impl Provider for MistralProvider {
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
        "mistral"
    }

    fn get_max_tokens(&self, model: &str) -> Option<u32> {
        match model {
            // Mistral models often have large context windows, and the max output
            // is not explicitly limited to a smaller value.
            "mistral-large-latest" => Some(32768),
            "open-mixtral-8x7b" => Some(32768),
            "open-mistral-7b" => Some(32768),
            _ => Some(8192), // A sensible default for other Mistral models
        }
    }
}
