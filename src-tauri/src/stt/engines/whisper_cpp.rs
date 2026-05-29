use crate::{error::AppError, stt::STTEngine};
use async_trait::async_trait;

pub struct WhisperCppEngine {
    pub model_path: String,
}

#[async_trait]
impl STTEngine for WhisperCppEngine {
    async fn transcribe(&self, _audio: Vec<f32>, _sample_rate: u32) -> Result<String, AppError> {
        todo!("WhisperCppEngine::transcribe — implement in v0.5 sprint (Privacy Mode)")
    }
    fn engine_id(&self) -> &'static str {
        "whisper_cpp"
    }
    fn requires_api_key(&self) -> bool {
        false
    }
}
