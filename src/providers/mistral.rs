use async_trait::async_trait;
use reqwest::Client;

use super::openai_compat;
use super::r#trait::Provider;
use crate::error::CrabError;
use crate::types::ModelInfo;

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

    fn static_models() -> Vec<ModelInfo> {
        vec![
            "mistral-large-latest",
            "mistral-medium-latest",
            "mistral-small-latest",
            "open-mistral-7b",
        ]
        .into_iter()
        .map(|id| {
            let mut info = ModelInfo::new(id);
            info.max_output_tokens = Some(8192);
            info
        })
        .collect()
    }
}

#[async_trait]
impl Provider for MistralProvider {
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
        "mistral"
    }
}
