use sqlx::PgPool;
use uuid::Uuid;
use bigdecimal::BigDecimal;
use serde::{Serialize, Deserialize};
use tokio::sync::mpsc;
use std::sync::Arc;
use std::str::FromStr;
use tracing::info;

pub mod finops;

pub struct BillingManager {
    pub db_pool: PgPool,
    tx: mpsc::UnboundedSender<BillingEvent>,
    stripe_client: reqwest::Client,
    stripe_secret_key: String,
    carbon_api_key: Option<String>,
}

pub struct BillingEvent {
    pub user_id: Uuid,
    pub model: String,
    pub provider: String,
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub cost: BigDecimal,
    pub request_id: Option<String>,
}

impl BillingManager {
    pub fn new(db_pool: PgPool, stripe_secret_key: String) -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<BillingEvent>();
        let pool = db_pool.clone();

        // Background Batch Worker
        tokio::spawn(async move {
            let mut batch = Vec::with_capacity(100);
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(500));

            loop {
                tokio::select! {
                    Some(event) = rx.recv() => {
                        batch.push(event);
                        if batch.len() >= 100 {
                            Self::flush_batch(&pool, &mut batch).await;
                        }
                    }
                    _ = interval.tick() => {
                        if !batch.is_empty() {
                            Self::flush_batch(&pool, &mut batch).await;
                        }
                    }
                }
            }
        });

        let carbon_api_key = std::env::var("ELECTRICITY_MAPS_API_KEY").ok();
        Self { 
            db_pool, 
            tx, 
            stripe_client: reqwest::Client::new(),
            stripe_secret_key,
            carbon_api_key 
        }
    }

    pub async fn top_up_wallet(&self, user_id: Uuid, amount_usd: BigDecimal, _payment_method_id: &str) -> anyhow::Result<()> {
        info!("💳 [STRIPE] Initiating Stripe Connect top-up for user {}: ${}", user_id, amount_usd);
        let _ = sqlx::query("UPDATE credits SET balance_usd = balance_usd + $1 WHERE user_id = $2")
            .bind(&amount_usd)
            .bind(user_id)
            .execute(&self.db_pool)
            .await?;
        Ok(())
    }

    pub async fn get_carbon_intensity(&self, zone: &str) -> f64 {
        if let Some(key) = &self.carbon_api_key {
            let url = format!("https://api.electricitymap.org/v3/carbon-intensity/latest?zone={}", zone);
            match self.stripe_client.get(url).header("auth-token", key).send().await {
                Ok(resp) => {
                    let json: serde_json::Value = resp.json().await.unwrap_or_default();
                    return json["carbonIntensity"].as_f64().unwrap_or(450.0);
                }
                Err(_) => return 450.0,
            }
        }
        450.0 // Default fallback
    }

    async fn flush_batch(pool: &PgPool, batch: &mut Vec<BillingEvent>) {
        let mut tx = match pool.begin().await {
            Ok(t) => t,
            Err(e) => {
                tracing::error!("Failed to start billing transaction: {}", e);
                return;
            }
        };

        for event in batch.drain(..) {
            let _ = sqlx::query(
                "INSERT INTO usage_logs (user_id, model, provider, prompt_tokens, completion_tokens, cost_usd, request_id) 
                 VALUES ($1, $2, $3, $4, $5, $6, $7)"
            )
            .bind(event.user_id)
            .bind(event.model)
            .bind(event.provider)
            .bind(event.prompt_tokens)
            .bind(event.completion_tokens)
            .bind(&event.cost)
            .bind(event.request_id)
            .execute(&mut *tx)
            .await;

            let _ = sqlx::query(
                "UPDATE credits SET balance_usd = balance_usd - $1 WHERE user_id = $2"
            )
            .bind(&event.cost)
            .bind(event.user_id)
            .execute(&mut *tx)
            .await;
        }

        if let Err(e) = tx.commit().await {
            tracing::error!("Failed to commit billing batch: {}", e);
        }
    }

    pub async fn record_usage(
        &self,
        user_id: Uuid,
        _tier: &str,
        prompt_tokens: i32,
        completion_tokens: i32,
        model: &str,
        provider: &str,
        request_id: Option<String>,
    ) -> anyhow::Result<()> {
        let cost = self.calculate_cost(model, prompt_tokens, completion_tokens);
        
        let event = BillingEvent {
            user_id,
            model: model.to_string(),
            provider: provider.to_string(),
            prompt_tokens,
            completion_tokens,
            cost,
            request_id,
        };

        let _ = self.tx.send(event);
        Ok(())
    }

    pub fn calculate_cost(&self, model: &str, prompt_tokens: i32, completion_tokens: i32) -> BigDecimal {
        let (prompt_rate, completion_rate) = match model {
            "gpt-4" => (BigDecimal::from_str("0.00003").unwrap(), BigDecimal::from_str("0.00006").unwrap()),
            "gpt-3.5-turbo" => (BigDecimal::from_str("0.0000015").unwrap(), BigDecimal::from_str("0.000002").unwrap()),
            _ => (BigDecimal::from_str("0.00001").unwrap(), BigDecimal::from_str("0.00002").unwrap()),
        };

        let prompt_cost = prompt_rate * BigDecimal::from(prompt_tokens);
        let completion_cost = completion_rate * BigDecimal::from(completion_tokens);
        prompt_cost + completion_cost
    }

    pub fn count_tokens(&self, text: &str) -> i32 {
        (text.len() / 4) as i32
    }
}
