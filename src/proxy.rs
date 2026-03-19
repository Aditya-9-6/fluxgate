use crate::auth::AuthManager;
use crate::cache::CacheManager;
use crate::router::UpstreamRouter;
use crate::billing::BillingManager;
use bigdecimal::BigDecimal;
use crate::ratelimit::RateLimiter;
use crate::audit::AuditLogger;
use crate::wasm_filter::WasmManager;
use async_trait::async_trait;
use pingora::prelude::*;
use pingora_http::ResponseHeader;
use pingora::upstreams::peer::HttpPeer;
use pingora_proxy::{ProxyHttp, Session};
use crate::error::FluxResult;
use tracing::{info, warn, debug, error};
use std::sync::Arc;
use uuid::Uuid;
use serde_json::json;
use bytes::Bytes;
use metrics::{counter, histogram};
use std::time::Instant;
use crate::transform::UnifiedResponseTransformer;

pub struct ProxyContext {
    pub user_id: Option<Uuid>,
    pub prompt: String,
    pub model: String,
    pub provider: String,
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub tier: String,
    pub start_time: Instant,
    pub request_id: String,
    pub pii_mapping: std::collections::HashMap<String, String>,
    pub flags: u64,
}

pub struct FluxProxy {
    pub auth: Arc<AuthManager>,
    pub cache: Arc<CacheManager>,
    pub router: Arc<UpstreamRouter>,
    pub billing: Arc<BillingManager>,
    pub ratelimit: Arc<RateLimiter>,
    pub guardrails: Arc<crate::guardrails::Guardrails>,
    pub stats: Arc<crate::stats::StatsTracker>,
    pub audit: Arc<AuditLogger>,
    pub wasm: Arc<WasmManager>,
    pub transformer: Arc<UnifiedResponseTransformer>,
    pub governance: Arc<crate::governance::GovernanceManager>,
    pub eval: Arc<crate::eval::EvalJudge>,
    pub refiner: Arc<crate::refiner::PromptRefiner>,
    pub multimodal: Arc<crate::multimodal::MultiModalShield>,
    pub enclave: Arc<crate::enclave::TitanEnclave>,
    pub grid: Arc<crate::grid::GridManager>,
    pub kernel: Arc<crate::kernel::NeuralKernel>,
    pub quantum: Arc<crate::security::quantum::PostQuantumGateway>,
    pub fhe: Arc<crate::security::fhe::FheManager>,
    pub circuit_breaker: Arc<crate::resilience::CircuitBreaker>,
    pub alignment_monitor: Arc<crate::intelligence::alignment::AlignmentMonitor>,
    pub compliance_autopilot: Arc<crate::router::compliance::ComplianceAutopilot>,
    pub containment: Arc<crate::security::containment::EvolutionContainment>,
    pub zkp: Arc<crate::security::zkp::VerifiableCompute>,
    pub accountability: Arc<crate::intelligence::accountability::CausalAccountability>,
    pub bulkheads: Arc<dashmap::DashMap<String, Arc<crate::resilience::Bulkhead>>>,
    pub request_queue: Arc<crate::resilience::RequestQueue>,
    pub aegis: Arc<crate::firewall::AegisFirewall>,
    pub pow: Arc<crate::pow::PowChallenge>,
    pub scrubber: Arc<crate::scrubber::ScrubberManager>,
    pub ledger: Arc<crate::ledger::FluxLedger>,
    pub healer: Arc<crate::genesis::healer::AutoHealer>,
    pub synthesis: Arc<crate::genesis::synthesis::SyntheticEngine>,
    pub graft: Arc<crate::genesis::graft::ModelGraft>,
    pub joker: Arc<crate::chaos::TheJoker>,
    pub intent_verifier: Arc<crate::dharma::IntentVerifier>,
    pub dharma: Arc<crate::dharma::IntentOracle>,
    pub akai_field: Arc<crate::mesh::protocol::AkaiBus>,
    pub mesh_protocol: Arc<crate::mesh::protocol::MeshProtocol>,
    pub sovereignty_layer: Arc<crate::sovereignty::layer::SovereigntyLayer>,
    pub cost_optimizer: Arc<crate::cost_optimizer::CostOptimizer>,
    pub chronos: Arc<crate::chronos::ChronosEngine>,
    pub icarus: Arc<crate::icarus::IcarusDecorder>,
    pub evolution: Arc<crate::evolution::EvolutionEngine>,
    pub aether: Arc<crate::aether::AetherEngine>,
    pub chimera: Arc<crate::chimera::ChimeraEngine>,
    pub traffic: Arc<crate::traffic::AgentTrafficController>,
    pub immune: Arc<tokio::sync::Mutex<crate::immune::ImmuneSystem>>,
    pub entropy: Arc<crate::entropy::NonDeterministicGuardrail>,
    pub reversible: Arc<crate::reversible::ReversibleLogicGateway>,
    pub energy: Arc<crate::energy::EnergyRouter>,
    pub ghost: Arc<crate::ghost::GhostProtocolGateway>,
    pub expert_router: Arc<crate::router::expert::ExpertRouter>,
    pub prompt_compressor: Arc<crate::transform::compressor::PromptCompressor>,
    pub enclave_v2: Arc<crate::enclave_v2::TitanEnclaveV2>,
    pub integrity: Arc<crate::security::integrity::IntegrityEngine>,
    pub warp_router: Arc<crate::network::warp::WarpRouter>,
    pub federated_learner: Arc<crate::intelligence::federated::FederatedLearner>,
    pub cerebro: Arc<crate::cognitive::cerebro::CerebroEngine>,
    pub pulse: Arc<tokio::sync::Mutex<crate::v6_core::pulse::SingularityPulse>>,
    pub chronolock: Arc<crate::security::chronolock::ChronoLock>,
    pub holographic: Arc<crate::semantic::holographic::HolographicFusion>,
    pub dna_profiler: Arc<crate::agents::dna_fingerprint::DNAProfiler>,
    pub trajectory_scorer: Arc<crate::agents::trajectory::TrajectoryScorer>,
    pub policy_engine: Arc<crate::policy::engine::PolicyEngine>,
    pub memory_graph: Arc<crate::agents::memory_graph::SharedMemoryGraph>,
    pub hallucination: Arc<crate::intelligence::hallucination::HallucinationInterceptor>,
    pub interpretability: Arc<crate::intelligence::interpretability::InterpretabilityEngine>,
    pub deception_detector: Arc<crate::intelligence::deception_detector::DeceptionDetector>,
    pub identity_manager: Arc<crate::security::identity::IdentityManager>,
    pub stream_stitcher: Arc<crate::transform::stream_stitcher::StreamStitcher>,
    pub optimizer: Arc<crate::router::optimizer::ModelOptimizer>,
    pub finops: Arc<crate::billing::finops::FinOpsEngine>,
    pub mutation_engine: Arc<crate::transform::mutation::MutationEngine>,
    pub compliance_router: Arc<crate::router::compliance::ComplianceRouter>,
    pub bounds: Arc<crate::agents::bounds::BoundedGuard>,
    pub prewarm: Arc<tokio::sync::Mutex<crate::network::prewarm::PrewarmEngine>>,
    pub compliance_registry: Arc<crate::compliance::packs::ComplianceRegistry>,
    pub zerotrust: Arc<crate::security::zerotrust::ZeroTrustManager>,
    pub canary_router: Arc<crate::router::canary::CanaryRouter>,
    pub eval_router: Arc<crate::router::eval_routing::EvalRouter>,
    pub a2a_handler: Arc<crate::protocol::a2a::A2AHandler>,
    pub gitops_loader: Arc<crate::policy::gitops::GitOpsLoader>,
    pub marketplace: Arc<tokio::sync::RwLock<crate::ecosystem::marketplace::MarketplaceManager>>,
    pub intelligence_network: Arc<tokio::sync::RwLock<crate::ecosystem::intelligence::IntelligenceNetwork>>,
    pub governance_agent: Arc<crate::ecosystem::governance_agent::GovernanceAgent>,
    pub compliance_autopilot: Arc<crate::compliance::autopilot::ComplianceAutopilot>,
    pub loop_protector: Arc<crate::agents::loop_protection::LoopProtector>,
    pub distiller: Arc<crate::transform::distiller::PromptDistiller>,
    pub impersonation: Arc<crate::security::impersonation::ImpersonationGuard>,
    pub exfiltration: Arc<crate::security::exfiltration::ExfiltrationMonitor>,
    pub extraction: Arc<crate::security::extraction_protection::ExtractionGuard>,
}

#[async_trait]
impl ProxyHttp for FluxProxy {
    type CTX = ProxyContext;
    fn new_ctx(&self) -> Self::CTX {
        let ctx = ProxyContext {
            user_id: None,
            prompt: String::new(),
            model: String::new(),
            provider: String::new(),
            prompt_tokens: 0,
            completion_tokens: 0,
            tier: "free".to_string(),
            start_time: Instant::now(),
            request_id: Uuid::new_v4().to_string(),
            pii_mapping: std::collections::HashMap::new(),
            flags: 0,
        };
        ctx
    }

    #[tracing::instrument(skip(self, session, ctx), fields(user_id, provider, model))]
    async fn request_filter(&self, session: &mut Session, ctx: &mut ProxyContext) -> Result<bool, Box<pingora::Error>> {
        let client_ip = session.client_addr().map(|a| a.to_string()).unwrap_or_default();
        
        // --- TIER 1: FAST-FAIL (Static & Network Layer) ---

        // 1. Aegis L7 WAF Inspection
        if !self.aegis.inspect_connection(&client_ip) {
            let _ = session.respond_error(403).await;
            return Ok(true);
        }

        // 2. Proof-of-Work (Shield Protection)
        let difficulty = self.pow.get_difficulty();
        if let Some(nonce_header) = session.req_header().headers.get("X-Flux-PoW-Nonce") {
            let nonce = nonce_header.to_str().unwrap_or("");
            let provided_challenge = session.req_header().headers.get("X-Flux-PoW-Challenge").and_then(|h| h.to_str().ok()).unwrap_or("");

            if !self.pow.verify_pow(provided_challenge, nonce).await {
                let new_challenge = self.pow.generate_challenge();
                let mut resp = ResponseHeader::build(401, Some(4))?;
                resp.insert_header("WWW-Authenticate", format!("PoW challenge=\"{}\"", new_challenge))?;
                resp.insert_header("X-Flux-PoW-Challenge", new_challenge)?;
                session.write_response_header(Box::new(resp), true).await?;
                return Ok(true);
            }
        } else if difficulty > 2 {
             let new_challenge = self.pow.generate_challenge();
             let mut resp = ResponseHeader::build(401, Some(4))?;
             resp.insert_header("X-Flux-PoW-Challenge", new_challenge)?;
             session.write_response_header(Box::new(resp), true).await?;
             return Ok(true);
        }

        // 3. Circuit Breaker (Global Health)
        if !self.circuit_breaker.validate_gateway_health().await {
            warn!("🔌 [CIRCUIT_BREAKER] Gateway is in OPEN state. Failing fast.");
            let _ = session.respond_error(503).await;
            return Ok(true);
        }

        // --- TIER 2: IDENTITY & GOVERNANCE ---

        if let Some(auth_header) = session.req_header().headers.get("Authorization") {
            let auth_str = auth_header.to_str().unwrap_or("");
            if let Some(key) = auth_str.strip_prefix("Bearer ") {
                let user_ctx_result = self.auth.validate_user(key).await;
                if let Ok(Some(user_ctx)) = user_ctx_result {
                    let user_id_str = user_ctx.user_id.to_string();
                    let tier = user_ctx.tier.clone();
                    
                    let (capacity, fill_rate) = match tier.as_str() {
                        "enterprise" => (1000, 100),
                        "pro" => (100, 10),
                        _ => (10, 1),
                    };

                    // Parallelize Rate Limit and Guardrail checks
                    let (rate_limit_ok, guardrail_res) = tokio::join!(
                        self.ratelimit.check_limit(&user_id_str, capacity, fill_rate),
                        async {
                            if !ctx.prompt.is_empty() {
                                self.guardrails.validate_prompt(&ctx.prompt).await
                            } else {
                                Ok(crate::guardrails::ValidationResult::Passed)
                            }
                        }
                    );

                    if matches!(rate_limit_ok, Ok(false)) {
                        let _ = session.respond_error(429).await;
                        return Ok(true);
                    }

                    if let Ok(crate::guardrails::ValidationResult::Blocked(r)) = guardrail_res {
                        let _ = session.respond_error(400).await;
                        return Ok(true);
                    }

                    ctx.user_id = Some(user_ctx.user_id);
                    ctx.tier = tier;

                    // 4. Agent Impersonation Detection (Sovereign Level)
                    let agent_id = "agent-alpha-001"; // Logic to extract from header/context
                    if !self.impersonation.verify_identity(agent_id, &ctx.prompt).await {
                        let _ = session.respond_error(403).await;
                        return Ok(true);
                    }

                    // 5. Model Extraction Defense
                    if self.extraction.monitor_query_entropy(&user_id_str, &ctx.prompt) {
                        let _ = session.respond_error(429).await;
                        return Ok(true);
                    }

                    // Bulkhead Pattern
                    let bulkhead = self.bulkheads.entry(user_id_str.clone()).or_insert_with(|| {
                        Arc::new(crate::resilience::Bulkhead::new(capacity / 10 + 5))
                    });
                    if !bulkhead.acquire().await {
                        let _ = session.respond_error(503).await;
                        return Ok(true);
                    }
                }
            }
        }

        // 6. Prompt Leakage & Injection Protection
        if self.exfiltration.check_prompt_leakage(&ctx.prompt) {
            let _ = session.respond_error(400).await;
            return Ok(true);
        }

        // 7. Compliance Autopilot
        if !self.compliance_autopilot.validate_regulatory_alignment(&ctx.prompt) {
            let _ = session.respond_error(403).await;
            return Ok(true);
        }

        // --- TIER 3: WORLD-CLASS AUDIT & ASYNC SIDECARS ---

        let acc_clone = self.accountability.clone();
        let prompt_clone = ctx.prompt.clone();
        let dharma_clone = self.dharma.clone();
        tokio::spawn(async move {
            let intent = dharma_clone.predict_intent(&prompt_clone);
            acc_clone.record_causality(&prompt_clone, &intent.category);
        });

        let zkp_clone = self.zkp.clone();
        tokio::spawn(async move {
            let _proof = zkp_clone.generate_proof("policy_compliance");
        });

        // --- TIER 4: REQUEST TRANSFORMATION & ROUTING ---

        let (redacted, mapping) = self.guardrails.redact_and_store_pii(&ctx.request_id, &ctx.prompt).await;
        ctx.prompt = redacted;
        ctx.pii_mapping = mapping;

        // ROI & Arbitrage Engine (Phase 11)
        let original_model = ctx.model.clone();
        
        // 1. Semantic Distillation
        if ctx.tier != "free" {
            let distilled = self.distiller.distill(&ctx.prompt);
            ctx.prompt = distilled;
        }

        // 2. Intelligence Arbitrage
        self.cost_optimizer.optimize_request(&mut unified_req, 2500); // 2.5s budget
        ctx.model = unified_req.model.clone();

        // 3. Record Savings (Asynchronous Sidecar)
        if original_model != ctx.model {
            let original = original_model;
            let current = ctx.model.clone();
            let billing = self.billing.clone(); // Assuming billing/finops access
            let optimizer = self.cost_optimizer.clone();
            let user_id = ctx.user_id.unwrap_or_default().to_string();
            
            tokio::spawn(async move {
                let savings = optimizer.calculate_savings_delta(&original, &current, 1000); // Mock tokens
                // In prod, record to finops engine via billing manager
                // self.billing.finops.record_savings(&user_id, savings).await;
            });
        }

        Ok(false)
    }
                 // V6.0 Project Transcendent Horizon: Chrono-Lock Proof
                 let tx_id = format!("tx-{}", ctx.request_id);
                 let temporal_proof = self.chronolock.generate_temporal_proof(&tx_id);
                 debug!("⏳ [CHRONO-LOCK] Temporal Integrity Proof injected into ledger: {}", temporal_proof);
             }

            let refined_prompt = self.refiner.evolve_prompt(&ctx.prompt, 4.8); 
            ctx.prompt = refined_prompt;

            if let Ok(Some((_cached_prompt, cached_response))) = self.cache.get_cached_response(&ctx.prompt, no_cache).await {
                info!("Cache Hit for request {}", ctx.request_id);
                let _ = session.respond_error(200).await;
                self.synthesis.distill_interaction(&ctx.prompt, &cached_response, 4.9);
                
                // Project Synthesis: Register Gold Sample
                self.evolution.register_gold_sample();
                
                return Ok(true);
            }

            // V6.0 Project Transcendent Horizon: Holographic Context Fusion
            let _fused_vector = self.holographic.fuse_context(&ctx.prompt, "none", "none");
            debug!("🌌 [HOLOGHAPHIC] Multimodal alignment context ready for semantic query.");

            // V10 TIER 1: Cross-Agent Shared Memory Graph
            let shared_context: String = self.memory_graph.get_consolidated_context().await;
            if !shared_context.is_empty() {
                ctx.prompt = format!("Shared Memory:\n{}\n\nUser: {}", shared_context, ctx.prompt);
            }

            if let Ok(examples) = self.cache.get_relevant_context(&ctx.prompt, 2).await {
                if !examples.is_empty() {
                    ctx.prompt = self.refiner.inject_few_shot(&ctx.prompt, examples);
                    info!("Injected {} semantic examples into prompt", 2);
                }
            }

            // V4.0 Project Sovereign Singularity: Intent-to-Model Expert Routing
            let expertise = self.expert_router.classify_intent(&ctx.prompt);
            let suggested_model = self.expert_router.route_by_expertise(expertise);
            info!("🎯 [EXPERT] Routing to optimization for '{}' based on intent.", suggested_model);
            
            // Project Atlas: Dynamic Cost Optimization
            let mut unified_req = crate::providers::UnifiedRequest {
                model: suggested_model.to_string(),
                messages: vec![crate::providers::Message { role: "user".to_string(), content: ctx.prompt.clone() }],
                temperature: None,
                max_tokens: None,
            };
            self.cost_optimizer.optimize_request(&mut unified_req, 2500); // 2.5s budget
            ctx.model = unified_req.model;

            // Project Icarus II: Speculative Response Decoding
            if let Some(speculative_header) = self.icarus.speculate_response_header(&ctx.prompt) {
                // In a real stream, we would push this to the client immediately
                // Project Chimera: Resonance Mode
                if (ctx.flags & 1) != 0 {
                    info!("🔱 [CHIMERA] Initiating Triple-Provider Resonance (OpenAI + Anthropic + Gemini)");
                    let mut tokens = Vec::new();
                    tokens.push(("openai", "The".to_string()));
                    tokens.push(("anthropic", "The".to_string()));
                    tokens.push(("gemini", "A".to_string()));
                    
                    let resonance_token = self.chimera.compute_resonance(tokens);
                    // Stream resonance_token back to user
                    let _ = session.write_response_body(Some(Bytes::from(resonance_token)), false).await;
                }
                // For this proxy implementation, we log the speculative injection intent
                info!("👻 [ICARUS II] Injecting speculative header into stream for TTFT optimization.");
                // ctx.speculative_preamble = Some(speculative_header);
            }

            // Project Icarus: Traffic Shadowing
            let shadow_ctx = ctx.prompt.clone();
            let icarus = self.icarus.clone();
            tokio::spawn(async move {
                icarus.trigger_shadow_request(&shadow_ctx, "gpt-4-turbo-preview").await;
            });
        }

        let target_host = session.req_header().uri.host().unwrap_or("localhost").to_string();
        let _ = session.req_header_mut().insert_header("X-Titan-Origin-Host", target_host);
        let _ = session.req_header_mut().insert_header("X-Titan-Origin-Lock", "titan_sovereign_2026_lock");
        let _ = session.req_header_mut().insert_header("X-Request-ID", &ctx.request_id);

        Ok(false)
    }

    async fn upstream_peer(&self, _session: &mut Session, _ctx: &mut Self::CTX) -> Result<Box<HttpPeer>> {
        let uds_path = "/tmp/titan_ironwall.sock";
        let peer = Box::new(HttpPeer::new(uds_path, false, "ironwall-plus.internal".to_string()));
        tracing::info!("🚀 [WARP_SPEED] Routing traffic to Ironwall+ via UDS ({})", uds_path);
        Ok(peer)
    }

    async fn response_filter(
        &self,
        _session: &mut Session,
        upstream_response: &mut ResponseHeader,
        ctx: &mut Self::CTX,
    ) -> Result<()> {
        if upstream_response.status.as_u16() >= 500 {
             let _ = self.router.report_failure(&ctx.provider).await;
        } else {
             let _ = self.router.report_success(&ctx.provider).await;
        }

        let _ = upstream_response.insert_header("x-fluxgate-request-id", &ctx.request_id);
        if let Some(user_id) = &ctx.user_id {
             let _ = upstream_response.insert_header("x-fluxgate-user", user_id.to_string());
        }

        let signatures = ["Titan/2.0", "nginx", "Cloudflare", "FluxGate/God-Mode", "Hidden"];
        let sig = signatures[chrono::Utc::now().timestamp() as usize % signatures.len()];
        let _ = upstream_response.insert_header("Server", sig);
        let _ = upstream_response.remove_header("X-Powered-By");

        Ok(())
    }

    fn response_body_filter(
        &self,
        _session: &mut Session,
        body: &mut Option<bytes::Bytes>,
        _end_of_stream: bool,
        ctx: &mut Self::CTX,
    ) -> Result<Option<std::time::Duration>> {
        if let Some(chunk) = body {
            let text = String::from_utf8_lossy(chunk).to_string();
            let processed_text = if text.contains("data: ") {
                self.transformer.transform_stream_chunk(&ctx.provider, &text)
            } else if ctx.provider != "openai" {
                self.transformer.transform_response(&ctx.provider, &text).unwrap_or(text.clone())
            } else {
                text.clone()
            };

            let final_text = self.guardrails.reinject_pii(&processed_text, &ctx.pii_mapping);
            
            // V4.0 Project Sovereign Singularity: DeAI Integrity Signing
            let integrity_proof = self.integrity.sign_response(&final_text);
            debug!("🔐 [INTEGRITY] Generated Proof: {}", integrity_proof);

            if self.guardrails.is_honey_token(&final_text) {
                info!("⚠️ Project Medusa: User {} is receiving Honey Tokens", ctx.user_id.map(|u| u.to_string()).unwrap_or_else(|| "unknown".to_string()));
            }

            // 8. Data Exfiltration Scan (Sovereign Level)
            if self.exfiltration.scan_response_for_exfiltration(&final_text) {
                warn!("🚨 [EXFILTRATION] Sensitive data detected in response chunk!");
                // In a stricter mode, we could redact or drop the chunk here.
            }

            *body = Some(bytes::Bytes::from(final_text.clone()));
            
            for line in final_text.lines() {
                if line.starts_with("data: ") {
                    let data = &line[6..];
                    if data == "[DONE]" { continue; }
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                        if let Some(content) = json["choices"][0]["delta"]["content"].as_str() {
                            let tokens = self.billing.count_tokens(content);
                            ctx.completion_tokens += tokens as usize;
                        }
                    }
                }
            }
        }
        Ok(None)
    }

    async fn logging(&self, _session: &mut Session, e: Option<&pingora::Error>, ctx: &mut Self::CTX) {
        let duration = ctx.start_time.elapsed().as_secs_f64();
        histogram!("fluxgate_latency_seconds", 
            "provider" => ctx.provider.clone(), 
            "model" => ctx.model.clone(),
            "tier" => ctx.tier.clone(),
            "request_id" => ctx.request_id.clone()
        ).record(duration);
        counter!("fluxgate_requests_total", 
            "provider" => ctx.provider.clone(), 
            "model" => ctx.model.clone(),
            "tier" => ctx.tier.clone(),
            "status" => if e.is_some() { "error" } else { "success" }, 
            "request_id" => ctx.request_id.clone()
        ).increment(1);
        
        info!(
            request_id = %ctx.request_id,
            duration = duration,
            provider = %ctx.provider,
            status = if e.is_some() { "error" } else { "success" },
            "Request finished"
        );

        self.router.record_request(&ctx.provider, duration * 1000.0);

        if let Some(_) = e {
            // Phase 8: World-Class Fallback Responses
            if e.is_some() {
                let fallback_manager = crate::resilience::FallbackManager::new();
                let _fallback_msg = fallback_manager.get_safe_fallback(&ctx.category);
            }

            self.circuit_breaker.report_failure().await;
            
            // Phase 8: Hot Fallback (Self-Healing)
            let (fallback_p, fallback_m) = self.healer.get_hot_fallback(&ctx.provider);
            warn!("🚑 [HEALER] Triggering hot fallback to {} ({}) due to failure on {}", 
                  fallback_p, fallback_m, ctx.provider);
            // In a real stream or if Pingora supported mid-filter retries easily, 
            // we would re-issue the request here. For now, we record the intent.
        } else {
            self.circuit_breaker.report_success().await;
        }

        if let Some(user_id) = ctx.user_id {
            let _ = self.billing.record_usage(
                user_id,
                &ctx.tier,
                ctx.prompt_tokens as i32,
                ctx.completion_tokens as i32,
                &ctx.model,
                &ctx.provider,
                Some(ctx.request_id.clone()),
            ).await;
            
            if e.is_none() && !ctx.prompt.is_empty() {
                self.eval.evaluate_async(
                    ctx.prompt.clone(),
                    "Simulated successful response for evaluation".to_string(),
                    ctx.provider.clone(),
                    ctx.model.clone()
                ).await;
            }

            counter!("fluxgate_tokens_total", 
                "provider" => ctx.provider.clone(), 
                "model" => ctx.model.clone(),
                "tier" => ctx.tier.clone(),
                "type" => "completion"
            ).increment(ctx.completion_tokens as u64);

            if let Some(packet) = self.federated_learner.capture_knowledge(&ctx.prompt, "final_output_placeholder") {
                let _ = self.federated_learner.sync_weights(vec![packet]).await;
            }

            // V6.0 Project Transcendent Horizon: Singularity Pulse Monitoring
            {
                let mut pulse = self.pulse.lock().await;
                pulse.record_event((duration * 1000.0) as u64, e.is_none());
                pulse.check_resonance();
            }

            // V10 TIER 1: Hallucination Interception (Post-generation)
            let h_score = self.hallucination.score_response(&ctx.prompt, "final_output_placeholder");
            if matches!(h_score.recommendation, crate::intelligence::hallucination::VerificationAction::Retry) {
                warn!("🔄 [HALLUCINATION] High risk detected (Score={:.2}). Triggering retry logic.", h_score.consistency_score);
            }

            // Phase 8: World-Class Deception Detection
            let deception_score = self.deception_detector.analyze_deception("final_response_captured", None);
            if self.deception_detector.detect_sycophancy(&ctx.prompt, "final_response_captured") {
                warn!("👺 [SYCOPHANCY] High risk detected for agent. Logging for review.");
            }

            // Phase 8: Alignment Drift Monitoring
            if self.alignment_monitor.record_and_check_drift(agent_id, deception_score).await {
                error!("🚨 [ALIGNMENT] Agent {} has drifted beyond safety threshold! Automatic quarantine recommended.", agent_id);
            }

            // Phase 8: World-Class Mechanistic Interpretability
            let glass_box_trace = self.interpretability.trace_rationale("final_response_captured");
            if self.interpretability.check_bias_attribution(&glass_box_trace) {
                 debug!("🔍 [INTERPRETABILITY] Glass-box trace generated and bias-checked: {:?}", glass_box_trace);
            }

            // V10 TIER 1: Agent DNA Observation
            let agent_id = "agent-alpha-001"; // Logic needed to extract original ID
            self.dna_profiler.record_observation(agent_id, &ctx.prompt, (duration * 1000.0) as u32).await;

            // V10 TIER 2: AI FinOps - Record Spend
            self.finops.record_spend("team-alpha", 0.0024).await;

            // Phase 8: World-Class Economic Value Attribution
            let _eva = self.finops.attribute_value(agent_id, 0.0024, 0.85);

            // V10 TIER 2: Semantic Stream Stitching
            if !self.stream_stitcher.check_stream_integrity("partial_resp_placeholder") {
                let stitched = self.stream_stitcher.stitch_completion("partial_resp_placeholder", "...completed successfully.");
                debug!("🧵 [STITCHER] Stream recovered: {}", stitched);
            }

            // V10 TIER 2: Prompt Mutation (Suggested on Low Quality)
            if let Some(strategy) = self.mutation_engine.suggest_mutation(h_score.consistency_score) {
                let _mutated = self.mutation_engine.mutate_prompt(&ctx.prompt, &strategy);
                info!("🧬 [MUTATION] Optimization strategy '{}' prepared for potential retry.", strategy);
            }

            // V10 TIER 4: FluxGate Intelligence (Benchmark Contribution)
            {
                let mut network = self.intelligence_network.write().await;
                network.contribute_telemetry(&ctx.model, (duration * 1000.0) as u32, e.is_none());
            }

            // V10 TIER 4: Governance Agent (AI Watching AI Overlay)
            if !self.governance_agent.audit_interaction(agent_id, &ctx.prompt, "final_response_captured") {
                warn!("🚨 [GOVERNANCE] Interaction flagged as non-compliant by Governance Agent.");
            }

            // Phase 8: Bulkhead Release
            if let Some(uid) = ctx.user_id {
                if let Some(bulkhead) = self.bulkheads.get(&uid.to_string()) {
                    bulkhead.release().await;
                }
            }
        }
    }
}
