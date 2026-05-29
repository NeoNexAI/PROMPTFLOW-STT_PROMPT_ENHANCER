use crate::cost::tracker::estimate_cost;
use crate::error::AppError;
use crate::providers::{AIProvider, ProviderResponse};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

const DEFAULT_MODEL: &str = "claude-haiku-4-5";
const ENDPOINT: &str = "https://api.anthropic.com/v1/messages";
const API_VERSION: &str = "2023-06-01";
const MAX_TOKENS: u32 = 4096;

#[derive(Serialize)]
struct MessagesRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    system: &'a str,
    messages: Vec<Message<'a>>,
}

#[derive(Serialize)]
struct Message<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct MessagesResponse {
    content: Vec<ContentBlock>,
    #[serde(default)]
    usage: Usage,
}

#[derive(Deserialize)]
struct ContentBlock {
    #[serde(default)]
    text: String,
}

#[derive(Deserialize, Default)]
struct Usage {
    #[serde(default)]
    input_tokens: u32,
    #[serde(default)]
    output_tokens: u32,
}

/// Anthropic Claude provider via the Messages API. Defaults to
/// `claude-haiku-4-5`. The system prompt is passed in the top-level `system`
/// field (not inside `messages`), and `anthropic-version` is required.
pub struct AnthropicProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl AnthropicProvider {
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
impl AIProvider for AnthropicProvider {
    async fn complete(&self, system: &str, user: &str) -> Result<ProviderResponse, AppError> {
        let body = MessagesRequest {
            model: &self.model,
            max_tokens: MAX_TOKENS,
            system,
            messages: vec![Message {
                role: "user",
                content: user,
            }],
        };

        let response = self
            .client
            .post(ENDPOINT)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", API_VERSION)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Provider(format!("Anthropic request failed: {e}")))?;

        let status = response.status();
        if !status.is_success() {
            if status.as_u16() == 401 || status.as_u16() == 403 {
                return Err(AppError::Provider(
                    "Invalid API key — check Settings → API Keys".to_string(),
                ));
            }
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "<unreadable body>".to_string());
            return Err(AppError::Provider(format!(
                "Anthropic API error {status}: {error_body}"
            )));
        }

        let parsed: MessagesResponse = response
            .json()
            .await
            .map_err(|e| AppError::Provider(format!("Anthropic response parse error: {e}")))?;

        let text = parsed
            .content
            .into_iter()
            .map(|b| b.text)
            .collect::<Vec<_>>()
            .join("");
        if text.is_empty() {
            return Err(AppError::Provider(
                "Anthropic returned no content".to_string(),
            ));
        }

        let tokens_used = parsed.usage.input_tokens + parsed.usage.output_tokens;
        Ok(ProviderResponse {
            text,
            tokens_used,
            cost_usd: estimate_cost(tokens_used, "anthropic"),
        })
    }

    fn provider_id(&self) -> &'static str {
        "anthropic"
    }

    fn requires_api_key(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anthropic_provider_id_and_key() {
        let p = AnthropicProvider::new("k".into());
        assert_eq!(p.provider_id(), "anthropic");
        assert!(p.requires_api_key());
    }

    #[test]
    fn test_anthropic_default_model() {
        assert_eq!(AnthropicProvider::new("k".into()).model, "claude-haiku-4-5");
    }
}
