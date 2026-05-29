use crate::error::AppError;
use crate::providers::openai_compatible::{self, Auth};
use crate::providers::{AIProvider, ProviderResponse};
use async_trait::async_trait;

const DEFAULT_MODEL: &str = "gpt-4o-mini";
const ENDPOINT: &str = "https://api.openai.com/v1/chat/completions";

/// OpenAI chat-completions provider. Defaults to `gpt-4o-mini`; an optional
/// model override may be supplied (e.g. `gpt-4o`).
pub struct OpenAIProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl OpenAIProvider {
    /// Creates a new [`OpenAIProvider`] with the default model.
    pub fn new(api_key: String) -> Self {
        Self::with_model(api_key, None)
    }

    /// Creates a new [`OpenAIProvider`] with an optional model override.
    pub fn with_model(api_key: String, model: Option<String>) -> Self {
        Self {
            api_key,
            model: model.unwrap_or_else(|| DEFAULT_MODEL.to_string()),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl AIProvider for OpenAIProvider {
    async fn complete(&self, system: &str, user: &str) -> Result<ProviderResponse, AppError> {
        openai_compatible::complete(
            &self.client,
            ENDPOINT,
            Auth::Bearer(&self.api_key),
            &[],
            &self.model,
            "openai",
            (system, user),
        )
        .await
    }

    fn provider_id(&self) -> &'static str {
        "openai"
    }

    fn requires_api_key(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_provider() -> OpenAIProvider {
        OpenAIProvider::new("test-key".to_string())
    }

    #[test]
    fn test_openai_provider_id_is_correct() {
        assert_eq!(make_provider().provider_id(), "openai");
    }

    #[test]
    fn test_openai_requires_api_key() {
        assert!(make_provider().requires_api_key());
    }

    #[test]
    fn test_openai_default_model() {
        assert_eq!(make_provider().model, "gpt-4o-mini");
    }

    #[test]
    fn test_openai_model_override() {
        let p = OpenAIProvider::with_model("k".to_string(), Some("gpt-4o".to_string()));
        assert_eq!(p.model, "gpt-4o");
    }
}
