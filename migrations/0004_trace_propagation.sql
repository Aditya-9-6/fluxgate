-- Phase 7: Traceability & Request Correlation
-- Add request_id to audit logs for precise correlation
ALTER TABLE audit_logs ADD COLUMN IF NOT EXISTS request_id TEXT;
CREATE INDEX IF NOT EXISTS idx_audit_logs_request_id ON audit_logs(request_id);

-- Add request_id to usage logs for billing transparency
ALTER TABLE usage_logs ADD COLUMN IF NOT EXISTS request_id TEXT;
CREATE INDEX IF NOT EXISTS idx_usage_logs_request_id ON usage_logs(request_id);

-- Add request_id to micro-transactions for financial audit trails
ALTER TABLE micro_transactions ADD COLUMN IF NOT EXISTS request_id TEXT;
CREATE INDEX IF NOT EXISTS idx_micro_tx_request_id ON micro_transactions(request_id);
