use serde::{Serialize, Deserialize};
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Effect {
    Allow,
    Deny,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    pub id: String,
    pub principal_role: String,
    pub resource: String,
    pub min_sensitivity: u8, // 0 = Public, 10 = PII/Secret
    pub effect: Effect,
}

pub struct PolicyEngine {
    pub rules: Vec<PolicyRule>,
}

impl PolicyEngine {
    pub fn new() -> Self {
        // Default Enterprise Policies
        let rules = vec![
            PolicyRule {
                id: "rule-001".to_string(),
                principal_role: "junior_agent".to_string(),
                resource: "financial_records".to_string(),
                min_sensitivity: 8,
                effect: Effect::Deny,
            },
            PolicyRule {
                id: "rule-002".to_string(),
                principal_role: "compliance_officer".to_string(),
                resource: "*".to_string(),
                min_sensitivity: 0,
                effect: Effect::Allow,
            }
        ];
        Self { rules }
    }

    /// Evaluates the request against the deterministic policy set.
    pub fn evaluate(&self, role: &str, resource: &str, sensitivity: u8) -> Effect {
        info!("⚖️ [POLICY] Evaluating request: Role={}, Resource={}, Sensitivity={}", 
              role, resource, sensitivity);

        for rule in &self.rules {
            if (rule.principal_role == role || rule.principal_role == "*") &&
               (rule.resource == resource || rule.resource == "*") {
                
                // If the data sensitivity exceeds the rule's threshold for a deny rule
                if rule.min_sensitivity <= sensitivity && matches!(rule.effect, Effect::Deny) {
                    warn!("🚫 [POLICY] Request DENIED by rule {}: sensitivity {} exceeds limit", rule.id, sensitivity);
                    return Effect::Deny;
                }
            }
        }

        info!("✅ [POLICY] Request ALLOWED by default policy.");
        Effect::Allow
    }
}
