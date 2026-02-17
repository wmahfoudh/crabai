use async_trait::async_trait;
use gcp_auth::provider;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::r#trait::Provider;
use crate::error::CrabError;

/// Google Vertex AI API.
///
/// Authentication: This provider uses Application Default Credentials (ADC). The user should
/// authenticate once by running `gcloud auth application-default login`. After that, `crabai`
/// will automatically discover and use the credentials to generate access tokens.
pub struct GoogleProvider {
    client: Client,
}

impl GoogleProvider {
    const SEND_URL: &'static str =
        "https://us-central1-aiplatform.googleapis.com/v1/projects/gemini-experimental/locations/us-central1/publishers/google/models";
    const LIST_MODELS_URL: &'static str =
        "https://us-central1-aiplatform.googleapis.com/v1/publishers/google/models";

    /// Creates a new provider instance.
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    async fn get_token(&self) -> Result<String, CrabError> {
        let provider = provider().await?;
        let scopes = &["https://www.googleapis.com/auth/cloud-platform"];
        let token = provider.token(scopes).await?;
        Ok(token.as_str().to_string())
    }

    fn static_models() -> Vec<String> {
        vec![
            "gemini-1.5-pro-latest".to_string(),
            "gemini-1.5-flash-latest".to_string(),
            "gemini-pro".to_string(),
            "gemini-1.0-pro".to_string(),
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
        let token = self.get_token().await?;
        let url = format!("{}/{}:streamGenerateContent", Self::SEND_URL, model);

        let request = GeminiRequest {
            contents: vec![GeminiContent {
                parts: vec![GeminiPart {
                    text: prompt.to_string(),
                }],
            }],
            generation_config: Some(GenerationConfig {
                temperature,
                max_output_tokens: max_tokens,
            }),
        };

        let resp = self
            .client
            .post(&url)
            .bearer_auth(token)
            .json(&request)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(CrabError::ProviderError {
                provider: "google".to_string(),
                message: format!("HTTP {status}: {body}"),
            });
        }

        // Vertex AI's streaming response is a bit different. We'll take the first part.
        let full_body = resp.text().await?;
        let first_json = full_body
            .lines()
            .find(|line| line.starts_with("data: "))
            .map(|line| line.strip_prefix("data: ").unwrap_or_default())
            .unwrap_or_default();

        let gemini_resp: GeminiResponse = serde_json::from_str(first_json)?;

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
        let token = match self.get_token().await {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Warning: Google authentication failed: {e}");
                eprintln!("Falling back to static model list. To fix, run: gcloud auth application-default login");
                return Ok(Self::static_models());
            }
        };

        let resp = self
            .client
            .get(Self::LIST_MODELS_URL)
            .bearer_auth(token)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(CrabError::ProviderError {
                provider: "google".to_string(),
                message: format!("Failed to list models: HTTP {status}: {body}"),
            });
        }

        let list: ModelsListResponse = resp.json().await?;

        let mut models: Vec<String> = list
            .models
            .into_iter()
            .filter_map(|m| m.name.split('/').last().map(|s| s.to_string()))
            .collect();

        models.sort();
        models.dedup();
        Ok(models)
    }

    fn name(&self) -> &str {
        "google"
    }
}

impl From<gcp_auth::Error> for CrabError {
    fn from(err: gcp_auth::Error) -> Self {
        CrabError::ConfigError(format!("Google Authentication Error: {err}"))
    }
}
