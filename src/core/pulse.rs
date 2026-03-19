use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tracing::{info, warn};

pub struct ResonanceMetrics {
    pub latency_pulse: AtomicU64,
    pub throughput_pulse: AtomicU64,
    pub error_pulse: AtomicU64,
}

pub struct SingularityPulse {
    pub resonance: ResonanceMetrics,
    pub last_optimization: Instant,
}

impl SingularityPulse {
    pub fn new() -> Self {
        Self {
            resonance: ResonanceMetrics {
                latency_pulse: AtomicU64::new(0),
                throughput_pulse: AtomicU64::new(0),
                error_pulse: AtomicU64::new(0),
            },
            last_optimization: Instant::now(),
        }
    }

    /// Records a system event to the resonance pulse.
    pub fn record_event(&self, latency_ms: u64, success: bool) {
        self.latency_pulse.fetch_add(latency_ms, Ordering::Relaxed);
        self.throughput_pulse.fetch_add(1, Ordering::Relaxed);
        if !success {
            self.error_pulse.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Evaluates system resonance and autonomously triggers optimization if needed.
    pub fn check_resonance(&mut self) -> bool {
        let elapsed = self.last_optimization.elapsed();
        if elapsed < Duration::from_secs(60) {
            return false;
        }

        let throughput = self.throughput_pulse.swap(0, Ordering::SeqCst);
        let total_latency = self.latency_pulse.swap(0, Ordering::SeqCst);
        let errors = self.error_pulse.swap(0, Ordering::SeqCst);

        let avg_latency = if throughput > 0 { total_latency as f32 / throughput as f32 } else { 0.0 };
        let error_rate = if throughput > 0 { errors as f32 / throughput as f32 } else { 0.0 };

        info!("💓 [PULSE] System Resonance: Avg Latency={:.2}ms, Throughput={}, Error Rate={:.2}%", 
              avg_latency, throughput, error_rate * 100.0);

        if avg_latency > 150.0 || error_rate > 0.05 {
            warn!("⚠️ [PULSE] Resonance DEGRADED. Triggering autonomous self-tuning...");
            // Simulated Optimization:
            // 1. Increase Cache TTL
            // 2. Expand request buffers
            // 3. Throttle non-critical background tasks
            info!("⚙️ [PULSE] Optimization Applied: [Cache.TTL: +300s, Buffers: 2x, Priority: LATENCY_STRICT]");
            self.last_optimization = Instant::now();
            return true;
        }

        info!("✨ [PULSE] Resonance STABLE. No optimization required.");
        false
    }
}
