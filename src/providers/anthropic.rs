use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::r#trait::Provider;
use crate::error::CrabError;

use crate::types::ModelInfo;

/// Anthropic Messages API. Uses a custom request format (not OpenAI-compatible).
/// Model listing returns a static fallback list; no API key required for that.
pub struct AnthropicProvider {
    client: Client,
    api_key: Option<String>,
}

impl AnthropicProvider {
    const API_URL: &'static str = "https://api.anthropic.com/v1/messages";
    const API_VERSION: &'static str = "2023-06-01";

    /// Creates a new provider instance with a custom environment variable name.
    pub fn new_with_env(env_var: &str) -> Result<Self, CrabError> {
        let api_key = std::env::var(env_var).ok();
        Ok(Self {
            client: Client::new(),
            api_key,
        })
    }

    fn require_key(&self) -> Result<&str, CrabError> {
        self.api_key
            .as_deref()
            .ok_or_else(|| CrabError::MissingApiKey("anthropic".to_string()))
    }

    fn static_models() -> Vec<ModelInfo> {
        vec![
            "claude-sonnet-4-20250514",
            "claude-opus-4-20250514",
            "claude-3-5-sonnet-20241022",
            "claude-3-5-haiku-20241022",
            "claude-3-opus-20240229",
        ]
        .into_iter()
        .map(|id| {
            ModelInfo::new(id)
        })
        .collect()
    }

    /// Fetch models from Anthropic's models API endpoint.
    async fn list_models_api(&self) -> Result<Vec<ModelInfo>, CrabError> {
        let api_key = self.require_key()?;
        let url = "https://api.anthropic.com/v1/models";

        let resp = self
            .client
            .get(url)
            .header("x-api-key", api_key)
            .header("anthropic-version", Self::API_VERSION)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(CrabError::ProviderError {
                provider: "anthropic".to_string(),
                message: format!("HTTP {}", resp.status()),
            });
        }

        #[derive(Deserialize)]
        struct ModelsResponse {
            data: Vec<ModelEntry>,
        }

        #[derive(Deserialize)]
        struct ModelEntry {
            id: String,
        }

        let response: ModelsResponse = resp.json().await?;
        let models: Vec<ModelInfo> = response
            .data
            .into_iter()
            .map(|m| {
                ModelInfo::new(&m.id)
            })
            .collect();

        Ok(models)
    }
}

#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<ContentBlock>,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: String,
}

#[async_trait]
impl Provider for AnthropicProvider {
    async fn send(
        &self,
        model: &str,
        prompt: &str,
        temperature: Option<f32>,
        max_tokens: u32,
        _max_tokens_key: Option<String>,
    ) -> Result<String, CrabError> {
        let request = AnthropicRequest {
            model: model.to_string(),
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            max_tokens,
            temperature,
        };

        let api_key = self.require_key()?;

        let resp = self
            .client
            .post(Self::API_URL)
            .header("x-api-key", api_key)
            .header("anthropic-version", Self::API_VERSION)
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(CrabError::ProviderError {
                provider: "anthropic".to_string(),
                message: format!("HTTP {status}: {body}"),
            });
        }

        let anthropic_resp: AnthropicResponse = resp.json().await?;
        anthropic_resp
            .content
            .into_iter()
            .next()
            .map(|b| b.text)
            .ok_or_else(|| CrabError::ProviderError {
                provider: "anthropic".to_string(),
                message: "Empty response".to_string(),
            })
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>, CrabError> {
        // Try to fetch from API first, fall back to static list on failure
        match self.list_models_api().await {
            Ok(models) => Ok(models),
            Err(_) => Ok(Self::static_models()),
        }
    }

    fn name(&self) -> &str {
        "anthropic"
    }

    fn sanitize_params(&self, _model: &str, temperature: f32, max_tokens: u32) -> (Option<f32>, u32) {
        (Some(temperature), max_tokens)
    }
}
