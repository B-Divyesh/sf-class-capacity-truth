# Independent verification 16 — FAIL

Verified on 2026-08-30 UTC against candidate commit
`283758f64e321a3037951b433f24bc79c0622ee6` and
<https://class-capacity-truth.sociobot.in>.

## Verdict

**FAIL — do not accept real school data.** The source candidate passes every
declared claim command and repository gate, the exact candidate is live, and
the ordinary product flows work while one replica remains available. Fresh
Azure readback nevertheless shows that production again has ephemeral local
SQLite/key storage and permits three replicas. This directly contradicts the
capacity-durability contract and the product's own production-topology claim.

The live demo also records CLS 0.122, above the required 0.1 budget. This is
secondary to the data-durability blocker.

No product code or infrastructure was modified during verification.

## Release-blocking finding

### P0 — production lost its durable, single-replica topology again

Fresh control-plane readback for the only active revision returned:

```text
revision:     sf-class-capacity-truth--0000046
image:        sociobotregistry.azurecr.io/sf-class-capacity-truth:283758f64e32
traffic:      100%
health:       Healthy / Provisioned
min/max:      1 / 3
replicas now: 1
environment:  PORT=8080 only
mounts:       null
volumes:      null
```

The environment still has a registered `cct-data` Azure Files definition, but
the candidate revision does not mount it. It also lacks `DATA_DIR` and
`DURABLE_BACKUP_PATH`. `scripts/verify-container-topology.sh` exits 1.

The running process independently logged:

```text
database_config="generated-default"
durable_backup="disabled"
cookie_signing_key="generated-and-persisted"
contact_encryption_key="generated-and-persisted"
```

Therefore a revision restart can lose the school ledger and generated keys.
Autoscaling above one replica can create divergent ledgers and keys. Either
condition can show the wrong number of seats, lose bookings, or make encrypted
contacts unreadable. One currently running replica does not mitigate the
configured `maxReplicas=3` or absent durable storage.

Required repair: deploy through `scripts/deploy-container.sh`, then read back
exactly one replica, the `cct-data` mount at `/mnt/cct`,
`DATA_DIR=/mnt/cct/keys`, and
`DURABLE_BACKUP_PATH=/mnt/cct/snapshots/class-capacity-truth.db`. Prove a real
booking and decrypted contact survive a revision restart before traffic is
accepted.

## Mandatory first-read and demo gate

**PASS.** A cold anonymous visit says “Show the right number of class seats,”
names schools whose booking calendar and room list disagree, and presents
**Try it with sample data** as the primary action. “Three sample classes open
next” explains the result. One click opens `/demo?demo=1`, shows realistic
open/full/cutoff classes, and keeps the “Demo — sample data, nothing is saved”
banner with Reset demo and Start for real visible.

In plain words: it keeps the public seat count aligned with a small language
school's bookings; it is for school operations staff; click **Try it with
sample data** first.

## Claims gate

`.factory/claims.json` exists with 22 entries. A preliminary invocation before
installing dependencies stopped at missing local test tools; after the required
clean `npm ci`, every exact manifest command passed independently. The
topology fixture passes locally, but the corresponding production claim is
false according to the fresh Azure readback above.

| Claim ID | Exact declared command | Result |
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
| `forwarded-ip-rate-limits` | `npm run test:api -- rate_limit_uses_forwarded_ip_and_returns_retry_after` | PASS local and live |
| `durable-one-replica-topology` | `npm run test:deployment` | PASS fixture; **FAIL live** |
| `operational-metrics-no-pii` | `npm run test:api -- regression_protected_operational_metrics_are_aggregated_and_contain_no_pii` | PASS |

The landing page and README claim review found no uncovered user-facing claim.

## Clean install, tests, and production build

| Check | Result |
| --- | --- |
| `npm ci` | PASS — 170 packages, 0 vulnerabilities. |
| `npm test` | PASS — 8 frontend, 6 Rust unit, 20 API/integration tests, and 2 deployment regressions. |
| `npm run typecheck` | PASS. |
| `npm run lint` | PASS — TypeScript, rustfmt, and Clippy with warnings denied. |
| `npm run build` | PASS — `dist/` and the optimized Rust binary produced. |
| `CI=1 npm run test:e2e -- --retries=0` | PASS — 26/26 in 1.0 minute. |

The production web build is 73.61 kB gzip initial JavaScript, 79.59 kB gzip
lazy staff/auth JavaScript, and 4.62 kB gzip CSS. First-load transfer measured
80.3 kB. Docker/Podman are unavailable in this verifier image; the exact live
ACR-built image and health identity were verified instead.

## Live product and backend evidence

- `/health` returns 200 with database `ready` and the exact full candidate SHA.
  The live image tag is the candidate's 12-character prefix. Local and live
  `index.html`, initial JS, and CSS are byte-identical by SHA-256.
- A valid sample booking changed two open seats to one. A malformed email was
  rejected by the labelled input and succeeded after correction. Full and
  cutoff classes removed the booking action and returned `409 class_full` and
  `409 booking_closed` at the API.
- Three concurrent booking requests for two open seats returned 201, 201, and
  409. The accepted responses reported one then zero seats; there was no
  oversell.
- Demo rate limiting allowed exactly 10 requests from one forwarded client,
  then returned 429 with `Retry-After: 5`. Protected metrics allowed 40
  requests, then returned 429 with `Retry-After`; a second client retained a
  fresh allowance. The observed allowances are **10 demo** and **40 school**
  requests per burst.
- The live checkout POST returned 200 from `api.sociobot.in` and navigated to
  `checkout.dodopayments.com`. No payment was submitted.
- Explicit sign-in used `sociobotcustomers.ciamlogin.com`, tenant
  `35c6fe40-0ec0-46b6-98c6-213ad4de6650`, client
  `25c704f4-465a-47af-80ab-2c489466b697`, `/auth/callback`, authorization code
  response, and PKCE S256. No credentials were entered.

## Privacy, security, accessibility, and routes

- Before explicit sign-in/checkout, home, demo, privacy, terms, and signed-out
  workspace requests were all same-origin. There were no analytics,
  advertising, third-party font, or third-party script requests.
- Demo cookies are `HttpOnly`, `Secure`, `SameSite=Strict`, with a 24-hour
  expiry. An unapproved CORS origin received no allow-origin header.
- CSP is delivered as a response header with `frame-ancestors 'none'`.
  `nosniff`, strict referrer policy, and a restrictive permissions policy are
  present. HTML/API responses are no-cache; hashed assets are immutable for one
  year.
- Factory `verify-url.sh` passed: load 623 ms, descriptive title, `lang=en`, one
  h1, one main, no missing image alt, no unnamed buttons, and no console/page
  errors.
- Live Axe checks found zero serious/critical findings on home, demo, privacy,
  terms, signed-out workspace, and the real 404. At 390 px dark/reduced-motion,
  there was no horizontal overflow, transitions/animations were 0 s, the menu
  measured 76.6 by 44.8 px, keyboard open/Escape worked, focus returned, and a
  visible 3 px focus outline remained.
- All 17 discovered internal/external links returned 200 or were explicit
  `mailto:` links. `/missing-page` returns an actual HTTP 404. Robots, sitemap,
  icons, social card, and public/deep-link routes returned their expected
  content types/statuses.
- There is no manifest or service-worker registration and no offline claim, so
  PWA offline/update testing is not applicable. This is not a library or CLI,
  so consumer package/CLI testing is not applicable.

## Secondary finding

### P2 — live demo exceeds the CLS budget

Mobile Lighthouse scored 91 performance, 100 accessibility, 100 best
practices, and 100 SEO. FCP/LCP were 1,291 ms, TBT 258.5 ms, transfer 80,305
bytes, and CLS **0.122**. The required CLS budget is below 0.1. Lighthouse
attributes the shift to the footer moving when the asynchronous demo class
content replaces the loading state. Reserve the demo result height or use a
stable skeleton, then remeasure the live `/demo?demo=1` route.

## Release decision and next verification

The candidate remains **FAIL** until the production topology is repaired and a
live restart durability drill passes. The CLS budget should be repaired in the
same release. After deployment, rerun all 22 claims, the full no-retry browser
suite, Azure topology readback, same-client rate limits, concurrent booking,
factory URL verification, and mobile Lighthouse.
