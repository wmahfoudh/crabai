use async_trait::async_trait;
use reqwest::Client;

use super::openai_compat;
use super::r#trait::Provider;
use crate::error::CrabError;

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

    fn get_max_tokens(&self, model: &str) -> Option<u32> {
        match model {
            // Sourced from Groq's model documentation.
            "llama-3.3-70b-versatile" => Some(32768),
            "llama-3.1-70b-versatile" => Some(131072),
            "llama-3.1-8b-instant" => Some(131072),
            "mixtral-8x7b-32768" => Some(32768),
            "gemma2-9b-it" => Some(65536),
            "gemma-7b-it" => Some(8192),
            _ => None,
        }
    }
}
