# API service scaffold

M1 creates the Rust 2021 Axum service here. The required layout is:

- src/main.rs for config, router, graceful shutdown, structured logs, and build identity;
- src/routes for health, demo, booking, and later protected handlers;
- src/db for SQLx repositories and tenant context;
- migrations for reversible schema changes; and
- tests for service integration.

The service must run on PORT with no other variables, defaulting to a persisted local SQLite database under /data. Production selects PostgreSQL with DATABASE_URL. Every route except health is rate-limited and returns Retry-After on 429. See .factory/plan.md before adding code.
