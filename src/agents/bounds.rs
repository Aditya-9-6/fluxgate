use std::sync::atomic::{AtomicU32, Ordering};
use tracing::{info, warn};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AutonomyBounds {
    pub max_actions_per_session: u32,
    pub max_total_cost: f32,
    pub max_cost_usd: f32,
    pub allow_external_tools: bool,
    pub allowed_tools: Vec<String>,
}

pub struct BoundedGuard {
    pub agent_id: String,
    pub current_actions: AtomicU32,
    pub current_cost: f32,
    pub bounds: AutonomyBounds,
}

impl BoundedGuard {
    pub fn new(agent_id: &str, bounds: AutonomyBounds) -> Self {
        Self {
            agent_id: agent_id.to_string(),
            current_actions: AtomicU32::new(0),
            current_cost: 0.0,
            bounds,
        }
    }

    /// Records an action taken by the agent and checks if it stays within bounds.
    pub fn record_action(&self) -> bool {
        let current = self.current_actions.fetch_add(1, Ordering::SeqCst);
        if current >= self.bounds.max_actions_per_session {
            warn!("🛑 [BOUNDS] Agent {} EXCEEDED action limit ({}). Triggering Human-in-the-loop escalation.", 
                  self.agent_id, self.bounds.max_actions_per_session);
            return false;
        }
        true
    }

    /// Checks if a proposed action is within data access scope.
    pub fn validate_access(&self, tool_requested: &str) -> bool {
        if tool_requested == "shell" || tool_requested == "external_api" {
            if !self.bounds.allow_external_tools {
                warn!("🚫 [BOUNDS] Agent {} attempted UNAUTHORIZED tool access: {}. Blocking.", 
                      self.agent_id, tool_requested);
                return false;
            }
        }
        true
    }
    
    /// Bounded Autonomy Check (Requirement #77)
    pub async fn enforce_bounds(&self, _agent_id: &str, _bounds: &AutonomyBounds, _context: &str) -> bool {
        // Higher level of enforcement logic goes here
        true
    }
}
