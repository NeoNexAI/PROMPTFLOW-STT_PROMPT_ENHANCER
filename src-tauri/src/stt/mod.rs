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

/// Construction parameters for an STT engine. `api_key` is needed by cloud
/// engines; `binary_path`/`model_path` configure the local whisper.cpp engine.
#[derive(Debug, Default, Clone)]
pub struct EngineConfig {
    pub api_key: Option<String>,
    pub binary_path: Option<String>,
    pub model_path: Option<String>,
}

/// Builds a boxed [`STTEngine`] for `engine_id` from `config`.
///
/// `whisper_api` (cloud) and `whisper_cpp` (local) are implemented; the
/// streaming engines return a clear "not yet implemented" error and
/// `web_speech` is rejected because it runs in the WebView.
pub fn make_engine(engine_id: &str, config: EngineConfig) -> Result<Box<dyn STTEngine>, AppError> {
    match engine_id {
        "whisper_api" => {
            let key = config.api_key.filter(|k| !k.is_empty()).ok_or_else(|| {
                AppError::Stt("No API key configured for whisper_api".to_string())
            })?;
            Ok(Box::new(engines::whisper_api::WhisperApiEngine::new(key)))
        }
        "whisper_cpp" => Ok(Box::new(engines::whisper_cpp::WhisperCppEngine::new(
            config.binary_path.unwrap_or_default(),
            config.model_path.unwrap_or_default(),
        ))),
        "web_speech" => Err(AppError::Stt(
            "web_speech runs in the browser, not the backend".to_string(),
        )),
        "deepgram" | "assembly_ai" | "google_stt" | "azure_stt" => Err(AppError::Stt(format!(
            "STT engine '{engine_id}' is not implemented yet"
        ))),
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

    fn cfg_key() -> EngineConfig {
        EngineConfig {
            api_key: Some("k".into()),
            ..Default::default()
        }
    }

    #[test]
    fn test_make_engine_whisper_api_ok() {
        let engine = make_engine("whisper_api", cfg_key()).unwrap();
        assert_eq!(engine.engine_id(), "whisper_api");
    }

    #[test]
    fn test_make_engine_whisper_api_requires_key() {
        assert!(matches!(
            make_engine("whisper_api", EngineConfig::default()),
            Err(AppError::Stt(_))
        ));
    }

    #[test]
    fn test_make_engine_whisper_cpp_ok_without_key() {
        let engine = make_engine(
            "whisper_cpp",
            EngineConfig {
                binary_path: Some("whisper-cli".into()),
                model_path: Some("model.bin".into()),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(engine.engine_id(), "whisper_cpp");
        assert!(!engine.requires_api_key());
    }

    #[test]
    fn test_make_engine_web_speech_rejected() {
        assert!(matches!(
            make_engine("web_speech", EngineConfig::default()),
            Err(AppError::Stt(_))
        ));
    }

    #[test]
    fn test_make_engine_unimplemented_and_unknown() {
        assert!(matches!(
            make_engine("deepgram", cfg_key()),
            Err(AppError::Stt(_))
        ));
        assert!(matches!(
            make_engine("bogus", EngineConfig::default()),
            Err(AppError::Stt(_))
        ));
    }

    #[test]
    fn test_known_engines_count() {
        assert_eq!(KNOWN_ENGINES.len(), 7);
    }
}
