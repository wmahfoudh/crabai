use async_trait::async_trait;
use reqwest::Client;

use crate::error::CrabError;
use super::openai_compat;
use super::r#trait::Provider;

/// Groq inference API. OpenAI-compatible.
pub struct GroqProvider {
    client: Client,
    api_key: Option<String>,
}

impl GroqProvider {
    const BASE_URL: &'static str = "https://api.groq.com/openai/v1";

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
            .ok_or_else(|| CrabError::MissingApiKey("groq".to_string()))
    }

    fn static_models() -> Vec<String> {
        vec![
            "llama-3.3-70b-versatile".to_string(),
            "llama-3.1-70b-versatile".to_string(),
            "mixtral-8x7b-32768".to_string(),
            "gemma2-9b-it".to_string(),
        ]
    }
}

#[async_trait]
impl Provider for GroqProvider {
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
        "groq"
    }
}
