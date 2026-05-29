use crate::{error::AppError, stt::STTEngine};
use async_trait::async_trait;

pub struct WhisperApiEngine {
    pub api_key: String,
}

#[async_trait]
impl STTEngine for WhisperApiEngine {
    async fn transcribe(&self, _audio: Vec<f32>, _sample_rate: u32) -> Result<String, AppError> {
        todo!("WhisperApiEngine::transcribe — implement in v0.2 sprint")
    }
    fn engine_id(&self) -> &'static str {
        "whisper_api"
    }
    fn requires_api_key(&self) -> bool {
        true
    }
}
