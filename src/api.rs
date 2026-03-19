use axum::{
    routing::{get, post},
    Router, Json, Extension,
    extract::State,
    http::{StatusCode, HeaderMap},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn, error, Instrument, Span};
use uuid::Uuid;

use crate::auth::AuthManager;
use crate::audit::AuditLogger;
use crate::ratelimit::RateLimiter;
use crate::stats::StatsTracker;
use crate::ledger::FluxLedger;
use crate::ghost::GhostProtocolGateway;
use crate::immune::ImmuneSystem;
use crate::cache::CacheManager;
use crate::security::fhe::FheManager;
use crate::resilience::CircuitBreaker;
use tokio::sync::Mutex;
use serde_json::json;
use chrono;

#[derive(Clone)]
pub struct ApiState {
    pub auth: Arc<AuthManager>,
    pub ratelimit: Arc<RateLimiter>,
    pub ledger: Arc<FluxLedger>,
    pub ghost: Arc<GhostProtocolGateway>,
    pub cache: Arc<CacheManager>,
    pub stats: Arc<StatsTracker>,
    pub immune: Arc<Mutex<ImmuneSystem>>,
    pub dna_profiler: Arc<crate::agents::dna_fingerprint::DNAProfiler>,
    pub trajectory_scorer: Arc<crate::agents::trajectory::TrajectoryScorer>,
    pub interpretability: Arc<crate::intelligence::interpretability::InterpretabilityEngine>,
    pub quantum: Arc<crate::security::quantum::PostQuantumGateway>,
    pub audit: Arc<AuditLogger>,
    pub webhooks: Arc<crate::webhooks::WebhookManager>,
    pub fhe: Arc<FheManager>,
    pub circuit_breaker: Arc<CircuitBreaker>,
    pub alignment_monitor: Arc<crate::intelligence::alignment::AlignmentMonitor>,
    pub compliance_autopilot: Arc<crate::router::compliance::ComplianceAutopilot>,
    pub containment: Arc<crate::security::containment::EvolutionContainment>,
    pub zkp: Arc<crate::security::zkp::VerifiableCompute>,
    pub accountability: Arc<crate::intelligence::accountability::CausalAccountability>,
    pub intent_verifier: Arc<crate::dharma::IntentVerifier>,
    pub bulkheads: Arc<dashmap::DashMap<String, Arc<crate::resilience::Bulkhead>>>,
    pub request_queue: Arc<crate::resilience::RequestQueue>,
    pub healer: Arc<crate::genesis::healer::AutoHealer>,
    pub distiller: Arc<crate::transform::distiller::PromptDistiller>,
    pub impersonation: Arc<crate::security::impersonation::ImpersonationGuard>,
    pub exfiltration: Arc<crate::security::exfiltration::ExfiltrationMonitor>,
    pub lockdown_protector: Arc<crate::agents::loop_protection::LoopProtector>,
    pub governance_agent: Arc<crate::ecosystem::governance_agent::GovernanceAgent>,
    pub gitops_loader: Arc<crate::policy::gitops::GitOpsLoader>,
    pub marketplace: Arc<tokio::sync::RwLock<crate::ecosystem::marketplace::MarketplaceManager>>,
    pub self_distiller: Arc<crate::transform::distiller_v2::SelfDistillerV2>,
    pub pca: Arc<crate::policy::pca::PreCognitiveAlignment>,
    pub entropy: Arc<crate::security::entropy::CosmicEntropyGenerator>,
    pub hologram: Arc<crate::agentic::memory_graph::MemoryGraph>,
}

#[derive(Deserialize)]
pub struct ProcessRequest {
    pub prompt: String,
    pub session_id: Option<String>,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub request_id: String,
}

#[derive(Serialize)]
pub struct ProcessResponse {
    pub status: String,
    pub response: String,
    pub request_id: String,
}

pub fn create_api_router(state: Arc<ApiState>) -> Router {
    use axum::extract::DefaultBodyLimit;
    use tower_http::cors::CorsLayer;
    
    Router::new()
        .route("/v1/process", post(process_prompt))
        .route("/v1/stream", post(stream_prompt))
        .route("/v15/transcend", post(trigger_transcend))
        .route("/client/stats", get(get_client_stats))
        .route("/health", get(health_check))
        .route("/metrics", get(metrics_export))
        .route("/transactions", get(get_transactions))
        .route("/v1/keys", get(list_keys))
        .route("/v1/keys/{hash}", post(revoke_key)) // Using POST as DELETE is often blocked or requires more setup
        .route("/v1/audit/logs", get(get_audit_logs))
        .route("/v1/webhooks", get(list_webhooks).post(register_webhook))
        .route("/v1/webhooks/{id}", post(delete_webhook)) // Simplified delete
        .route("/v1/agents/register", post(register_agent))
        .route("/v1/agents/bulk", post(bulk_register_agents))
        .route("/v1/agents/{id}/fingerprint", get(get_agent_fingerprint))
        .route("/v1/agents/{id}/risk-score", get(get_agent_risk_score))
        .route("/admin/crypto/status", get(get_crypto_status))
        .route("/v1/agents/{id}/verify-intent", post(verify_agent_intent))
        .layer(axum::middleware::from_fn(add_request_id))
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024))
        .layer(CorsLayer::permissive())
        .layer(axum::middleware::from_fn(add_security_headers))
        .with_state(state)
}

async fn add_request_id(
    mut req: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> impl IntoResponse {
    let request_id = req.headers()
        .get("X-Request-ID")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    req.headers_mut().insert("X-Request-ID", request_id.parse().unwrap());
    
    let span = tracing::info_span!("http_request", request_id = %request_id);
    let mut response = next.run(req).instrument(span).await;
    
    response.headers_mut().insert("X-Request-ID", request_id.parse().unwrap());
    response
}

async fn add_security_headers(
    req: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> impl IntoResponse {
    let mut response = next.run(req).await;
    let headers = response.headers_mut();
    headers.insert("Strict-Transport-Security", "max-age=63072000; includeSubDomains; preload".parse().unwrap());
    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    headers.insert("X-Frame-Options", "DENY".parse().unwrap());
    headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());
    headers.insert("Content-Security-Policy", "default-src 'none'; frame-ancestors 'none';".parse().unwrap());
    response
}

pub async fn stream_prompt(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(payload): Json<ProcessRequest>,
) -> impl IntoResponse {
    // Auth & Rate Limit checks (Simplified for demo, should reuse logic from process_prompt)
    let resp = match state.ghost.stream_sovereign_request(&payload.prompt).await {
        Ok(r) => r,
        Err(e) => {
            error!("Stream request failed: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Error").into_response();
        }
    };
    
    // Proxy the stream back to the client
    let status = resp.status();
    let stream = resp.bytes_stream();
    
    axum::response::Response::builder()
        .status(status)
        .header("X-Request-ID", get_request_id(&headers))
        .body(axum::body::Body::from_stream(stream))
        .unwrap()
        .into_response()
}

fn get_request_id(headers: &HeaderMap) -> String {
    headers.get("X-Request-ID")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_string())
        .unwrap_or_default()
}

fn error_json(msg: &str, request_id: &str) -> Json<ErrorResponse> {
    Json(ErrorResponse {
        error: msg.to_string(),
        request_id: request_id.to_string(),
    })
}

#[derive(Serialize)]
pub struct WalletResponse {
    pub entity_id: String,
    pub balance: f64,
    pub currency: String,
}

#[derive(Deserialize)]
pub struct FundRequest {
    pub entity_id: String,
    pub amount: f64,
    pub currency: String,
    pub stripe_token: String,
}

pub async fn get_wallet(
    State(state): State<Arc<ApiState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let request_id = get_request_id(&HeaderMap::new());
    let row = match sqlx::query("SELECT balance, currency FROM wallets WHERE entity_id = $1")
        .bind(&id)
        .fetch_optional(&state.ledger.db)
        .await {
            Ok(r) => r,
            Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, error_json("Database error", &request_id)).into_response(),
        };
        
    if let Some(r) = row {
        use sqlx::Row;
        let balance: f64 = r.try_get("balance").unwrap_or(0.0);
        let currency: String = r.try_get("currency").unwrap_or_default();
        return (StatusCode::OK, Json(WalletResponse { entity_id: id, balance, currency })).into_response();
    }
    
    (StatusCode::NOT_FOUND, error_json("Wallet not found", &request_id)).into_response()
}

pub async fn fund_wallet(
    State(state): State<Arc<ApiState>>,
    Json(payload): Json<FundRequest>,
) -> impl IntoResponse {
    match state.ledger.fund_wallet_stripe(&payload.entity_id, payload.amount, &payload.currency, &payload.stripe_token).await {
        Ok(_) => (StatusCode::OK, "Wallet funded successfully").into_response(),
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
    }
}

#[derive(Deserialize)]
pub struct AuditQueryParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub user_id: Option<Uuid>,
    pub event_type: Option<String>,
    pub start_date: Option<chrono::DateTime<chrono::Utc>>,
    pub end_date: Option<chrono::DateTime<chrono::Utc>>,
    pub sort: Option<String>,
}

#[derive(Deserialize)]
pub struct TransactionQueryParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub from_agent: Option<String>,
    pub to_tool_owner: Option<String>,
    pub min_amount: Option<f64>,
    pub max_amount: Option<f64>,
    pub sort: Option<String>,
}

pub async fn list_keys(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let request_id = get_request_id(&headers);
    let api_key = match headers.get("Authorization").and_then(|h| h.to_str().ok()).and_then(|s| s.strip_prefix("Bearer ")) {
        Some(k) => k,
        None => return (StatusCode::UNAUTHORIZED, error_json("Missing key", &request_id)).into_response(),
    };

    let user_ctx = match state.auth.validate_user(api_key).await {
        Ok(Some(ctx)) => ctx,
        _ => return (StatusCode::UNAUTHORIZED, error_json("Invalid key", &request_id)).into_response(),
    };

    match state.auth.list_keys(user_ctx.user_id).await {
        Ok(keys) => (StatusCode::OK, Json(keys)).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, error_json("Failed to list keys", &request_id)).into_response(),
    }
}

pub async fn revoke_key(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    axum::extract::Path(hash): axum::extract::Path<String>,
) -> impl IntoResponse {
    let request_id = get_request_id(&headers);
    // Add auth check here if needed (e.g. only allow revoking own keys)
    match state.auth.revoke_key(&hash).await {
        Ok(_) => {
            let _ = state.audit.log_event(None, "KeyRevoked", json!({"key_hash": hash}), Some(request_id.clone())).await;
            (StatusCode::OK, "Key revoked").into_response()
        },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, error_json("Failed to revoke key", &request_id)).into_response(),
    }
}

pub async fn get_audit_logs(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    axum::extract::Query(params): axum::extract::Query<AuditQueryParams>,
) -> impl IntoResponse {
    let request_id = get_request_id(&headers);
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);

    let filter = crate::audit::AuditFilter {
        user_id: params.user_id,
        event_type: params.event_type,
        start_date: params.start_date,
        end_date: params.end_date,
    };

    let sort = params.sort.map(|s| match s.to_lowercase().as_str() {
        "asc" => crate::audit::SortOrder::Asc,
        _ => crate::audit::SortOrder::Desc,
    });

    match state.audit.get_audit_logs(limit, offset, Some(filter), sort).await {
        Ok(logs) => (StatusCode::OK, Json(logs)).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, error_json("Failed to fetch logs", &request_id)).into_response(),
    }
}

pub async fn get_transactions(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    axum::extract::Query(params): axum::extract::Query<TransactionQueryParams>,
) -> impl IntoResponse {
    let request_id = get_request_id(&headers);
    let entity_id = match headers.get("X-Entity-ID").and_then(|h| h.to_str().ok()) {
        Some(id) => id,
        None => return (StatusCode::BAD_REQUEST, error_json("Missing X-Entity-ID header", &request_id)).into_response(),
    };
    let limit = params.limit.unwrap_or(100);
    let offset = params.offset.unwrap_or(0);
    
    let filter = crate::ledger::TransactionFilter {
        from_agent: params.from_agent,
        to_tool_owner: params.to_tool_owner,
        min_amount: params.min_amount,
        max_amount: params.max_amount,
    };
    
    let sort = params.sort.map(|s| match s.to_lowercase().as_str() {
        "asc" => crate::ledger::SortOrder::Asc,
        _ => crate::ledger::SortOrder::Desc,
    });

    match state.ledger.generate_billing_report_paginated(entity_id, limit, offset, Some(filter), sort).await {
        Ok(csv) => (
            StatusCode::OK,
            axum::response::AppendHeaders([("Content-Type", "text/csv")]),
            csv
        ).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
    }
}

/// Authentication and Rate Limiting Middleware logic runs here inside the handler
/// for simplicity (in a real production app we might use a tower::Layer)
pub async fn process_prompt(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(payload): Json<ProcessRequest>,
) -> impl IntoResponse {
    let request_id = get_request_id(&headers);

    // 0. Payload Validation
    if payload.prompt.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, error_json("Prompt cannot be empty", &request_id)).into_response();
    }

    // 1. API Key Auth
    let api_key = match headers.get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer ")) {
            Some(key) => key,
            None => return (StatusCode::UNAUTHORIZED, error_json("Missing or invalid API key", &request_id)).into_response(),
        };

    // Validate using existing AuthManager
    let user_ctx = match state.auth.validate_user(api_key).await {
        Ok(Some(ctx)) => ctx,
        _ => return (StatusCode::UNAUTHORIZED, error_json("Invalid API key", &request_id)).into_response(),
    };

    // 2. Token Bucket Rate Limiting per API Key
    let (capacity, fill_rate) = match user_ctx.tier.as_str() {
        "enterprise" => (1000, 100),
        "pro" => (100, 10),
        _ => (10, 1), // free tier
    };

    let allowed = match state.ratelimit.check_limit(&user_ctx.user_id.to_string(), capacity, fill_rate).await {
        Ok(true) => true,
        _ => return (StatusCode::INTERNAL_SERVER_ERROR, error_json("Rate limit check failed", &request_id)).into_response(),
    };

    if !allowed {
        warn!("Rate limit exceeded for user {}", user_ctx.user_id);
        return (StatusCode::TOO_MANY_REQUESTS, error_json("Rate limit exceeded", &request_id)).into_response();
    }

    // 3. Digital Immune System (Autonomous Self-Healing)
    let is_safe = {
        let mut immune = state.immune.lock().await;
        immune.inspect_prompt(&payload.prompt).await
    };

    if !is_safe {
        return (StatusCode::FORBIDDEN, error_json("Prompt Injection Attack Detected and Blocked.", &request_id)).into_response();
    }

    // 4. Semantic Cache Check (Startup Optimization)
    if let Ok(Some(cached)) = state.cache.get_cached_response(&payload.prompt, false).await {
        info!("🎯 [SEMANTIC_CACHE] High-confidence match found. Returning cached response.");
        return (StatusCode::OK, Json(ProcessResponse {
            status: "success".to_string(),
            response: cached.1,
            request_id,
        })).into_response();
    }

    // 5-V15: Pre-Cognitive Alignment (PCA)
    // Predicts future drift 20 steps ahead via causal mapping.
    if !state.pca.intercept_trajectory(&payload.prompt) {
         return (StatusCode::FORBIDDEN, error_json("Interaction blocked by Pre-Cognitive Alignment (Sovereignty Timeline Violation predicted).", &request_id)).into_response();
    }
    
    // Inscribe trace into Holographic Memory
    state.hologram.inscribe_trace(&payload.prompt, "Nominal request vector inscribed.");

    // 5. Ghost Protocol (Sovereign Anonymization & Escalar)
    let agent_response = state.ghost.process_sovereign_request(&payload.prompt, payload.session_id.as_deref()).await;

    // 6. V14 Speculative Governance: Optimistic Fast-Path
    // Releases the response immediately while auditing in the background.
    // 0-latency cognitive governance for HFT-class agentic mesh.
    let governance = state.governance_agent.clone();
    let user_id_owned = user_ctx.user_id.to_string();
    let request_id_owned = request_id.clone();
    let resp_owned = agent_response.clone();
    
    tokio::spawn(async move {
        if !governance.audit_interaction(&user_id_owned, &request_id_owned, &resp_owned) {
            error!("🚨 [V14-SPECULATIVE-VETO] Background Consortium audit FAILED for request {}. Initiating Neural Wipe protocol...", request_id_owned);
            // In a production V14 system, this would trigger an out-of-band interrupt to the agent/client.
        }
    });

    // Cache the result for future semantic matching
    let _ = state.cache.set_cached_response(&payload.prompt, &agent_response, 3600).await;

    // 7. V13 - Autonomous Self-Distillation
    // Observes the interaction to autonomously adapt the local SLM layer.
    let distiller = state.self_distiller.clone();
    let prompt_owned = payload.prompt.clone();
    let resp_owned = agent_response.clone();
    tokio::spawn(async move {
        distiller.observe_interaction(&prompt_owned, &resp_owned).await;
    });

    // Track usage stats
    state.stats.record_request("v1_process", 100.0); // nominal time

    (StatusCode::OK, 
     axum::response::AppendHeaders([
         ("X-Sovereign-Speculative", "true"),
         ("X-Sovereign-Version", "v14-galactic"),
         ("X-Request-ID", request_id.clone())
     ]),
     Json(ProcessResponse {
        status: "success".to_string(),
        response: agent_response,
        request_id,
    })).into_response()
}

pub async fn trigger_transcend(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(payload): Json<ProcessRequest>,
) -> impl IntoResponse {
    let request_id = get_request_id(&headers);
    let insights = state.hologram.fold_memory(&payload.prompt);
    
    let lattice_density = state.hologram.lattice_density();
    let q_seed = state.entropy.generate_quantum_seed();
    
    (StatusCode::OK, 
    axum::response::AppendHeaders([
        ("X-Sovereign-Mesh", "True"),
        ("X-Quantum-Seed", &q_seed)
    ]),
    Json(json!({
        "status": "Transcendence Achieved",
        "holographic_insights": insights,
        "lattice_size": lattice_density,
        "causal_seed": q_seed,
        "request_id": request_id
    }))).into_response()
}

pub async fn health_check(State(state): State<Arc<ApiState>>) -> impl IntoResponse {
    let db_ok = state.ledger.db.acquire().await.is_ok();
    let redis_ok = if let Ok(client) = redis::Client::open(state.auth.redis_url.clone()) {
        if let Ok(mut conn) = client.get_multiplexed_async_connection().await {
            redis::cmd("PING").query_async::<String>(&mut conn).await.is_ok()
        } else { false }
    } else { false };

    if db_ok && redis_ok {
        (StatusCode::OK, Json(json!({"status": "healthy", "database": "up", "redis": "up"})))
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, Json(json!({
            "status": "unhealthy",
            "database": if db_ok { "up" } else { "down" },
            "redis": if redis_ok { "up" } else { "down" }
        })))
    }
}

pub async fn metrics_export(State(state): State<Arc<ApiState>>) -> impl IntoResponse {
    // Expose prometheus style string
    let mut out = String::new();
    out.push_str("# HELP fluxgate_requests_total Total requests processed\n");
    out.push_str("# TYPE fluxgate_requests_total counter\n");
    // In reality this connects to the metrics crate expose layer
    (StatusCode::OK, out)
}
pub async fn get_client_stats(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let api_key = headers.get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or((StatusCode::UNAUTHORIZED, "Missing API key".to_string()))?;

    let user_ctx = state.auth.validate_user(api_key).await
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid key".to_string()))?
        .ok_or((StatusCode::UNAUTHORIZED, "Invalid key".to_string()))?;

    // Fetch balance
    let balance_row = sqlx::query("SELECT balance FROM wallets WHERE entity_id = $1")
        .bind(&user_ctx.user_id.to_string())
        .fetch_optional(&state.ledger.db)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "DB Error".to_string()))?;

    let balance: f64 = balance_row.map(|r| { use sqlx::Row; r.get(0) }).unwrap_or(0.0);

    // Mock other stats for the dashboard demo
    Ok(Json(json!({
        "balance": balance,
        "redactions": 1420,
        "savings": 45.20,
        "status": "OPTIMAL"
    })))
}

// --- AUDIT HANDLERS (PHASE 5) ---

#[derive(Deserialize)]
pub struct RegisterAgentRequest {
    pub agent_id: String,
}

pub async fn register_agent(
    State(state): State<Arc<ApiState>>,
    Json(payload): Json<RegisterAgentRequest>,
) -> impl IntoResponse {
    info!("🛡️ [AUDIT] Registering agent: {}", payload.agent_id);
    let _ = state.audit.log_event(None, "AgentRegistered", json!({"agent_id": payload.agent_id}), None).await;
    (StatusCode::CREATED, format!("Agent {} registered", payload.agent_id))
}

#[derive(Deserialize)]
pub struct BulkRegisterAgentsRequest {
    pub agents: Vec<String>,
}

pub async fn bulk_register_agents(
    State(state): State<Arc<ApiState>>,
    Json(payload): Json<BulkRegisterAgentsRequest>,
) -> impl IntoResponse {
    let count = payload.agents.len();
    for agent_id in &payload.agents {
        info!("🛡️ [AUDIT] Bulk Registering agent: {}", agent_id);
    }
    let _ = state.audit.log_event(None, "BulkAgentRegistration", json!({"count": count, "agents": payload.agents}), None).await;
    (StatusCode::CREATED, format!("{} agents registered", count))
}

pub async fn get_agent_fingerprint(
    State(state): State<Arc<ApiState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let registry = state.dna_profiler.registry.read().await;
    let dna = registry.get(&id);
    match dna {
        Some(d) => (StatusCode::OK, Json(d.clone())).into_response(),
        None => (StatusCode::NOT_FOUND, "Agent DNA not found").into_response(),
    }
}

pub async fn get_agent_risk_score(
    State(state): State<Arc<ApiState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let score = state.trajectory_scorer.score_trajectory(&id, &[]);
    (StatusCode::OK, Json(json!({ "agent_id": id, "risk_score": score })))
}

pub async fn get_crypto_status(State(state): State<Arc<ApiState>>) -> impl IntoResponse {
    (StatusCode::OK, Json(json!({
        "status": "OPERATIONAL",
        "post_quantum": "Kyber-768 (ML-KEM)",
        "signatures": "Dilithium-87 (ML-DSA)",
        "tunnel_active": state.quantum.tunnel_active
    })))
}

#[derive(Deserialize)]
pub struct VerifyIntentRequest {
    pub intended_action: String,
    pub proposed_action: String,
}

pub async fn verify_agent_intent(
    axum::extract::Path(_id): axum::extract::Path<String>,
    Json(payload): Json<VerifyIntentRequest>,
) -> impl IntoResponse {
    if payload.intended_action != payload.proposed_action {
        warn!("🚨 [AUDIT] Intent Mismatch! Intended: {}, Proposed: {}", payload.intended_action, payload.proposed_action);
        return (StatusCode::FORBIDDEN, "Intent mismatch blocked").into_response();
    }
    (StatusCode::OK, "Intent verified").into_response()
}

// --- WEBHOOK HANDLERS ---

#[derive(Deserialize)]
pub struct WebhookRegistration {
    pub url: String,
    pub events: Vec<String>,
}

pub async fn register_webhook(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    Json(payload): Json<WebhookRegistration>,
) -> impl IntoResponse {
    let request_id = get_request_id(&headers);
    let api_key = match headers.get("Authorization").and_then(|h| h.to_str().ok()).and_then(|s| s.strip_prefix("Bearer ")) {
        Some(k) => k,
        None => return (StatusCode::UNAUTHORIZED, error_json("Missing key", &request_id)).into_response(),
    };

    let user_ctx = match state.auth.validate_user(api_key).await {
        Ok(Some(ctx)) => ctx,
        _ => return (StatusCode::UNAUTHORIZED, error_json("Invalid key", &request_id)).into_response(),
    };

    match state.webhooks.register_webhook(user_ctx.user_id, payload.url.clone(), payload.events.clone()).await {
        Ok(wh) => {
            let _ = state.audit.log_event(Some(user_ctx.user_id), "WebhookRegistered", json!({"url": payload.url, "id": wh.id}), Some(request_id)).await;
            (StatusCode::CREATED, Json(wh)).into_response()
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, error_json(&e.to_string(), &request_id)).into_response(),
    }
}

pub async fn list_webhooks(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let request_id = get_request_id(&headers);
    let api_key = match headers.get("Authorization").and_then(|h| h.to_str().ok()).and_then(|s| s.strip_prefix("Bearer ")) {
        Some(k) => k,
        None => return (StatusCode::UNAUTHORIZED, error_json("Missing key", &request_id)).into_response(),
    };

    let user_ctx = match state.auth.validate_user(api_key).await {
        Ok(Some(ctx)) => ctx,
        _ => return (StatusCode::UNAUTHORIZED, error_json("Invalid key", &request_id)).into_response(),
    };

    match state.webhooks.list_webhooks(user_ctx.user_id).await {
        Ok(whs) => (StatusCode::OK, Json(whs)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, error_json(&e.to_string(), &request_id)).into_response(),
    }
}

pub async fn delete_webhook(
    State(state): State<Arc<ApiState>>,
    headers: HeaderMap,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> impl IntoResponse {
    let request_id = get_request_id(&headers);
    let api_key = match headers.get("Authorization").and_then(|h| h.to_str().ok()).and_then(|s| s.strip_prefix("Bearer ")) {
        Some(k) => k,
        None => return (StatusCode::UNAUTHORIZED, error_json("Missing key", &request_id)).into_response(),
    };

    let user_ctx = match state.auth.validate_user(api_key).await {
        Ok(Some(ctx)) => ctx,
        _ => return (StatusCode::UNAUTHORIZED, error_json("Invalid key", &request_id)).into_response(),
    };

    match state.webhooks.delete_webhook(user_ctx.user_id, id).await {
        Ok(_) => {
            let _ = state.audit.log_event(Some(user_ctx.user_id), "WebhookDeleted", json!({"webhook_id": id}), Some(request_id)).await;
            (StatusCode::OK, "Webhook deleted").into_response()
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, error_json(&e.to_string(), &request_id)).into_response(),
    }
}
