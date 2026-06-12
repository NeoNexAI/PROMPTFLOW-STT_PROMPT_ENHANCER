use crate::error::AppError;
use crate::providers::openai_compatible::{self, Auth};
use crate::providers::{AIProvider, ProviderResponse};
use async_trait::async_trait;

const DEFAULT_MODEL: &str = "llama-3.1-8b-instant";
const ENDPOINT: &str = "https://api.groq.com/openai/v1/chat/completions";

/// Groq chat-completions provider via Groq's OpenAI-compatible API. Defaults to
/// `llama-3.1-8b-instant` for sub-500 ms LPU inference.
pub struct GroqProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl GroqProvider {
    pub fn new(api_key: String) -> Self {
        Self::with_model(api_key, None)
    }

    pub fn with_model(api_key: String, model: Option<String>) -> Self {
        Self {
            api_key,
            model: model.unwrap_or_else(|| DEFAULT_MODEL.to_string()),
            client: crate::http::client(),
        }
    }
}

#[async_trait]
impl AIProvider for GroqProvider {
    async fn complete(&self, system: &str, user: &str) -> Result<ProviderResponse, AppError> {
        openai_compatible::complete(
            &self.client,
            ENDPOINT,
            Auth::Bearer(&self.api_key),
            &[],
            &self.model,
            "groq",
            (system, user),
        )
        .await
    }

    fn provider_id(&self) -> &'static str {
        "groq"
    }

    fn requires_api_key(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_provider() -> GroqProvider {
        GroqProvider::new("test-key".to_string())
    }

    #[test]
    fn test_groq_provider_id_is_correct() {
        assert_eq!(make_provider().provider_id(), "groq");
    }

    #[test]
    fn test_groq_requires_api_key() {
        assert!(make_provider().requires_api_key());
    }

    #[test]
    fn test_groq_default_model() {
        assert_eq!(make_provider().model, "llama-3.1-8b-instant");
    }
}
