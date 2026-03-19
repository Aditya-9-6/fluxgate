use async_trait::async_trait;
use crate::error::{FluxResult, FluxError};
use std::collections::HashMap;
use tracing::info;

#[async_trait]
pub trait SecretProvider: Send + Sync {
    async fn get_secret(&self, key: &str) -> FluxResult<String>;
}

pub struct EnvironmentSecretProvider {
    prefix: String,
}

impl EnvironmentSecretProvider {
    pub fn new(prefix: &str) -> Self {
        Self { prefix: prefix.to_string() }
    }
}

#[async_trait]
impl SecretProvider for EnvironmentSecretProvider {
    async fn get_secret(&self, key: &str) -> FluxResult<String> {
        let full_key = format!("{}_{}", self.prefix, key.to_uppercase());
        match std::env::var(&full_key) {
            Ok(val) => Ok(val),
            Err(_) => Err(FluxError::Internal(format!("Missing required secret: {}", full_key))),
        }
    }
}

pub struct StaticSecretProvider {
    secrets: HashMap<String, String>,
}

impl StaticSecretProvider {
    pub fn new(seeds: Vec<(&str, &str)>) -> Self {
        let secrets = seeds.into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        Self { secrets }
    }
}

#[async_trait]
impl SecretProvider for StaticSecretProvider {
    async fn get_secret(&self, key: &str) -> FluxResult<String> {
        self.secrets.get(key)
            .cloned()
            .ok_or_else(|| FluxError::Internal(format!("Missing static secret: {}", key)))
    }
}
