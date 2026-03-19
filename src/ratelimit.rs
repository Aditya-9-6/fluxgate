use redis::AsyncCommands;
use crate::error::FluxResult;

pub struct RateLimiter {
    redis_client: redis::Client,
}

impl RateLimiter {
    pub fn new(redis_url: &str) -> redis::RedisResult<Self> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self { redis_client: client })
    }

    pub async fn check_limit(&self, key: &str, limit: u32, _fill_rate: u32) -> FluxResult<bool> {
        if std::env::var("FLUXGATE_DRY_RUN").is_ok() {
            return Ok(true);
        }
        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        let redis_key = format!("ratelimit:{}", key);
        
        // Simple token bucket implementation in Redis (mocked)
        let current: u32 = conn.get(&redis_key).await.unwrap_or(0);
        if current >= limit {
            return Ok(false);
        }
        
        let _: () = conn.incr(&redis_key, 1).await?;
        let _: () = conn.expire(&redis_key, 1).await?; // 1 second window
        
        Ok(true)
    }
}
