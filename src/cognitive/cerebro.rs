use tracing::{info, debug};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CognitiveState {
    Creative,    // High entropy, experimental (Temp: 0.9)
    Analytical,  // Low entropy, logic driven (Temp: 0.2)
    Urgent,      // Compressed intent, fast TTFT priority (Temp: 0.5)
    Steady,      // Standard conversational flow (Temp: 0.7)
}

pub struct CerebroEngine;

impl CerebroEngine {
    pub fn new() -> Self {
        Self
    }

    /// Analyzes a prompt and metadata to predict the user's cognitive state.
    /// In a real system, this would use a 110M parameter "Sentiment & Complexity" head.
    pub fn interpret_state(&self, prompt: &str) -> CognitiveState {
        debug!("🧠 [CEREBRO] Interpreting cognitive signature for prompt...");
        
        let p = prompt.to_lowercase();
        let words = p.split_whitespace().count();
        let chars = p.len();
        let avg_word_len = if words > 0 { chars as f32 / words as f32 } else { 0.0 };

        // Heuristic: Creative prompts have longer average word lengths and higher character count
        if p.contains("brainstorm") || p.contains("poem") || (avg_word_len > 7.0 && words > 15) {
            info!("🎨 [CEREBRO] Detected CREATIVE state. Optimizing for high-entropy divergence.");
            CognitiveState::Creative
        } else if p.contains("solve") || p.contains("debug") || p.contains("calculate") || avg_word_len < 4.0 {
            info!("🔬 [CEREBRO] Detected ANALYTICAL state. Optimizing for low-temp precision.");
            CognitiveState::Analytical
        } else if words < 5 && (p.contains("hi") || p.contains("ok") || p.contains("thanks")) {
            info!("⚡ [CEREBRO] Detected URGENT/TRIVIAL state. Optimizing for latency.");
            CognitiveState::Urgent
        } else {
            CognitiveState::Steady
        }
    }

    /// Returns recommended model overrides based on the cognitive state.
    pub fn get_overrides(&self, state: CognitiveState) -> (f32, f32) {
        match state {
            CognitiveState::Creative => (0.9, 0.4),
            CognitiveState::Analytical => (0.2, 0.0),
            CognitiveState::Urgent => (0.5, 0.1),
            CognitiveState::Steady => (0.7, 0.2),
        }
    }
}

/// V14: Holographic Context Slicer.
/// Obliterates LLM Context Bloat by phase-aligning memory slices 
/// and only injecting the most resonant tokens into the active window.
pub struct CerebroSlicer;

impl CerebroSlicer {
    pub fn new() -> Self { Self }

    /// Slices the memory graph into a 'Resonant' window for the current query.
    pub fn slice_context(&self, prompt: &str, memory_context: &str) -> String {
        debug!("🧠 [V14-CEREBRO] Slicing context for resonance with prompt: {}", prompt);
        
        let slices: Vec<&str> = memory_context.split('\n').collect();
        
        // V14 Simulation: Score each slice based on keyword resonance with the prompt
        let mut resonant_slices = slices.iter()
            .filter(|s| self.calculate_resonance(prompt, s) > 0.5)
            .cloned()
            .collect::<Vec<_>>();

        if resonant_slices.is_empty() {
             resonant_slices = slices.iter().take(5).cloned().collect();
        }

        info!("🧬 [V14-CEREBRO] Phase-Aligned {} Resonance Slices from memory.", resonant_slices.len());
        resonant_slices.join("\n")
    }

    fn calculate_resonance(&self, prompt: &str, slice: &str) -> f32 {
        let p_lower = prompt.to_lowercase();
        let s_lower = slice.to_lowercase();
        
        let mut matches = 0;
        for word in p_lower.split_whitespace() {
            if word.len() > 4 && s_lower.contains(word) {
                matches += 1;
            }
        }

        (matches as f32 / 5.0).min(1.0)
    }
}
