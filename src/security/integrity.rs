use sha2::{Sha256, Digest};
use hmac::{Hmac, Mac};
use tracing::{info, debug};

type HmacSha256 = Hmac<Sha256>;

/// Integrity Engine for Sovereign DeAI.
/// Provides cryptographic proofs that AI responses are untampered.
pub struct IntegrityEngine {
    pub secret: Vec<u8>,
}

impl IntegrityEngine {
    pub fn new(secret: &str) -> Self {
        Self { secret: secret.as_bytes().to_vec() }
    }

    /// Sign a response to generate an integrity proof for the client.
    pub fn sign_response(&self, response: &str) -> String {
        debug!("🔐 [INTEGRITY] Signing response to generate sovereignty proof...");
        let mut mac = HmacSha256::new_from_slice(&self.secret)
            .expect("HMAC can take key of any size");
        mac.update(response.as_bytes());
        let result = mac.finalize();
        let code_bytes = result.into_bytes();
        hex::encode(code_bytes)
    }

    /// Verify an inbound response from a 'Sovereign' cloud provider.
    pub fn verify_sovereign_response(&self, response: &str, proof: &str) -> bool {
        let expected = self.sign_response(response);
        if expected == proof {
            info!("✅ [INTEGRITY] Sovereign response verified! Proof matches.");
            true
        } else {
            tracing::warn!("❌ [INTEGRITY] WARNING: Response integrity proof mismatch! Possible tampering.");
            false
        }
    }
}
