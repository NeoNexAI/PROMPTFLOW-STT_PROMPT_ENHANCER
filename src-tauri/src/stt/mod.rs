pub mod engines;

use crate::error::AppError;
use async_trait::async_trait;

#[async_trait]
pub trait STTEngine: Send + Sync {
    /// Transcribe audio samples (f32, mono) at `sample_rate` Hz to text.
    async fn transcribe(&self, audio: Vec<f32>, sample_rate: u32) -> Result<String, AppError>;
    fn engine_id(&self) -> &'static str;
    fn requires_api_key(&self) -> bool;
}

/// All STT engine ids known to the application (mirrors `STTEngine` in
/// `src/types/index.ts`).
pub const KNOWN_ENGINES: &[&str] = &[
    "whisper_api",
    "whisper_cpp",
    "deepgram",
    "assembly_ai",
    "google_stt",
    "azure_stt",
    "web_speech",
];

/// Whether `engine_id` needs an API key from the keychain.
/// `whisper_cpp` (local) and `web_speech` (browser) do not.
pub fn requires_api_key(engine_id: &str) -> bool {
    !matches!(engine_id, "whisper_cpp" | "web_speech")
}

/// Engines that run in the Rust backend (i.e. that `start_recording` drives).
/// `web_speech` is handled entirely in the WebView and never reaches Rust.
pub fn is_backend_engine(engine_id: &str) -> bool {
    engine_id != "web_speech"
}

/// The keychain account id that stores the key for a given engine. Several
/// engines reuse a provider's key (e.g. `whisper_api` uses the OpenAI key).
pub fn key_account(engine_id: &str) -> &str {
    match engine_id {
        "whisper_api" => "openai",
        other => other,
    }
}

/// Builds a boxed [`STTEngine`] for `engine_id`. `api_key` is required for
/// engines where [`requires_api_key`] is true.
///
/// Only `whisper_api` is implemented in the Rust backend so far; the streaming
/// and local engines return a clear "not yet implemented" error, and
/// `web_speech` is rejected because it runs in the WebView.
pub fn make_engine(
    engine_id: &str,
    api_key: Option<String>,
) -> Result<Box<dyn STTEngine>, AppError> {
    match engine_id {
        "whisper_api" => {
            let key = api_key.filter(|k| !k.is_empty()).ok_or_else(|| {
                AppError::Stt("No API key configured for whisper_api".to_string())
            })?;
            Ok(Box::new(engines::whisper_api::WhisperApiEngine::new(key)))
        }
        "web_speech" => Err(AppError::Stt(
            "web_speech runs in the browser, not the backend".to_string(),
        )),
        "whisper_cpp" | "deepgram" | "assembly_ai" | "google_stt" | "azure_stt" => Err(
            AppError::Stt(format!("STT engine '{engine_id}' is not implemented yet")),
        ),
        other => Err(AppError::Stt(format!("Unknown STT engine: {other}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_requires_api_key() {
        assert!(requires_api_key("whisper_api"));
        assert!(!requires_api_key("whisper_cpp"));
        assert!(!requires_api_key("web_speech"));
    }

    #[test]
    fn test_key_account_shares_openai_for_whisper() {
        assert_eq!(key_account("whisper_api"), "openai");
        assert_eq!(key_account("deepgram"), "deepgram");
    }

    #[test]
    fn test_make_engine_whisper_api_ok() {
        let engine = make_engine("whisper_api", Some("k".into())).unwrap();
        assert_eq!(engine.engine_id(), "whisper_api");
    }

    #[test]
    fn test_make_engine_whisper_api_requires_key() {
        assert!(matches!(
            make_engine("whisper_api", None),
            Err(AppError::Stt(_))
        ));
    }

    #[test]
    fn test_make_engine_web_speech_rejected() {
        assert!(matches!(
            make_engine("web_speech", None),
            Err(AppError::Stt(_))
        ));
    }

    #[test]
    fn test_make_engine_unimplemented_and_unknown() {
        assert!(matches!(
            make_engine("deepgram", Some("k".into())),
            Err(AppError::Stt(_))
        ));
        assert!(matches!(make_engine("bogus", None), Err(AppError::Stt(_))));
    }

    #[test]
    fn test_known_engines_count() {
        assert_eq!(KNOWN_ENGINES.len(), 7);
    }
}
