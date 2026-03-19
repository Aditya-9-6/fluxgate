use fluxgate::auth::AuthManager;
use fluxgate::cache::CacheManager;
use fluxgate::router::UpstreamRouter;
use fluxgate::billing::BillingManager;
use fluxgate::proxy::FluxProxy;
use fluxgate::config::AppConfig;
use fluxgate::audit::AuditLogger;
use fluxgate::transform::UnifiedResponseTransformer;
use fluxgate::telemetry;
use fluxgate::guardrails::Guardrails;
use fluxgate::stats::StatsTracker;
use fluxgate::ratelimit::RateLimiter;
use fluxgate::wasm_filter::WasmManager;
use fluxgate::error::{FluxError, FluxResult};
use fluxgate::security::secrets::{SecretProvider, EnvironmentSecretProvider, StaticSecretProvider};
// use fluxgate::proxy::ProxyContext; // Removed unused

use pingora::server::Server;
use pingora_proxy::http_proxy_service;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tracing::info;
use pingora::services::Service;
use pingora::server::ShutdownWatch;
use async_trait::async_trait;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize Telemetry
    telemetry::init_telemetry()?;
    info!("FluxGate Telemetry Initialized");

    // Initialize Akai Field signaling bus (Project Warp Speed)
    let akai_field = Arc::new(fluxgate::mesh::protocol::AkaiBus::new("akai_pulse_v1", 1024 * 1024)?);
    // 2. Load Configuration
    let config = AppConfig::new()?;
    info!("Configuration Loaded");

    // 3. Initialize DB pools
    let dry_run = std::env::var("FLUXGATE_DRY_RUN").is_ok();
    let db_pool = if dry_run {
        info!("⚠️ [DRY_RUN] Operating in dry-run mode. Database persistence is disabled.");
        PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy("postgres://localhost/fluxgate")?
    } else {
        PgPoolOptions::new()
            .max_connections(50)
            .min_connections(5)
            .acquire_timeout(std::time::Duration::from_secs(5))
            .connect(&config.database.url)
            .await
            .map_err(FluxError::from)?
    };

    if !dry_run {
        sqlx::migrate!("./migrations")
            .run(&db_pool)
            .await
            .map_err(|e: sqlx::migrate::MigrateError| FluxError::Internal(e.to_string()))?;
        info!("Database migrations executed successfully");
    }

    // 4. Initialize Secrets Provider
    let secret_provider: Arc<dyn SecretProvider> = if dry_run {
        info!("🔐 [SECURITY] Using StaticSecretProvider for structural verification.");
        Arc::new(StaticSecretProvider::new(vec![
            ("STRIPE_KEY", "sk_test_mock_123"),
            ("FRONTIER_KEY", "sk-frontier-mock-abc"),
            ("SOVEREIGN_SECRET", "flux_sovereign_mock_xyz"),
        ]))
    } else {
        info!("🔐 [SECURITY] Initializing EnvironmentSecretProvider...");
        Arc::new(EnvironmentSecretProvider::new("FLUX_SEC"))
    };

    let stripe_key = secret_provider.get_secret("STRIPE_KEY").await.unwrap_or_else(|_| "sk_test_fallback".to_string());
    let frontier_key = secret_provider.get_secret("FRONTIER_KEY").await.unwrap_or_else(|_| "sk-frontier-fallback".to_string());
    let sovereign_secret = secret_provider.get_secret("SOVEREIGN_SECRET").await.unwrap_or_else(|_| "flux_sovereign_fallback".to_string());

    // 5. Initialize Core Managers
    let stats = Arc::new(StatsTracker::new(db_pool.clone()));
    let energy = Arc::new(fluxgate::energy::EnergyRouter::new());
    let auth = Arc::new(AuthManager::new(db_pool.clone(), Some(&config.redis.url)));
    let cache = Arc::new(CacheManager::new(&config.redis.url, db_pool.clone(), 0.90, Arc::new(fluxgate::cache::LocalTrigramProvider))?);
    let router = Arc::new(UpstreamRouter::new(stats.clone()));
    let billing = Arc::new(BillingManager::new(db_pool.clone(), stripe_key));
    let ratelimit = Arc::new(RateLimiter::new(&config.redis.url)?);
    let guardrails = Arc::new(Guardrails::new(Some(&config.redis.url)));
    let audit = Arc::new(AuditLogger::new(db_pool.clone()));
    let webhooks = Arc::new(fluxgate::webhooks::WebhookManager::new(db_pool.clone()));

    // Redis Pre-warming (Project Warp Speed)
    if let Ok(mut conn) = redis::Client::open(config.redis.url.clone())?.get_multiplexed_async_connection().await {
        let _: String = redis::cmd("PING").query_async(&mut conn).await.unwrap_or_default();
        info!("🚀 [WARP_SPEED] Redis connection pre-warmed for core managers.");
    }
    let transformer = Arc::new(UnifiedResponseTransformer::new());
    let wasm = Arc::new(WasmManager::new()?);
    let governance = Arc::new(fluxgate::governance::GovernanceManager::new());
    let eval = Arc::new(fluxgate::eval::EvalJudge::new(db_pool.clone()));
    let refiner = Arc::new(fluxgate::refiner::PromptRefiner::new());
    let multimodal = Arc::new(fluxgate::multimodal::MultiModalShield::new());
    
    // 5. Security & Sovereignty
    let quantum = Arc::new(fluxgate::security::quantum::PostQuantumGateway::new());
    let fhe = Arc::new(fluxgate::security::fhe::FheManager::new());
    let circuit_breaker = Arc::new(fluxgate::resilience::CircuitBreaker::new(5, std::time::Duration::from_secs(30)));
    let aegis = Arc::new(fluxgate::firewall::AegisFirewall::new());
    let scrubbed_pow_redis = Arc::new(redis::Client::open(config.redis.url.clone())?);
    let pow = Arc::new(fluxgate::pow::PowChallenge::new(scrubbed_pow_redis));
    let scrubber = Arc::new(fluxgate::scrubber::ScrubberManager::new());
    let integrity = Arc::new(fluxgate::security::integrity::IntegrityEngine::new(&sovereign_secret));
    let chronolock = Arc::new(fluxgate::security::chronolock::ChronoLock::new());
    let identity = Arc::new(fluxgate::security::identity::IdentityManager::new());
    
    // 6. Agents & Autonomous Logic
    let dna_profiler = Arc::new(fluxgate::agents::dna_fingerprint::DNAProfiler::new());
    let trajectory_scorer = Arc::new(fluxgate::agents::trajectory::TrajectoryScorer::new());
    let policy_engine = Arc::new(fluxgate::policy::engine::PolicyEngine::new());
    let immune_raw = fluxgate::immune::ImmuneSystem::new(db_pool.clone(), &config.redis.url).await;
    let immune = Arc::new(tokio::sync::Mutex::new(immune_raw));
    
    // V10 TIER Agentic Core Initialization
    let memory_graph = Arc::new(fluxgate::agents::memory_graph::SharedMemoryGraph::new());
    let mcp_hub = Arc::new(fluxgate::agents::mcp::McpHub::new());
    let session_manager = Arc::new(fluxgate::agents::session::SessionManager::new());
    
    // V15 COSMIC SOVEREIGN - Next Gen Reality Engines
    let pca = Arc::new(fluxgate::policy::pca::PreCognitiveAlignment::new());
    let entropy = Arc::new(fluxgate::security::entropy::CosmicEntropyGenerator::new());
    let hologram = Arc::new(fluxgate::agentic::memory_graph::MemoryGraph::new());
    
    let ghost = Arc::new(fluxgate::ghost::GhostProtocolGateway::new(
        1024, 
        energy.clone(), 
        stats.clone(),
        mcp_hub.clone(),
        session_manager.clone(),
        memory_graph.clone(),
        frontier_key,
        scrubber.clone(),
        accountability.clone(),
    ));
    
    // 7. Advanced Protocols & Intelligence
    let icarus = Arc::new(fluxgate::icarus::IcarusDecorder::new(Arc::new(fluxgate::dharma::IntentOracle::new())));
    let evolution = Arc::new(fluxgate::evolution::EvolutionEngine::new(100));
    let aether = Arc::new(fluxgate::aether::AetherEngine::new());
    let chimera = Arc::new(fluxgate::chimera::ChimeraEngine::new());
    let federated_learner = Arc::new(fluxgate::intelligence::federated::FederatedLearner::new("default-tenant"));
    let hallucination = Arc::new(fluxgate::intelligence::hallucination::HallucinationInterceptor::new());
    let interpretability = Arc::new(fluxgate::intelligence::interpretability::InterpretabilityEngine::new());
    let deception_detector = Arc::new(fluxgate::intelligence::deception_detector::DeceptionDetector::new());
    let alignment_monitor = Arc::new(fluxgate::intelligence::alignment::AlignmentMonitor::new(0.5));
    let compliance_autopilot = Arc::new(fluxgate::router::compliance::ComplianceAutopilot::new());
    let healer = Arc::new(fluxgate::genesis::healer::AutoHealer::new());
    let bulkheads = Arc::new(dashmap::DashMap::new());
    let request_queue = Arc::new(fluxgate::resilience::RequestQueue::new(5000));
    let accountability = Arc::new(fluxgate::intelligence::accountability::CausalAccountability::new());
    let containment = Arc::new(fluxgate::security::containment::EvolutionContainment::new(&sovereign_secret));
    let zkp = Arc::new(fluxgate::security::zkp::VerifiableCompute::new());
    
    // 8. Performance & Mesh
    let stream_stitcher = Arc::new(fluxgate::transform::stream_stitcher::StreamStitcher::new());
    let grid = Arc::new(fluxgate::grid::GridManager::new());
    let kernel = Arc::new(fluxgate::kernel::NeuralKernel::new());
    let distiller = Arc::new(fluxgate::transform::distiller::PromptDistiller::new());
    let impersonation = Arc::new(fluxgate::security::impersonation::ImpersonationGuard::new(dna_profiler.clone()));
    let exfiltration = Arc::new(fluxgate::security::exfiltration::ExfiltrationMonitor::new());
    let extraction = Arc::new(fluxgate::security::extraction_protection::ExtractionGuard::new());
    let mesh_protocol = Arc::new(fluxgate::mesh::protocol::MeshProtocol::new());
    let self_distiller = Arc::new(fluxgate::transform::distiller_v2::SelfDistillerV2::new());
    
    // V13: Start QKD Mesh Link Rotation
    let mesh_cloned = mesh_protocol.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            mesh_cloned.rotate_qkd_keys().await;
        }
    });

    let sovereignty_layer = Arc::new(fluxgate::sovereignty::layer::SovereigntyLayer::new());
    let cost_optimizer = Arc::new(fluxgate::cost_optimizer::CostOptimizer::new());
    let chronos = Arc::new(fluxgate::chronos::ChronosEngine::new());
    let ledger = Arc::new(fluxgate::ledger::FluxLedger::new(db_pool.clone()));

    // 9. Startup Services
    let mut my_server = Server::new(None)?;
    my_server.bootstrap();

    let flux_proxy = FluxProxy {
        auth: auth.clone(),
        cache: cache.clone(),
        router: router.clone(),
        billing: billing.clone(),
        ratelimit: ratelimit.clone(),
        guardrails: guardrails.clone(),
        stats: stats.clone(),
        audit: audit.clone(),
        wasm: wasm.clone(),
        transformer: transformer.clone(),
        governance: governance.clone(),
        eval: eval.clone(),
        refiner: refiner.clone(),
        multimodal: multimodal.clone(),
        enclave: Arc::new(fluxgate::enclave::TitanEnclave::new()),
        grid: grid.clone(),
        kernel: kernel.clone(),
        quantum: quantum.clone(),
        fhe: fhe.clone(),
        circuit_breaker: circuit_breaker.clone(),
        aegis: aegis.clone(),
        pow: pow.clone(),
        scrubber: scrubber.clone(),
        ledger: ledger.clone(),
        healer: healer.clone(),
        synthesis: Arc::new(fluxgate::genesis::synthesis::SyntheticEngine::new()),
        graft: Arc::new(fluxgate::genesis::graft::ModelGraft::new()),
        joker: Arc::new(fluxgate::chaos::TheJoker::new()),
        intent_verifier: Arc::new(fluxgate::dharma::IntentVerifier::new()),
        dharma: Arc::new(fluxgate::dharma::IntentOracle::new()),
        akai_field: akai_field.clone(),
        mesh_protocol: mesh_protocol.clone(),
        sovereignty_layer: sovereignty_layer.clone(),
        cost_optimizer: cost_optimizer.clone(),
        chronos: chronos.clone(),
        icarus: icarus.clone(),
        evolution: evolution.clone(),
        aether: aether.clone(),
        chimera: chimera.clone(),
        traffic: Arc::new(fluxgate::traffic::AgentTrafficController::new(&config.redis.url).map_err(|e| {
            tracing::error!("Traffic Controller Init Failed: {}", e);
            e
        })?),
        immune: immune.clone(),
        entropy: Arc::new(fluxgate::entropy::NonDeterministicGuardrail::new(0.7)),
        reversible: Arc::new(fluxgate::reversible::ReversibleLogicGateway::new()),
        energy: energy.clone(),
        ghost: ghost.clone(),
        expert_router: Arc::new(fluxgate::router::expert::ExpertRouter::new()),
        prompt_compressor: Arc::new(fluxgate::transform::compressor::PromptCompressor::new()),
        enclave_v2: Arc::new(fluxgate::enclave_v2::TitanEnclaveV2::new()),
        integrity: integrity.clone(),
        warp_router: Arc::new(fluxgate::network::warp::WarpRouter::new()),
        federated_learner: federated_learner.clone(),
        cerebro: Arc::new(fluxgate::cognitive::cerebro::CerebroEngine::new()),
        pulse: Arc::new(tokio::sync::Mutex::new(fluxgate::v6_core::pulse::SingularityPulse::new())),
        chronolock: chronolock.clone(),
        holographic: Arc::new(fluxgate::semantic::holographic::HolographicFusion::new()),
        dna_profiler: dna_profiler.clone(),
        trajectory_scorer: trajectory_scorer.clone(),
        policy_engine: policy_engine.clone(),
        memory_graph: memory_graph.clone(),
        hallucination: hallucination.clone(),
        interpretability: interpretability.clone(),
        deception_detector: deception_detector.clone(),
        alignment_monitor: alignment_monitor.clone(),
        identity_manager: identity.clone(),
        stream_stitcher: stream_stitcher.clone(),
        optimizer: Arc::new(fluxgate::router::optimizer::ModelOptimizer::new()),
        finops: Arc::new(fluxgate::billing::finops::FinOpsEngine::new()),
        mutation_engine: Arc::new(fluxgate::transform::mutation::MutationEngine::new()),
        compliance_router: Arc::new(fluxgate::router::compliance::ComplianceRouter::new()),
        bulkheads: bulkheads.clone(),
        bounds: Arc::new(fluxgate::agents::bounds::BoundedGuard::new("global_guard", fluxgate::agents::bounds::AutonomyBounds {
            max_actions_per_session: 100,
            max_total_cost: 10.0,
            max_cost_usd: 1.0,
            allow_external_tools: true,
            allowed_tools: vec!["search".to_string(), "read".to_string()],
        })),
        prewarm: Arc::new(tokio::sync::Mutex::new(fluxgate::network::prewarm::PrewarmEngine::new())),
        compliance_registry: Arc::new(fluxgate::compliance::packs::ComplianceRegistry::new()),
        zerotrust: Arc::new(fluxgate::security::zerotrust::ZeroTrustManager::new(zkp.clone())),
        canary_router: Arc::new(fluxgate::router::canary::CanaryRouter::new("blue", "green", 0.5)),
        eval_router: Arc::new(fluxgate::router::eval_routing::EvalRouter::new()),
        a2a_handler: Arc::new(fluxgate::protocol::a2a::A2AHandler::new()),
        gitops_loader: Arc::new(fluxgate::policy::gitops::GitOpsLoader::new()),
        marketplace: Arc::new(tokio::sync::RwLock::new(fluxgate::ecosystem::marketplace::MarketplaceManager::new(wasm.clone()))),
        intelligence_network: Arc::new(tokio::sync::RwLock::new(fluxgate::ecosystem::intelligence::IntelligenceNetwork::new())),
        governance_agent: Arc::new(fluxgate::ecosystem::governance_agent::GovernanceAgent::new(accountability.clone())),
        compliance_autopilot: Arc::new(fluxgate::compliance::autopilot::ComplianceAutopilot::new()),
        loop_protector: Arc::new(fluxgate::agents::loop_protection::LoopProtector::new()),
    };

    let mut flux_service = http_proxy_service(&my_server.configuration, flux_proxy);
    let addr = format!("0.0.0.0:{}", config.server.port);
    info!("Starting FluxGate Proxy on {}", addr);
    flux_service.add_tcp(&addr);
    my_server.add_service(flux_service);

    // 10. Start API Service
    let api_state = Arc::new(fluxgate::api::ApiState {
        auth: auth.clone(),
        ratelimit: ratelimit.clone(),
        ledger: ledger.clone(),
        ghost: ghost.clone(),
        cache: cache.clone(),
        stats: stats.clone(),
        immune: immune.clone(),
        dna_profiler: dna_profiler.clone(),
        trajectory_scorer: trajectory_scorer.clone(),
        interpretability: interpretability.clone(),
        quantum: quantum.clone(),
        audit: audit.clone(),
        webhooks: webhooks.clone(),
        fhe: fhe.clone(),
        circuit_breaker: circuit_breaker.clone(),
        alignment_monitor: alignment_monitor.clone(),
        compliance_autopilot: compliance_autopilot.clone(),
        accountability: accountability.clone(),
        containment: containment.clone(),
        zkp: zkp.clone(),
        intent_verifier: Arc::new(fluxgate::dharma::IntentVerifier::new()),
        bulkheads: bulkheads.clone(),
        request_queue: request_queue.clone(),
        healer: healer.clone(),
        distiller: distiller.clone(),
        impersonation: impersonation.clone(),
        exfiltration: exfiltration.clone(),
        extraction: extraction.clone(),
        lockdown_protector: loop_protector.clone(),
        governance_agent: governance_agent.clone(),
        gitops_loader: gitops_loader.clone(),
        marketplace: marketplace.clone(),
        self_distiller: self_distiller.clone(),
        pca: pca.clone(),
        entropy: entropy.clone(),
        hologram: hologram.clone(),
    });

    let api_router = fluxgate::api::create_api_router(api_state);
    let api_service = AxumService {
        router: api_router,
        addr: "0.0.0.0:8081".to_string(),
    };
    my_server.add_service(api_service);

    info!("Services added to Pingora. Now entering run_forever.");
    my_server.run_forever();
    // info!("Pingora run_forever exited!");
}

pub struct AxumService {
    router: axum::Router,
    addr: String,
}

#[async_trait]
impl Service for AxumService {
    async fn start_service(
        &mut self,
        mut _shutdown: ShutdownWatch,
        _listeners_per_fd: usize,
    ) {
        let router = self.router.clone();
        let addr = self.addr.clone();
        
        info!("🚀 [API_SERVER] Starting Axum API on {}", addr);
        info!("🚀 [V15-COSMIC] FluxGate Transcendence Complete. 2040 Readiness Tier ACTIVE.");
    
        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .expect("Failed to bind API port");
        if let Err(e) = axum::serve(listener, router).await {
            tracing::error!("API Server failed: {}", e);
        }
    }
    fn name(&self) -> &str {
        "Axum API Service"
    }
}
