use crate::cost::tracker::estimate_cost;
use crate::error::AppError;
use crate::providers::{AIProvider, ProviderResponse};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

const DEFAULT_MODEL: &str = "gemini-1.5-flash";
const BASE: &str = "https://generativelanguage.googleapis.com/v1beta/models";

#[derive(Serialize)]
struct GenerateRequest<'a> {
    system_instruction: Content<'a>,
    contents: Vec<Content<'a>>,
}

#[derive(Serialize)]
struct Content<'a> {
    parts: Vec<Part<'a>>,
}

#[derive(Serialize)]
struct Part<'a> {
    text: &'a str,
}

#[derive(Deserialize)]
struct GenerateResponse {
    #[serde(default)]
    candidates: Vec<Candidate>,
    #[serde(rename = "usageMetadata", default)]
    usage_metadata: UsageMetadata,
}

#[derive(Deserialize)]
struct Candidate {
    content: ContentOut,
}

#[derive(Deserialize)]
struct ContentOut {
    #[serde(default)]
    parts: Vec<PartOut>,
}

#[derive(Deserialize)]
struct PartOut {
    #[serde(default)]
    text: String,
}

#[derive(Deserialize, Default)]
struct UsageMetadata {
    #[serde(rename = "promptTokenCount", default)]
    prompt_token_count: u32,
    #[serde(rename = "candidatesTokenCount", default)]
    candidates_token_count: u32,
}

/// Google Gemini provider via `generateContent`. Defaults to
/// `gemini-1.5-flash`. The model is part of the URL path and the system prompt
/// is passed via `system_instruction`.
pub struct GeminiProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl GeminiProvider {
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
impl AIProvider for GeminiProvider {
    async fn complete(&self, system: &str, user: &str) -> Result<ProviderResponse, AppError> {
        let url = format!("{BASE}/{}:generateContent", self.model);
        let body = GenerateRequest {
            system_instruction: Content {
                parts: vec![Part { text: system }],
            },
            contents: vec![Content {
                parts: vec![Part { text: user }],
            }],
        };

        let response = self
            .client
            .post(&url)
            .header("x-goog-api-key", &self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Provider(format!("Gemini request failed: {e}")))?;

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
                "Gemini API error {status}: {error_body}"
            )));
        }

        let parsed: GenerateResponse = response
            .json()
            .await
            .map_err(|e| AppError::Provider(format!("Gemini response parse error: {e}")))?;

        let text = parsed
            .candidates
            .into_iter()
            .next()
            .map(|c| {
                c.content
                    .parts
                    .into_iter()
                    .map(|p| p.text)
                    .collect::<Vec<_>>()
                    .join("")
            })
            .ok_or_else(|| AppError::Provider("Gemini returned no candidates".to_string()))?;

        let tokens_used =
            parsed.usage_metadata.prompt_token_count + parsed.usage_metadata.candidates_token_count;
        Ok(ProviderResponse {
            text,
            tokens_used,
            cost_usd: estimate_cost(tokens_used, "gemini"),
        })
    }

    fn provider_id(&self) -> &'static str {
        "gemini"
    }

    fn requires_api_key(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gemini_provider_id_and_key() {
        let p = GeminiProvider::new("k".into());
        assert_eq!(p.provider_id(), "gemini");
        assert!(p.requires_api_key());
    }

    #[test]
    fn test_gemini_default_model() {
        assert_eq!(GeminiProvider::new("k".into()).model, "gemini-1.5-flash");
    }
}
