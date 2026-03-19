use async_trait::async_trait;
use crate::providers::{UnifiedRequest, UnifiedResponse, ProviderError};

pub struct AnthropicProvider {
    pub api_key: String,
    pub base_url: String,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.anthropic.com/v1".to_string(),
        }
    }
}

#[async_trait]
impl super::LlmProvider for AnthropicProvider {
    async fn send_request(&self, request: UnifiedRequest) -> Result<UnifiedResponse, ProviderError> {
        // Implementation for Anthropic API
        Err(ProviderError::Api("Anthropic implementation pending".to_string()))
    }
}
