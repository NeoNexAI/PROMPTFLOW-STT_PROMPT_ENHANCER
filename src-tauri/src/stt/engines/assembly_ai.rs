use crate::{error::AppError, stt::STTEngine};
use async_trait::async_trait;

pub struct AssemblyAiEngine {
    pub api_key: String,
}

#[async_trait]
impl STTEngine for AssemblyAiEngine {
    async fn transcribe(&self, _audio: Vec<f32>, _sample_rate: u32) -> Result<String, AppError> {
        todo!("AssemblyAiEngine::transcribe — implement in v0.4 sprint")
    }
    fn engine_id(&self) -> &'static str {
        "assembly_ai"
    }
    fn requires_api_key(&self) -> bool {
        true
    }
}
