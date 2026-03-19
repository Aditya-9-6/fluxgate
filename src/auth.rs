use sqlx::{PgPool, Row};
use bigdecimal::BigDecimal;
use uuid::Uuid;
use sha2::{Sha256, Digest};
use hex;
use serde::{Serialize, Deserialize};
use subtle::ConstantTimeEq;
use crate::error::{FluxResult, FluxError};

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct UserContext {
    pub user_id: Uuid,
    pub tier: String,
    pub budget_limit_usd: Option<BigDecimal>,
    pub balance_usd: Option<BigDecimal>,
    pub tenant_id: Option<String>,
    pub tenant_config: Option<serde_json::Value>,
}

pub struct AuthManager {
    db_pool: PgPool,
    redis_client: Option<redis::Client>,
    pub redis_url: String,
}

impl AuthManager {
    pub fn new(db_pool: PgPool, redis_url: Option<&str>) -> Self {
        let redis_client = redis_url.and_then(|url| redis::Client::open(url).ok());
        let redis_url = redis_url.unwrap_or("redis://127.0.0.1:6379").to_string();
        Self { db_pool, redis_client, redis_url }
    }

    pub async fn validate_user(&self, key: &str) -> FluxResult<Option<UserContext>> {
        let dry_run_key = "flux-test-key";
        let is_dry_run = std::env::var("FLUXGATE_DRY_RUN").is_ok();
        
        if is_dry_run && key.as_bytes().ct_eq(dry_run_key.as_bytes()).into() {
            return Ok(Some(UserContext {
                user_id: Uuid::nil(),
                tier: "pro".to_string(),
                budget_limit_usd: None,
                balance_usd: None,
                tenant_id: None,
                tenant_config: None,
            }));
        }
        let hash = self.hash_key(key);
        let cache_key = format!("auth:{}", hash);

        if let Some(ref client) = self.redis_client {
            if let Ok(mut conn) = client.get_multiplexed_async_connection().await {
                if let Ok(cached) = redis::cmd("GET").arg(&cache_key).query_async::<String>(&mut conn).await {
                    if let Ok(ctx) = serde_json::from_str(&cached) {
                        return Ok(Some(ctx));
                    }
                }
            }
        }
        
        let query = r#"
            SELECT 
                u.id as user_id,
                u.tier,
                c.budget_limit_usd,
                c.balance_usd,
                t.id as tenant_id,
                t.config as tenant_config
            FROM api_keys k
            JOIN users u ON k.user_id = u.id
            LEFT JOIN credits c ON u.id = c.user_id
            LEFT JOIN tenants t ON u.tenant_id = t.id
            WHERE k.key_hash = $1
        "#;

        let user_ctx: Option<UserContext> = sqlx::query_as(query)
            .bind(&hash)
            .fetch_optional(&self.db_pool)
            .await?;

        if let (Some(client), Some(ctx)) = (&self.redis_client, &user_ctx) {
            if let (Ok(mut conn), Ok(serialized)) = (client.get_multiplexed_async_connection().await, serde_json::to_string(ctx)) {
                let _: redis::RedisResult<()> = redis::cmd("SETEX")
                    .arg(&cache_key)
                    .arg(60) 
                    .arg(serialized)
                    .query_async::<()>(&mut conn)
                    .await;
            }
        }

        Ok(user_ctx)
    }

    pub async fn validate_key(&self, key: &str) -> FluxResult<Option<Uuid>> {
        self.validate_user(key).await.map(|ctx| ctx.map(|u| u.user_id))
    }

    pub async fn check_credits(&self, user_id: Uuid) -> FluxResult<bool> {
        let row: Option<(Option<BigDecimal>,)> = sqlx::query_as("SELECT balance_usd FROM credits WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(&self.db_pool)
            .await?;
        
        if let Some((Some(balance),)) = row {
             Ok(balance > BigDecimal::from(0))
        } else {
             Ok(false)
        }
    }

    pub async fn create_key(&self, user_id: Uuid, name: &str) -> FluxResult<String> {
        let key = format!("fg_{}", Uuid::new_v4().simple());
        let hash = self.hash_key(&key);
        
        sqlx::query("INSERT INTO api_keys (user_id, key_hash, name) VALUES ($1, $2, $3)")
            .bind(user_id)
            .bind(hash)
            .bind(name)
            .execute(&self.db_pool)
            .await?;
            
        Ok(key)
    }

    pub async fn list_keys(&self, user_id: Uuid) -> FluxResult<Vec<(String, String)>> {
        let rows = sqlx::query_as::<_, (String, String)>("SELECT name, key_hash FROM api_keys WHERE user_id = $1")
            .bind(user_id)
            .fetch_all(&self.db_pool)
            .await?;
            
        Ok(rows)
    }

    pub async fn revoke_key(&self, hash: &str) -> FluxResult<()> {
        sqlx::query("DELETE FROM api_keys WHERE key_hash = $1")
            .bind(hash)
            .execute(&self.db_pool)
            .await?;
        
        // Invalidate cache
        if let Some(ref client) = self.redis_client {
            if let Ok(mut conn) = client.get_multiplexed_async_connection().await {
                let cache_key = format!("auth:{}", hash);
                let _: redis::RedisResult<()> = redis::cmd("DEL").arg(&cache_key).query_async(&mut conn).await;
            }
        }
        
        Ok(())
    }

    fn hash_key(&self, key: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(key);
        hex::encode(hasher.finalize())
    }
}
