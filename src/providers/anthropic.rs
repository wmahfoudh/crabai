use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::CrabError;
use super::r#trait::Provider;

/// Anthropic Messages API. Uses a custom request format (not OpenAI-compatible).
/// Model listing returns a static fallback list; no API key required for that.
pub struct AnthropicProvider {
    client: Client,
    api_key: Option<String>,
}

impl AnthropicProvider {
    const API_URL: &'static str = "https://api.anthropic.com/v1/messages";
    const API_VERSION: &'static str = "2023-06-01";

    pub fn new() -> Result<Self, CrabError> {
        let api_key = std::env::var("ANTHROPIC_API_KEY").ok();
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
            "claude-sonnet-4-5-20250514".to_string(),
            "claude-3-5-haiku-20241022".to_string(),
            "claude-3-5-sonnet-20241022".to_string(),
            "claude-3-opus-20240229".to_string(),
        ]
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
        Ok(Self::static_models())
    }

    fn name(&self) -> &str {
        "anthropic"
    }
}
