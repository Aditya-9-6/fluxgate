use dashmap::DashMap;
use tracing::{info, debug};

#[derive(Debug, Clone)]
pub struct CausalLink {
    pub trigger: String,
    pub confidence: f32,
    pub depth: u32,
}

/// V11 Causal Accountability Layer.
/// Traces the logical 'Recursive Intent' chain with depth-aware multi-step auditing.
pub struct CausalAccountability {
    pub causal_nodes: DashMap<String, CausalLink>,
}

impl CausalAccountability {
    pub fn new() -> Self {
        Self { causal_nodes: DashMap::new() }
    }

    /// Records a causal link with V11 depth and confidence.
    pub fn record_causality(&self, trigger: &str, action: &str, confidence: f32, depth: u32) {
        info!("🔗 [V11-CAUSAL] Linking trigger '{}' -> action '{}' (Conf: {:.2}, Depth: {})", 
              trigger, action, confidence, depth);
        self.causal_nodes.insert(action.to_string(), CausalLink {
            trigger: trigger.to_string(),
            confidence,
            depth,
        });
    }

    /// Generates a deep causal trace for a given action.
    pub fn trace_accountability(&self, action: &str) -> Vec<CausalLink> {
        let mut trace = Vec::new();
        let mut current = action.to_string();

        while let Some(link_ref) = self.causal_nodes.get(&current) {
            let link = link_ref.value().clone();
            trace.push(link.clone());
            current = link.trigger;
        }

        trace
    }
}
