use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::openai_compat;
use super::r#trait::Provider;
use crate::config::Config;
use crate::error::CrabError;

use crate::types::ModelInfo;

pub struct OpenAIProvider {
    client: Client,
    api_key: Option<String>,
    max_tokens_param: String,
}

impl OpenAIProvider {
    const BASE_URL: &'static str = "https://api.openai.com/v1";

    /// Creates a new provider instance from the application config.
    pub fn new(config: &Config) -> Result<Self, CrabError> {
        let api_key_var = config.api_key_var("openai");
        let max_tokens_param = config
            .advanced
            .as_ref()
            .and_then(|a| a.openai.as_ref())
            .and_then(|o| o.max_tokens_param.as_ref())
            .cloned()
            .unwrap_or_else(|| "max_tokens".to_string());

        Ok(Self {
            client: Client::new(),
            api_key: std::env::var(api_key_var).ok(),
            max_tokens_param,
        })
    }

    fn require_key(&self) -> Result<&str, CrabError> {
        self.api_key
            .as_deref()
            .ok_or_else(|| CrabError::MissingApiKey("openai".to_string()))
    }

    fn static_models() -> Vec<ModelInfo> {
        // A comprehensive fallback list of common models.
        vec![
            "gpt-4o",
            "gpt-4o-mini",
            "gpt-4-turbo",
            "gpt-4-turbo-preview",
            "gpt-4-vision-preview",
            "gpt-4-32k",
            "gpt-4",
            "gpt-3.5-turbo",
            "gpt-3.5-turbo-16k",
        ]
        .into_iter()
        .map(|id| {
            let mut info = ModelInfo::new(id);
            // Enrich with known limits
            if id == "gpt-4o" || id == "gpt-4o-mini" {
                info.max_output_tokens = Some(16384);
            } else if id.contains("gpt-4") {
                info.max_output_tokens = Some(4096);
            }
            info
        })
        .collect()
    }
}

// Custom request/response structs to avoid dependency on openai_compat's send method.
#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatResponseMessage,
}

#[derive(Deserialize)]
struct ChatResponseMessage {
    content: String,
}

#[async_trait]
impl Provider for OpenAIProvider {
    async fn send(
        &self,
        model: &str,
        prompt: &str,
        temperature: Option<f32>,
        max_tokens: u32,
        max_tokens_key: Option<String>,
    ) -> Result<String, CrabError> {
        let mut request_body = serde_json::json!({
            "model": model,
            "messages": [
                ChatMessage {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                }
            ],
        });

        if let Some(t) = temperature {
            request_body["temperature"] = serde_json::json!(t);
        }

        // Use the explicitly provided key name, or determine the correct one based on the model.
        let key = if let Some(k) = max_tokens_key {
            k
        } else if model.starts_with("o1") || model.starts_with("o3") || model == "gpt-5" {
            "max_completion_tokens".to_string()
        } else {
            self.max_tokens_param.clone()
        };

        // Add the max tokens parameter with the resolved name.
        request_body[key] = serde_json::json!(max_tokens);


        let url = format!("{}/chat/completions", Self::BASE_URL);
        let resp = self
            .client
            .post(&url)
            .bearer_auth(self.require_key()?)
            .json(&request_body)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(CrabError::ProviderError {
                provider: url,
                message: format!("HTTP {status}: {body}"),
            });
        }

        let chat_resp: ChatResponse = resp.json().await?;
        chat_resp
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or_else(|| CrabError::ProviderError {
                provider: "openai".to_string(),
                message: "Empty response".to_string(),
            })
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>, CrabError> {
        let api_key = match self.require_key() {
            Ok(k) => k,
            Err(_) => return Ok(Self::static_models()),
        };
        match openai_compat::list_models_api(&self.client, Self::BASE_URL, api_key).await {
            Ok(mut models) => {
                // Enrich discovered models with known capabilities
                for m in &mut models {
                    if m.id.starts_with("o1-") || m.id.starts_with("o3-") {
                        m.supports_temperature = false;
                        m.max_tokens_param = Some("max_completion_tokens".to_string());
                    }
                }
                Ok(models)
            }
            Err(_) => Ok(Self::static_models()),
        }
    }

    fn name(&self) -> &str {
        "openai"
    }

    fn sanitize_params(&self, model: &str, temperature: f32, max_tokens: u32) -> (Option<f32>, u32) {
        let mut final_temp = Some(temperature);

        // Reasoning models do not support temperature (it must be 1.0 or omitted).
        // For simplicity and safety, we omit it.
        if model.starts_with("o1") || model.starts_with("o3") {
            final_temp = None;
        } else if model == "gpt-5" {
            // Placeholder for future models with specific constraints.
            final_temp = Some(1.0);
        } else if model.starts_with("gpt-") && temperature > 1.0 {
            final_temp = Some(1.0);
        }

        (final_temp, max_tokens)
    }
}
