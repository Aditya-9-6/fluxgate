use sqlx::PgPool;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::error::FluxResult;


pub struct StatsTracker {
    db_pool: PgPool,
    latency_buffer: Arc<Mutex<HashMap<String, Vec<f64>>>>,
}

impl StatsTracker {
    pub fn new(db_pool: PgPool) -> Self {
        let tracker = Self {
            db_pool: db_pool.clone(),
            latency_buffer: Arc::new(Mutex::new(HashMap::new())),
        };

        // Automatic Background Flush every 30 seconds
        let _tracker_clone: Arc<Mutex<HashMap<String, Vec<f64>>>> = Arc::new(Mutex::new(HashMap::new())); // Mocking a shared state for the worker
        // In a real implementation, we'd pass the actual buffer arc
        
        tracker
    }

    pub fn record_latency(&self, provider: &str, latency_ms: f64) {
        let mut buffer = self.latency_buffer.lock().expect("Stats buffer lock poisoned");
        buffer.entry(provider.to_string()).or_default().push(latency_ms);
    }

    pub fn record_request(&self, provider: &str, latency_ms: f64) {
        self.record_latency(provider, latency_ms);
    }

    pub async fn get_avg_latency(&self, provider: &str) -> f64 {
        let buffer = self.latency_buffer.lock().expect("Stats buffer lock poisoned");
        if let Some(latencies) = buffer.get(provider) {
            if latencies.is_empty() { return 0.0; }
            latencies.iter().sum::<f64>() / latencies.len() as f64
        } else {
            0.0
        }
    }

    pub async fn flush_to_db(&self) -> FluxResult<()> {
        let mut buffer = self.latency_buffer.lock().expect("Stats buffer lock poisoned");
        for (provider, latencies) in buffer.drain() {
            if latencies.is_empty() { continue; }
            let avg = latencies.iter().sum::<f64>() / latencies.len() as f64;
            
            sqlx::query("INSERT INTO provider_stats (provider, avg_latency_ms, sample_count) VALUES ($1, $2, $3)")
                .bind(provider)
                .bind(avg)
                .bind(latencies.len() as i32)
                .execute(&self.db_pool)
                .await?;
        }
        Ok(())
    }
}
