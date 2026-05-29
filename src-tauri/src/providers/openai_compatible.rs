//! Shared implementation for providers that speak the OpenAI
//! `/v1/chat/completions` request/response format.
//!
//! OpenAI, Groq, Ollama, Mistral, OpenRouter and Custom all use the same
//! `messages` body and `choices[].message.content` + `usage.total_tokens`
//! response shape, differing only in endpoint URL, model id, auth scheme and
//! a few extra headers. This module centralizes that logic so each provider
//! file stays a thin configuration wrapper. Anthropic and Gemini use distinct
//! formats and have their own modules.

use crate::cost::tracker::estimate_cost;
use crate::error::AppError;
use crate::providers::ProviderResponse;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage<'a>>,
}

#[derive(Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
    #[serde(default)]
    usage: Usage,
}

#[derive(Deserialize)]
struct Choice {
    message: MessageOut,
}

#[derive(Deserialize)]
struct MessageOut {
    #[serde(default)]
    content: String,
}

#[derive(Deserialize, Default)]
struct Usage {
    #[serde(default)]
    total_tokens: u32,
}

/// How the API key is presented to an OpenAI-compatible endpoint.
pub enum Auth<'a> {
    /// `Authorization: Bearer <key>` (OpenAI, Groq, Mistral, OpenRouter).
    Bearer(&'a str),
    /// No authentication header (local Ollama).
    None,
}

/// Sends a chat completion to any OpenAI-compatible endpoint and maps the
/// reply into a [`ProviderResponse`]. Cost is derived centrally from
/// [`crate::cost::tracker::estimate_cost`] using `cost_provider_id`, so pricing
/// lives in exactly one place.
pub async fn complete(
    client: &reqwest::Client,
    endpoint: &str,
    auth: Auth<'_>,
    extra_headers: &[(&str, &str)],
    model: &str,
    cost_provider_id: &str,
    // (system, user) travel together — kept as a tuple to stay within clippy's
    // argument-count limit and signal that they are one logical input.
    prompt: (&str, &str),
) -> Result<ProviderResponse, AppError> {
    let (system, user) = prompt;
    let body = ChatRequest {
        model,
        messages: vec![
            ChatMessage {
                role: "system",
                content: system,
            },
            ChatMessage {
                role: "user",
                content: user,
            },
        ],
    };

    let mut req = client.post(endpoint).json(&body);
    if let Auth::Bearer(key) = auth {
        req = req.bearer_auth(key);
    }
    for (name, value) in extra_headers {
        req = req.header(*name, *value);
    }

    let response = req
        .send()
        .await
        .map_err(|e| AppError::Provider(format!("{cost_provider_id} request failed: {e}")))?;

    let status = response.status();
    if !status.is_success() {
        // Authentication failures get an actionable message (per the error
        // handling contract in docs/specs/06_AI_INTEGRATIONS.md §3).
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
            "{cost_provider_id} API error {status}: {error_body}"
        )));
    }

    let parsed: ChatResponse = response
        .json()
        .await
        .map_err(|e| AppError::Provider(format!("{cost_provider_id} response parse error: {e}")))?;

    let text = parsed
        .choices
        .into_iter()
        .next()
        .map(|c| c.message.content)
        .ok_or_else(|| AppError::Provider(format!("{cost_provider_id} returned no choices")))?;

    let tokens_used = parsed.usage.total_tokens;
    let cost_usd = estimate_cost(tokens_used, cost_provider_id);

    Ok(ProviderResponse {
        text,
        tokens_used,
        cost_usd,
    })
}
