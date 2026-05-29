use crate::error::AppError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub mod anthropic;
pub mod custom;
pub mod gemini;
pub mod groq;
pub mod mistral;
pub mod ollama;
pub mod openai;
pub mod openai_compatible;
pub mod openrouter;

/// The structured response returned by every [`AIProvider`] implementation.
#[derive(Debug, Serialize, Deserialize)]
pub struct ProviderResponse {
    pub text: String,
    pub tokens_used: u32,
    pub cost_usd: f64,
}

/// Common interface implemented by all AI text-completion providers.
#[async_trait]
pub trait AIProvider: Send + Sync {
    async fn complete(&self, system: &str, user: &str) -> Result<ProviderResponse, AppError>;
    fn provider_id(&self) -> &'static str;
    fn requires_api_key(&self) -> bool;
}

/// Construction parameters for a provider. `api_key` is required for cloud
/// providers; `model` and `base_url` are optional overrides (most providers
/// fall back to a sensible default model and fixed endpoint).
#[derive(Debug, Default, Clone)]
pub struct ProviderConfig {
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub base_url: Option<String>,
}

/// All provider ids known to the application.
pub const KNOWN_PROVIDERS: &[&str] = &[
    "openai",
    "anthropic",
    "gemini",
    "ollama",
    "groq",
    "mistral",
    "openrouter",
    "custom",
];

/// Whether the given provider requires an API key stored in the keychain.
/// Local providers (`ollama`) and user-configured gateways (`custom`) do not.
pub fn requires_api_key(provider_id: &str) -> bool {
    !matches!(provider_id, "ollama" | "custom")
}

/// Providers that can run fully offline and are permitted in Privacy Mode.
/// `custom` qualifies only when pointed at a loopback endpoint, which is
/// validated separately at dispatch time.
pub fn is_offline_capable(provider_id: &str) -> bool {
    matches!(provider_id, "ollama" | "custom")
}

/// Creates a boxed [`AIProvider`] for the given provider id and config.
///
/// # Errors
/// Returns [`AppError::Provider`] if `provider_id` is unknown, a required API
/// key is missing, or a required field (OpenRouter model, custom base URL) is
/// absent.
pub fn make_provider(
    provider_id: &str,
    config: ProviderConfig,
) -> Result<Box<dyn AIProvider>, AppError> {
    let ProviderConfig {
        api_key,
        model,
        base_url,
    } = config;

    // Validate the key up front for providers that need one.
    let require_key = || -> Result<String, AppError> {
        api_key
            .clone()
            .filter(|k| !k.is_empty())
            .ok_or_else(|| AppError::Provider(format!("No API key configured for: {provider_id}")))
    };

    match provider_id {
        "openai" => Ok(Box::new(openai::OpenAIProvider::with_model(
            require_key()?,
            model,
        ))),
        "groq" => Ok(Box::new(groq::GroqProvider::with_model(
            require_key()?,
            model,
        ))),
        "anthropic" => Ok(Box::new(anthropic::AnthropicProvider::with_model(
            require_key()?,
            model,
        ))),
        "gemini" => Ok(Box::new(gemini::GeminiProvider::with_model(
            require_key()?,
            model,
        ))),
        "mistral" => Ok(Box::new(mistral::MistralProvider::with_model(
            require_key()?,
            model,
        ))),
        "ollama" => Ok(Box::new(ollama::OllamaProvider::new(model, base_url))),
        "openrouter" => {
            let model = model.filter(|m| !m.is_empty()).ok_or_else(|| {
                AppError::Provider(
                    "OpenRouter requires a model (e.g. anthropic/claude-3-haiku) — set it in Settings"
                        .to_string(),
                )
            })?;
            Ok(Box::new(openrouter::OpenRouterProvider::new(
                require_key()?,
                model,
            )))
        }
        "custom" => {
            let base_url = base_url.filter(|u| !u.is_empty()).ok_or_else(|| {
                AppError::Provider(
                    "Custom provider requires a base URL — set it in Settings".to_string(),
                )
            })?;
            let model = model.filter(|m| !m.is_empty()).ok_or_else(|| {
                AppError::Provider(
                    "Custom provider requires a model — set it in Settings".to_string(),
                )
            })?;
            Ok(Box::new(custom::CustomProvider::new(
                base_url,
                api_key.unwrap_or_default(),
                model,
            )))
        }
        other => Err(AppError::Provider(format!("Unknown provider: {other}"))),
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg_with_key() -> ProviderConfig {
        ProviderConfig {
            api_key: Some("key".to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn test_make_provider_key_providers_succeed() {
        for id in ["openai", "groq", "anthropic", "gemini", "mistral"] {
            let provider = make_provider(id, cfg_with_key()).unwrap();
            assert_eq!(provider.provider_id(), id);
            assert!(provider.requires_api_key());
        }
    }

    #[test]
    fn test_make_provider_ollama_needs_no_key() {
        let provider = make_provider("ollama", ProviderConfig::default()).unwrap();
        assert_eq!(provider.provider_id(), "ollama");
        assert!(!provider.requires_api_key());
    }

    #[test]
    fn test_make_provider_missing_key_errors() {
        let result = make_provider("openai", ProviderConfig::default());
        assert!(matches!(result, Err(AppError::Provider(_))));
    }

    #[test]
    fn test_make_provider_openrouter_requires_model() {
        match make_provider("openrouter", cfg_with_key()) {
            Err(AppError::Provider(msg)) => assert!(msg.contains("model")),
            Err(other) => panic!("Expected model-required error, got {other:?}"),
            Ok(_) => panic!("Expected error when model is missing"),
        }
        let ok = make_provider(
            "openrouter",
            ProviderConfig {
                api_key: Some("k".into()),
                model: Some("anthropic/claude-3-haiku".into()),
                ..Default::default()
            },
        );
        assert!(ok.is_ok());
    }

    #[test]
    fn test_make_provider_custom_requires_base_url_and_model() {
        let result = make_provider("custom", ProviderConfig::default());
        assert!(matches!(result, Err(AppError::Provider(_))));
        let ok = make_provider(
            "custom",
            ProviderConfig {
                api_key: None,
                model: Some("m".into()),
                base_url: Some("http://localhost:8000/v1".into()),
            },
        );
        assert!(ok.is_ok());
    }

    #[test]
    fn test_make_provider_unknown_returns_error() {
        let result = make_provider("unknown-llm", cfg_with_key());
        match result {
            Err(AppError::Provider(msg)) => assert!(msg.contains("unknown-llm")),
            Err(other) => panic!("Expected AppError::Provider, got {other:?}"),
            Ok(_) => panic!("Expected error, got Ok"),
        }
    }

    #[test]
    fn test_requires_api_key_helper() {
        assert!(requires_api_key("openai"));
        assert!(!requires_api_key("ollama"));
        assert!(!requires_api_key("custom"));
    }

    #[test]
    fn test_known_providers_count() {
        assert_eq!(KNOWN_PROVIDERS.len(), 8);
    }
}
