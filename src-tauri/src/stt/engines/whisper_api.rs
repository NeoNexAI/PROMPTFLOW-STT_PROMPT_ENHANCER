use crate::audio::wav::encode_wav_pcm16;
use crate::{error::AppError, stt::STTEngine};
use async_trait::async_trait;
use serde::Deserialize;

const ENDPOINT: &str = "https://api.openai.com/v1/audio/transcriptions";
const MODEL: &str = "whisper-1";

#[derive(Deserialize)]
struct TranscriptionResponse {
    #[serde(default)]
    text: String,
}

/// OpenAI Whisper (cloud, batch) transcription engine. Audio is wrapped in a
/// WAV container and uploaded as multipart/form-data. Shares the OpenAI API key
/// (`promptflow-stt/openai`) with the OpenAI text provider.
pub struct WhisperApiEngine {
    api_key: String,
    client: reqwest::Client,
}

impl WhisperApiEngine {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl STTEngine for WhisperApiEngine {
    async fn transcribe(&self, audio: Vec<f32>, sample_rate: u32) -> Result<String, AppError> {
        if audio.is_empty() {
            return Err(AppError::Stt("No audio captured".to_string()));
        }
        let wav = encode_wav_pcm16(&audio, sample_rate);

        let part = reqwest::multipart::Part::bytes(wav)
            .file_name("audio.wav")
            .mime_str("audio/wav")
            .map_err(|e| AppError::Stt(e.to_string()))?;
        let form = reqwest::multipart::Form::new()
            .text("model", MODEL)
            .part("file", part);

        let response = self
            .client
            .post(ENDPOINT)
            .bearer_auth(&self.api_key)
            .multipart(form)
            .send()
            .await
            .map_err(|e| AppError::Stt(format!("Whisper request failed: {e}")))?;

        let status = response.status();
        if !status.is_success() {
            if status.as_u16() == 401 || status.as_u16() == 403 {
                return Err(AppError::Stt(
                    "Invalid API key — check Settings → API Keys".to_string(),
                ));
            }
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "<unreadable body>".to_string());
            return Err(AppError::Stt(format!("Whisper API error {status}: {body}")));
        }

        let parsed: TranscriptionResponse = response
            .json()
            .await
            .map_err(|e| AppError::Stt(format!("Whisper response parse error: {e}")))?;
        Ok(parsed.text)
    }

    fn engine_id(&self) -> &'static str {
        "whisper_api"
    }

    fn requires_api_key(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whisper_api_metadata() {
        let e = WhisperApiEngine::new("k".into());
        assert_eq!(e.engine_id(), "whisper_api");
        assert!(e.requires_api_key());
    }

    #[tokio::test]
    async fn test_empty_audio_rejected() {
        let e = WhisperApiEngine::new("k".into());
        let result = e.transcribe(vec![], 16_000).await;
        assert!(matches!(result, Err(AppError::Stt(_))));
    }
}
