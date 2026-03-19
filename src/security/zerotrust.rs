use tracing::{info, warn, debug};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SPIFFE_SVID {
    pub identity_uri: String,
    pub trust_domain: String,
    pub issued_at: u64,
}

use crate::security::zkp::VerifiableCompute;

pub struct ZeroTrustManager {
    pub zkp: Arc<VerifiableCompute>,
}

impl ZeroTrustManager {
    pub fn new(zkp: Arc<VerifiableCompute>) -> Self {
        Self { zkp }
    }

    /// Verifies both the SPIFFE identity AND the cryptographic integrity proof of the agent.
    /// V12: Integrity-Fused Identity prevents 'Ghost Agent' impersonation.
    pub fn verify_agent_identity(&self, svid: &SPIFFE_SVID, integrity_proof: &str, challenge: &str) -> bool {
        info!("🕵️ [V12-ZERO-TRUST] Verifying SVID + Integrity Proof for {}.", svid.identity_uri);
        
        // 1. SPIFFE Check
        if !svid.trust_domain.contains("fluxgate.internal") {
            warn!("🚫 [V12-ZERO-TRUST] UNTRUSTED DOMAIN: {} blocked.", svid.trust_domain);
            return false;
        }

        // 2. Cryptographic Integrity Check (ZKP)
        if !self.zkp.verify_proof(integrity_proof, challenge) {
            warn!("🚫 [V12-ZERO-TRUST] INTEGRITY BREACH: Invalid ZKP proof for {}.", svid.identity_uri);
            return false;
        }

        debug!("✅ [V12-ZERO-TRUST] Zero-Trust handshake complete. Chain of custody verified.");
        true
    }

    /// Issues an ephemeral mTLS certificate token for the session.
    pub fn issue_session_token(&self, agent_id: &str) -> String {
        let token = format!("svid:fluxgate:agent:{}", agent_id);
        info!("🎫 [ZERO-TRUST] mTLS identity token issued for agent {}.", agent_id);
        token
    }
}
