use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::providers::{UnifiedRequest, UnifiedResponse, ProviderError};

pub struct OpenAIProvider {
    pub api_key: String,
    pub base_url: String,
}

impl OpenAIProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.openai.com/v1".to_string(),
        }
    }
}

#[async_trait]
impl super::LlmProvider for OpenAIProvider {
    async fn send_request(&self, request: UnifiedRequest) -> Result<UnifiedResponse, ProviderError> {
        let client = reqwest::Client::new();
        let url = format!("{}/chat/completions", self.base_url);
        
        let response = client.post(&url)
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()
            .await
            .map_err(|e| ProviderError::Network(e.to_string()))?;

        if response.status().is_success() {
            let res: UnifiedResponse = response.json().await
                .map_err(|e| ProviderError::Serialization(e.to_string()))?;
            Ok(res)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(ProviderError::Api(error_text))
        }
    }
}
