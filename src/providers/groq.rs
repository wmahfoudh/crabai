use async_trait::async_trait;
use reqwest::Client;

use super::openai_compat;
use super::r#trait::Provider;
use crate::error::CrabError;
use crate::types::ModelInfo;

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

    fn static_models() -> Vec<ModelInfo> {
        vec![
            "llama-3.3-70b-versatile",
            "llama-3.1-70b-versatile",
            "mixtral-8x7b-32768",
            "gemma2-9b-it",
        ]
        .into_iter()
        .map(|id| {
            let mut info = ModelInfo::new(id);
            if id.contains("llama") {
                info.max_output_tokens = Some(32768);
            }
            info
        })
        .collect()
    }
}

#[async_trait]
impl Provider for GroqProvider {
    async fn send(
        &self,
        model: &str,
        prompt: &str,
        temperature: Option<f32>,
        max_tokens: u32,
        max_tokens_key: Option<String>,
    ) -> Result<String, CrabError> {
        let api_key = self.require_key()?;
        openai_compat::send_chat_request(
            &self.client,
            Self::BASE_URL,
            api_key,
            model,
            prompt,
            temperature,
            max_tokens,
            max_tokens_key,
        )
        .await
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>, CrabError> {
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
