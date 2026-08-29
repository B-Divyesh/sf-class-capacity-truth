# Verification 7 handoff — FAIL

Candidate: `023bc90148efd22542aa1fb99c81588686e7aac4`

Live URL: <https://class-capacity-truth.sociobot.in>

Verified: 2026-08-29 UTC

Decision: **FAIL — do not accept real school data**

## Release blocker

The live artifact matches the candidate and `/health` reports the full
candidate SHA, but active Azure revision
`sf-class-capacity-truth--0000036` still has `maxReplicas: 3`, only
`PORT=8080`, and no volume or mount. Startup reports durable backup disabled
and newly generated cookie/contact keys. The existing `cct-data` Azure Files
environment storage is not attached.

This makes the real-school SQLite ledger and encryption keys ephemeral. A
replacement loses data; scale-out can create conflicting ledgers. The local
deployment fixture and durable restart claim pass, but the repaired topology
was not applied to the active revision.

Required operator repair:

1. Attach `cct-data` at `/mnt/cct`.
2. Set `DATA_DIR=/mnt/cct/keys` and
   `DURABLE_BACKUP_PATH=/mnt/cct/snapshots/class-capacity-truth.db`.
3. Set `minReplicas: 1` and `maxReplicas: 1`.
4. Read the effective Azure template back after deployment.
5. Prove one real booked record and its decrypted contact survive a new
   production revision, then remove the synthetic workspace.

## Verification summary

- All 21 `.factory/claims.json` commands passed from the clean candidate
  checkout. The live topology nevertheless disproves the topology claim.
- `npm ci`, `npm test`, `npm run typecheck`, `npm run lint`, `npm run build`,
  `npm run test:e2e` (24/24), and `npm run test:cold-claim` (205 s) passed.
- Live sample booking/reset, full/cutoff rejection, invalid input recovery,
  privacy request capture, security headers, caching, link crawl, desktop,
  390px mobile, keyboard, 200% text, reduced motion, and dark mode passed.
- Live rate limits: demo request 11 returned 429 with `Retry-After: 5` after a
  10-request allowance; school request 41 returned 429 after a 40-request
  allowance. The 100-request smoke was 10 accepted / 90 rate-limited.
- Axe found no serious/critical findings. Lighthouse mobile scored 100 in
  performance, accessibility, best practices, and SEO; LCP 1.3 s, TBT 30 ms,
  CLS 0.
- The Entra redirect uses the required Sociobot CIAM tenant, client ID,
  production callback, and PKCE S256. The live Sociobot checkout returned a
  hosted Dodo session URL.
- Docker image build was not run because no Docker-compatible runtime exists
  in this verifier. No product code was modified.

## Secondary documentation finding

P2: `.factory/plan.md` contradicts itself about delivered milestones and
PostgreSQL versus the shipped single-replica SQLite/Azure Files architecture.
Align it when repairing the deployment.

Full evidence and commands are in
[`.factory/verification-7.md`](verification-7.md).
