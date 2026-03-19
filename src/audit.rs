use sqlx::PgPool;
use uuid::Uuid;
use serde_json::Value;

use tokio::sync::mpsc;
use std::sync::Arc;
use metrics::counter;
use crate::error::FluxResult;

pub struct AuditLogger {
    db_pool: PgPool,
    tx: mpsc::UnboundedSender<AuditEvent>,
    notifications: Arc<crate::notification::NotificationHub>,
}

pub struct AuditFilter {
    pub user_id: Option<Uuid>,
    pub event_type: Option<String>,
    pub start_date: Option<chrono::DateTime<chrono::Utc>>,
    pub end_date: Option<chrono::DateTime<chrono::Utc>>,
}

pub enum SortOrder {
    Asc,
    Desc,
}

struct AuditEvent {
    user_id: Option<Uuid>,
    event_type: String,
    metadata: Value,
    request_id: Option<String>,
}

impl AuditLogger {
    pub fn new(db_pool: PgPool) -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<AuditEvent>();
        let pool = db_pool.clone();

        let notifications = Arc::new(crate::notification::NotificationHub::new());
        let notifications_clone = notifications.clone();

        tokio::spawn(async move {
            let mut batch = Vec::with_capacity(100);
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(1000));

            loop {
                tokio::select! {
                    Some(event) = rx.recv() => {
                        if event.event_type.contains("Attack") || event.event_type == "NovelInjection" {
                            let msg = format!("Security Event Detected: {:?}", event.metadata);
                            let n = notifications_clone.clone();
                            counter!("fluxgate_security_events_total", "event_type" => event.event_type.clone(), "severity" => "critical").increment(1);
                            tokio::spawn(async move {
                                n.alert("Critical Security Attack", &msg, "critical").await;
                            });
                        } else {
                            counter!("fluxgate_security_events_total", "event_type" => event.event_type.clone(), "severity" => "info").increment(1);
                        }
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

        Self { db_pool, tx, notifications }
    }

    async fn flush_batch(pool: &PgPool, batch: &mut Vec<AuditEvent>) {
        let mut query_builder = sqlx::QueryBuilder::new("INSERT INTO audit_logs (user_id, event_type, metadata) ");
        
        query_builder.push_values(batch.drain(..), |mut b, event| {
            b.push_bind(event.user_id)
             .push_bind(event.event_type)
             .push_bind(event.metadata)
             .push_bind(event.request_id);
        });

        let query = query_builder.build();
        if let Err(e) = query.execute(pool).await {
            tracing::error!("Failed to flush audit batch: {}", e);
        }
    }

    pub async fn log_event(&self, user_id: Option<Uuid>, event_type: &str, metadata: Value, request_id: Option<String>) -> FluxResult<()> {
        let event = AuditEvent {
            user_id,
            event_type: event_type.to_string(),
            metadata: metadata.clone(),
            request_id,
        };
        let _ = self.tx.send(event);
        Ok(())
    }

    pub async fn get_audit_logs(&self, limit: i64, offset: i64, filter: Option<AuditFilter>, sort: Option<SortOrder>) -> FluxResult<Vec<Value>> {
        let mut query_builder = sqlx::QueryBuilder::new(
            "SELECT user_id, event_type, metadata, created_at FROM audit_logs "
        );

        let mut has_where = false;
        if let Some(f) = filter {
            if let Some(uid) = f.user_id {
                query_builder.push(" WHERE user_id = ");
                query_builder.push_bind(uid);
                has_where = true;
            }
            if let Some(et) = f.event_type {
                if has_where { query_builder.push(" AND "); } else { query_builder.push(" WHERE "); }
                query_builder.push(" event_type = ");
                query_builder.push_bind(et);
                has_where = true;
            }
            if let Some(sd) = f.start_date {
                if has_where { query_builder.push(" AND "); } else { query_builder.push(" WHERE "); }
                query_builder.push(" created_at >= ");
                query_builder.push_bind(sd);
                has_where = true;
            }
            if let Some(ed) = f.end_date {
                if has_where { query_builder.push(" AND "); } else { query_builder.push(" WHERE "); }
                query_builder.push(" created_at <= ");
                query_builder.push_bind(ed);
            }
        }

        let order = match sort {
            Some(SortOrder::Asc) => " ASC ",
            _ => " DESC ",
        };

        query_builder.push(" ORDER BY created_at ");
        query_builder.push(order);
        query_builder.push(" LIMIT ");
        query_builder.push_bind(limit);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset);

        let rows = query_builder.build().fetch_all(&self.db_pool).await?;

        let mut logs = Vec::new();
        for row in rows {
            use sqlx::Row;
            let log = serde_json::json!({
                "user_id": row.get::<Option<Uuid>, _>("user_id"),
                "event_type": row.get::<String, _>("event_type"),
                "metadata": row.get::<Value, _>("metadata"),
                "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
            });
            logs.push(log);
        }
        Ok(logs)
    }
}
