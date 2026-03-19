use dashmap::DashMap;
use std::sync::Arc;
use tracing::{info, debug};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryFact {
    pub key: String,
    pub value: String,
    pub originator: String,
    pub caused_by: Option<String>, 
    pub timestamp: u64,
    pub logical_clock: u64,
    /// V15: Quantum Phase (Holographic Resonance).
    pub phase: (f32, f32), // Simulated complex number (Real, Imaginary)
    pub superposition_depth: u8,
}

/// V15 Quantum-Holographic Memory Graph.
/// Facts exist in super-position until aligned by a sovereign observer.
pub struct SharedMemoryGraph {
    pub graph: Arc<DashMap<String, Vec<MemoryFact>>>, // Key -> List of super-positioned facts
    pub vector_clock: Arc<DashMap<String, u64>>,
}

impl SharedMemoryGraph {
    pub fn new() -> Self {
        Self {
            graph: Arc::new(DashMap::new()),
            vector_clock: Arc::new(DashMap::new()),
        }
    }

    /// Publishes a fact into the quantum super-position of the graph.
    pub async fn publish_fact(&self, agent_id: &str, key: &str, value: &str, causal_link: Option<String>) {
        let mut clock_entry = self.vector_clock.entry(agent_id.to_string()).or_insert(0);
        *clock_entry += 1;
        let logical_clock = *clock_entry;

        // V15: Phase calculation based on 'Causal Resonance' (Simulated)
        let phase = (rand::random::<f32>(), rand::random::<f32>());

        let fact = MemoryFact {
            key: key.to_string(),
            value: value.to_string(),
            originator: agent_id.to_string(),
            caused_by: causal_link,
            timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            logical_clock,
            phase,
            superposition_depth: 0,
        };

        let mut entries = self.graph.entry(key.to_string()).or_insert_with(Vec::new);
        entries.push(fact);
        
        info!("🧠 [V15-HOLOGRAPHIC] Fact collapsed into super-position: {} (Phase: {:.2?})", key, phase);
    }

    /// Retrieves the most 'Resonant' fact from the quantum stack.
    pub async fn subscribe_fact(&self, key: &str) -> Option<MemoryFact> {
        let entries = self.graph.get(key)?;
        // V15: Selection based on phase alignment (Simulating 'Alignment collapse')
        entries.iter()
            .max_by(|a, b| a.phase.0.partial_cmp(&b.phase.0).unwrap())
            .cloned()
    }

    pub async fn get_consolidated_context(&self) -> String {
        self.graph.iter()
            .map(|r| {
                let best = r.value().iter().max_by(|a, b| a.phase.0.partial_cmp(&b.phase.0).unwrap()).unwrap();
                format!("{}: {}", r.key(), best.value)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// V15: Reconciles remote facts into the local quantum stack.
    pub async fn reconcile_remote_fact(&self, mut remote_fact: MemoryFact) {
        remote_fact.superposition_depth += 1;
        let mut entries = self.graph.entry(remote_fact.key.clone()).or_insert_with(Vec::new);
        entries.push(remote_fact);
        debug!("🛰️ [V15-SYNC] Remote fact integrated into holographic stack.");
    }
}
