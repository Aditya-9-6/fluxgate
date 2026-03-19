use std::collections::HashMap;
use tracing::{info, debug};

pub struct EvalRouter {
    pub quality_scores: HashMap<String, f32>, // ModelName -> QualityScore (0.0 to 1.0)
}

impl EvalRouter {
    pub fn new() -> Self {
        let mut scores = HashMap::new();
        scores.insert("gpt-4o".to_string(), 0.92);
        scores.insert("claude-3-5-sonnet".to_string(), 0.95);
        scores.insert("llama-3-70b".to_string(), 0.88);
        
        Self { quality_scores: scores }
    }

    /// Selects the best performing model based on real-time evaluation data.
    pub fn route_by_quality(&self) -> String {
        let mut best_model = "gpt-4o-mini".to_string();
        let mut max_score = -1.0;

        for (model, score) in &self.quality_scores {
            if *score > max_score {
                max_score = *score;
                best_model = model.clone();
            }
        }

        info!("🎯 [EVAL-ROUTING] Dynamic steering to highest quality model: {} (Score: {:.2})", 
              best_model, max_score);
        best_model
    }

    /// Updates the quality score for a model based on new evaluation telemetry.
    pub fn record_eval(&mut self, model: &str, score: f32) {
        debug!("📊 [EVAL-ROUTING] Telemetry update: {} quality adjusted to {:.2}.", model, score);
        self.quality_scores.insert(model.to_string(), score);
    }
}
