use crate::{error::AppError, stt::STTEngine};
use async_trait::async_trait;

pub struct AzureSttEngine {
    pub api_key: String,
    pub region: String,
}

#[async_trait]
impl STTEngine for AzureSttEngine {
    async fn transcribe(&self, _audio: Vec<f32>, _sample_rate: u32) -> Result<String, AppError> {
        todo!("AzureSttEngine::transcribe — implement in v0.4 sprint")
    }
    fn engine_id(&self) -> &'static str {
        "azure_stt"
    }
    fn requires_api_key(&self) -> bool {
        true
    }
}
