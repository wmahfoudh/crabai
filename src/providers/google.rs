use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::r#trait::Provider;
use crate::error::CrabError;

/// Google Gemini API. Uses a custom request format (not OpenAI-compatible).
/// Authentication is via query parameter, not Authorization header.
pub struct GoogleProvider {
    client: Client,
    api_key: Option<String>,
}

impl GoogleProvider {
    const BASE_URL: &'static str = "https://generativelanguage.googleapis.com/v1beta";

    pub fn new_with_env(env_var: &str) -> Result<Self, CrabError> {
        Ok(Self {
            client: Client::new(),
            api_key: std::env::var(env_var).ok(),
        })
    }

    fn require_key(&self) -> Result<&str, CrabError> {
        self.api_key
            .as_deref()
            .ok_or_else(|| CrabError::MissingApiKey("google".to_string()))
    }

    fn static_models() -> Vec<String> {
        vec![
            "gemini-1.5-pro-latest".to_string(),
            "gemini-1.5-flash-latest".to_string(),
            "gemini-pro".to_string(),
        ]
    }
}

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(rename = "generationConfig", skip_serializing_if = "Option::is_none")]
    generation_config: Option<GenerationConfig>,
}

#[derive(Serialize)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

#[derive(Serialize)]
struct GeminiPart {
    text: String,
}

#[derive(Serialize)]
struct GenerationConfig {
    temperature: f32,
    #[serde(rename = "maxOutputTokens")]
    max_output_tokens: u32,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<Candidate>>,
}

#[derive(Deserialize)]
struct Candidate {
    content: CandidateContent,
}

#[derive(Deserialize)]
struct CandidateContent {
    parts: Vec<CandidatePart>,
}

#[derive(Deserialize)]
struct CandidatePart {
    text: String,
}

#[derive(Deserialize)]
struct ModelsListResponse {
    models: Vec<ModelEntry>,
}

#[derive(Deserialize)]
struct ModelEntry {
    name: String,
}

#[async_trait]
impl Provider for GoogleProvider {
    async fn send(
        &self,
        model: &str,
        prompt: &str,
        temperature: f32,
        max_tokens: u32,
    ) -> Result<String, CrabError> {
        let api_key = self.require_key()?;
        let url = format!(
            "{}/models/{}:generateContent?key={}",
            Self::BASE_URL,
            model,
            api_key
        );

        let request = GeminiRequest {
            contents: vec![GeminiContent {
                role: "user".to_string(),
                parts: vec![GeminiPart {
                    text: prompt.to_string(),
                }],
            }],
            generation_config: Some(GenerationConfig {
                temperature,
                max_output_tokens: max_tokens,
            }),
        };

        let resp = self.client.post(&url).json(&request).send().await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(CrabError::ProviderError {
                provider: "google".to_string(),
                message: format!("HTTP {status}: {body}"),
            });
        }

        let gemini_resp: GeminiResponse = resp.json().await?;
        gemini_resp
            .candidates
            .and_then(|c| c.into_iter().next())
            .and_then(|c| c.content.parts.into_iter().next())
            .map(|p| p.text)
            .ok_or_else(|| CrabError::ProviderError {
                provider: "google".to_string(),
                message: "Empty response".to_string(),
            })
    }

    async fn list_models(&self) -> Result<Vec<String>, CrabError> {
        let api_key = match self.require_key() {
            Ok(k) => k,
            Err(_) => return Ok(Self::static_models()),
        };
        let url = format!("{}/models?key={}", Self::BASE_URL, api_key);
        let resp = self.client.get(&url).send().await?;

        let status = resp.status();
        if !status.is_success() {
            // Don't error, just fall back to static list.
            return Ok(Self::static_models());
        }

        match resp.json::<ModelsListResponse>().await {
            Ok(list) => {
                let mut models: Vec<String> = list
                    .models
                    .into_iter()
                    .map(|m| {
                        m.name
                            .strip_prefix("models/")
                            .unwrap_or(&m.name)
                            .to_string()
                    })
                    .collect();
                models.sort();
                Ok(models)
            }
            Err(_) => Ok(Self::static_models()),
        }
    }

    fn name(&self) -> &str {
        "google"
    }
}
