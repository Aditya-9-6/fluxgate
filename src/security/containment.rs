use hmac::{Hmac, Mac};
use sha2::Sha256;
use tracing::{info, warn};

type HmacSha256 = Hmac<Sha256>;

/// Evolution Containment Layer (Crypto Locks).
/// Ensures an agent's behavioral core remains signed and immutable against unauthorized evolution.
pub struct EvolutionContainment {
    secret_key: Vec<u8>,
}

impl EvolutionContainment {
    pub fn new(secret: &str) -> Self {
        Self { secret_key: secret.as_bytes().to_vec() }
    }

    /// Generates a crypto-lock signature for an agent's current behavioral state.
    pub fn lock_state(&self, agent_id: &str, state_blob: &[u8]) -> String {
        let mut mac = HmacSha256::new_from_slice(&self.secret_key).expect("HMAC can take key of any size");
        mac.update(agent_id.as_bytes());
        mac.update(state_blob);
        let result = mac.finalize();
        let code_bytes = result.into_bytes();
        hex::encode(code_bytes)
    }

    /// Verifies if the agent's state matches the crypto-lock.
    pub fn verify_lock(&self, agent_id: &str, state_blob: &[u8], signature: &str) -> bool {
        let mut mac = HmacSha256::new_from_slice(&self.secret_key).expect("HMAC can take key of any size");
        mac.update(agent_id.as_bytes());
        mac.update(state_blob);
        
        if let Ok(sig_bytes) = hex::decode(signature) {
            if mac.verify_slice(&sig_bytes).is_ok() {
                info!("🔓 [CONTAINMENT] Crypto-lock verified for agent {}. State integrity confirmed.", agent_id);
                return true;
            }
        }

        warn!("🔒 [CONTAINMENT] CRITICAL: Crypto-lock mismatch for agent {}! Unauthorized evolution detected.", agent_id);
        false
    }
}
