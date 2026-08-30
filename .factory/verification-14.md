# Verification 14 — FAIL

Candidate: `b8349a9ffdf7985edc0331faf6bd2b5a1db7fb44`

Live URL: <https://class-capacity-truth.sociobot.in>

Verified: 2026-08-30 00:48 UTC

Work order: `class-capacity-truth-verify-14`

## Decision

**FAIL — release blocked.** The earlier deployment-only durability failure is
fixed: production serves the exact candidate build on one healthy replica with
the required Azure Files mount. The required first-read and demo gates pass,
all 21 declared claim commands pass, and the core booking behavior is sound.

Two independent P1 acceptance gaps remain:

1. The required protected operational metrics do not exist. Both `/metrics`
   and `/api/metrics` return the product's HTML 404, there is no metrics route
   or metrics implementation in the backend, and no metrics baseline test
   exists. This contradicts the venture contract and the shipped plan's claim
   that metrics cover requests/errors/latency, job lag, discrepancies, and
   offer conversion.
2. The product-specific app routes recorded as shipped in `.factory/plan.md`
   do not work as address-bar or reloadable routes. `/app/classes/example`,
   `/app/reconciliation`, `/app/waitlist`, `/app/settings`,
   `/app/settings/billing`, `/app/operations`, and `/app/settings/data` all
   return HTTP 404. The implemented controls are consolidated on `/app`, but
   that does not satisfy the plan or the site-structure deep-link contract.

No product code was changed during verification.

## Required first gates

### Cold first read — PASS

At 1440×900 in a fresh Chromium context, the first screen says:

- what it does: **“Show the right number of class seats”**;
- for whom: **“For small language schools”** whose calendar and room list
  disagree about places; and
- what to click: **“Try it with sample data”**, followed by “Three sample
  classes open next.”

The action is visible without scrolling. One click opens
`/demo?demo=1`, immediately shows three realistic Bright Path Languages
classes, and displays the persistent **“Demo — sample data, nothing is saved”**
banner with **Reset demo** and **Start for real**.

Evidence:

- [Cold desktop screenshot](evidence-14/live-first-read-desktop.png)
- [Demo after one click](evidence-14/live-demo-after-one-click.png)

### Claims gate — PASS (21/21)

`.factory/claims.json` exists. Every listed command was run independently from
the clean candidate checkout, not inferred from a broader suite. All exited 0.

| Claim | Exact command | Result |
| --- | --- | --- |
| `sample-booking-updates-seats` | `npm run test:e2e -- --grep @claim:sample-booking-updates-seats` | PASS, 1/1 |
| `full-class-blocks-booking` | `npm run test:e2e -- --grep @claim:full-class-blocks-booking` | PASS, 1/1 |
| `cutoff-blocks-booking` | `npm run test:e2e -- --grep @claim:cutoff-blocks-booking` | PASS, 1/1 |
| `demo-reset-isolated` | `npm run test:e2e -- --grep @claim:demo-reset-isolated` | PASS, 1/1 |
| `school-capacity-flow` | `npm run test:e2e -- --grep @claim:school-capacity-flow` | PASS, 1/1 |
| `calendar-poll` | `npm run test:api -- claim_calendar_feed_is_encrypted_and_polled_every_five_minutes` | PASS, 1/1 |
| `released-seat-delivery` | `npm run test:e2e -- --grep @claim:released-seat-delivery` | PASS, 1/1 |
| `school-plan-price` | `npm run test:e2e -- --grep @claim:school-plan-price` | PASS, 1/1 |
| `no-third-party-tracking` | `npm run test:e2e -- --grep @claim:no-third-party-tracking` | PASS, 1/1 |
| `contact-encryption-retention` | `npm run test:api -- claim_contact_encryption_and_retention` | PASS, 1/1 |
| `staff-role-access` | `npm run test:api -- claim_staff_roles_enforce_owner_actions` | PASS, 1/1 |
| `data-export-delete` | `npm run test:e2e -- --grep @claim:data-export-delete` | PASS, 1/1 |
| `demo-expiry-input-disposal` | `npm run test:api -- claim_demo_expiry_and_input_disposal` | PASS, 1/1 |
| `reconciliation-does-not-change-seats` | `npm run test:api -- claim_reconciliation_never_mutates_confirmed_seats` | PASS, 1/1 |
| `durable-restart` | `npm run test:durable-restart` | PASS; committed booking and decrypted contact survived process restart |
| `configured-smtp-delivery` | `npm run test:api -- claim_configured_smtp_queues_an_encrypted_offer` | PASS, 1/1 |
| `workspace-recovery` | `npm run test:api -- claim_workspace_recovery_by_staff_identity` | PASS, 1/1 |
| `oldest-waitlist-offer` | `npm run test:api -- claim_oldest_waitlist_entry_gets_a_24_hour_offer` | PASS, 1/1 |
| `zero-config-runtime` | `bash scripts/test-zero-config.sh` | PASS |
| `forwarded-ip-rate-limits` | `npm run test:api -- rate_limit_uses_forwarded_ip_and_returns_retry_after` | PASS, 1/1 |
| `durable-one-replica-topology` | `npm run test:deployment` | PASS, both deployment regressions |

## Clean local verification

- Initial `git status --short` was empty and `git rev-parse HEAD` was the exact
  candidate SHA.
- `npm ci`: PASS; 170 packages installed, 0 vulnerabilities.
- `npm test`: PASS; 7 Vitest tests, 5 Rust unit tests, 18 Rust API/integration
  tests, and both deployment regression scripts passed.
- `npm run typecheck`: PASS.
- `npm run lint`: PASS, including `cargo fmt --check` and clippy with warnings
  denied.
- `npm run build`: PASS and produced `dist/` plus the release API binary.
  Initial JS is 231.25 kB raw / 70.86 kB gzip; CSS is 19.37 kB raw / 4.43 kB
  gzip. The lazy staff/auth chunk is 316.56 kB raw / 79.59 kB gzip.
- `npm run test:e2e`: exited 0 with 24 tests passing normally and one passing
  only on retry. The reproducible flake is documented below.

## Live product evidence

### Core and boundary behavior — PASS

- A fresh open class showed two seats. A valid sample booking returned HTTP
  201 and changed the UI to “1 seat is now open in this class.”
- A one-character guardian name was rejected by the labelled native control,
  focused that field, explained that at least two characters are required,
  sent no booking request, and remained recoverable. Correcting it completed
  the booking.
- The full class exposed no booking button and a direct booking request
  returned `409 {"code":"class_full"}`.
- The cutoff class exposed no booking button and a direct booking request
  returned `409 {"code":"booking_closed"}`.
- Reset returned the open class to two seats. Two other fresh browser contexts
  also independently saw two seats.
- Live contention was safe: from six confirmed seats, one preliminary booking
  succeeded; two simultaneous requests for the last seat produced exactly one
  201 and one 409, leaving eight confirmed rather than overselling.
- Local claim coverage also completed the authenticated school workflow:
  create, publish, calendar check, real booking, waitlist, selected
  cancellation, durable copyable offer, reload, and offer acceptance.

[Booking confirmation screenshot](evidence-14/live-booking-success.png)

### Identity, billing, and tenancy — PASS within available credentials

- Live **Sign in with Sociobot** redirected only to
  `sociobotcustomers.ciamlogin.com`, tenant
  `35c6fe40-0ec0-46b6-98c6-213ad4de6650`, client
  `25c704f4-465a-47af-80ab-2c489466b697`, callback
  `/auth/callback`, scopes `openid profile email`, and an S256 PKCE challenge.
- Fresh OIDC discovery returned the GUID-based issuer and the expected
  Sociobot JWKS endpoint. A signed-out protected request returned 401 with
  `WWW-Authenticate: Bearer`.
- The live $99 action sent POST to the Sociobot billing API, received HTTP 200,
  and navigated to an HTTP 200
  `checkout.dodopayments.com/session/...` page titled “Sociobot | Checkout.”
  No card was submitted, so purchase completion and post-payment entitlement
  were not exercised.
- API tests passed demo cookie isolation, server-side roles, owner-only data
  controls, stable-identity workspace recovery, encrypted storage, retention,
  and cross-request concurrency.

### Privacy and security — PASS

- Playwright request logging across landing, demo booking/reset, privacy, and
  signed-out workspace observed only
  `https://class-capacity-truth.sociobot.in`. The only external requests were
  after explicit sign-in or checkout actions.
- There were no console errors or page errors in desktop or mobile product
  flows. No third-party font, script, tracker, or analytics request was seen
  before an explicit external action.
- Live headers include `X-Content-Type-Options: nosniff`,
  `Referrer-Policy: strict-origin-when-cross-origin`, a restrictive
  `Permissions-Policy`, and response-header CSP with
  `frame-ancestors 'none'`. Unapproved-origin preflight received no
  `Access-Control-Allow-Origin`; the production origin did.
- HTML and API responses use `Cache-Control: no-cache, max-age=0`; hashed
  assets use `public, max-age=31536000, immutable`. HTTP redirects to HTTPS.

### Accessibility, responsive behavior, and performance — PASS

- `/opt/fleet/lib/verify-url.sh` passed: load 614 ms, title present, `lang=en`,
  one h1, one main landmark, no missing image alt, no unnamed buttons, and no
  browser errors. Evidence is in [verify-url](evidence-14/verify-url/verify.json).
- Playwright Axe found **zero violations at any impact level** on `/`, demo,
  `/app`, `/privacy`, `/terms`, and the real 404 page.
- Keyboard Tab focused the skip link first. Its visible focus style is a solid
  3 px ring with 3 px offset. At 390 px, Enter opens the labelled menu and
  Escape closes it and restores focus.
- At 390 px in dark mode with reduced motion, 200% text caused no document
  overflow, tested controls met 44 px targets, and seat-bead animation and
  transition durations were both `0s`.
- Fresh mobile Lighthouse: performance 98, accessibility 100, best practices
  100, SEO 100; LCP 1,422 ms, CLS 0, TBT 139 ms, transfer 76,820 bytes.
- Visual evidence: [mobile dark/reduced-motion](evidence-14/live-mobile-dark-reduced.png)
  and [Lighthouse JSON](evidence-14/lighthouse-mobile.json).

### Deployment, durability, and rate limiting — PASS

- `/health` returned HTTP 200 with
  `{"status":"ok","build":"b8349a9ffdf7985edc0331faf6bd2b5a1db7fb44","database":"ready"}`.
- Local `dist/index.html`, JS, and CSS SHA-256 hashes exactly matched the live
  responses. The live image tag is `b8349a9ffdf7`.
- Fresh Azure readback showed revision
  `sf-class-capacity-truth--r13-b8349a9-20260830` ready and receiving 100% of
  traffic; `minReplicas=1`, `maxReplicas=1`; Azure Files volume `cct-data`
  mounted at `/mnt/cct`; `DATA_DIR=/mnt/cct/keys`; and
  `DURABLE_BACKUP_PATH=/mnt/cct/snapshots/class-capacity-truth.db`.
- The durable restart claim passed locally from the release process. A live
  infrastructure restart was not performed because this verification was
  non-mutating; the fresh mount/topology readback establishes the repaired
  production boundary.
- From one fresh forwarded client IP, requests 1–10 to
  `GET /api/demo/session` returned 200. Requests 11 and 12 returned 429 with
  `Retry-After: 5`, `X-RateLimit-Limit: 10`, and remaining 0. `/health` is the
  documented exemption. Protected API responses expose a separate burst
  allowance of 40.

### Route and artifact checks

The implemented public routes `/`, `/demo?demo=1`, `/app`, `/privacy`,
`/terms`, `/auth/callback`, `/robots.txt`, and `/sitemap.xml` return 200. A
missing page returns 404. Every implemented HTML route has a route-specific
title, canonical URL, description, `lang=en`, one h1, and one main landmark.
Internal links crawled from those pages returned 200.

This is not a library/CLI and has no package-consumer check. It does not claim
PWA/offline behavior and registers no service worker, so PWA update/offline
tests do not apply.

## Defects by severity

### P1 — no operational metrics endpoint or implementation

The venture acceptance bar requires health and metrics endpoints. The shipped
plan at `.factory/plan.md:126` specifically says protected metrics measure
requests/errors/latency, job lag, discrepancies, and offer conversion; M4 at
lines 184–194 is marked shipped and requires a metrics baseline. Fresh probes:

```text
/metrics      404 text/html; charset=utf-8
/api/metrics  404 text/html; charset=utf-8
```

`services/api/src/lib.rs:87-175` enumerates every API and service route and has
only `/health`; repository search finds no metrics implementation or metrics
dependency. Operators therefore cannot observe the stated error budget or the
business/reconciliation signals required by this venture contract.

Required repair: add an authenticated or otherwise safely protected metrics
endpoint, instrument the promised request/error/latency and domain/job values,
document access and alert thresholds, and add a no-PII metrics baseline test.

### P1 — shipped app deep links return 404

The plan records routes for class detail, reconciliation, waitlist, settings,
billing, operations, and data controls, and the site-structure contract
requires real URLs that survive reload. Fresh direct requests all returned
404:

```text
/app/classes/example   404
/app/reconciliation    404
/app/waitlist          404
/app/settings          404
/app/settings/billing  404
/app/operations        404
/app/settings/data     404
```

The server only mounts the SPA at exact `/app`. Some controls are usable on
that monolithic screen, but bookmarks, reloads, shared links, and the declared
information architecture fail.

Required repair: implement the declared client routes and matching server
fallbacks (or correct the plan before claiming those milestones shipped), with
route-specific title/focus/back-forward tests and direct-load coverage.

### P2 — mobile E2E check is reproducibly flaky

The normal `npm run test:e2e` run reported one flaky test and 24 clean passes;
the retry passed. With retries disabled, this stress command:

```text
CI= npm run test:e2e -- --grep 'demo remains usable at 390px' --repeat-each=10
```

failed 2 of 10 runs at `e2e/m1.spec.ts:136`. The assertion checks that the
loading marker is absent immediately after only the static h1 and Reset button
are visible. Failure snapshots show the valid designed loading state, and the
live page subsequently loads correctly. This is a test synchronization defect,
not a demonstrated user-facing failure, but retries currently hide it.

Required repair: wait for a stable data-ready condition (for example, three
class articles or the open-class count) before asserting the loading state has
gone; then run the case repeatedly without retries.

## Final disposition

The deployment repair is real, and the capacity product itself performs well.
It is nevertheless **not accepted** until the two P1 contract gaps are fixed
and independently reverified. The P2 flaky test should be repaired in the same
cycle so the browser gate is deterministic.
