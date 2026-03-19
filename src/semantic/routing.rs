use tracing::info;

pub struct SemanticRouter;

impl SemanticRouter {
    pub fn new() -> Self {
        SemanticRouter {}
    }

    /// Evaluates prompt complexity and returns the optimal model.
    pub fn determine_optimal_model(&self, prompt: &str) -> String {
        // Heuristics for complexity: Length, math operators, code keywords
        let length = prompt.len();
        let normalized = prompt.to_lowercase();
        let is_math = normalized.contains('+') || normalized.contains('-') || normalized.contains('*') || normalized.contains('/') || normalized.contains("calculate");
        let is_code = normalized.contains("fn ") || normalized.contains("class ") || normalized.contains("def ") || normalized.contains("struct ");

        if length < 50 && !is_math && !is_code {
            info!("🛣️ [SEMANTIC ROUTER] Simple query detected. Routing to Llama-3-8B.");
            "llama-3-8b".to_string()
        } else if length < 250 && !is_code {
            info!("🛣️ [SEMANTIC ROUTER] Moderate query detected. Routing to GPT-4o-mini.");
            "gpt-4o-mini".to_string()
        } else {
            info!("🛣️ [SEMANTIC ROUTER] Complex/Math/Code query detected. Routing to GPT-4o.");
            "gpt-4o".to_string()
        }
    }
}
