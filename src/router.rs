pub mod expert;
pub mod optimizer;
pub mod compliance;
pub mod canary;
pub mod eval_routing;
use pingora::upstreams::peer::HttpPeer;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::info;

#[derive(Debug, Clone, Copy, PartialEq, serde::Deserialize)]
pub enum RoutingStrategy {
    Latency,
    Frugal,
    Resilient,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

struct BreakerState {
    state: CircuitState,
    failures: u32,
    last_failure: Option<Instant>,
}

impl BreakerState {
    fn new() -> Self {
        Self {
            state: CircuitState::Closed,
            failures: 0,
            last_failure: None,
        }
    }
}

pub struct UpstreamRouter {
    breakers: Arc<tokio::sync::RwLock<HashMap<String, BreakerState>>>,
    stats: Arc<crate::stats::StatsTracker>,
    failure_threshold: u32,
    reset_timeout: Duration,
    oracle: crate::oracle::OracleEngine,
    fallback: crate::fallback::FallbackManager,
}

impl UpstreamRouter {
    pub fn new(stats: Arc<crate::stats::StatsTracker>) -> Self {
        let mut fallback = crate::fallback::FallbackManager::new();
        
        Self {
            breakers: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            stats,
            failure_threshold: 3,
            reset_timeout: Duration::from_secs(300),
            oracle: crate::oracle::OracleEngine::new(),
            fallback,
        }
    }

    pub async fn select_best_upstream(&self, model: &str, strategy: RoutingStrategy) -> (HttpPeer, String) {
        let providers = match strategy {
            RoutingStrategy::Frugal => vec!["ollama", "deepseek", "openai"],
            RoutingStrategy::Resilient => vec!["anthropic", "openai", "azure-openai"],
            RoutingStrategy::Latency => vec!["openai", "anthropic", "deepseek"],
        };

        for provider_id in providers {
            if self.is_healthy(provider_id).await {
                return (self.create_peer(provider_id), provider_id.to_string());
            }
        }

        (self.create_peer("openai"), "openai".to_string())
    }

    pub async fn select_with_oracle(
        &self, 
        request: &crate::providers::UnifiedRequest,
        strategy: RoutingStrategy
    ) -> (HttpPeer, String, String) {
        let (intent, confidence) = self.oracle.classify_intent(request);
        let suggested_model = self.oracle.suggest_model(&intent);
        
        tracing::info!("Oracle Match: {:?} (Confidence: {:.2}) | Strategy: {:?}", intent, confidence, strategy);
        
        let (peer, provider) = self.select_best_upstream(&suggested_model, strategy).await;
        (peer, provider, suggested_model)
    }

    async fn is_healthy(&self, provider: &str) -> bool {
        let mut breakers = self.breakers.write().await;
        let breaker = breakers.entry(provider.to_string()).or_insert_with(BreakerState::new);

        match breaker.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(last) = breaker.last_failure {
                    if last.elapsed() >= self.reset_timeout {
                        breaker.state = CircuitState::HalfOpen;
                        return true; 
                    }
                }
                false
            }
            CircuitState::HalfOpen => false,
        }
    }

    pub async fn report_success(&self, provider: &str) {
        let mut breakers = self.breakers.write().await;
        if let Some(breaker) = breakers.get_mut(provider) {
            if breaker.state == CircuitState::HalfOpen {
                breaker.state = CircuitState::Closed;
                breaker.failures = 0;
            }
        }
    }

    pub async fn report_failure(&self, provider: &str) {
        let mut breakers = self.breakers.write().await;
        let breaker = breakers.entry(provider.to_string()).or_insert_with(BreakerState::new);

        breaker.failures += 1;
        breaker.last_failure = Some(Instant::now());

        if breaker.state == CircuitState::HalfOpen || breaker.failures >= self.failure_threshold {
            breaker.state = CircuitState::Open;
        }
    }

    pub fn record_traffic(&self, count: u64) {
        self.oracle.record_traffic_pulse(count);
    }

    fn create_peer(&self, provider: &str) -> HttpPeer {
        match provider {
            "openai" => HttpPeer::new("api.openai.com:443", true, "api.openai.com".to_string()),
            "anthropic" => HttpPeer::new("api.anthropic.com:443", true, "api.anthropic.com".to_string()),
            _ => HttpPeer::new("api.openai.com:443", true, "api.openai.com".to_string()), 
        }
    }

    pub fn record_request(&self, provider: &str, latency: f64) {
        self.stats.record_latency(provider, latency);
    }

    pub async fn run_health_checks(&self) {
        let providers = vec!["openai", "anthropic"];
        loop {
            for provider in &providers {
                let host = match *provider {
                    "openai" => "api.openai.com:443",
                    "anthropic" => "api.anthropic.com:443",
                    _ => "api.openai.com:443",
                };

                match tokio::net::TcpStream::connect(host).await {
                    Ok(_) => {
                        self.report_success(provider).await;
                    }
                    Err(_) => {
                        tracing::warn!("Health check failed for {}", provider);
                        self.report_failure(provider).await;
                    }
                }
            }
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    }
}
