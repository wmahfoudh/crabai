use async_trait::async_trait;
use reqwest::Client;

use crate::error::CrabError;
use super::openai_compat;
use super::r#trait::Provider;

/// Together AI inference API. OpenAI-compatible.
pub struct TogetherProvider {
    client: Client,
    api_key: Option<String>,
}

impl TogetherProvider {
    const BASE_URL: &'static str = "https://api.together.xyz/v1";

    pub fn new() -> Result<Self, CrabError> {
        Ok(Self {
            client: Client::new(),
            api_key: std::env::var("TOGETHER_API_KEY").ok(),
        })
    }

    fn require_key(&self) -> Result<&str, CrabError> {
        self.api_key
            .as_deref()
            .ok_or_else(|| CrabError::MissingApiKey("together".to_string()))
    }
}

#[async_trait]
impl Provider for TogetherProvider {
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
        openai_compat::list_models_api(&self.client, Self::BASE_URL, self.require_key()?).await
    }

    fn name(&self) -> &str {
        "together"
    }
}
