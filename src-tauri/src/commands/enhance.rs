use crate::commands::settings;
use crate::enhancement::{build_prompt, EnhancementMode};
use crate::error::AppError;
use crate::providers::{self, ProviderConfig};
use crate::storage::db::{Database, UsageEntry};
use crate::storage::keychain::KeychainStore;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

#[derive(Debug, Serialize, Deserialize)]
pub struct EnhanceResponse {
    pub result: String,
    pub tokens_used: u32,
    pub cost_usd: f64,
}

/// Parses a mode string into an [`EnhancementMode`]. `"custom"` carries the
/// user-supplied system prompt (empty when none was provided).
fn parse_mode(mode: &str, custom_prompt: Option<String>) -> Result<EnhancementMode, AppError> {
    if mode == "custom" {
        return Ok(EnhancementMode::Custom(custom_prompt.unwrap_or_default()));
    }
    let mode_value = serde_json::Value::String(mode.to_string());
    serde_json::from_value(mode_value)
        .map_err(|_| AppError::Provider(format!("Unknown enhancement mode: {mode}")))
}

/// Enhances `text` using the specified AI provider and enhancement mode.
///
/// Steps: parse mode → enforce Privacy Mode → fetch the API key (only for
/// providers that need one) → build the provider → build the prompt → call the
/// model → log usage to SQLite (fire-and-forget) → return the result.
///
/// `custom_prompt` supplies the system prompt for `mode == "custom"`; `model`
/// and `base_url` are optional provider overrides (a non-default model, or the
/// Ollama/custom endpoint).
///
/// # Errors
/// - [`AppError::Provider`] — unknown mode/provider, missing key, missing
///   required config, a Privacy Mode violation, or an upstream API error.
/// - [`AppError::Storage`] — keychain access failure.
#[tauri::command]
pub async fn enhance_text(
    app: AppHandle,
    text: String,
    mode: String,
    provider: String,
    custom_prompt: Option<String>,
    model: Option<String>,
    base_url: Option<String>,
) -> Result<EnhanceResponse, AppError> {
    let enhancement_mode = parse_mode(&mode, custom_prompt)?;

    // Privacy Mode enforcement (server-side, independent of the UI selector):
    // reject any cloud provider while Privacy Mode is enabled.
    let settings = settings::load(&app)?;
    if settings.privacy_mode && !providers::is_offline_capable(&provider) {
        return Err(AppError::Provider(format!(
            "Privacy Mode is on — {provider} would send data off-device. Use ollama or a localhost custom endpoint."
        )));
    }

    // Fetch the API key only for providers that require one.
    let api_key = if providers::requires_api_key(&provider) {
        Some(
            KeychainStore::new()
                .get_api_key(&provider)?
                .ok_or_else(|| {
                    AppError::Provider(format!("No API key configured for provider: {provider}"))
                })?,
        )
    } else {
        None
    };

    let ai_provider = providers::make_provider(
        &provider,
        ProviderConfig {
            api_key,
            model,
            base_url,
        },
    )?;

    let (system, user) = build_prompt(&enhancement_mode, &text);
    let input_len = text.chars().count() as u32;
    let response = ai_provider.complete(&system, &user).await?;
    let output_len = response.text.chars().count() as u32;

    // Fire-and-forget usage logging: must never delay or fail the response.
    log_usage_async(&app, &mode, &provider, &response, input_len, output_len);

    Ok(EnhanceResponse {
        result: response.text,
        tokens_used: response.tokens_used,
        cost_usd: response.cost_usd,
    })
}

/// Spawns a background thread that appends a row to the usage database.
/// Any error is swallowed (logged in debug builds) — logging is best-effort and
/// must not affect the user-facing enhancement path.
fn log_usage_async(
    app: &AppHandle,
    mode: &str,
    provider: &str,
    response: &providers::ProviderResponse,
    input_len: u32,
    output_len: u32,
) {
    let Ok(dir) = app.path().app_data_dir() else {
        return;
    };
    let entry = UsageEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        mode: mode.to_string(),
        provider: provider.to_string(),
        tokens: response.tokens_used,
        cost_usd: response.cost_usd,
        input_len,
        output_len,
    };
    std::thread::spawn(move || {
        let result = std::fs::create_dir_all(&dir)
            .map_err(|e| AppError::Storage(e.to_string()))
            .and_then(|_| Database::open(dir.join("usage.db")))
            .and_then(|db| db.log_usage(&entry));
        #[cfg(debug_assertions)]
        if let Err(e) = result {
            eprintln!("[PromptFlow] usage logging failed: {e:?}");
        }
        #[cfg(not(debug_assertions))]
        let _ = result;
    });
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enhancement::{build_prompt, EnhancementMode};

    // ── Mode parsing helpers (pure logic, no AppHandle required) ──────────────
    // Thin wrapper over the production `parse_mode` so the existing call sites
    // (which pass no custom prompt) exercise the real implementation.

    fn parse_mode(mode: &str) -> Result<EnhancementMode, AppError> {
        super::parse_mode(mode, None)
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

    #[test]
    fn test_parse_mode_custom_carries_supplied_prompt() {
        let result = super::parse_mode("custom", Some("You are a pirate.".to_string()));
        match result.unwrap() {
            EnhancementMode::Custom(inner) => assert_eq!(inner, "You are a pirate."),
            other => panic!("Expected Custom, got {other:?}"),
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
