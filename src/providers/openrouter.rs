use async_trait::async_trait;
use reqwest::Client;

use super::openai_compat;
use super::r#trait::Provider;
use crate::error::CrabError;

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
        match self.list_models_api().await {
            Ok(models) => Ok(models.into_iter().map(|m| m.id).collect()),
            Err(_) => Ok(Self::static_models()),
        }
    }

    fn name(&self) -> &str {
        "openrouter"
    }

    async fn fetch_max_tokens(&self, model_id: &str) -> Result<Option<u32>, CrabError> {
        let models = self.list_models_api().await?;

        let model = models.into_iter().find(|m| m.id == model_id);

        Ok(model.and_then(|m| m.top_provider.max_completion_tokens))
    }
}

// Add the structs needed for deserialization of the model list
#[derive(serde::Deserialize)]
struct OpenRouterModel {
    id: String,
    #[serde(rename = "top_provider")]
    top_provider: TopProvider,
}

#[derive(serde::Deserialize)]
struct TopProvider {
    #[serde(rename = "max_completion_tokens")]
    max_completion_tokens: Option<u32>,
}

// Modify the existing list_models_api to use the new structs
impl OpenRouterProvider {
    async fn list_models_api(&self) -> Result<Vec<OpenRouterModel>, CrabError> {
        #[derive(serde::Deserialize)]
        struct ModelsList {
            data: Vec<OpenRouterModel>,
        }

        let api_key = self.require_key()?;
        let resp = self
            .client
            .get(format!("{}/models", Self::BASE_URL))
            .bearer_auth(api_key)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(CrabError::ProviderError {
                provider: "openrouter".to_string(),
                message: format!("API error: {}", resp.status()),
            });
        }

        let models_list: ModelsList = resp.json().await.map_err(|e| CrabError::ProviderError {
            provider: "openrouter".to_string(),
            message: format!("Failed to parse models list: {}", e),
        })?;

        Ok(models_list.data)
    }
}
