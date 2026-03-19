use redis::AsyncCommands;
use sha2::{Sha256, Digest};
use hex;
use sqlx::{PgPool, Row};
use pgvector::Vector;
use std::sync::Arc;
use metrics::counter;
use async_trait::async_trait;
use serde_json::json;
use crate::error::FluxResult;

#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn generate(&self, text: &str) -> FluxResult<Vec<f32>>;
    fn dimension(&self) -> usize;
}

pub struct LocalTrigramProvider;
#[async_trait]
impl EmbeddingProvider for LocalTrigramProvider {
    fn dimension(&self) -> usize { 1536 }
    async fn generate(&self, text: &str) -> FluxResult<Vec<f32>> {
        let mut embedding = vec![0.0; 1536];
        let normalized = text.to_lowercase();
        let chars: Vec<char> = normalized.chars().collect();
        for i in 0..chars.len().saturating_sub(2) {
            let trigram = format!("{}{}{}", chars[i], chars[i+1], chars[i+2]);
            let mut h = 0u64;
            for b in trigram.bytes() { h = h.wrapping_add(b as u64); }
            let idx = (h % 1536) as usize;
            embedding[idx] += 1.0;
        }
        let norm = embedding.iter().map(|x| x*x).sum::<f32>().sqrt();
        if norm > 0.0 { for val in &mut embedding { *val /= norm; } }
        Ok(embedding)
    }
}

pub struct OpenAIEmbeddingProvider {
    pub api_key: String,
    pub client: reqwest::Client,
}
#[async_trait]
impl EmbeddingProvider for OpenAIEmbeddingProvider {
    fn dimension(&self) -> usize { 1536 }
    async fn generate(&self, text: &str) -> FluxResult<Vec<f32>> {
        let resp = self.client.post("https://api.openai.com/v1/embeddings")
            .bearer_auth(&self.api_key)
            .json(&json!({ "input": text, "model": "text-embedding-3-small" }))
            .send().await?;
        
        let json: serde_json::Value = resp.json().await?;
        let vec = json["data"][0]["embedding"].as_array()
            .ok_or_else(|| crate::error::FluxError::Internal("Invalid OpenAI Response".to_string()))?
            .iter().map(|v| v.as_f64().unwrap_or(0.0) as f32).collect();
        Ok(vec)
    }
}

use dashmap::DashMap;

pub struct CacheManager {
    redis_client: redis::Client,
    db_pool: PgPool,
    similarity_threshold: f64,
    provider: Arc<dyn EmbeddingProvider>,
    l1_cache: DashMap<String, String>, // Hot prompt hash -> Response
}

impl CacheManager {
    pub fn new(redis_url: &str, db_pool: PgPool, similarity_threshold: f64, provider: Arc<dyn EmbeddingProvider>) -> redis::RedisResult<Self> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self { 
            redis_client: client, 
            db_pool, 
            similarity_threshold, 
            provider,
            l1_cache: DashMap::new(),
        })
    }

    pub async fn get_cached_response(&self, prompt: &str, no_cache: bool) -> FluxResult<Option<(String, String)>> {
        if no_cache { return Ok(None); }
        let hash = self.hash_prompt(prompt);
        
        // L1 Cache check
        if let Some(cached) = self.l1_cache.get(&hash) {
            counter!("fluxgate_cache_performance", "result" => "l1_hit").increment(1);
            return Ok(Some((prompt.to_string(), cached.value().clone())));
        }

        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        let cached: Option<String> = conn.get(format!("cache:{}", hash)).await?;
        if let Some(res) = cached { 
            counter!("fluxgate_cache_performance", "result" => "redis_hit").increment(1);
            self.l1_cache.insert(hash, res.clone());
            return Ok(Some((prompt.to_string(), res))); 
        }
        self.get_semantic_match(prompt).await
    }

    pub async fn set_cached_response(&self, prompt: &str, response: &str, ttl_seconds: usize) -> FluxResult<()> {
        let hash = self.hash_prompt(prompt);
        self.l1_cache.insert(hash.clone(), response.to_string());

        let mut conn = self.redis_client.get_multiplexed_async_connection().await?;
        conn.set_ex::<_, _, ()>(format!("cache:{}", hash), response, ttl_seconds as u64).await?;
        
        let pool = self.db_pool.clone();
        let prompt_text = prompt.to_string();
        let response_text = response.to_string();
        let prompt_hash = hash.clone();
        let provider = self.provider.clone();

        tokio::spawn(async move {
            if let Ok(embedding) = provider.generate(&prompt_text).await {
                let query = "INSERT INTO semantic_cache (prompt_hash, prompt_text, response_text, embedding) VALUES ($1, $2, $3, $4) ON CONFLICT (prompt_hash) DO NOTHING";
                let _ = sqlx::query(query)
                    .bind(prompt_hash)
                    .bind(prompt_text)
                    .bind(response_text)
                    .bind(Vector::from(embedding))
                    .execute(&pool).await;
            }
        });
        Ok(())
    }

    fn hash_prompt(&self, prompt: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(prompt);
        hex::encode(hasher.finalize())
    }

    pub async fn get_semantic_match(&self, prompt: &str) -> FluxResult<Option<(String, String)>> {
        let embedding = self.provider.generate(prompt).await?;
        let query = "SELECT prompt_text, response_text, 1 - (embedding <=> $1) as similarity FROM semantic_cache ORDER BY embedding <=> $1 LIMIT 1";
        let row: Option<(String, String, f64)> = sqlx::query_as(query).bind(Vector::from(embedding)).fetch_optional(&self.db_pool).await?;

        if let Some((prompt_text, response, similarity)) = row {
            if similarity > self.similarity_threshold { 
                counter!("fluxgate_cache_performance", "result" => "semantic_hit").increment(1);
                return Ok(Some((prompt_text, response))); 
            }
        }
        counter!("fluxgate_cache_performance", "result" => "miss").increment(1);
        Ok(None)
    }

    pub async fn get_relevant_context(&self, prompt: &str, _limit: usize) -> FluxResult<Vec<(String, String)>> {
        if let Some(res) = self.get_semantic_match(prompt).await? {
            Ok(vec![res])
        } else {
            Ok(vec![])
        }
    }
}
