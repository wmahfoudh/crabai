//! Shared request/response handling for OpenAI-compatible APIs.
//! Used by: OpenAI, OpenRouter, Groq, Together, Mistral, DeepSeek.

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::CrabError;

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

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
    temperature: f32,
    max_tokens: u32,
) -> Result<String, CrabError> {
    let request = ChatRequest {
        model: model.to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: prompt.to_string(),
        }],
        temperature: Some(temperature),
        max_tokens: Some(max_tokens),
    };

    let url = format!("{base_url}/chat/completions");
    let resp = client
        .post(&url)
        .bearer_auth(api_key)
        .json(&request)
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

/// GET {base_url}/models. Returns sorted model IDs.
pub async fn list_models_api(
    client: &Client,
    base_url: &str,
    api_key: &str,
) -> Result<Vec<String>, CrabError> {
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
    let mut models: Vec<String> = models_resp.data.into_iter().map(|m| m.id).collect();
    models.sort();
    Ok(models)
}
