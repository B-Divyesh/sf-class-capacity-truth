# API service

The Rust 2021 Axum service owns the temporary M1 demo ledger. It provides:

- `GET /health` with build identity and dependency state;
- `GET /api/demo/session` for a cookie-scoped seeded workspace;
- `POST /api/demo/classes/:publicId/book` for an atomic one-seat booking;
- `POST /api/demo/reset` and `/api/demo/leave` for scoped deletion; and
- expiry cleanup plus forwarded-IP rate limiting on every API route.

The service starts with only `PORT`, defaulting to a SQLite database and a
persisted generated cookie key under `/data`. `DATA_DIR`, `DATABASE_URL`,
`FRONTEND_DIST`, and `COOKIE_SIGNING_KEY` are optional overrides. M1 has no
durable customer tenant; PostgreSQL tenancy begins in M2 as specified by the
plan.
