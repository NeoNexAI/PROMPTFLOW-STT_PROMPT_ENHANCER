use crate::error::AppError;
use crate::providers::openai_compatible::{self, Auth};
use crate::providers::{AIProvider, ProviderResponse};
use async_trait::async_trait;

const DEFAULT_MODEL: &str = "mistral-small-latest";
const ENDPOINT: &str = "https://api.mistral.ai/v1/chat/completions";

/// Mistral AI chat-completions provider (OpenAI-compatible). Defaults to
/// `mistral-small-latest`.
pub struct MistralProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl MistralProvider {
    pub fn new(api_key: String) -> Self {
        Self::with_model(api_key, None)
    }

    pub fn with_model(api_key: String, model: Option<String>) -> Self {
        Self {
            api_key,
            model: model.unwrap_or_else(|| DEFAULT_MODEL.to_string()),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl AIProvider for MistralProvider {
    async fn complete(&self, system: &str, user: &str) -> Result<ProviderResponse, AppError> {
        openai_compatible::complete(
            &self.client,
            ENDPOINT,
            Auth::Bearer(&self.api_key),
            &[],
            &self.model,
            "mistral",
            system,
            user,
        )
        .await
    }

    fn provider_id(&self) -> &'static str {
        "mistral"
    }

    fn requires_api_key(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mistral_provider_id() {
        assert_eq!(MistralProvider::new("k".into()).provider_id(), "mistral");
    }

    #[test]
    fn test_mistral_default_model() {
        assert_eq!(
            MistralProvider::new("k".into()).model,
            "mistral-small-latest"
        );
    }
}
