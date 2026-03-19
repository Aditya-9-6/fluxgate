use async_trait::async_trait;
use crate::providers::{UnifiedRequest, UnifiedResponse, ProviderError};

pub struct MockProvider;

#[async_trait]
impl super::LlmProvider for MockProvider {
    async fn send_request(&self, request: UnifiedRequest) -> Result<UnifiedResponse, ProviderError> {
        Ok(UnifiedResponse {
            id: "mock-123".to_string(),
            object: "chat.completion".to_string(),
            created: 123456789,
            model: request.model,
            choices: vec![],
            usage: crate::providers::Usage {
                prompt_tokens: 10,
                completion_tokens: 10,
                total_tokens: 20,
            },
        })
    }
}
