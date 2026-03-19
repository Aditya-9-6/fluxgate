use std::collections::HashMap;
use std::sync::RwLock;
use tracing::info;

pub struct SessionManager {
    sessions: RwLock<HashMap<String, Vec<serde_json::Value>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        SessionManager {
            sessions: RwLock::new(HashMap::new()),
        }
    }

    /// Appends a new message to the stateful session and returns the full reconstructed history
    pub fn append_and_reconstruct(&self, session_id: &str, new_message: serde_json::Value) -> Vec<serde_json::Value> {
        let mut sessions = self.sessions.write().unwrap();
        let history = sessions.entry(session_id.to_string()).or_insert_with(Vec::new);
        
        history.push(new_message);
        
        info!("💾 [SESSION STORE] Reconstructed full history for session {} ({} messages).", session_id, history.len());
        history.clone()
    }

    pub fn clear_session(&self, session_id: &str) {
        let mut sessions = self.sessions.write().unwrap();
        sessions.remove(session_id);
        info!("🗑️ [SESSION STORE] Cleared session history for {}", session_id);
    }
}
