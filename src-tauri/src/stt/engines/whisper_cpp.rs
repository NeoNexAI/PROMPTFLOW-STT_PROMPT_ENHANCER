use crate::audio::wav::{encode_wav_pcm16, resample_linear};
use crate::{error::AppError, stt::STTEngine};
use async_trait::async_trait;
use std::process::Command;

const TARGET_RATE: u32 = 16_000;

/// Local, offline STT via the `whisper.cpp` CLI (`whisper-cli` / `main`). Audio
/// is resampled to 16 kHz mono, written to a temp WAV, and transcribed by
/// invoking the binary with `-nt` (no timestamps); stdout is the transcript.
/// Requires no API key and is eligible for Privacy Mode.
pub struct WhisperCppEngine {
    binary: String,
    model: String,
}

impl WhisperCppEngine {
    pub fn new(binary: String, model: String) -> Self {
        Self { binary, model }
    }
}

#[async_trait]
impl STTEngine for WhisperCppEngine {
    async fn transcribe(&self, audio: Vec<f32>, sample_rate: u32) -> Result<String, AppError> {
        if self.binary.trim().is_empty() || self.model.trim().is_empty() {
            return Err(AppError::Stt(
                "Configure the whisper.cpp binary and model paths in Settings".to_string(),
            ));
        }
        if audio.is_empty() {
            return Err(AppError::Stt("No audio captured".to_string()));
        }

        let resampled = resample_linear(&audio, sample_rate, TARGET_RATE);
        let wav = encode_wav_pcm16(&resampled, TARGET_RATE);

        let path = std::env::temp_dir().join(format!("promptflow-{}.wav", std::process::id()));
        std::fs::write(&path, &wav).map_err(|e| AppError::Stt(e.to_string()))?;

        let output = Command::new(&self.binary)
            .args(["-m", &self.model, "-f", &path.to_string_lossy(), "-nt"])
            .output();
        let _ = std::fs::remove_file(&path);

        let output = output.map_err(|e| {
            AppError::Stt(format!("Failed to run whisper.cpp ({}): {e}", self.binary))
        })?;
        if !output.status.success() {
            return Err(AppError::Stt(format!(
                "whisper.cpp failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )));
        }
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    fn engine_id(&self) -> &'static str {
        "whisper_cpp"
    }

    fn requires_api_key(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata() {
        let e = WhisperCppEngine::new("whisper-cli".into(), "model.bin".into());
        assert_eq!(e.engine_id(), "whisper_cpp");
        assert!(!e.requires_api_key());
    }

    #[tokio::test]
    async fn test_missing_paths_error() {
        let e = WhisperCppEngine::new(String::new(), String::new());
        let r = e.transcribe(vec![0.1, 0.2], 16_000).await;
        assert!(matches!(r, Err(AppError::Stt(_))));
    }
}
