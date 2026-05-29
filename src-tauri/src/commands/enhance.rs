use crate::enhancement::{build_prompt, EnhancementMode};
use crate::error::AppError;
use crate::providers;
use crate::storage::keychain::KeychainStore;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

#[derive(Debug, Serialize, Deserialize)]
pub struct EnhanceResponse {
    pub result: String,
    pub tokens_used: u32,
    pub cost_usd: f64,
}

/// Enhances the given text using the specified AI provider and enhancement mode.
///
/// # Steps
/// 1. Deserialize `mode` string into [`EnhancementMode`].
///    `"custom"` is handled explicitly because its serde representation
///    requires an inner string payload that the frontend does not supply in v0.1.
/// 2. Retrieve the API key for `provider` from the OS keychain.
/// 3. Instantiate the provider via [`providers::make_provider`].
/// 4. Build the (system, user) prompt pair via [`enhancement::build_prompt`].
/// 5. Call the provider's `complete()` method and return the result.
///
/// # Errors
/// - [`AppError::Provider`] — unknown mode, unknown provider, or missing API key.
/// - [`AppError::Storage`] — keychain access failure.
#[tauri::command]
pub async fn enhance_text(
    _app: AppHandle,
    text: String,
    mode: String,
    provider: String,
) -> Result<EnhanceResponse, AppError> {
    // Step 1: parse mode string to EnhancementMode.
    // "custom" requires special handling because its serde form needs an inner
    // string (e.g. `{"custom": "...prompt..."}`), which the frontend doesn't
    // send in v0.1.  We default to an empty inner prompt; the real prompt will
    // be wired up in a future iteration.
    let enhancement_mode = if mode == "custom" {
        EnhancementMode::Custom(String::new())
    } else {
        let mode_value = serde_json::Value::String(mode.clone());
        serde_json::from_value(mode_value)
            .map_err(|_| AppError::Provider(format!("Unknown enhancement mode: {mode}")))?
    };

    // Step 2: retrieve API key from OS keychain.
    let keychain = KeychainStore::new();
    let api_key = keychain.get_api_key(&provider)?.ok_or_else(|| {
        AppError::Provider(format!("No API key configured for provider: {provider}"))
    })?;

    // Step 3: build provider instance.
    let ai_provider = providers::make_provider(&provider, api_key)?;

    // Step 4: build prompt and call the AI API.
    let (system, user) = build_prompt(&enhancement_mode, &text);
    let response = ai_provider.complete(&system, &user).await?;

    // Step 5: return structured response.
    Ok(EnhanceResponse {
        result: response.text,
        tokens_used: response.tokens_used,
        cost_usd: response.cost_usd,
    })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enhancement::{build_prompt, EnhancementMode};

    // ── Mode parsing helpers (pure logic, no AppHandle required) ──────────────

    fn parse_mode(mode: &str) -> Result<EnhancementMode, AppError> {
        if mode == "custom" {
            return Ok(EnhancementMode::Custom(String::new()));
        }
        let mode_value = serde_json::Value::String(mode.to_string());
        serde_json::from_value(mode_value)
            .map_err(|_| AppError::Provider(format!("Unknown enhancement mode: {mode}")))
    }

    #[test]
    fn test_enhance_text_mode_fix_grammar_parses() {
        let result = parse_mode("fix_grammar");
        assert!(result.is_ok(), "fix_grammar should parse successfully");
        assert!(matches!(result.unwrap(), EnhancementMode::FixGrammar));
    }

    #[test]
    fn test_enhance_text_mode_formalize_parses() {
        let result = parse_mode("formalize");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), EnhancementMode::Formalize));
    }

    #[test]
    fn test_enhance_text_mode_shorten_parses() {
        assert!(parse_mode("shorten").is_ok());
    }

    #[test]
    fn test_enhance_text_mode_expand_parses() {
        assert!(parse_mode("expand").is_ok());
    }

    #[test]
    fn test_enhance_text_mode_translate_parses() {
        assert!(parse_mode("translate").is_ok());
    }

    #[test]
    fn test_enhance_text_mode_brainstorm_parses() {
        assert!(parse_mode("brainstorm").is_ok());
    }

    #[test]
    fn test_enhance_text_mode_action_items_parses() {
        assert!(parse_mode("action_items").is_ok());
    }

    #[test]
    fn test_enhance_text_mode_summarize_parses() {
        assert!(parse_mode("summarize").is_ok());
    }

    #[test]
    fn test_enhance_text_mode_code_review_parses() {
        assert!(parse_mode("code_review").is_ok());
    }

    #[test]
    fn test_enhance_text_mode_simplify_parses() {
        assert!(parse_mode("simplify").is_ok());
    }

    #[test]
    fn test_enhance_text_mode_reframe_parses() {
        assert!(parse_mode("reframe").is_ok());
    }

    #[test]
    fn test_enhance_text_mode_custom_returns_empty_inner_prompt() {
        let result = parse_mode("custom");
        assert!(result.is_ok());
        match result.unwrap() {
            EnhancementMode::Custom(inner) => assert!(
                inner.is_empty(),
                "Custom mode in v0.1 should default to an empty inner prompt"
            ),
            other => panic!("Expected Custom, got {other:?}"),
        }
    }

    #[test]
    fn test_enhance_text_mode_unknown_returns_provider_error() {
        let result = parse_mode("does_not_exist");
        assert!(result.is_err());
        match result {
            Err(AppError::Provider(msg)) => {
                assert!(
                    msg.contains("does_not_exist"),
                    "Error message should name the bad mode"
                );
            }
            Err(other) => panic!("Expected AppError::Provider, got {other:?}"),
            Ok(_) => panic!("Expected error, got Ok"),
        }
    }

    // ── Prompt building (pure, no I/O) ────────────────────────────────────────

    #[test]
    fn test_enhance_text_build_prompt_fix_grammar_non_empty() {
        let (system, user) = build_prompt(&EnhancementMode::FixGrammar, "Hello wrold");
        assert!(!system.is_empty(), "system prompt must not be empty");
        assert_eq!(user, "Hello wrold");
    }

    #[test]
    fn test_enhance_text_build_prompt_custom_uses_inner_as_system() {
        let custom_prompt = "You are a pirate.".to_string();
        let (system, user) = build_prompt(&EnhancementMode::Custom(custom_prompt.clone()), "Ahoy");
        assert_eq!(system, custom_prompt);
        assert_eq!(user, "Ahoy");
    }

    #[test]
    fn test_enhance_text_build_prompt_custom_empty_inner() {
        // v0.1 path: custom with no inner prompt — system is empty, user is the text.
        let (system, user) = build_prompt(&EnhancementMode::Custom(String::new()), "some text");
        assert!(system.is_empty());
        assert_eq!(user, "some text");
    }

    // ── EnhanceResponse derives Serialize + Deserialize ───────────────────────

    #[test]
    fn test_enhance_response_serde_round_trip() {
        let resp = EnhanceResponse {
            result: "Hello world".to_string(),
            tokens_used: 42,
            cost_usd: 0.001,
        };
        let json = serde_json::to_string(&resp).expect("serialize failed");
        let back: EnhanceResponse = serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(back.result, resp.result);
        assert_eq!(back.tokens_used, resp.tokens_used);
        assert!((back.cost_usd - resp.cost_usd).abs() < f64::EPSILON);
    }
}
