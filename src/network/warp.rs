use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, debug};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegionalPoP {
    pub region_id: String,
    pub latency_ms: u32,
    pub healthy: bool,
    pub endpoint: String,
}

pub struct WarpRouter {
    pub pops: Arc<RwLock<HashMap<String, RegionalPoP>>>,
}

impl WarpRouter {
    pub fn new() -> Self {
        let mut initial_pops = HashMap::new();
        initial_pops.insert("us-east".to_string(), RegionalPoP {
            region_id: "us-east".to_string(),
            latency_ms: 15,
            healthy: true,
            endpoint: "use-flux.warp.ai:443".to_string(),
        });
        initial_pops.insert("eu-west".to_string(), RegionalPoP {
            region_id: "eu-west".to_string(),
            latency_ms: 85,
            healthy: true,
            endpoint: "euw-flux.warp.ai:443".to_string(),
        });
        initial_pops.insert("asia-south".to_string(), RegionalPoP {
            region_id: "asia-south".to_string(),
            latency_ms: 220,
            healthy: true,
            endpoint: "as-flux.warp.ai:443".to_string(),
        });

        Self {
            pops: Arc::new(RwLock::new(initial_pops)),
        }
    }

    /// Steers the request to the optimal regional PoP based on detected user region.
    /// In a real system, this would use Cloudflare-style anycast or GeoIP steering.
    pub async fn steer_traffic(&self, user_region: &str) -> Option<RegionalPoP> {
        debug!("🌐 [WARP] Steering request for user region: {}", user_region);
        let pops = self.pops.read().await;
        
        // Return exact match if available and healthy
        if let Some(pop) = pops.get(user_region) {
            if pop.healthy {
                return Some(pop.clone());
            }
        }

        // Fallback to lowest latency healthy PoP
        pops.values()
            .filter(|p| p.healthy)
            .min_by_key(|p| p.latency_ms)
            .cloned()
    }

    /// Records a heartbeat from a regional PoP.
    pub async fn report_health(&self, region_id: &str, healthy: bool, latency: u32) {
        let mut pops = self.pops.write().await;
        if let Some(pop) = pops.get_mut(region_id) {
            pop.healthy = healthy;
            pop.latency_ms = latency;
            info!("📡 [WARP] Regional PoP {} status updated: Healthy={}, Latency={}ms", 
                  region_id, healthy, latency);
        }
    }
}
