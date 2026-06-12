use crate::error::AppError;
use crate::providers::openai_compatible::{self, Auth};
use crate::providers::{AIProvider, ProviderResponse};
use async_trait::async_trait;

const DEFAULT_MODEL: &str = "llama3.2";
const DEFAULT_BASE_URL: &str = "http://localhost:11434";

/// Ollama local-inference provider via its OpenAI-compatible endpoint
/// (`{base_url}/v1/chat/completions`). Requires no API key and qualifies for
/// Privacy Mode because all inference stays on-device. Cost is always `0.0`.
pub struct OllamaProvider {
    endpoint: String,
    model: String,
    client: reqwest::Client,
}

impl OllamaProvider {
    pub fn new(model: Option<String>, base_url: Option<String>) -> Self {
        let base = base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_string());
        let base = base.trim_end_matches('/');
        Self {
            endpoint: format!("{base}/v1/chat/completions"),
            model: model.unwrap_or_else(|| DEFAULT_MODEL.to_string()),
            client: crate::http::client(),
        }
    }
}

#[async_trait]
impl AIProvider for OllamaProvider {
    async fn complete(&self, system: &str, user: &str) -> Result<ProviderResponse, AppError> {
        openai_compatible::complete(
            &self.client,
            &self.endpoint,
            Auth::None,
            &[],
            &self.model,
            "ollama",
            (system, user),
        )
        .await
    }

    fn provider_id(&self) -> &'static str {
        "ollama"
    }

    fn requires_api_key(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_does_not_require_key() {
        assert!(!OllamaProvider::new(None, None).requires_api_key());
    }

    #[test]
    fn test_ollama_default_endpoint_and_model() {
        let p = OllamaProvider::new(None, None);
        assert_eq!(p.endpoint, "http://localhost:11434/v1/chat/completions");
        assert_eq!(p.model, "llama3.2");
    }

    #[test]
    fn test_ollama_base_url_override_trims_trailing_slash() {
        let p = OllamaProvider::new(None, Some("http://127.0.0.1:1234/".to_string()));
        assert_eq!(p.endpoint, "http://127.0.0.1:1234/v1/chat/completions");
    }
}
