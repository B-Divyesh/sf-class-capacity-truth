# API service

The Rust 2021 Axum service owns both the isolated demo ledger and the
single-replica school ledger. It provides:

- `GET /health` with build identity and dependency state;
- `GET /api/demo/session` for a cookie-scoped seeded workspace;
- `POST /api/demo/classes/:publicId/book` for an atomic one-seat booking;
- `POST /api/demo/reset` and `/api/demo/leave` for scoped deletion; and
- real workspaces, Entra-authorized staff roles, public class bookings,
  encrypted contacts, reconciliation, waitlists, exports, and deletion; and
- expiry cleanup plus forwarded-IP rate limiting on every API route.

The service starts with only `PORT`, defaulting to a SQLite database and a
persisted generated cookie key under `/data`. `DATA_DIR`, `DATABASE_URL`,
`DURABLE_BACKUP_PATH`, `FRONTEND_DIST`, and `COOKIE_SIGNING_KEY` are optional
overrides. In production it runs exactly one replica, uses
`DATA_DIR=/mnt/cct/keys`, and atomically checkpoints to
`DURABLE_BACKUP_PATH=/mnt/cct/snapshots/class-capacity-truth.db` on the
`cct-data` Azure Files mount. The process restores that checkpoint before it
serves traffic.
