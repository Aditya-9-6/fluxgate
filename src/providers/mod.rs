use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnifiedRequest {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnifiedResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Choice {
    pub message: Message,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Usage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

#[derive(Debug)]
pub enum ProviderError {
    Network(String),
    Api(String),
    Serialization(String),
    Auth(String),
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn send_request(&self, request: UnifiedRequest) -> Result<UnifiedResponse, ProviderError>;
}

pub mod openai;
pub mod deepseek;
pub mod anthropic;
pub mod google_gemini;
pub mod ollama;
// pub mod aws_bedrock;
// pub mod azure_openai;
// pub mod cohere;
// pub mod mistral;
// pub mod together_ai;
pub mod mock;
