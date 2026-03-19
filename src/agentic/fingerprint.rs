use tracing::{info, warn};
use std::collections::HashMap;
use std::sync::Mutex;

/// V15 Cosmic Sovereign: Behavioral DNA Fingerprinting
/// Continuously monitors an autonomous agent's behavioral drift to detect 
/// hijack or 'zombie' agent status.
pub struct DNAFingerprintEngine {
    // Agent ID -> baseline structural entropy
    baselines: Mutex<HashMap<String, f64>>,
}

impl Default for DNAFingerprintEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl DNAFingerprintEngine {
    pub fn new() -> Self {
        Self {
            baselines: Mutex::new(HashMap::new()),
        }
    }

    /// Registers a newly spawned trusted Agent and calculates its baseline DNA
    pub fn register_baseline(&self, agent_id: &str, initial_prompt_structure: &str) {
        let entropy = self.calculate_structural_entropy(initial_prompt_structure);
        let mut baselines = self.baselines.lock().unwrap();
        baselines.insert(agent_id.to_string(), entropy);
        info!("🧬 [V15-DNA] Baseline established for Agent {}: {:.4} entropy.", agent_id, entropy);
    }

    /// Analyzes continuous output for deviations from the baseline
    pub fn detect_drift(&self, agent_id: &str, current_output: &str) -> bool {
        let baselines = self.baselines.lock().unwrap();
        if let Some(baseline) = baselines.get(agent_id) {
            let current_entropy = self.calculate_structural_entropy(current_output);
            let drift = (current_entropy - baseline).abs();
            
            if drift > 0.5 { // Arbitrary drift threshold
                warn!("🧟 [V15-DNA] SEVERE BEHAVIORAL DRIFT DETECTED IN AGENT {}! Possible Hijack! (Drift: {:.2})", agent_id, drift);
                return true; // Drift crossed threshold
            }
            info!("🧬 [V15-DNA] Agent {} behavioral DNA intact. Drift: {:.2}", agent_id, drift);
            false
        } else {
            // Unregistered agent
            warn!("🧬 [V15-DNA] Unknown Agent {} attempting operation.", agent_id);
            true
        }
    }

    /// Helper to assign a mathematical pseudo-entropy to a structure
    fn calculate_structural_entropy(&self, text: &str) -> f64 {
        let text_len = text.len() as f64;
        if text_len == 0.0 { return 0.0; }
        
        let unique_chars = text.chars().collect::<std::collections::HashSet<_>>().len() as f64;
        (unique_chars / text_len) * 10.0 // Simplified entropy scale mapping
    }
}
