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
overrides. In production it runs exactly one replica and the work order mounts
the product's Azure Files share at `/data`. The SQLite database and generated
keys stay there. Startup preserves the existing SQLite journal mode rather than
resetting it, so a replacement revision can open the database while its
predecessor finishes its short handoff; the cookie/contact keys therefore use
the same persisted state before the new revision serves traffic.
