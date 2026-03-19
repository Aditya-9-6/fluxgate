use std::sync::Arc;
use tracing::{warn, info};
use crate::agents::dna_fingerprint::DNAProfiler;

pub struct ImpersonationGuard {
    dna_profiler: Arc<DNAProfiler>,
}

impl ImpersonationGuard {
    pub fn new(dna_profiler: Arc<DNAProfiler>) -> Self {
        Self { dna_profiler }
    }

    /// Verifies if the agent's current behavior matches its known DNA profile.
    /// If drift is too high, it indicates a potential impersonation or hijack.
    pub async fn verify_identity(&self, agent_id: &str, prompt: &str) -> bool {
        if self.dna_profiler.check_drift(agent_id, prompt).await {
            warn!("🛑 [IMPERSONATION] Agent {} failed behavioral verification. DNA mismatch detected!", agent_id);
            return false;
        }
        
        info!("✅ [IMPERSONATION] Behavioral signature verified for agent {}.", agent_id);
        true
    }
}
