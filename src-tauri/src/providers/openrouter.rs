use crate::error::AppError;
use crate::providers::openai_compatible::{self, Auth};
use crate::providers::{AIProvider, ProviderResponse};
use async_trait::async_trait;

const ENDPOINT: &str = "https://openrouter.ai/api/v1/chat/completions";
const REFERER: &str = "https://github.com/NeoNexAI/PROMPTFLOW-STT_PROMPT_ENHANCER";
const TITLE: &str = "PromptFlow STT";

/// OpenRouter provider (OpenAI-compatible). Unlike other providers it has no
/// default model — the routed model (e.g. `anthropic/claude-3-haiku`) selects
/// both the model and the underlying provider, so the user must supply one.
/// Sends OpenRouter's required attribution headers.
pub struct OpenRouterProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl OpenRouterProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            api_key,
            model,
            client: crate::http::client(),
        }
    }
}

#[async_trait]
impl AIProvider for OpenRouterProvider {
    async fn complete(&self, system: &str, user: &str) -> Result<ProviderResponse, AppError> {
        openai_compatible::complete(
            &self.client,
            ENDPOINT,
            Auth::Bearer(&self.api_key),
            &[("HTTP-Referer", REFERER), ("X-Title", TITLE)],
            &self.model,
            "openrouter",
            (system, user),
        )
        .await
    }

    fn provider_id(&self) -> &'static str {
        "openrouter"
    }

    fn requires_api_key(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openrouter_provider_id() {
        let p = OpenRouterProvider::new("k".into(), "anthropic/claude-3-haiku".into());
        assert_eq!(p.provider_id(), "openrouter");
        assert_eq!(p.model, "anthropic/claude-3-haiku");
    }
}
