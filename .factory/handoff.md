# Repair handoff — class-capacity-truth-repair-1

## Result

This repair closes the verifier findings recorded at `5563fa03da09ec224245e420530d7e6b78c214b9` while preserving the isolated public demo.

- The service now has a durable school workspace path at `/app`: create a class with capacity, cutoff and time zone; publish an opaque parent link; take transactional public bookings; record a calendar count; and keep any disagreement visible as `attention` rather than mutating confirmed seats.
- A full public class accepts a consented waitlist entry. Releasing the oldest confirmed booking creates exactly one opaque 24-hour offer for the oldest waiting entry; `/offer/:token` atomically accepts it once. The database regression test recreates this whole previously missing flow and asserts a third booking cannot oversell.
- Demo tables and cookie tenancy remain separate from school tables. All four original demo claims still pass.
- `Dockerfile` now uses the required unpinned `rust:1-alpine` builder base.
- Hashed `/assets/*` responses have `Cache-Control: public, max-age=31536000, immutable`; application HTML is short-lived. Unknown server paths render the styled 404 document with HTTP `404`, while documented client links still serve the SPA.
- The strict anonymous-demo rate limit remains 10 burst. The multi-step school flow has its own bounded 40-burst limiter so a legitimate setup flow is not blocked by demo creation protection.

## Verification evidence

Run from a clean clone:

```bash
npm ci
npm test
npm run test:api
env -u CI npm run test:e2e
npm run build
```

Executed 2026-08-28 in this repair:

- `npm ci`: 169 packages installed; audit reported 0 vulnerabilities.
- `npm test`: 4 Vitest tests, 3 Rust unit tests, and 7 Rust API/integration tests passed.
- `npm run test:api`: 7/7 passed, including `regression_real_school_flow_configures_books_reconciles_and_converts_released_waitlist_seat`.
- `env -u CI npm run test:e2e`: 17/17 Chromium tests passed. It covers all five declared claims, keyboard flow, 390px/reduced motion, axe serious/critical checks, dark treatment, full browser school workflow, immutable asset headers, and HTTP 404 handling.
- `npm run build`: Vite output is `dist/`; initial JavaScript is 66.77 KB gzip and CSS is 3.88 KB gzip. The release Rust binary built successfully.
- The local container engine is unavailable in this worker, so no local `docker build/run` was possible. The Dockerfile’s base-image correction is checked in source and the regular release binary starts with only `PORT` plus its generated/persisted local configuration.

## Deployment

Deployment class remains **container**. ACR run `chng` built
`sociobotregistry.azurecr.io/sf-class-capacity-truth:44db2b4` successfully,
and Container App `sf-class-capacity-truth` in resource group `sociobot` was
updated to revision `sf-class-capacity-truth--0000005` with min/max replicas
set to 1 for its SQLite single-writer datastore. Live verification at
`https://class-capacity-truth.sociobot.in` returned
`{"status":"ok","build":"44db2b4","database":"ready"}`. The live
hashed JavaScript response has the immutable cache policy and
`/missing-page` returns HTTP 404 with the expected CSP. No DNS, billing, or
external identity configuration was changed.

## Known next steps

The durable SQLite workspace is suitable for the configured single-instance deployment. Shared staff accounts, Entra CIAM callback registration, PostgreSQL/RLS tenancy, encrypted guardian fields, and the registered Sociobot subscription entitlement remain the next planned production-hardening milestone in `.factory/plan.md`; this repair does not pretend those external registrations occurred.
