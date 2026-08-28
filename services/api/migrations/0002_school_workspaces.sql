-- Durable, single-school workspaces.  Demo rows remain in the 0001 tables and
-- never join these tables.
CREATE TABLE workspaces (
  id TEXT PRIMARY KEY NOT NULL,
  school_name TEXT NOT NULL,
  access_key_hash TEXT NOT NULL UNIQUE,
  created_at INTEGER NOT NULL
);

CREATE TABLE real_classes (
  id TEXT PRIMARY KEY NOT NULL,
  workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
  public_id TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL,
  starts_at INTEGER NOT NULL,
  booking_cutoff INTEGER NOT NULL,
  timezone TEXT NOT NULL,
  capacity INTEGER NOT NULL CHECK (capacity > 0 AND capacity <= 500),
  confirmed INTEGER NOT NULL DEFAULT 0 CHECK (confirmed >= 0 AND confirmed <= capacity),
  published INTEGER NOT NULL DEFAULT 0 CHECK (published IN (0, 1)),
  created_at INTEGER NOT NULL
);
CREATE INDEX real_classes_workspace_idx ON real_classes(workspace_id, starts_at);

CREATE TABLE real_bookings (
  id TEXT PRIMARY KEY NOT NULL,
  class_id TEXT NOT NULL REFERENCES real_classes(id) ON DELETE CASCADE,
  idempotency_key TEXT NOT NULL,
  guardian_name TEXT NOT NULL,
  guardian_email TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('confirmed', 'cancelled')),
  created_at INTEGER NOT NULL,
  UNIQUE(class_id, idempotency_key)
);
CREATE INDEX real_bookings_class_idx ON real_bookings(class_id, status);

CREATE TABLE calendar_connections (
  id TEXT PRIMARY KEY NOT NULL,
  workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
  label TEXT NOT NULL,
  provider TEXT NOT NULL CHECK (provider IN ('manual_calendar')),
  enabled INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
  created_at INTEGER NOT NULL,
  UNIQUE(workspace_id)
);

CREATE TABLE reconciliation_runs (
  id TEXT PRIMARY KEY NOT NULL,
  class_id TEXT NOT NULL REFERENCES real_classes(id) ON DELETE CASCADE,
  calendar_confirmed INTEGER NOT NULL CHECK (calendar_confirmed >= 0),
  local_confirmed INTEGER NOT NULL CHECK (local_confirmed >= 0),
  status TEXT NOT NULL CHECK (status IN ('matched', 'attention')),
  created_at INTEGER NOT NULL
);
CREATE INDEX reconciliation_runs_class_idx ON reconciliation_runs(class_id, created_at DESC);

CREATE TABLE waitlist_entries (
  id TEXT PRIMARY KEY NOT NULL,
  class_id TEXT NOT NULL REFERENCES real_classes(id) ON DELETE CASCADE,
  guardian_name TEXT NOT NULL,
  guardian_email TEXT NOT NULL,
  consented_at INTEGER NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('waiting', 'offered', 'accepted', 'expired')),
  created_at INTEGER NOT NULL
);
CREATE INDEX waitlist_queue_idx ON waitlist_entries(class_id, status, created_at);

CREATE TABLE seat_offers (
  id TEXT PRIMARY KEY NOT NULL,
  waitlist_entry_id TEXT NOT NULL UNIQUE REFERENCES waitlist_entries(id) ON DELETE CASCADE,
  class_id TEXT NOT NULL REFERENCES real_classes(id) ON DELETE CASCADE,
  token_hash TEXT NOT NULL UNIQUE,
  expires_at INTEGER NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('open', 'accepted', 'expired')),
  created_at INTEGER NOT NULL
);
CREATE INDEX seat_offers_token_idx ON seat_offers(token_hash);
