use async_trait::async_trait;

use crate::error::CrabError;

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
        temperature: f32,
        max_tokens: u32,
    ) -> Result<String, CrabError>;

    /// Returns a list of available model identifiers for this provider.
    ///
    /// Implementation varies by provider:
    /// - Some query a models API endpoint (OpenAI, Google, OpenRouter, etc.)
    /// - Some return a static fallback list (Anthropic, when no API key set)
    /// - Some use a hybrid approach (Google, DeepSeek - API with fallback)
    ///
    /// Model IDs are returned in a provider-specific format suitable for
    /// use with the send() method.
    async fn list_models(&self) -> Result<Vec<String>, CrabError>;

    /// Returns the lowercase provider identifier.
    /// Examples: "openai", "anthropic", "google", "mistral"
    fn name(&self) -> &str;

    /// Returns the maximum number of tokens for a given model.
    /// Default implementation returns None.
    fn get_max_tokens(&self, _model: &str) -> Option<u32> {
        None
    }

    /// Fetches the maximum number of tokens for a given model from the API.
    /// Default implementation calls the synchronous get_max_tokens.
    async fn fetch_max_tokens(&self, model: &str) -> Result<Option<u32>, CrabError> {
        Ok(self.get_max_tokens(model))
    }
}
