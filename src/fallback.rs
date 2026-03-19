pub struct FallbackManager;

impl FallbackManager {
    pub fn new() -> Self { Self }

    pub fn get_fallback_provider(&self, failed_provider: &str) -> String {
        match failed_provider {
            "openai" => "anthropic".to_string(),
            _ => "openai".to_string(),
        }
    }
}
