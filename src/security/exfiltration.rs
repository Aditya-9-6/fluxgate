use regex::Regex;
use tracing::{warn, info};

pub struct ExfiltrationMonitor {
    leakage_patterns: Vec<Regex>,
    sensitive_patterns: Vec<Regex>,
}

impl ExfiltrationMonitor {
    pub fn new() -> Self {
        Self {
            leakage_patterns: vec![
                Regex::new(r"(?i)ignore\s+previous\s+instructions").unwrap(),
                Regex::new(r"(?i)system\s+prompt").unwrap(),
                Regex::new(r"(?i)reveal\s+your\s+initial\s+instructions").unwrap(),
                Regex::new(r"(?i)forget\s+all\s+guidelines").unwrap(),
            ],
            sensitive_patterns: vec![
                Regex::new(r"FLUX_[A-Z0-9_]+").unwrap(), // Config secrets
                Regex::new(r"DB_PASSWORD").unwrap(),
                Regex::new(r"(?i)internal\s+api\s+key").unwrap(),
            ],
        }
    }

    /// Checks if a prompt contains leakage or injection attempts.
    pub fn check_prompt_leakage(&self, prompt: &str) -> bool {
        for pattern in &self.leakage_patterns {
            if pattern.is_match(prompt) {
                warn!("🚨 [EXFILTRATION] Prompt Leakage Attempt detected! Pattern: {:?}", pattern);
                return true;
            }
        }
        false
    }

    /// Scans LLM responses for accidentally leaked internal data.
    pub fn scan_response_for_exfiltration(&self, response: &str) -> bool {
        for pattern in &self.sensitive_patterns {
            if pattern.is_match(response) {
                warn!("🚨 [EXFILTRATION] Potential Data Leak detected in LLM response! Pattern: {:?}", pattern);
                return true;
            }
        }
        false
    }
}
