use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::CrabError;
use super::r#trait::Provider;

/// Google Gemini API. Uses a custom request format (not OpenAI-compatible).
/// Authentication is via query parameter, not Authorization header.
/// Model listing falls back to a static list if no API key is set.
pub struct GoogleProvider {
    client: Client,
    api_key: Option<String>,
}

impl GoogleProvider {
    const BASE_URL: &'static str = "https://generativelanguage.googleapis.com/v1beta";

    pub fn new() -> Result<Self, CrabError> {
        Ok(Self {
            client: Client::new(),
            api_key: std::env::var("GOOGLE_API_KEY").ok(),
        })
    }

    fn require_key(&self) -> Result<&str, CrabError> {
        self.api_key
            .as_deref()
            .ok_or_else(|| CrabError::MissingApiKey("google".to_string()))
    }

    fn static_models() -> Vec<String> {
        vec![
            "gemini-2.0-flash".to_string(),
            "gemini-1.5-pro".to_string(),
            "gemini-1.5-flash".to_string(),
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
    #[serde(rename = "supportedGenerationMethods", default)]
    supported_generation_methods: Vec<String>,
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
        let resp = self.client.get(&url).send().await;

        match resp {
            Ok(r) if r.status().is_success() => {
                let list: ModelsListResponse = r.json().await?;
                // API returns names like "models/gemini-2.0-flash"; strip the prefix.
                let mut models: Vec<String> = list
                    .models
                    .into_iter()
                    .filter(|m| m.supported_generation_methods.contains(&"generateContent".to_string()))
                    .map(|m| m.name.strip_prefix("models/").unwrap_or(&m.name).to_string())
                    .collect();
                models.sort();
                Ok(models)
            }
            _ => Ok(Self::static_models()),
        }
    }

    fn name(&self) -> &str {
        "google"
    }
}
