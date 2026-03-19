use tracing::{info, warn, debug};

pub struct StreamStitcher;

impl StreamStitcher {
    pub fn new() -> Self {
        Self
    }

    /// Intercepts a partial stream and detects if it's incomplete or failed.
    pub fn check_stream_integrity(&self, partial_resp: &str) -> bool {
        debug!("🌊 [STITCHER] Checking semantic integrity of partial stream...");
        
        let trimmed = partial_resp.trim();
        if trimmed.is_empty() { return true; }

        // Heuristic: If it doesn't end with a punctuation or it ends with a dangling phrase
        let ends_with_punct = trimmed.ends_with('.') || trimmed.ends_with('!') || trimmed.ends_with('?') || trimmed.ends_with('}');
        
        if !ends_with_punct && trimmed.len() > 100 {
            warn!("⚠️ [STITCHER] Mid-stream failure detected (Dangling thought). Attempting stitch...");
            return false;
        }

        true
    }

    /// Stitches a completion phrase onto a failed stream.
    pub fn stitch_completion(&self, original: &str, completion: &str) -> String {
        info!("🧵 [STITCHER] Seamlessly stitching recovery completion onto original stream.");
        format!("{} {}", original.trim_end(), completion.trim_start())
    }
}
