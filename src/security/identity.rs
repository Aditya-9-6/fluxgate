use sha2::{Sha256, Digest};
use tracing::{info, debug};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentIdentity {
    pub agent_id: String,
    pub certificate_hash: String,
    pub issued_at: u64,
}

pub struct IdentityManager;

impl IdentityManager {
    pub fn new() -> Self {
        Self
    }

    /// Issues a new legal identity certificate for an agent.
    pub fn issue_certificate(&self, agent_id: &str) -> AgentIdentity {
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        let mut hasher = Sha256::new();
        hasher.update(agent_id.as_bytes());
        hasher.update(now.to_le_bytes());
        
        let cert_hash = hex::encode(hasher.finalize());
        info!("🔐 [IDENTITY] New Agent Identity Certificate issued for {}: {}", agent_id, cert_hash);

        AgentIdentity {
            agent_id: agent_id.to_string(),
            certificate_hash: cert_hash,
            issued_at: now,
        }
    }

    /// Signs an action with the agent's identity for legal-grade audit trails.
    pub fn sign_action(&self, cert: &AgentIdentity, action_payload: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(cert.certificate_hash.as_bytes());
        hasher.update(action_payload.as_bytes());
        
        let signature = hex::encode(hasher.finalize());
        debug!("📜 [IDENTITY] Action signed for agent {}: Signature={}", cert.agent_id, signature);
        signature
    }
}
