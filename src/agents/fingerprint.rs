use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, debug};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentDNA {
    pub agent_id: String,
    pub entropy_score: f32,      // Structural complexity of prompts
    pub avg_jitter_ms: u32,       // Variance in request timing
    pub focus_vector: Vec<f32>,   // Semantic topic distribution
    pub last_seen: u64,
}

pub struct DNAProfiler {
    pub registry: Arc<RwLock<HashMap<String, AgentDNA>>>,
}

impl DNAProfiler {
    pub fn new() -> Self {
        Self {
            registry: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Records an observation of an agent's behavior and updates its DNA.
    pub async fn record_observation(&self, agent_id: &str, prompt: &str, latency_ms: u32) {
        let mut registry = self.registry.write().await;
        let dna = registry.entry(agent_id.to_string()).or_insert(AgentDNA {
            agent_id: agent_id.to_string(),
            entropy_score: 0.5,
            avg_jitter_ms: 100,
            focus_vector: vec![0.0; 8],
            last_seen: 0,
        });

        // Update entropy (simplified: length-based complexity)
        let current_entropy = (prompt.len() as f32 / 1000.0).min(1.0);
        dna.entropy_score = (dna.entropy_score * 0.9) + (current_entropy * 0.1);

        // Update jitter
        dna.avg_jitter_ms = (dna.avg_jitter_ms as f32 * 0.9 + latency_ms as f32 * 0.1) as u32;

        debug!("🧬 [DNA] Updated fingerprint for agent {}: Entropy={:.2}, Jitter={}ms", 
               agent_id, dna.entropy_score, dna.avg_jitter_ms);
    }

    /// Checks if the current behavior deviates from the stored DNA.
    pub async fn check_drift(&self, agent_id: &str, current_prompt: &str) -> bool {
        let registry = self.registry.read().await;
        if let Some(dna) = registry.get(agent_id) {
            let current_entropy = (current_prompt.len() as f32 / 1000.0).min(1.0);
            let drift = (dna.entropy_score - current_entropy).abs();

            if drift > 0.4 {
                warn!("🚨 [DNA] ALERT: Behavioral drift detected for agent {}! Drift={:.2}", agent_id, drift);
                return true; // Drift indicates possible hijack or memory poisoning
            }
        }
        false
    }
}
