ALTER TABLE workspaces ADD COLUMN subscription_status TEXT NOT NULL DEFAULT 'trial'
  CHECK (subscription_status IN ('trial', 'active', 'grace', 'inactive'));
ALTER TABLE workspaces ADD COLUMN trial_ends_at INTEGER;

CREATE TABLE workspace_members (
  workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
  oid TEXT NOT NULL,
  role TEXT NOT NULL CHECK (role IN ('owner', 'operator', 'viewer')),
  created_at INTEGER NOT NULL,
  PRIMARY KEY (workspace_id, oid)
);

ALTER TABLE real_bookings ADD COLUMN contact_expires_at INTEGER;
ALTER TABLE waitlist_entries ADD COLUMN contact_expires_at INTEGER;
ALTER TABLE waitlist_entries ADD COLUMN idempotency_key TEXT;
CREATE UNIQUE INDEX waitlist_idempotency_idx ON waitlist_entries(class_id, idempotency_key);

ALTER TABLE calendar_connections RENAME TO calendar_connections_legacy;
CREATE TABLE calendar_connections (
  id TEXT PRIMARY KEY NOT NULL,
  workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
  label TEXT NOT NULL,
  provider TEXT NOT NULL CHECK (provider IN ('ical_feed')),
  enabled INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
  created_at INTEGER NOT NULL,
  feed_url_encrypted TEXT,
  last_polled_at INTEGER,
  next_poll_at INTEGER,
  last_error TEXT,
  UNIQUE(workspace_id)
);
DROP TABLE calendar_connections_legacy;

CREATE TABLE email_outbox (
  id TEXT PRIMARY KEY NOT NULL,
  workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
  recipient_encrypted TEXT NOT NULL,
  subject TEXT NOT NULL,
  text_body TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('pending', 'sent', 'captured', 'failed')),
  attempts INTEGER NOT NULL DEFAULT 0,
  next_attempt_at INTEGER NOT NULL,
  sent_at INTEGER,
  last_error TEXT,
  created_at INTEGER NOT NULL
);
CREATE INDEX email_outbox_due_idx ON email_outbox(status, next_attempt_at);

CREATE TABLE billing_events (
  id TEXT PRIMARY KEY NOT NULL,
  workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
  external_reference_hash TEXT NOT NULL UNIQUE,
  status TEXT NOT NULL,
  created_at INTEGER NOT NULL
);
