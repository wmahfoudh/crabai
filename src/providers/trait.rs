use async_trait::async_trait;
use crate::error::CrabError;
use crate::types::ModelInfo;

/// Common interface for all LLM provider implementations.
///
/// Each provider is self-contained: it owns its HTTP client, manages API
/// authentication, and handles request/response formatting according to
/// the provider's specific API contract.
///
/// Providers must be Send + Sync to support async/tokio runtime.
#[async_trait]
pub trait Provider: Send + Sync {
    /// Sends a prompt to the model and returns the text response.
    ///
    /// This method performs the core LLM inference operation. The prompt
    /// is sent as a single user message (no conversation history). The
    /// temperature and max_tokens parameters control response generation.
    ///
    /// Returns the raw text content from the model without additional
    /// formatting or metadata.
    async fn send(
        &self,
        model: &str,
        prompt: &str,
        temperature: Option<f32>,
        max_tokens: u32,
        max_tokens_key: Option<String>,
    ) -> Result<String, CrabError>;

    /// Returns a list of available model identifiers and their capabilities.
    async fn list_models(&self) -> Result<Vec<ModelInfo>, CrabError>;

    /// Returns the lowercase provider identifier.
    /// Examples: "openai", "anthropic", "google", "mistral"
    fn name(&self) -> &str;

    /// Sanitizes parameters based on model-specific constraints.
    /// Returns (final_temperature, final_max_tokens).
    /// Returning None for temperature signals the provider to omit the parameter from the request.
    fn sanitize_params(&self, _model: &str, temperature: f32, max_tokens: u32) -> (Option<f32>, u32) {
        (Some(temperature), max_tokens)
    }
}
