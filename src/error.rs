use thiserror::Error;
use pingora::Error as PingoraError;

#[derive(Error, Debug)]
pub enum FluxError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Network/IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP client error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Authorization error: {0}")]
    Auth(String),

    #[error("Proxy session error: {0}")]
    Pingora(#[from] PingoraError),

    #[error("Internal logic error: {0}")]
    Internal(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Agent Impersonation detected for user: {0}")]
    ImpersonationAttempt(String),

    #[error("Prompt Leakage detected in request")]
    PromptLeakage,

    #[error("Model Extraction probing detected for user: {0}")]
    ModelExtractionProbing(String),

    #[error("Generic error: {0}")]
    Other(#[from] anyhow::Error),
}

pub type FluxResult<T> = Result<T, FluxError>;
