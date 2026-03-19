use tracing::debug;

pub struct PromptCompressor;

impl PromptCompressor {
    pub fn new() -> Self {
        PromptCompressor {}
    }

    pub fn compress(&self, prompt: &str) -> String {
        let mut words: Vec<&str> = prompt.split_whitespace().collect();
        let original_len = words.len();

        let stop_words = ["please", "can", "you", "tell", "me", "the", "a", "an", "is", "are"];
        
        words.retain(|&w| !stop_words.contains(&w.to_lowercase().as_str()));
        
        let compressed = words.join(" ");
        let compressed_len = words.len();

        if original_len > 0 {
            let savings = (original_len - compressed_len) as f32 / original_len as f32 * 100.0;
            debug!("🗜️ [PROMPT COMPRESSION] Compressed prompt by {:.1}% ({} -> {} words)", savings, original_len, compressed_len);
        }

        compressed
    }
}
