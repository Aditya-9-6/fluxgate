use regex::Regex;
use metrics::counter;
use std::collections::HashMap;
use std::sync::Arc;
use dashmap::DashMap;

/// Robust De-identification mapping for sovereign privacy.
/// Stores placeholders used in external cloud calls to their original PII values.
#[derive(Debug, Default)]
pub struct DeIdentifier {
    // Map of placeholder -> original value
    mappings: DashMap<String, String>,
}

impl DeIdentifier {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&self, placeholder: String, original: String) {
        self.mappings.insert(placeholder, original);
    }

    pub fn recover(&self, scrubbed_text: &str) -> String {
        let mut recovered = scrubbed_text.to_string();
        for pair in self.mappings.iter() {
            recovered = recovered.replace(pair.key(), pair.value());
        }
        recovered
    }
}

pub struct ScrubberManager {
    rules: Vec<(String, Regex)>,
}

impl ScrubberManager {
    pub fn new() -> Self {
        let mut rules = Vec::new();
        rules.push(("email".to_string(), Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}").unwrap()));
        rules.push(("ssn".to_string(), Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").unwrap()));
        rules.push(("credit_card".to_string(), Regex::new(r"\b(?:\d[ -]*?){13,16}\b").unwrap()));
        rules.push(("phone".to_string(), Regex::new(r"\b(?:\+?1[-. ]?)?\(?([2-9][0-8][0-9])\)?[-. ]?([2-9][0-9]{2})[-. ]?([0-9]{4})\b").unwrap()));
        rules.push(("ip_address".to_string(), Regex::new(r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b").unwrap()));
        
        Self { rules }
    }

    /// Scrubs text and populates a DeIdentifier mapping for later reconstruction.
    pub fn scrub_and_map(&self, text: &str, de_id: &DeIdentifier) -> String {
        let mut scrubbed = text.to_string();
        let mut counter = 1;

        for (name, re) in &self.rules {
            let mut replacements = Vec::new();
            
            for mat in re.find_iter(&scrubbed) {
                let original = mat.as_str().to_string();
                let placeholder = format!("[REDACTED_{}_{}]", name.to_uppercase(), counter);
                replacements.push((original, placeholder));
                counter += 1;
            }

            for (original, placeholder) in replacements {
                scrubbed = scrubbed.replace(&original, &placeholder);
                de_id.insert(placeholder, original);
                counter!("fluxgate_pii_detections_total", "pii_type" => name.clone()).increment(1);
            }
        }
        scrubbed
    }

    pub fn scrub_sensitive_data(&self, text: &str) -> String {
        let mut scrubbed = text.to_string();
        for (name, re) in &self.rules {
            if re.is_match(&scrubbed) {
                counter!("fluxgate_pii_detections_total", "pii_type" => name.clone()).increment(1);
                scrubbed = re.replace_all(&scrubbed, "[REDACTED]").into_owned();
            }
        }
        scrubbed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pii_de_identification_cycle() {
        let scrubber = ScrubberManager::new();
        let de_id = DeIdentifier::new();
        let input = "Contact John at john.doe@example.com or call 555-0199. My SSN is 123-45-6789.";
        
        let scrubbed = scrubber.scrub_and_map(input, &de_id);
        assert!(scrubbed.contains("[REDACTED_EMAIL_1]"));
        assert!(scrubbed.contains("[REDACTED_PHONE_2]"));
        assert!(scrubbed.contains("[REDACTED_SSN_3]"));
        assert!(!scrubbed.contains("john.doe@example.com"));

        let recovered = de_id.recover(&scrubbed);
        assert_eq!(recovered, input);
    }
}
