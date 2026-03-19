use tracing::{info, warn};

pub struct CanaryRouter {
    pub blue_model: String,
    pub green_model: String,
    pub green_weight: f32, // 0.0 to 1.0
}

impl CanaryRouter {
    pub fn new(blue: &str, green: &str, weight: f32) -> Self {
        Self {
            blue_model: blue.to_string(),
            green_model: green.to_string(),
            green_weight: weight,
        }
    }

    /// Determines which model to route to based on the canary weight.
    pub fn route_canary(&self) -> String {
        let rand_val: f32 = rand::random::<f32>();
        
        if rand_val < self.green_weight {
            info!("🟢 [CANARY] Routing to GREEN model (experimental): {}", self.green_model);
            self.green_model.clone()
        } else {
            info!("🔵 [CANARY] Routing to BLUE model (stable): {}", self.blue_model);
            self.blue_model.clone()
        }
    }

    /// Forces a switch to the GREEN model (Blue/Green Deployment final step).
    pub fn cutover_to_green(&mut self) {
        info!("🚀 [BLUE/GREEN] Cutting over 100% traffic to GREEN model: {}", self.green_model);
        self.green_weight = 1.0;
    }

    /// Triggers an automatic rollback if quality metrics drop below threshold.
    pub fn rollback(&mut self) {
        warn!("🚨 [CANARY] Quality drop detected! Rolling back 100% traffic to stable BLUE model.");
        self.green_weight = 0.0;
    }
}
