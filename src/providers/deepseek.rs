use async_trait::async_trait;
use reqwest::Client;

use super::openai_compat;
use super::r#trait::Provider;
use crate::error::CrabError;

use crate::types::ModelInfo;

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

    fn static_models() -> Vec<ModelInfo> {
        vec!["deepseek-chat", "deepseek-reasoner"]
            .into_iter()
            .map(|id| {
                let mut info = ModelInfo::new(id);
                if id == "deepseek-reasoner" {
                    info.supports_temperature = false;
                }
                info
            })
            .collect()
    }
}

#[async_trait]
impl Provider for DeepSeekProvider {
    async fn send(
        &self,
        model: &str,
        prompt: &str,
        temperature: Option<f32>,
        max_tokens: u32,
        max_tokens_key: Option<String>,
    ) -> Result<String, CrabError> {
        openai_compat::send_chat_request(
            &self.client,
            Self::BASE_URL,
            self.require_key()?,
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
            Ok(mut models) => {
                for m in &mut models {
                    if m.id == "deepseek-reasoner" {
                        m.supports_temperature = false;
                    }
                }
                Ok(models)
            }
            Err(_) => Ok(Self::static_models()),
        }
    }

    fn name(&self) -> &str {
        "deepseek"
    }

    fn sanitize_params(&self, model: &str, temperature: f32, max_tokens: u32) -> (Option<f32>, u32) {
        if model == "deepseek-reasoner" {
            // Omit temperature for deepseek-reasoner.
            (None, max_tokens)
        } else {
            (Some(temperature), max_tokens)
        }
    }
}
