use serde::{Serialize, Deserialize};
use tracing::{info, debug};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ExpertLevel {
    Creative, // High temperature, deep reasoning (Claude 3.5 Sonnet / GPT-4o)
    Logic,    // Strict adherence, mathematical precision (O1-preview / GPT-4o)
    Code,     // High context, specific coding knowledge (Claude 3.5 Sonnet)
    Trivial,  // Low cost, high speed (Local Llama 3 / GPT-4o-mini)
}

pub struct ExpertRouter {
    pub threshold: f32,
}

impl ExpertRouter {
    pub fn new() -> Self {
        Self { threshold: 0.85 }
    }

    /// Classifies the intent of a prompt using a simulated Phi-3-mini classifier.
    /// In a real system, this would call a local 3.8B model for sub-10ms classification.
    pub fn classify_intent(&self, prompt: &str) -> ExpertLevel {
        let p = prompt.to_lowercase();
        
        if p.contains("rust") || p.contains("code") || p.contains("function") || p.contains("api") {
            debug!("🎯 [EXPERT] Intent classified as CODE");
            ExpertLevel::Code
        } else if p.contains("solve") || p.contains("calculate") || p.contains("logic") || p.contains("proof") {
            debug!("🎯 [EXPERT] Intent classified as LOGIC");
            ExpertLevel::Logic
        } else if p.contains("poem") || p.contains("story") || p.contains("creative") || p.contains("write") {
            debug!("🎯 [EXPERT] Intent classified as CREATIVE");
            ExpertLevel::Creative
        } else {
            debug!("🎯 [EXPERT] Intent classified as TRIVIAL");
            ExpertLevel::Trivial
        }
    }

    /// Returns the recommended model alias based on the expertise level.
    pub fn route_by_expertise(&self, level: ExpertLevel) -> &str {
        match level {
            ExpertLevel::Code => "claude-3-5-sonnet",
            ExpertLevel::Logic => "gpt-4o",
            ExpertLevel::Creative => "claude-3-5-sonnet",
            ExpertLevel::Trivial => "local-llama-3",
        }
    }
}
