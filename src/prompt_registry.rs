use sqlx::PgPool;
use std::collections::HashMap;

pub struct PromptRegistry {
    db_pool: PgPool,
}

impl PromptRegistry {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    pub async fn get_prompt(&self, name: &str) -> anyhow::Result<Option<String>> {
        let row: Option<(String,)> = sqlx::query_as("SELECT template FROM prompts WHERE name = $1")
            .bind(name)
            .fetch_optional(&self.db_pool)
            .await?;
        Ok(row.map(|(t,)| t))
    }
}
