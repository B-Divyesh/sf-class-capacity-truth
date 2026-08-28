PRAGMA foreign_keys = ON;

CREATE TABLE demo_tenants (
  id TEXT PRIMARY KEY NOT NULL,
  created_at INTEGER NOT NULL,
  expires_at INTEGER NOT NULL
);

CREATE TABLE class_sessions (
  id TEXT PRIMARY KEY NOT NULL,
  demo_tenant_id TEXT NOT NULL REFERENCES demo_tenants(id) ON DELETE CASCADE,
  public_id TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL,
  starts_at INTEGER NOT NULL,
  booking_cutoff INTEGER NOT NULL,
  timezone TEXT NOT NULL,
  capacity INTEGER NOT NULL CHECK (capacity > 0),
  confirmed INTEGER NOT NULL CHECK (confirmed >= 0 AND confirmed <= capacity),
  sort_order INTEGER NOT NULL,
  UNIQUE (demo_tenant_id, name)
);

CREATE INDEX class_sessions_tenant_idx ON class_sessions(demo_tenant_id, sort_order);

CREATE TABLE bookings (
  id TEXT PRIMARY KEY NOT NULL,
  demo_tenant_id TEXT NOT NULL REFERENCES demo_tenants(id) ON DELETE CASCADE,
  class_session_id TEXT NOT NULL REFERENCES class_sessions(id) ON DELETE CASCADE,
  idempotency_key TEXT NOT NULL,
  guardian_name TEXT NOT NULL,
  guardian_email TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  UNIQUE (demo_tenant_id, idempotency_key)
);

CREATE INDEX bookings_session_idx ON bookings(class_session_id);

