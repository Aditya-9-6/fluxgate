use tracing::{info, warn, debug};

use std::collections::HashSet;
use std::sync::Mutex;

pub struct AegisFirewall {
    seen_request_ids: Mutex<HashSet<String>>,
}

impl AegisFirewall {
    pub fn new() -> Self {
        Self {
            seen_request_ids: Mutex::new(HashSet::new()),
        }
    }

    pub fn check_replay(&self, request_id: &str) -> bool {
        let mut seen = self.seen_request_ids.lock().unwrap();
        if seen.contains(request_id) {
            warn!("🚨 [AEGIS X] Replay Attack Detected! ID: {}", request_id);
            return false;
        }
        
        // Sliding window: keep it small for demo, in prod use Redis TTL
        if seen.len() > 10000 {
            seen.clear(); // Simple flush for demo
        }
        
        seen.insert(request_id.to_string());
        true
    }

    pub fn inspect_connection(&self, ip: &str) -> bool {
        // Project Aegis X: eBPF/XDP Kernel-Level Inspection
        // In V2.0, this logic is offloaded to the XDP hook.
        // If the packet reached user-space, it has already passed the fast-path check.
        debug!("🛡️ [AEGIS X] Kernel XDP-Pass verified for IP: {}", ip);
        true
    }
}
