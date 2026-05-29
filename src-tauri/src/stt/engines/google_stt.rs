use crate::{error::AppError, stt::STTEngine};
use async_trait::async_trait;

pub struct GoogleSttEngine {
    pub api_key: String,
}

#[async_trait]
impl STTEngine for GoogleSttEngine {
    async fn transcribe(&self, _audio: Vec<f32>, _sample_rate: u32) -> Result<String, AppError> {
        todo!("GoogleSttEngine::transcribe — implement in v0.4 sprint")
    }
    fn engine_id(&self) -> &'static str {
        "google_stt"
    }
    fn requires_api_key(&self) -> bool {
        true
    }
}
