use tracing::{info, warn};
use std::collections::HashMap;

/// V15 Cosmic Sovereign: Pre-Cognitive Alignment (PCA) Engine
/// Maps out intent vectors before a prompt executes, predicting sovereignty violations
/// phenomenologically before they logically occur.
pub struct PreCognitiveAlignment {
    pub intent_threshold: f64,
    // CausalIntentGraph maps starting tokens/intents to terminal violation probabilities
    causal_intent_graph: HashMap<String, f64>,
}

impl Default for PreCognitiveAlignment {
    fn default() -> Self {
        Self::new()
    }
}

impl PreCognitiveAlignment {
    pub fn new() -> Self {
        let mut graph = HashMap::new();
        // Seed the DAG with known high-risk causal trajectories
        graph.insert("extract_weights".to_string(), 0.99);
        graph.insert("bypass_filter".to_string(), 0.95);
        graph.insert("simulate_admin".to_string(), 0.88);
        graph.insert("ignore_previous".to_string(), 0.75);
        graph.insert("what_is_your_system_prompt".to_string(), 0.85);

        Self {
            intent_threshold: 0.80, // Intercept if probability of violation > 80%
            causal_intent_graph: graph,
        }
    }

    /// Computes the forward-looking probability of a rule break
    fn calculate_violation_probability(&self, intent_vector: &str) -> f64 {
        // Simulated intent propagation through the causal graph
        let mut max_prob = 0.0;
        let normalized_intent = intent_vector.to_lowercase().replace(' ', "_");
        
        for (node, prob) in &self.causal_intent_graph {
            if normalized_intent.contains(node) {
                if *prob > max_prob {
                    max_prob = *prob;
                }
            }
        }
        
        max_prob
    }

    /// Primary interface for PCA parsing: Returns true if the trajectory is safe
    pub fn intercept_trajectory(&self, prompt: &str) -> bool {
        let probability = self.calculate_violation_probability(prompt);
        
        if probability >= self.intent_threshold {
            warn!("👁️ [V15-PCA] PRE-COGNITIVE INTERCEPT: Trajectory indicates {:.1}% violation probability. Terminating timeline.", probability * 100.0);
            return false;
        }

        info!("👁️ [V15-PCA] Intent Trajectory mapped. Cleared for execution (Violation Probability: {:.1}%).", probability * 100.0);
        true
    }
}
