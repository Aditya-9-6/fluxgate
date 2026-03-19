use async_trait::async_trait;
use crate::providers::{UnifiedRequest, UnifiedResponse, ProviderError};

pub struct GeminiProvider {
    pub api_key: String,
}

impl GeminiProvider {
    pub fn new(api_key: String) -> Self { Self { api_key } }
}

#[async_trait]
impl super::LlmProvider for GeminiProvider {
    async fn send_request(&self, request: UnifiedRequest) -> Result<UnifiedResponse, ProviderError> {
        // Implementation for Google Gemini API
        Err(ProviderError::Api("Gemini implementation pending".to_string()))
    }
}
