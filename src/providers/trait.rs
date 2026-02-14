use async_trait::async_trait;

use crate::error::CrabError;

/// Interface that each LLM provider must implement. Providers are
/// isolated: each owns its HTTP client and API key independently.
#[async_trait]
pub trait Provider: Send + Sync {
    /// Send a single user-role message and return the model's text response.
    async fn send(
        &self,
        model: &str,
        prompt: &str,
        temperature: f32,
        max_tokens: u32,
    ) -> Result<String, CrabError>;

    /// Return available model IDs. May query the provider API or return
    /// a static fallback list depending on the implementation.
    async fn list_models(&self) -> Result<Vec<String>, CrabError>;

    /// Lowercase provider identifier (e.g. "openai", "anthropic").
    fn name(&self) -> &str;
}
