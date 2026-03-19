use sqlx::{PgPool, Row};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use crate::error::FluxResult;
use std::sync::Arc;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use tracing::{info, error, warn};

#[derive(Serialize, Deserialize, Clone)]
pub struct Webhook {
    pub id: Uuid,
    pub user_id: Uuid,
    pub url: String,
    pub secret: String,
    pub events: Vec<String>,
}

pub struct WebhookManager {
    db: PgPool,
    client: reqwest::Client,
}

impl WebhookManager {
    pub fn new(db: PgPool) -> Self {
        Self {
            db,
            client: reqwest::Client::new(),
        }
    }

    pub async fn register_webhook(&self, user_id: Uuid, url: String, events: Vec<String>) -> FluxResult<Webhook> {
        let secret = Uuid::new_v4().to_string();
        let id = Uuid::new_v4();
        
        sqlx::query("INSERT INTO webhooks (id, user_id, url, secret, events) VALUES ($1, $2, $3, $4, $5)")
            .bind(id)
            .bind(user_id)
            .bind(&url)
            .bind(&secret)
            .bind(&events)
            .execute(&self.db).await?;
            
        Ok(Webhook { id, user_id, url, secret, events })
    }

    pub async fn list_webhooks(&self, user_id: Uuid) -> FluxResult<Vec<Webhook>> {
        let rows = sqlx::query("SELECT id, user_id, url, secret, events FROM webhooks WHERE user_id = $1")
            .bind(user_id)
            .fetch_all(&self.db).await?;
            
        let mut webhooks = Vec::new();
        for row in rows {
            webhooks.push(Webhook {
                id: row.get("id"),
                user_id: row.get("user_id"),
                url: row.get("url"),
                secret: row.get("secret"),
                events: row.get("events"),
            });
        }
        Ok(webhooks)
    }

    pub async fn delete_webhook(&self, user_id: Uuid, id: Uuid) -> FluxResult<()> {
        sqlx::query("DELETE FROM webhooks WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(&self.db).await?;
        Ok(())
    }

    pub async fn dispatch_event(&self, event_type: &str, user_id: Uuid, payload: Value) {
        let webhooks = match self.list_webhooks(user_id).await {
            Ok(w) => w,
            Err(_) => return,
        };

        for webhook in webhooks {
            if webhook.events.contains(&"*".to_string()) || webhook.events.contains(&event_type.to_string()) {
                let client = self.client.clone();
                let webhook_clone = webhook.clone();
                let payload_clone = payload.clone();
                let event_type_str = event_type.to_string();

                tokio::spawn(async move {
                    Self::deliver_with_retry(client, webhook_clone, event_type_str, payload_clone).await;
                });
            }
        }
    }

    async fn deliver_with_retry(client: reqwest::Client, webhook: Webhook, event_type: String, payload: Value) {
        let mut attempts = 0;
        let max_attempts = 5;
        let payload_str = serde_json::to_string(&payload).unwrap_or_default();
        
        loop {
            attempts += 1;
            
            // Sign the payload
            let signature = {
                type HmacSha256 = Hmac<Sha256>;
                let mut mac = HmacSha256::new_from_slice(webhook.secret.as_bytes()).expect("HMAC can take key of any size");
                mac.update(payload_str.as_bytes());
                hex::encode(mac.finalize().into_bytes())
            };

            let resp = client.post(&webhook.url)
                .header("X-FluxGate-Event", &event_type)
                .header("X-FluxGate-Signature", signature)
                .json(&payload)
                .send().await;

            match resp {
                Ok(r) if r.status().is_success() => {
                    info!("✅ [WEBHOOK] Delivered {} to {} on attempt {}", event_type, webhook.url, attempts);
                    break;
                }
                Ok(r) => {
                    warn!("⚠️ [WEBHOOK] Failed delivery to {} (Status: {}). Attempt {}/{}", webhook.url, r.status(), attempts, max_attempts);
                }
                Err(e) => {
                    error!("❌ [WEBHOOK] Error delivering to {} ({}). Attempt {}/{}", webhook.url, e, attempts, max_attempts);
                }
            }

            if attempts >= max_attempts {
                error!("💀 [WEBHOOK] Exhausted retries for {} to {}", event_type, webhook.url);
                break;
            }

            // Exponential backoff
            tokio::time::sleep(std::time::Duration::from_secs(2u64.pow(attempts))).await;
        }
    }
}
