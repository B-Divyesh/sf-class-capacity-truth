DROP TABLE IF EXISTS billing_events;
DROP TABLE IF EXISTS email_outbox;
DROP INDEX IF EXISTS waitlist_idempotency_idx;
DROP TABLE IF EXISTS workspace_members;
-- SQLite cannot drop the added columns on every supported runtime. The
-- previous schema remains readable after the feature tables are removed.
