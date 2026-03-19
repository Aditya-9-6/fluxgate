use async_trait::async_trait;
use crate::providers::{UnifiedRequest, UnifiedResponse, ProviderError};

pub struct OllamaProvider {
    pub base_url: String,
}

impl OllamaProvider {
    pub fn new(base_url: String) -> Self { Self { base_url } }
}

#[async_trait]
impl super::LlmProvider for OllamaProvider {
    async fn send_request(&self, request: UnifiedRequest) -> Result<UnifiedResponse, ProviderError> {
        // Implementation for Ollama Local API
        Err(ProviderError::Api("Ollama implementation pending".to_string()))
    }
}
