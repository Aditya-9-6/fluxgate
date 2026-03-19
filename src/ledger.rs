use sqlx::{PgPool, Postgres, Transaction, FromRow};
use tracing::{info, debug, error, warn};
use tokio::sync::mpsc;
use std::sync::Arc;

/*
#[derive(Debug, FromRow)]
pub struct TransactionHistory {
    pub from_agent: String,
    pub to_tool_owner: String,
    pub amount: f64,
    pub currency: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, FromRow)]
pub struct Wallet {
    pub entity_id: String,
    pub balance: f64,
    pub currency: String,
}
*/

pub struct TransactionFilter {
    pub from_agent: Option<String>,
    pub to_tool_owner: Option<String>,
    pub min_amount: Option<f64>,
    pub max_amount: Option<f64>,
}

pub enum SortOrder {
    Asc,
    Desc,
}

pub struct SettlementTask {
    pub agent: String,
    pub tool_owner: String,
    pub cost: f64,
    pub currency: String,
    pub request_id: Option<String>,
}

/// Implement "Micro-Settlement" Ledger with PostgreSQL (sqlx)
/// Replaces in-memory HashMaps for ACID guarantees.
pub struct FluxLedger {
    pub db: PgPool,
    pub tx_sender: mpsc::UnboundedSender<SettlementTask>,
}

impl FluxLedger {
    pub fn new(db: PgPool) -> Self {
        let (tx_sender, mut rx) = mpsc::unbounded_channel::<SettlementTask>();
        let db_clone = db.clone();

        // Background Worker for Asynchronous Micro-Settlements
        tokio::spawn(async move {
            info!("🚀 [LEDGER_WORKER] Background Settlement Worker Started.");
            while let Some(task) = rx.recv().await {
                if let Err(e) = Self::execute_settlement(&db_clone, &task.agent, &task.tool_owner, task.cost, &task.currency, task.request_id).await {
                    error!("❌ [LEDGER_WORKER] Background Settlement Failed: {}", e);
                }
            }
        });

        Self { db, tx_sender }
    }

    /// Internal synchronous executor used by the background worker
    async fn execute_settlement(db: &PgPool, agent: &str, tool_owner: &str, cost: f64, currency: &str, request_id: Option<String>) -> Result<(), &'static str> {
        debug!("💸 [WORKER] Executing Settlement: '{}' for {:.2} {}", agent, cost, currency);
        
        let mut tx: Transaction<'_, Postgres> = db.begin().await
            .map_err(|_| "Failed to begin transaction")?;

        // 1. Check and deduct from Agent
        let agent_row = sqlx::query(
            "UPDATE wallets SET balance = balance - $1 WHERE entity_id = $2 AND balance >= $1 RETURNING balance"
        )
        .bind(cost)
        .bind(agent)
        .fetch_optional(&mut *tx).await.map_err(|_| "Query failed")?;

        if let Some(row) = agent_row {
            use sqlx::Row;
            let remaining_balance: f64 = row.try_get("balance").unwrap_or(0.0);
            if remaining_balance < 10.0 {
                warn!("⚠️ LOW BALANCE: Agent '{}' has only {:.2} remaining.", agent, remaining_balance);
            }
        } else {
            let _ = tx.rollback().await;
            return Err("Insufficient Funds");
        }

        // 2. Credit to Tool Owner
        sqlx::query("INSERT INTO wallets (entity_id, balance) VALUES ($1, $2) ON CONFLICT (entity_id) DO UPDATE SET balance = wallets.balance + $2")
            .bind(tool_owner)
            .bind(cost)
            .execute(&mut *tx).await.map_err(|_| "Credit failed")?;

        // 3. Record History
        sqlx::query("INSERT INTO micro_transactions (from_agent, to_tool_owner, amount, request_id) VALUES ($1, $2, $3, $4)")
            .bind(agent)
            .bind(tool_owner)
            .bind(cost)
            .bind(request_id)
            .execute(&mut *tx).await.map_err(|_| "History failed")?;

        tx.commit().await.map_err(|_| "Commit failed")?;
        Ok(())
    }

    /// Non-blocking settlement call. Returns immediately, processing happens in background.
    pub async fn settle_tool_usage(&self, agent: &str, tool_owner: &str, cost: f64, currency: &str, request_id: Option<String>) -> Result<(), &'static str> {
        debug!("Pushing settlement task to background queue for agent '{}'", agent);
        
        let task = SettlementTask {
            agent: agent.to_string(),
            tool_owner: tool_owner.to_string(),
            cost,
            currency: currency.to_string(),
            request_id,
        };

        self.tx_sender.send(task).map_err(|_| "Ledger worker channel closed")?;
        Ok(())
    }

    /// Seed a balance from Stripe fiat top-ups
    pub async fn fund_wallet_stripe(&self, entity: &str, amount: f64, currency: &str, stripe_token: &str) -> Result<(), &'static str> {
        info!("💳 Processing Stripe Connect payment for '{}': {} {}", entity, amount, currency);
        if stripe_token.is_empty() { return Err("Invalid Stripe token"); }
        if amount <= 0.0 { return Err("Amount must be positive"); }
        self.fund_wallet(entity, amount, currency).await.map_err(|_| "DB Error during funding")?;
        Ok(())
    }

    pub async fn fund_wallet(&self, entity: &str, amount: f64, currency: &str) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO wallets (entity_id, balance, currency) VALUES ($1, $2, $3) ON CONFLICT (entity_id) DO UPDATE SET balance = wallets.balance + $2")
            .bind(entity)
            .bind(amount)
            .bind(currency)
            .execute(&self.db).await?;
        Ok(())
    }

    pub async fn generate_billing_report(&self, entity_id: &str) -> Result<String, &'static str> {
        let rows = sqlx::query("SELECT from_agent, to_tool_owner, amount FROM micro_transactions WHERE from_agent = $1 OR to_tool_owner = $1 ORDER BY id DESC LIMIT 500")
            .bind(entity_id)
            .fetch_all(&self.db).await.map_err(|_| "Failed to fetch transactions")?;

        let mut csv_output = String::from("From,To,Amount,Currency\n");
        for row in rows {
            use sqlx::Row;
            let from: String = row.try_get("from_agent").unwrap_or_default();
            let to: String = row.try_get("to_tool_owner").unwrap_or_default();
            let amt: f64 = row.try_get("amount").unwrap_or(0.0);
            csv_output.push_str(&format!("{},{},{:.2},USD\n", from, to, amt));
        }
        Ok(csv_output)
    }

    pub async fn generate_billing_report_paginated(&self, entity_id: &str, limit: i64, offset: i64, filter: Option<TransactionFilter>, sort: Option<SortOrder>) -> Result<String, String> {
        let mut query_builder = sqlx::QueryBuilder::<Postgres>::new(
            "SELECT from_agent, to_tool_owner, amount FROM micro_transactions "
        );

        query_builder.push(" WHERE (from_agent = ");
        query_builder.push_bind(entity_id);
        query_builder.push(" OR to_tool_owner = ");
        query_builder.push_bind(entity_id);
        query_builder.push(")");

        if let Some(f) = filter {
            if let Some(from) = f.from_agent {
                query_builder.push(" AND from_agent = ");
                query_builder.push_bind(from);
            }
            if let Some(to) = f.to_tool_owner {
                query_builder.push(" AND to_tool_owner = ");
                query_builder.push_bind(to);
            }
            if let Some(min) = f.min_amount {
                query_builder.push(" AND amount >= ");
                query_builder.push_bind(min);
            }
            if let Some(max) = f.max_amount {
                query_builder.push(" AND amount <= ");
                query_builder.push_bind(max);
            }
        }

        let order = match sort {
            Some(SortOrder::Asc) => " ASC ",
            _ => " DESC ",
        };

        query_builder.push(" ORDER BY id ");
        query_builder.push(order);
        query_builder.push(" LIMIT ");
        query_builder.push_bind(limit);
        query_builder.push(" OFFSET ");
        query_builder.push_bind(offset);

        let rows = query_builder.build().fetch_all(&self.db).await.map_err(|e| format!("Failed to fetch transactions: {}", e))?;

        let mut csv_output = String::from("From,To,Amount,Currency\n");
        for row in rows {
            use sqlx::Row;
            let from: String = row.try_get("from_agent").unwrap_or_default();
            let to: String = row.try_get("to_tool_owner").unwrap_or_default();
            let amt: f64 = row.try_get("amount").unwrap_or(0.0);
            csv_output.push_str(&format!("{},{},{:.2},USD\n", from, to, amt));
        }
        Ok(csv_output)
    }

    pub async fn transfer(&self, from: &str, to: &str, amount: i64, _reason: &str) -> Result<(), &'static str> {
        self.settle_tool_usage(from, to, amount as f64, "USD", None).await
    }
}

