# Independent verification 7 — FAIL

Verified 2026-08-29 UTC against candidate commit
`023bc90148efd22542aa1fb99c81588686e7aac4` and
<https://class-capacity-truth.sociobot.in>.

## Verdict

**FAIL — do not accept real school data.** The candidate source, claims,
one-click demo, live UI, accessibility, privacy behavior, security headers,
identity redirect, rate limits, and performance checks pass. The active Azure
revision still stores the SQLite database and generated encryption/signing keys
only on disposable container storage and may scale to three replicas.

This is fresh evidence from the active candidate revision, not a carry-over of
the prior verifier result. The repair is present and tested in source, but it is
not applied to the live Container App template.

## Release-blocking defect

### P0 — the active candidate revision is still ephemeral and may scale SQLite to three replicas

The live `/health` response is HTTP 200 and identifies the exact requested
candidate:

```json
{"status":"ok","build":"023bc90148efd22542aa1fb99c81588686e7aac4","database":"ready"}
```

A fresh read-only Azure control-plane query on 2026-08-29 found active revision
`sf-class-capacity-truth--0000036` and image
`sociobotregistry.azurecr.io/sf-class-capacity-truth:023bc90148ef`. It also
found:

```text
minReplicas: 1
maxReplicas: 3
environment: PORT=8080 only
volumeMounts: null
volumes: null
```

The `cct-data` environment storage exists and points to the intended
`class-capacity-truth` Azure Files share, but the active revision does not mount
it. Its fresh startup log says:

```text
database_config="generated-default"
durable_backup="disabled"
cookie_signing_key="generated-and-persisted"
contact_encryption_key="generated-and-persisted"
```

Consequences:

- a replacement replica loses real workspaces, classes, bookings, waitlists,
  offer receipts, and billing state;
- regenerated keys make any surviving encrypted contact data unreadable;
- scale-out can create independent SQLite ledgers and keys, so public seat
  counts may disagree and rate-limit allowances multiply per replica.

The checked-in deployment contract and deployment regression correctly require
one replica, `/mnt/cct`, `DATA_DIR=/mnt/cct/keys`, and
`DURABLE_BACKUP_PATH=/mnt/cct/snapshots/class-capacity-truth.db`. Local tests
prove that contract and the release-process restart. They do not prove that the
live control plane accepted it. The active Azure template is the acceptance
failure.

Required repair: deploy the candidate with the existing `cct-data` environment
storage mounted at `/mnt/cct`, set both durable paths, set min/max replicas to
one, read the effective template back, then prove a real booked record and its
decrypted contact survive a new production revision.

## Other finding

### P2 — the venture plan contradicts the shipped architecture and milestone status

`.factory/plan.md` says in its opening status that M2/M3 core and M4 controls
were delivered, while its milestone table still marks M2 through M5 as
`Planned`. Its architecture section also names PostgreSQL as production while
the same section, README, deployment contract, and implementation use
single-replica SQLite with Azure Files checkpoints. This does not cause the
runtime failure, but it makes the operating contract ambiguous and should be
corrected with the deployment repair.

## Mandatory first-read and demo gate

**PASS.** In a cold 1440×900 browser, the first screen answers all three
questions in plain words:

- What it does: “Show the right number of class seats.”
- For whom: small language schools whose booking calendar and room list
  disagree about places.
- What to click first: “Try it with sample data,” with “Three sample classes
  open next.” immediately beside it.

One click opens `/demo?demo=1` with realistic available, full, and cutoff
classes. The persistent banner says “Demo — sample data, nothing is saved” and
provides **Reset demo** and **Start for real**. Starting for real removes the
signed demo cookie and returns to the school-plan section.

## Claims gate

`.factory/claims.json` exists with 21 entries. Immediately after `npm ci`, every
listed command was executed separately from this clean candidate checkout.
All passed:

| Claim ID | Exact test | Result |
| --- | --- | --- |
| `sample-booking-updates-seats` | `npm run test:e2e -- --grep @claim:sample-booking-updates-seats` | PASS |
| `full-class-blocks-booking` | `npm run test:e2e -- --grep @claim:full-class-blocks-booking` | PASS |
| `cutoff-blocks-booking` | `npm run test:e2e -- --grep @claim:cutoff-blocks-booking` | PASS |
| `demo-reset-isolated` | `npm run test:e2e -- --grep @claim:demo-reset-isolated` | PASS |
| `school-capacity-flow` | `npm run test:e2e -- --grep @claim:school-capacity-flow` | PASS |
| `calendar-poll` | `npm run test:api -- claim_calendar_feed_is_encrypted_and_polled_every_five_minutes` | PASS |
| `released-seat-delivery` | `npm run test:e2e -- --grep @claim:released-seat-delivery` | PASS |
| `school-plan-price` | `npm run test:e2e -- --grep @claim:school-plan-price` | PASS |
| `no-third-party-tracking` | `npm run test:e2e -- --grep @claim:no-third-party-tracking` | PASS |
| `contact-encryption-retention` | `npm run test:api -- claim_contact_encryption_and_retention` | PASS |
| `staff-role-access` | `npm run test:api -- claim_staff_roles_enforce_owner_actions` | PASS |
| `data-export-delete` | `npm run test:e2e -- --grep @claim:data-export-delete` | PASS |
| `demo-expiry-input-disposal` | `npm run test:api -- claim_demo_expiry_and_input_disposal` | PASS |
| `reconciliation-does-not-change-seats` | `npm run test:api -- claim_reconciliation_never_mutates_confirmed_seats` | PASS |
| `durable-restart` | `npm run test:durable-restart` | PASS locally |
| `configured-smtp-delivery` | `npm run test:api -- claim_configured_smtp_queues_an_encrypted_offer` | PASS |
| `workspace-recovery` | `npm run test:api -- claim_workspace_recovery_by_staff_identity` | PASS |
| `oldest-waitlist-offer` | `npm run test:api -- claim_oldest_waitlist_entry_gets_a_24_hour_offer` | PASS |
| `zero-config-runtime` | `bash scripts/test-zero-config.sh` | PASS |
| `forwarded-ip-rate-limits` | `npm run test:api -- rate_limit_uses_forwarded_ip_and_returns_retry_after` | PASS |
| `durable-one-replica-topology` | `npm run test:deployment` | PASS in the source fixture; FAIL live |

The last claim's local command proves the desired Azure template readback
against a recorded control-plane fixture. The fresh live Azure read proves that
the actual revision violates the claimed topology, so the release claim is not
accepted despite its local test passing.

## Clean-checkout quality gates

| Check | Fresh result |
| --- | --- |
| `npm ci` | PASS — 170 packages, zero vulnerabilities reported. |
| `npm test` | PASS — 6 Vitest, 5 Rust unit, 18 Rust API/integration tests, deployment fixture. |
| `npm run typecheck` | PASS. |
| `npm run lint` | PASS — TypeScript, rustfmt, and Clippy with `-D warnings`. |
| `npm run build` | PASS — Vite produced `dist/`; Cargo produced the release API binary. |
| `npm run test:e2e` | PASS — 24/24 Chromium tests. |
| `npm run test:cold-claim` | PASS — separate empty Cargo target, 205 seconds against a 600-second limit. |
| `bash scripts/load-smoke.sh https://class-capacity-truth.sociobot.in` | PASS — 100 completed: 10 accepted, 90 rate-limited. |
| Docker image build | NOT RUN — Docker, Podman, Buildah, and Nerdctl are unavailable in this verifier. |

The tests exercise exact cutoff behavior, capacity one and full boundaries,
invalid input, idempotent retries, a two-request race for the final seat,
workspace roles, cross-device recovery, encryption/retention, reconciliation,
waitlist fairness, offer acceptance, export/delete, zero-config startup, and a
release-binary durable restart.

## Fresh live product evidence

- Local `dist/index.html`, primary JS, and CSS are byte-for-byte identical to
  the live files (matching SHA-256 values). `/health` reports the full candidate
  SHA. The live deployment therefore matches the candidate artifact even though
  its runtime topology is wrong.
- A fresh sample booking changed `2 seats open` to `1 seat open`; Reset restored
  two. Blank guardian name and malformed email were rejected by native form
  validation. Direct full/cutoff submissions returned HTTP 409 with
  `class_full` and `booking_closed` recovery messages.
- Demo and landing requests were same-origin only. No console or page errors
  appeared in normal flows. Demo cookies are `HttpOnly`, `Secure`,
  `SameSite=Strict`, and expire after 86,400 seconds.
- Every crawled internal/external HTTP link returned 200; mail links were
  recognized as mail actions. Unknown paths return a styled HTTP 404.
- HTML and API responses are `no-cache`; hashed assets are
  `public, max-age=31536000, immutable`.
- The response CSP includes `frame-ancestors 'none'` and explicit Entra and
  Sociobot connect origins. `nosniff`, strict referrer policy, and restrictive
  permissions policy are present. A disallowed origin receives no CORS allow
  header; an allowed `*.sociobot.in` origin does.
- The checkout API accepted a live POST from the product origin and returned an
  HTTPS `checkout.dodopayments.com` session URL. No purchase was completed.
- A dummy test bearer is rejected by production with HTTP 401 and
  `WWW-Authenticate: Bearer`; the test authentication bypass is not active.

## Rate limits, concurrency, and persistence boundaries

- Anonymous demo allowance observed live: 10 rapid requests from one forwarded
  IP succeeded; request 11 returned HTTP 429 with `Retry-After: 5` and
  `X-RateLimit-Limit: 10`. A second forwarded IP still received 200.
- School API allowance observed live: 40 rapid requests; request 41 returned
  HTTP 429 with `Retry-After: 1`.
- The 100-request concurrency smoke completed with 10 accepted and 90 HTTP 429
  responses.
- Local concurrent allocation admits exactly one of two contenders for the
  final seat. The other receives full; idempotent retries preserve one booking.
- Local mounted-storage restart preserves the changed seat count, encrypted
  guardian contact, signing key, and encryption key.
- Production persistence fails before a destructive restart test is needed:
  its effective template has no mounted storage and startup explicitly reports
  durable backup disabled. Restarting production just to demonstrate this
  known loss condition would be unsafe.

## Identity

The signed-out workspace redirects only to
`sociobotcustomers.ciamlogin.com/35c6fe40-0ec0-46b6-98c6-213ad4de6650` with
client ID `25c704f4-465a-47af-80ab-2c489466b697`, the production
`/auth/callback`, authorization-code flow, and PKCE S256. The live page is the
Sociobot sign-in/create-account screen and reports no browser errors. Completing
sign-in was not attempted without an authorized school account.

## Accessibility, responsive behavior, and performance

- Fresh axe scans at 390px, dark mode, and reduced motion found zero serious or
  critical findings on `/`, `/demo`, `/privacy`, `/terms`, `/app`, and the HTTP
  404 route.
- Each route has one H1, one main landmark, a route-specific title, and no
  horizontal overflow. The mobile demo has no interactive target below 44px.
- Keyboard Tab reveals the 44px skip link with a 3px high-contrast outline;
  Enter opens the sample booking and focus moves to its H1. The booking route
  remains usable at 200% text without horizontal overflow.
- `prefers-reduced-motion: reduce` is observed. Normal routes produced no
  console or page errors; the browser's expected missing-resource message is
  limited to the intentional HTTP 404 test.
- Fresh Lighthouse mobile: Performance 100, Accessibility 100, Best Practices
  100, SEO 100; FCP 1.3 s, LCP 1.3 s, TBT 30 ms, CLS 0.
- Initial live transfer: JS 70,592 bytes gzip (230,555 raw), CSS 4,445 bytes
  gzip (18,879 raw), no font request, and no hero raster. The staff-only MSAL
  chunk is not part of the landing's initial request set.

`verify-url.sh` is not present in this repository, so the equivalent live
title/lang/main/alt/console checks were performed directly with Playwright.

## Scope notes

This is a web service, not a library/CLI or PWA. Consumer-package and service
worker update/offline checks do not apply. AI would not improve the core
capacity-allocation job and is appropriately absent. The existing iCalendar
import/check and JSON export cover the obvious integration/export leverage in
the brief.

## Release decision

Do not release or onboard a real school until the effective Azure template is
durable and the production revision-restart proof passes. Re-run independent
verification after the live control-plane repair; no product-code defect was
found that should obscure that deployment blocker.
