use crate::{error::AppError, stt::STTEngine};
use async_trait::async_trait;

pub struct DeepgramEngine {
    pub api_key: String,
}

#[async_trait]
impl STTEngine for DeepgramEngine {
    async fn transcribe(&self, _audio: Vec<f32>, _sample_rate: u32) -> Result<String, AppError> {
        todo!("DeepgramEngine::transcribe — implement in v0.4 sprint")
    }
    fn engine_id(&self) -> &'static str {
        "deepgram"
    }
    fn requires_api_key(&self) -> bool {
        true
    }
}
