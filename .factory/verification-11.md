# Verification 11 — FAIL

Verified candidate: `89d35c47ee376d75d92c42b7c839f6da323e35b3`
Live URL: <https://class-capacity-truth.sociobot.in>
Verification date: 2026-08-29

## Decision

**FAIL — release blocked.** The deployed revision serves the requested commit, but its live Container App topology is not durable and can scale the SQLite application to multiple replicas.

## P0 release blocker

Fresh Azure control-plane readback, not a fixture:

```text
revision: sf-class-capacity-truth--0000042
image: sociobotregistry.azurecr.io/sf-class-capacity-truth:89d35c47ee37
traffic: 100%
scale: minReplicas=1, maxReplicas=3
volumes: null
volumeMounts: null
environment: PORT=8080 only
```

`/health` simultaneously returned build `89d35c47ee376d75d92c42b7c839f6da323e35b3`, so this is the candidate actually receiving traffic. `scripts/verify-container-topology.sh` exits 1 against this live app. The mandatory deployment contract requires exactly one replica, an Azure Files `cct-data` mount at `/mnt/cct`, `DATA_DIR=/mnt/cct/keys`, and `DURABLE_BACKUP_PATH=/mnt/cct/snapshots/class-capacity-truth.db`.

Impact: SQLite, generated cookie keys, and contact-encryption keys are revision-local/ephemeral; a restart can lose state and a scale-out to up to three replicas can return inconsistent seat counts or let competing replicas oversell. This directly violates the product's capacity-truth and privacy promises.

Required fix: deploy the candidate through `scripts/deploy-container.sh` (or equivalent) and read the effective Azure template back until it has the one-replica Azure Files topology above. Then repeat a controlled live persistence/revision drill before release.

## Required claim gate — PASS

`.factory/claims.json` exists with 21 claims. From the clean checkout, every exact declared command passed through the product demo entry point:

`sample-booking-updates-seats`, `full-class-blocks-booking`, `cutoff-blocks-booking`, `demo-reset-isolated`, `school-capacity-flow`, `calendar-poll`, `released-seat-delivery`, `school-plan-price`, `no-third-party-tracking`, `contact-encryption-retention`, `staff-role-access`, `data-export-delete`, `demo-expiry-input-disposal`, `reconciliation-does-not-change-seats`, `durable-restart`, `configured-smtp-delivery`, `workspace-recovery`, `oldest-waitlist-offer`, `zero-config-runtime`, `forwarded-ip-rate-limits`, and `durable-one-replica-topology`.

The extra cold-cache check also passed: `npm run test:cold-claim` completed in 209 seconds (limit 600 seconds).

## Product and quality evidence — PASS

- First-read, cold live page: “Show the right number of class seats”; it says this is for small language schools whose booking calendar and room list disagree; the first action is the visible one-click **Try it with sample data**. It leads to the isolated fictional Bright Path Languages demo.
- Local checks passed: `npm ci`, `npm test` (7 frontend + 23 Rust tests), `npm run test:e2e` (25 tests), `npm run lint`, `npm run build`, and all claim commands above. Production build produced `dist/` and a release API binary.
- End-to-end coverage includes normal booking, full and cutoff rejection, demo reset/isolation, input constraint recovery, owner flow, calendar reconciliation without seat mutation, waitlist offer acceptance, role/data controls, concurrent booking race, zero-config boot, and release-process durable restart.
- Live `/health` was 200 with `status: ok`, `database: ready`, and the candidate SHA. Live routes `/`, `/demo?demo=1`, `/app`, `/privacy`, `/terms`, `/auth/callback`, `robots.txt`, and `sitemap.xml` were 200; an unknown route was a real 404.
- Desktop and 390px mobile/reduced-motion checks passed: visible labelled mobile menu, keyboard use, no horizontal overflow, route focus, and 44px targets in the browser suite. Axe found no serious/critical findings on landing, demo, booking, app, privacy, terms, and 404 routes. No page or console errors were observed.
- A fresh live Playwright request log across landing/demo/privacy/app observed only same-origin requests; no analytics/tracking request was observed. The demo page itself made only its same-origin session request. No third-party fonts/scripts load.
- Live headers include `X-Content-Type-Options: nosniff`, strict referrer policy, restrictive permissions policy, and response-header CSP with `frame-ancestors 'none'`. Hashed JS has `Cache-Control: public, max-age=31536000, immutable`; HTML is no-cache. First-load JavaScript is 70.86 kB gzip for the loaded application chunk.
- Live Lighthouse: performance 100, accessibility 100, best practices 100, SEO 100; LCP 1.3 s, CLS 0, TBT 0 ms.
- Rate-limit check against live `GET /api/demo/session`, using one distinct `X-Forwarded-For` client: requests 1–10 returned 200; requests 11–12 returned 429. The 429 contained `Retry-After: 5`, `X-RateLimit-Limit: 10`, and `X-RateLimit-Remaining: 0`.
- Auth implementation is scoped to `https://sociobotcustomers.ciamlogin.com/...` through MSAL; no other sign-in provider is present.

## Scope note

No product source code was modified. Docker was unavailable in this verifier container, so the Docker image build itself was not rerun; the repository's exact application production build, deployment fixture tests, live runtime identity, and live control-plane topology were independently checked. The control-plane P0 is sufficient to reject this candidate.
