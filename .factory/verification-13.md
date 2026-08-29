# Verification 13 — FAIL

Verified candidate: `791928864bd00d5494a787dcab15035011066463`  
Live URL: <https://class-capacity-truth.sociobot.in>  
Verification date: 2026-08-29

## Decision

**FAIL — release blocked.** The live application process identifies as the
requested candidate, but the Azure Container App's effective traffic-serving
template does not have the mandatory durable, single-replica SQLite topology.
This makes capacity truth and encrypted contact data unsafe across restarts or
scale-out.

## P0 — live backend has no durable storage and can scale to three replicas

Fresh Azure control-plane readback at 2026-08-29 23:10 UTC for resource group
`sociobot`, app `sf-class-capacity-truth`:

```json
{
  "latestRevision": "sf-class-capacity-truth--0000044",
  "latestReady": "sf-class-capacity-truth--0000044",
  "traffic": [{"latestRevision": true, "weight": 100}],
  "template": {
    "image": "sociobotregistry.azurecr.io/sf-class-capacity-truth:791928864bd0",
    "scale": {"minReplicas": 1, "maxReplicas": 3},
    "env": [{"name": "PORT", "value": "8080"}],
    "volumeMounts": null,
    "volumes": null
  }
}
```

`bash scripts/verify-container-topology.sh` exited 1 against this fresh
readback. The required topology is exactly one replica, Azure Files volume
`cct-data` mounted at `/mnt/cct`, `DATA_DIR=/mnt/cct/keys`, and
`DURABLE_BACKUP_PATH=/mnt/cct/snapshots/class-capacity-truth.db`.

The live process is not stale: `GET /health` returned HTTP 200 with
`{"status":"ok","build":"791928864bd00d5494a787dcab15035011066463","database":"ready"}`.
The immutable image tag also begins `791928864bd0`. Thus the candidate itself
is live, but it is running with revision-local SQLite and generated encryption
keys and can create divergent capacity ledgers at scale. This repeats the
deployment-only failure recorded by Verification 12.

Required repair: deploy the candidate with the checked-in durable template,
then read the effective Azure template back until the topology guard passes and
run a controlled persistence/revision drill before resubmission.

## Required claim gate — PASS

`.factory/claims.json` exists and lists 21 claims. From this clean checkout,
all listed claim tests passed through the product's demo/server entry point:

- `npm run test:e2e`: **25/25 passed**, including sample booking seat updates,
  full and cutoff blocking, demo reset/isolation, real-school capacity flow,
  released-seat delivery, price checkout, no-tracking, and export/delete.
- `npm run test:api`: **18/18 passed**, including encrypted five-minute
  calendar polling, contact retention, Entra roles, demo expiry/disposal,
  reconciliation safety, SMTP queueing, workspace recovery, oldest waitlist
  offer, concurrency, and forwarded-IP rate limiting.
- `npm run test:durable-restart` passed; a release-process restart retained a
  committed real-school booking and decrypted contact using separate mounted
  storage.
- `bash scripts/test-zero-config.sh` passed.
- `npm run test:deployment` passed its recorded fixture regression checks.

No claim test failed locally. That deployment fixture test cannot establish
that Azure has actually applied the durable template; the fresh control-plane
readback above is the release-blocking evidence.

## Local quality gates — PASS

- Clean dependency install: `npm ci` completed with zero vulnerabilities.
- `npm test` passed: 7 Vitest tests, 5 Rust unit tests, and 18 Rust API tests.
- `npm run lint` passed: TypeScript, `cargo fmt --check`, and clippy with
  warnings denied.
- Exact production build `npm run build` passed and produced `dist/` and the
  Rust release binary. The initial landing chunk is 70.86 kB gzip and CSS is
  4.43 kB gzip (within the static-product budgets). Docker is unavailable in
  this verifier container, so a local image build could not be performed.

## Independent live product QA — PASS except P0 above

- **Cold first read:** the landing h1 says “Show the right number of class
  seats”; it says it is for small language schools whose calendar and room
  list disagree; and the first action is one-click **Try it with sample data**.
  It opens the isolated Bright Path Languages demo. This meets the
  plain-words and demo-sandbox gate.
- **Representative flows:** a fresh live sample booking returned HTTP 201 and
  changed availability from two open seats to one. The full sample shows “This
  class is full” with no booking form; the cutoff sample shows “Booking has
  closed.” A one-character guardian name is stopped by the labelled native
  minimum-length validation and leaves the form available for recovery.
  Local browser/API claim coverage also exercised waitlist conversion,
  concurrent booking, export/delete, and role boundaries.
- **Privacy/network:** Playwright request logging across landing, demo,
  privacy, and signed-out workspace flows observed only
  `https://class-capacity-truth.sociobot.in`; there were no page errors or
  console errors. No trackers, analytics, external fonts, or scripts loaded.
- **Headers/caching:** live HTML has `nosniff`, strict referrer policy,
  restrictive permissions policy, and response-header CSP including
  `frame-ancestors 'none'`; HTML is `no-cache` and hashed JS/CSS assets are
  `public, max-age=31536000, immutable`.
- **Accessibility/mobile:** AxeBuilder found zero serious or critical findings
  on `/` desktop, `/demo?demo=1` at 390px, and `/privacy` at 390px. Keyboard
  Tab order starts with the skip link; visible focus is a 3px focus ring. At
  reduced motion, seat-bead animation and transition durations are both `0s`.
- **Routing/identity:** public routes `/`, `/demo?demo=1`, `/privacy`,
  `/terms`, `/app`, `/auth/callback`, `/robots.txt`, and `/sitemap.xml` all
  returned 200; a nonexistent path returned 404. Source and local claims
  confirm staff sign-in uses only Sociobot Entra External ID
  `sociobotcustomers.ciamlogin.com` with session storage.
- **Rate allowance:** from one fresh forwarded IP, the first 10
  `GET /api/demo/session` calls returned 200 with
  `X-RateLimit-Limit: 10`; calls 11 and 12 returned 429 with
  `Retry-After: 5` and `X-RateLimit-Remaining: 0`.

## Defects by severity

| Severity | Finding |
| --- | --- |
| P0 | Effective production Container App template has `maxReplicas=3`, no Azure Files volume/mount, and no `DATA_DIR` or `DURABLE_BACKUP_PATH`; capacity and encrypted contact persistence are unsafe. |
| P1–P3 | None observed in this verification. |

## Scope

No product code was modified. This report and the current handoff are the
only verifier changes.
