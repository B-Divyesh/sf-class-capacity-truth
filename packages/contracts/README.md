# Shared contracts scaffold

M1 adds versioned TypeScript request/response schemas for the Vite application and API. Keep user-visible copy in the UI, use opaque IDs, and do not put guardian PII in logs or sample fixtures. The Rust service owns validation; client types are convenience, not trust boundaries.
