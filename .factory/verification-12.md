# Verification 12 — FAIL

Verified candidate: `28fcd19f33b513f4a3b365be90bda7ec457340c7`  
Live URL: <https://class-capacity-truth.sociobot.in>  
Verification date: 2026-08-29

## Decision

**FAIL — release blocked.** The requested commit is serving live, but the
effective Azure Container App template lacks the mandatory single-replica,
durable SQLite topology. That leaves capacity truth and encrypted data unsafe.

## P0 — unsafe live topology

Fresh Azure control-plane readback at 2026-08-29 21:38 UTC for resource group
`sociobot`, app `sf-class-capacity-truth`, returned:

```text
revision: sf-class-capacity-truth--0000043
ready revision: sf-class-capacity-truth--0000043
image: sociobotregistry.azurecr.io/sf-class-capacity-truth:28fcd19f33b5
traffic: latest revision, 100%
scale: minReplicas=1, maxReplicas=3
environment: PORT=8080 only
volumes: null
volumeMounts: null
```

Live `GET /health` returned HTTP 200:

```json
{"status":"ok","build":"28fcd19f33b513f4a3b365be90bda7ec457340c7","database":"ready"}
```

Thus this is the requested candidate, not a stale deployment. The repository's
`scripts/verify-container-topology.sh` fails against this live readback. The
contract requires `maxReplicas=1`, Azure Files `cct-data` mounted at `/mnt/cct`,
`DATA_DIR=/mnt/cct/keys`, and
`DURABLE_BACKUP_PATH=/mnt/cct/snapshots/class-capacity-truth.db`.

Impact: SQLite and generated cookie/contact-encryption keys are
revision-local/ephemeral. A restart can lose or make data unreadable; scale-out
can yield divergent availability and oversell seats. This breaks the product's
core capacity, privacy, and backend-service contracts.

Required fix: deploy the durable one-replica topology, read the effective
template back until the guard passes, then run the controlled production
durability drill.

## Required claim gate — PASS

`.factory/claims.json` exists with 21 claims. From this clean checkout every
exact declared command passed: the booking/full/cutoff/demo claims; real school
capacity, released-seat, price, privacy, data-rights claims; all encryption,
retention, role, calendar, reconciliation, SMTP, recovery, and waitlist API
claims; durable restart; zero-config runtime; forwarded-IP rate limiting; and
the deployment topology regression claim. No declared claim command failed.

## Other QA evidence — PASS

- Cold first read passes: the live h1 says “Show the right number of class
  seats,” names small language schools whose calendar and room list disagree,
  and exposes one-click **Try it with sample data**. It opens isolated Bright
  Path Languages data.
- `npm ci`, `npm test` (7 Vitest and 23 Rust tests), `npm run lint`,
  `npm run build`, and the full `npm run test:e2e` (25/25) passed. `dist/` and
  the release binary were produced. Docker is unavailable in this verifier
  image, so no local Docker build was possible.
- A fresh live sample booking returned 201 and changed availability from two
  seats to one; reset restored two. Full and cutoff samples expose no booking
  action. Local browser/API coverage additionally exercised invalid input,
  recovery, waitlist conversion, role boundaries, data deletion/export, and
  concurrent booking.
- Local release load smoke: 100 requests, 10 accepted and 90 rate-limited.
  The durable restart claim passed with booking and encrypted contact surviving
  a release-process restart on separate mounted storage.
- Live rate limit: ten `GET /api/demo/session` calls from one fresh forwarded
  IP returned 200; request 11 returned 429 with `Retry-After: 5`,
  `X-RateLimit-Limit: 10`, and `X-RateLimit-Remaining: 0`.
- Fresh Playwright logging across landing, demo, app, privacy, and terms saw
  only same-origin requests. No tracker or third-party font/script loaded.
  CSP is a response header with `frame-ancestors 'none'`; nosniff, strict
  referrer policy, permissions policy, HTML no-cache, and immutable hashed
  asset caching are present.
- Live axe found zero serious/critical issues on `/`, `/demo?demo=1`, `/app`,
  `/privacy`, and `/terms`. At 390px/reduced motion there was no horizontal
  overflow; the skip link and route-focus behavior worked. A clean landing
  navigation had no console/page errors.
- Fresh live Lighthouse: Performance 100, Accessibility 100, Best Practices
  100, SEO 100; LCP 1.277 s, CLS 0, TBT 24.5 ms. Initial JS is 70.86 kB gzip;
  CSS is 4.43 kB gzip.
- The implementation uses only Sociobot Entra External ID
  `sociobotcustomers.ciamlogin.com`; no other sign-in provider was found.

## Scope

No product source was changed. This report and the handoff are the only
verifier changes. The P0 deployment defect alone blocks release.
