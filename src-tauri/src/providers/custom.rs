use crate::error::AppError;
use crate::providers::openai_compatible::{self, Auth};
use crate::providers::{AIProvider, ProviderResponse};
use async_trait::async_trait;

/// Custom provider targeting any OpenAI-compatible endpoint (vLLM, LM Studio,
/// LocalAI, self-hosted gateways, …). The user supplies the base URL, model and
/// API key. When the base URL is `localhost`/`127.0.0.1` it qualifies for
/// Privacy Mode. Cost tracking is disabled (rates are unknown).
pub struct CustomProvider {
    endpoint: String,
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl CustomProvider {
    /// `base_url` is the API root (e.g. `https://my-gateway.example/v1`); the
    /// `/chat/completions` path is appended.
    pub fn new(base_url: String, api_key: String, model: String) -> Self {
        let base = base_url.trim_end_matches('/');
        Self {
            endpoint: format!("{base}/chat/completions"),
            api_key,
            model,
            client: crate::http::client(),
        }
    }

    /// True when the configured endpoint is loopback, qualifying for Privacy Mode.
    pub fn is_local(&self) -> bool {
        self.endpoint.contains("localhost") || self.endpoint.contains("127.0.0.1")
    }
}

#[async_trait]
impl AIProvider for CustomProvider {
    async fn complete(&self, system: &str, user: &str) -> Result<ProviderResponse, AppError> {
        // Bearer auth if a key is present; otherwise unauthenticated (local gateways).
        let auth = if self.api_key.is_empty() {
            Auth::None
        } else {
            Auth::Bearer(&self.api_key)
        };
        openai_compatible::complete(
            &self.client,
            &self.endpoint,
            auth,
            &[],
            &self.model,
            "custom",
            (system, user),
        )
        .await
    }

    fn provider_id(&self) -> &'static str {
        "custom"
    }

    fn requires_api_key(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_endpoint_construction() {
        let p = CustomProvider::new(
            "https://gw.example/v1/".into(),
            "k".into(),
            "my-model".into(),
        );
        assert_eq!(p.endpoint, "https://gw.example/v1/chat/completions");
        assert!(!p.is_local());
    }

    #[test]
    fn test_custom_localhost_is_local() {
        let p = CustomProvider::new("http://localhost:8000/v1".into(), "".into(), "m".into());
        assert!(p.is_local());
    }
}
