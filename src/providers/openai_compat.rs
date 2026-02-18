//! Shared request/response handling for OpenAI-compatible APIs.
//! Used by: OpenAI, OpenRouter, Groq, Together, Mistral, DeepSeek.

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::CrabError;
use crate::types::ModelInfo;

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}

#[derive(Deserialize)]
struct ModelsResponse {
    data: Vec<ModelEntry>,
}

#[derive(Deserialize)]
struct ModelEntry {
    id: String,
}

/// POST {base_url}/chat/completions with a single user message.
pub async fn send_chat_request(
    client: &Client,
    base_url: &str,
    api_key: &str,
    model: &str,
    prompt: &str,
    temperature: Option<f32>,
    max_tokens: u32,
    max_tokens_key: Option<String>,
) -> Result<String, CrabError> {
    let mut request_body = serde_json::json!({
        "model": model,
        "messages": [
            Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }
        ],
    });

    if let Some(t) = temperature {
        request_body["temperature"] = serde_json::json!(t);
    }

    let key = max_tokens_key.unwrap_or_else(|| "max_tokens".to_string());
    request_body[key] = serde_json::json!(max_tokens);

    let url = format!("{base_url}/chat/completions");
    let resp = client
        .post(&url)
        .bearer_auth(api_key)
        .json(&request_body)
        .send()
        .await?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(CrabError::ProviderError {
            provider: base_url.to_string(),
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
            provider: base_url.to_string(),
            message: "Empty response from API".to_string(),
        })
}

/// GET {base_url}/models. Returns sorted model info.
pub async fn list_models_api(
    client: &Client,
    base_url: &str,
    api_key: &str,
) -> Result<Vec<ModelInfo>, CrabError> {
    let url = format!("{base_url}/models");
    let resp = client.get(&url).bearer_auth(api_key).send().await?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(CrabError::ProviderError {
            provider: base_url.to_string(),
            message: format!("HTTP {status}: {body}"),
        });
    }

    let models_resp: ModelsResponse = resp.json().await?;
    let mut models: Vec<ModelInfo> = models_resp
        .data
        .into_iter()
        .map(|m| ModelInfo::new(&m.id))
        .collect();
    models.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(models)
}
