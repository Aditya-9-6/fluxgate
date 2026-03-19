use std::sync::Arc;
use axum::{Router, routing::{get, post}, Json, extract::{State, Path}};

use serde_json::{Value, json};

pub struct AdminState {
    pub auth: Arc<crate::auth::AuthManager>,
    pub prompts: Arc<tokio::sync::Mutex<crate::prompt_registry::PromptRegistry>>,
    pub stats: Arc<crate::stats::StatsTracker>,
    pub audit: Arc<crate::audit::AuditLogger>,
}

pub async fn start_admin_api(state: Arc<AdminState>, port: u16) {
    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/stats", get(get_stats))
        .route("/tenants", post(create_tenant))                     // Create org
        .route("/tenants/:id/keys", get(list_tenant_keys))          // Tenant keys
        .route("/tenants/:id/keys/rotate", post(rotate_tenant_key)) // Key rotation
        .route("/tenants/:id/config", post(update_tenant_config))   // Security configs
        .with_state(state);

    match tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await {
        Ok(listener) => {
            if let Err(e) = axum::serve(listener, app).await {
                tracing::error!("Admin Server failed: {}", e);
            }
        }
        Err(e) => {
            tracing::error!("Failed to bind admin server on port {}: {}", port, e);
        }
    }
}

async fn get_stats(State(state): State<Arc<AdminState>>) -> Json<Value> {
    Json(json!({ "status": "operational" }))
}

// ---------------- Multi-Tenancy Admin Handlers ----------------

#[derive(serde::Deserialize)]
pub struct CreateTenantReq {
    pub name: String,
    pub tier: String,
}

async fn create_tenant(State(_state): State<Arc<AdminState>>, Json(req): Json<CreateTenantReq>) -> Json<Value> {
    // Mock DB insert
    let tenant_id = uuid::Uuid::new_v4().to_string();
    Json(json!({ "status": "created", "tenant_id": tenant_id, "name": req.name, "tier": req.tier }))
}

async fn list_tenant_keys(State(_state): State<Arc<AdminState>>, Path(id): Path<String>) -> Json<Value> {
    // In reality, SELECT name, key_hash, created_at FROM api_keys WHERE tenant_id = $id
    Json(json!({ "tenant_id": id, "keys": [{"name": "prod-key-1", "prefix": "fg_a1b2..."}] }))
}

async fn rotate_tenant_key(State(_state): State<Arc<AdminState>>, Path(id): Path<String>) -> Json<Value> {
    // Generate new key, mark old as expired after 24h
    let new_key = format!("fg_{}", uuid::Uuid::new_v4().simple());
    Json(json!({ "tenant_id": id, "new_key": new_key, "message": "Old key expires in 24 hours" }))
}

#[derive(serde::Deserialize)]
pub struct TenantConfigReq {
    pub entropy_threshold: f64,
    pub enable_pii_redaction: bool,
    pub provider_zone: String,
}

async fn update_tenant_config(State(_state): State<Arc<AdminState>>, Path(id): Path<String>, Json(req): Json<TenantConfigReq>) -> Json<Value> {
    // UPDATE tenants SET config = $1 WHERE id = $2
    Json(json!({ "tenant_id": id, "status": "updated", "config": {
        "entropy": req.entropy_threshold,
        "pii": req.enable_pii_redaction,
        "routing": req.provider_zone
    }}))
}
