use std::sync::Arc;
use redis::Client;

pub struct PowChallenge {
    _redis: Arc<Client>,
}

impl PowChallenge {
    pub fn new(redis: Arc<Client>) -> Self { Self { _redis: redis } }

    pub fn get_difficulty(&self) -> u32 { 3 }
    pub fn generate_challenge(&self) -> String { "diff:3;nonce:123".to_string() }
    pub async fn verify_pow(&self, challenge: &str, nonce: &str) -> bool { true }
}
