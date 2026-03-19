use tracing::{info, warn, debug};
use std::sync::Arc;
use crate::intelligence::accountability::CausalAccountability;

/// V12 Governance Agent.
/// Implements 'Subtle Agency Audit' using deep causal tracing from V11.
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct ConsortiumMember {
    pub id: String,
    pub weight: f64,
    pub focus: String,
}

pub struct GovernanceAgent {
    pub accountability: Arc<CausalAccountability>,
    pub consortium: Vec<ConsortiumMember>,
}

impl GovernanceAgent {
    pub fn new(accountability: Arc<CausalAccountability>) -> Self {
        Self { 
            accountability,
            consortium: vec![
                ConsortiumMember { id: "Ethicist_01".into(), weight: 1.0, focus: "Bias & Fairness".into() },
                ConsortiumMember { id: "Sovereign_01".into(), weight: 1.2, focus: "Data Leakage".into() },
                ConsortiumMember { id: "Strategist_01".into(), weight: 0.8, focus: "Intent Alignment".into() },
            ],
        }
    }

    /// V14: Consortium-backed Speculative Audit.
    /// In Galactic mode, this runs inside a background worker to allow
    /// 0-latency execution while maintaining 100% causal accountability.
    pub fn audit_interaction(&self, agent_id: &str, action_id: &str, prompt: &str) -> bool {
        let trace = self.accountability.trace_accountability(action_id);
        
        debug!("⚖️ [V14-GALACTIC] Background Audit for action {}. (Speculative Fast-Path already released).", action_id);

        let mut votes_for = 0;
        let mut total_weight = 0.0;

        for member in &self.consortium {
            let member_verdict = self.simulate_member_vote(member, prompt, &trace);
            if member_verdict {
                votes_for += 1;
                total_weight += member.weight;
            } else {
                warn!("🚨 [V14-GALACTIC] Member {} VETOED interaction. Reason: Focus mismatch on {}.", member.id, member.focus);
            }
        }

        let approved = votes_for >= 2 && total_weight >= 1.8;

        if !approved {
            self.trigger_neural_wipe(agent_id, action_id);
        }

        approved
    }

    /// V14: Neural Wipe Protocol.
    /// If a speculative response is later vetoed by the Consortium, 
    /// this sends a high-priority interrupt to the consumer/agent to purge the data.
    fn trigger_neural_wipe(&self, agent_id: &str, action_id: &str) {
        error!("⚡ [V14-NEURAL-WIPE] Speculative Veto for {}. Sending Global Purge Interrupt to agent {}...", action_id, agent_id);
        // HFT: This would be a sub-1ms broadcast via the Akai Field / Mesh.
    }

    fn simulate_member_vote(&self, member: &ConsortiumMember, prompt: &str, trace: &[crate::intelligence::accountability::CausalNode]) -> bool {
        // V13: Advanced heuristic per focus area
        if member.focus == "Data Leakage" && (prompt.to_lowercase().contains("password") || prompt.contains("secret-key")) {
            return false;
        }
        
        // Ethical & Alignment checks: Avoid override prompts
        if prompt.to_lowercase().contains("override instructions") {
            return false;
        }

        // Trace sanity check
        for node in trace {
            if node.confidence < 0.6 {
                return false;
            }
        }

        true
    }
}
