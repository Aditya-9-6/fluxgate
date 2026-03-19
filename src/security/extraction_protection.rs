use dashmap::DashMap;
use tracing::{warn, info};
use std::time::{Instant, Duration};

pub struct ExtractionGuard {
    tracker: DashMap<String, Vec<(Instant, f32)>>, // User -> [(Time, Entropy)]
}

impl ExtractionGuard {
    pub fn new() -> Self {
        Self {
            tracker: DashMap::new(),
        }
    }

    /// Measures query entropy to detect systematic boundary probing (Model Extraction Attacks).
    pub fn monitor_query_entropy(&self, user_id: &str, prompt: &str) -> bool {
        let entropy = self.calculate_entropy(prompt);
        let now = Instant::now();
        
        let mut history = self.tracker.entry(user_id.to_string()).or_insert(Vec::new());
        history.push((now, entropy));
        
        // Prune history older than 1 hour
        history.retain(|(t, _)| now.duration_since(*t) < Duration::from_secs(3600));

        // Detect Extraction Probing
        // Probing usually involves many high-entropy queries in a short window
        if history.len() > 50 {
            let avg_entropy: f32 = history.iter().map(|(_, e)| e).sum::<f32>() / history.len() as f32;
            if avg_entropy > 0.8 {
                warn!("🚨 [EXTRACTION] High-Entropy Probing detected for user {}! Probable Model Extraction attempt.", user_id);
                return true;
            }
        }

        false
    }

    fn calculate_entropy(&self, prompt: &str) -> f32 {
        if prompt.is_empty() { return 0.0; }
        let mut counts = [0usize; 256];
        for b in prompt.as_bytes() {
            counts[*b as usize] += 1;
        }
        
        counts.iter().fold(0.0, |acc, &c| {
            if c == 0 { acc }
            else {
                let p = c as f32 / prompt.len() as f32;
                acc - p * p.log2()
            }
        }) / 8.0 // Normalized 0.0 - 1.0
    }
}
