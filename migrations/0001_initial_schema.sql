-- Create micro-ledger wallets table
CREATE TABLE IF NOT EXISTS wallets (
    entity_id VARCHAR(255) PRIMARY KEY,
    balance NUMERIC(10, 4) NOT NULL DEFAULT 0.0
);

-- Create transactions history table
CREATE TABLE IF NOT EXISTS micro_transactions (
    id SERIAL PRIMARY KEY,
    from_agent VARCHAR(255) NOT NULL,
    to_tool_owner VARCHAR(255) NOT NULL,
    amount NUMERIC(10, 4) NOT NULL,
    timestamp TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Note: The Immune System signatures will be stored in PostgreSQL instead of just Redis 
-- or Redis can be used for fast-path, PG for persistence. Let's create a table for it.
CREATE TABLE IF NOT EXISTS immune_signatures (
    signature VARCHAR(255) PRIMARY KEY,
    detected_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
