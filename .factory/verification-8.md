# Independent verification 8 — FAIL

Verified 2026-08-29 UTC against candidate commit
`11a728e6b2f481506753caef919347958512c124` and
<https://class-capacity-truth.sociobot.in>.

## Verdict

**FAIL — do not accept real school data.** The candidate source, all 21
manifested claim commands, one-click demo, live product flows, identity,
accessibility, privacy behavior, rate limiting, security headers, and
performance gates pass. The active candidate deployment does not use the
repository's mandatory durable one-replica topology.

This is fresh evidence from candidate revision `0000039`, not a carry-over
from verification 7. During this verification Azure scaled the active revision
to two healthy replicas. Each replica has its own disposable SQLite database
and independently generated signing/encryption keys.

## Release-blocking defect

### P0 — candidate deployment is an ephemeral, multi-replica SQLite service

The live health endpoint is HTTP 200 and identifies the exact requested
candidate:

```json
{"status":"ok","build":"11a728e6b2f481506753caef919347958512c124","database":"ready"}
```

A fresh Azure control-plane read at 2026-08-29T12:36Z found:

```text
revision: sf-class-capacity-truth--0000039
latest ready revision: sf-class-capacity-truth--0000039
image: sociobotregistry.azurecr.io/sf-class-capacity-truth:11a728e6b2f4
minReplicas/maxReplicas: 1/3
environment: PORT=8080 only
volumeMounts: null
volumes: null
running status: Running
```

The registered `cct-data` environment storage still exists and points to the
`class-capacity-truth` Azure Files share, but revision `0000039` does not mount
it. Startup logs state:

```text
database_config="generated-default"
durable_backup="disabled"
cookie_signing_key="generated-and-persisted"
contact_encryption_key="generated-and-persisted"
```

The revision initially had one replica. At 12:36Z the control plane reported
two Ready/Running replicas:

```text
sf-class-capacity-truth--0000039-5cdc6555d8-wt84z  created 11:32:49Z
sf-class-capacity-truth--0000039-5cdc6555d8-dc895  created 12:32:01Z
```

The second replica's fresh startup log repeats `durable_backup="disabled"`
and both generated-key messages. This directly violates the
`durable-one-replica-topology` claim and the README/design/plan operating
contract. Consequences include:

- live requests can reach different, disagreeing seat ledgers;
- a replacement replica loses workspaces, classes, bookings, waitlists,
  offer receipts, and billing state;
- independently generated keys make another replica's signed demo cookie or
  encrypted guardian data unusable; and
- process-local rate-limit allowances can multiply across replicas.

The checked-in deploy script and regression fixture correctly demand one
replica, `/mnt/cct`, `DATA_DIR=/mnt/cct/keys`, and
`DURABLE_BACKUP_PATH=/mnt/cct/snapshots/class-capacity-truth.db`. The local
tests prove that desired state, not the active deployment. A production
restart drill was not run because the known effective template can lose data.

Required repair: deploy the exact candidate image through the checked-in
durable deployment path; read back one replica, the `cct-data` mount, and both
durable path variables; then run the production persistence drill and verify
that a booked record plus decrypted synthetic contact survives a new revision.

### P3 — the 390 px header does not follow its recorded responsive rule

`.factory/design.md` says the navigation collapses to an accessible labelled
menu at 390 px. The live header instead wraps all four navigation links into
two rows. It remains readable, has no overflow, and does not prevent the
headline, audience sentence, or demo action from appearing in the first
844 px, so this is a visual-contract defect rather than an accessibility or
release blocker.

## Mandatory first-read and demo gate

**PASS.** In a cold 1440×900 browser and again at 390×844, the first screen
answers all three required questions in plain words:

- What it does: “Show the right number of class seats.”
- For whom: small language schools whose booking calendar and room list
  disagree about places.
- What to click: “Try it with sample data,” followed by “Three sample classes
  open next.”

One click reaches `/demo?demo=1`. The page immediately shows realistic
available, full, and cutoff classes. The persistent banner says “Demo — sample
data, nothing is saved” and exposes **Reset demo** and **Start for real**.

## Claims gate

`.factory/claims.json` exists with 21 entries. After `npm ci`, every listed
command was executed exactly as written from this candidate checkout. All
commands passed. The live truth check nevertheless rejects the final topology
claim because the active Azure template contradicts it.

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
| `durable-one-replica-topology` | `npm run test:deployment` | PASS fixture; **FAIL live** |

The landing page and README claims map to the manifest. No additional
unlisted public product claim was found. The README's one-replica/durable
checkpoint statement is listed, but false in the live candidate.

## Clean-checkout quality gates

| Check | Fresh result |
| --- | --- |
| `npm ci` | PASS — 170 packages, zero vulnerabilities. |
| `npm test` | PASS — 6 Vitest, 5 Rust unit, 18 API/integration, and both deployment regressions. |
| `npm run typecheck` | PASS. |
| `npm run lint` | PASS — TypeScript, rustfmt, and Clippy with `-D warnings`. |
| `npm run build` | PASS — exact Vite production build and optimized Rust API; `dist/` produced. |
| `npm run test:e2e` | PASS — 24/24 Chromium tests. |
| `npm run test:cold-claim` | PASS — 187 seconds against the 600-second limit with an empty Cargo target. |
| `npm run test:durable-restart` | PASS — booked real-school data and generated keys survived a local release-process restart. |
| `bash scripts/test-zero-config.sh` | PASS. |
| `bash scripts/load-smoke.sh https://class-capacity-truth.sociobot.in` | PASS — 100 completed: 10 accepted, 90 rate-limited. |
| Docker image build | NOT RUN — Docker, Podman, Buildah, and Nerdctl are unavailable. |

The locally built `index.html`, primary JavaScript, and CSS are byte-for-byte
identical to the live files by SHA-256. Combined with `/health` and the active
image tag, the live application artifact matches the requested candidate.

## Functional, privacy, and error-path evidence

- A fresh live sample booking changed two open seats to one; Reset restored
  two. Blank name/email and malformed email were stopped with native recovery
  messages. The booking POST returned 201.
- The full and cutoff pages remove the booking action. Direct submissions
  returned HTTP 409 with `class_full` and `booking_closed` plus a next step.
- Local tests cover capacity one, exact cutoff time, idempotent retries, two
  contenders for the final seat, role boundaries, tenant recovery,
  reconciliation, retention, waitlist fairness, offer acceptance, export, and
  deletion.
- Landing and demo flows made only same-origin requests. No advertising,
  analytics, font, or CDN request occurred. Explicit sign-in and checkout were
  the only observed third-party transitions.
- The demo cookie is HttpOnly, Secure, SameSite=Strict, and has
  `Max-Age=86400`. Fresh API data contains the documented Bright Path sample.
- HTML/API responses are `no-cache`; hashed assets are
  `public, max-age=31536000, immutable`.
- CSP, `nosniff`, strict referrer policy, and restrictive permissions policy
  are response headers. A disallowed CORS origin receives no allow header; a
  `*.sociobot.in` origin does.
- Every crawled HTTP link returned 200. Mail links were recognized as mail
  actions. Unknown paths return a styled HTTP 404.
- A dummy bearer received HTTP 401 and `WWW-Authenticate: Bearer`; the test
  authentication bypass is absent from the live environment.
- Live checkout used POST to the Sociobot API and reached an HTTPS Dodo hosted
  session. No payment was attempted.

## Rate limits and concurrency

- Demo API: requests 1–10 from one forwarded client IP returned 200; request
  11 returned 429 with `Retry-After: 5` and `X-RateLimit-Limit: 10`. A second
  forwarded IP received 200.
- Staff API: requests 1–40 reached authentication; request 41 returned 429
  with `Retry-After: 1` and `X-RateLimit-Limit: 40`.
- Live 100-request concurrency smoke: 10 accepted and 90 rate-limited.
- The external Sociobot checkout gateway also enforced a shared limit: after
  prior QA traffic, 50 concurrent no-product requests admitted 12 to the
  handler and returned 429 for 38, with `Retry-After: 0`. This confirms the
  header and enforcement; the shared/refilling bucket prevents treating 12 as
  a documented fresh-client allowance.
- Local concurrent-allocation integration testing admits one of two requests
  for the final seat and rejects the other without overselling.

## Identity

The only staff sign-in action redirected to
`sociobotcustomers.ciamlogin.com/35c6fe40-0ec0-46b6-98c6-213ad4de6650`
with client ID `25c704f4-465a-47af-80ab-2c489466b697`, callback
`https://class-capacity-truth.sociobot.in/auth/callback`, authorization-code
flow, state, and PKCE S256. No other identity provider is offered. Sign-in was
not completed without an authorized school account; the full protected flow
passed locally with recorded test identity.

## Accessibility, responsive behavior, and performance

- `/opt/fleet/lib/verify-url.sh` passed: HTTPS 200, title, `lang=en`, one H1,
  main landmark, no missing alt text, no unlabeled buttons, and no console
  errors.
- Fresh Playwright axe scans found zero serious or critical findings on `/`,
  `/demo`, `/privacy`, `/terms`, `/app`, the booking route, and the 404 in
  light, dark, mobile, and reduced-motion contexts.
- All tested routes have one H1, one main landmark, route-specific titles, no
  horizontal overflow at 390 px, and no visible interactive target below
  44×44 px.
- Keyboard Tab exposes the 44 px skip link with a 3 px visible outline. Enter
  opens the sample booking and focus moves to its H1. The booking route remains
  usable at 200% text.
- Lighthouse mobile: Performance 100, Accessibility 100, Best Practices 100,
  SEO 100; FCP 1.3 s, LCP 1.3 s, TBT 40 ms, CLS 0.
- Initial live transfer is 70,505 bytes gzip JavaScript and 4,343 bytes gzip
  CSS. No font or hero-raster request is made. The staff-only MSAL chunk is
  lazy and absent from the landing request set.

## Scope notes

This is a web service, not a library/CLI or PWA. Consumer-package and service
worker/offline-update checks do not apply. AI would not improve the core
allocation decision and is appropriately absent. The iCalendar input, public
booking link, waitlist offer, and JSON export cover the obvious leverage in
the brief.

## Release decision

Do not release or onboard a real school. Re-run independent verification only
after the active Azure template is durably mounted and fixed to one replica,
and the candidate passes a production revision-restart proof.
