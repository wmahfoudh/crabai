use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::r#trait::Provider;
use crate::error::CrabError;

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

    fn static_models() -> Vec<String> {
        vec![
            "claude-sonnet-4-20250514".to_string(),
            "claude-opus-4-20250514".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
            "claude-3-5-haiku-20241022".to_string(),
            "claude-3-opus-20240229".to_string(),
        ]
    }

    /// Fetch models from Anthropic's models API endpoint.
    async fn list_models_api(&self) -> Result<Vec<String>, CrabError> {
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
        let models: Vec<String> = response.data.into_iter().map(|m| m.id).collect();

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
        temperature: f32,
        max_tokens: u32,
    ) -> Result<String, CrabError> {
        let request = AnthropicRequest {
            model: model.to_string(),
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            max_tokens,
            temperature: Some(temperature),
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

    async fn list_models(&self) -> Result<Vec<String>, CrabError> {
        // Try to fetch from API first, fall back to static list on failure
        match self.list_models_api().await {
            Ok(models) => Ok(models),
            Err(_) => Ok(Self::static_models()),
        }
    }

    fn name(&self) -> &str {
        "anthropic"
    }

    fn get_max_tokens(&self, model: &str) -> Option<u32> {
        // Newer Haiku models support a much larger output, according to API error messages.
        if model.contains("haiku") {
            Some(64000)
        }
        // Other Claude 3 models have a documented limit of 8192.
        else if model.contains("claude-3") || model.contains("claude-3.5") {
            Some(8192)
        }
        // Provide a safe fallback for any other claude models.
        else if model.starts_with("claude-") {
            Some(8192)
        } else {
            None
        }
    }
}
