use sqlx::PgPool;

pub struct EvalJudge {
    _db_pool: PgPool,
}

impl EvalJudge {
    pub fn new(db_pool: PgPool) -> Self { Self { _db_pool: db_pool } }

    pub async fn evaluate_async(&self, prompt: String, response: String, provider: String, model: String) {
        // Asynchronous evaluation of model output quality
    }
}
