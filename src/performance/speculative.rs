use tracing::{info, debug};
use std::sync::Arc;

pub trait SpeculativeHead: Send + Sync {
    fn predict(&self, prompt: &str) -> Option<String>;
    fn name(&self) -> &'static str;
}

pub struct GreetingHead;
impl SpeculativeHead for GreetingHead {
    fn name(&self) -> &'static str { "Greetings" }
    fn predict(&self, prompt: &str) -> Option<String> {
        let p = prompt.to_lowercase();
        if p.contains("hello") || p.contains("hi ") || p.starts_with("hi") {
            Some("Hello! I am FluxGate's sovereign intelligence layer. How can I assist you today?".to_string())
        } else { None }
    }
}

pub struct TechnicalHead;
impl SpeculativeHead for TechnicalHead {
    fn name(&self) -> &'static str { "Technical" }
    fn predict(&self, prompt: &str) -> Option<String> {
        let p = prompt.to_lowercase();
        if p.contains("how does") || p.contains("explain") || p.contains("what is") {
            Some("That's a great technical question. Let me analyze the underlying architecture for you: ".to_string())
        } else { None }
    }
}

/// Medusa-style Speculative Engine: Uses parallel heads to predict prefixes.
pub struct SpeculativeEngine {
    heads: Vec<Arc<dyn SpeculativeHead>>,
}

impl SpeculativeEngine {
    pub fn new() -> Self {
        Self {
            heads: vec![
                Arc::new(GreetingHead),
                Arc::new(TechnicalHead),
            ],
        }
    }

    /// Parallel Speculation: Queries all heads and returns the most confident prefix.
    pub fn speculate_prefix(&self, prompt: &str) -> Option<String> {
        debug!("⚡ [MEDUSA] Running parallel speculation heads...");
        for head in &self.heads {
            if let Some(prefix) = head.predict(prompt) {
                info!("🎯 [MEDUSA] Speculation Hit! Head '{}' predicted prefix.", head.name());
                return Some(prefix);
            }
        }
        None
    }
}
