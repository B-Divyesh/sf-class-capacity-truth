# Independent verification 17 — PASS

Verified on 2026-09-01 UTC against candidate commit
`f8b545ad0efc4b1972d3f3447958b7baf5a413f6` and
<https://class-capacity-truth.sociobot.in>.

## Verdict

**PASS — accept the candidate.** The exact candidate is live, the database is
ready, all 22 declared claim commands pass from the clean checkout, all local
quality gates pass, and fresh live product checks found no release-blocking or
lower-severity defect.

No product code or infrastructure was changed during verification. Read-only
cloud checks were limited to `sf-class-capacity-truth`.

## Mandatory first-read and demo gate

**PASS.** A cold anonymous desktop context loaded the live home page with no
stored state. The first screen says **“Show the right number of class seats”**
and **“For schools whose booking calendar and room list disagree about
places.”** The primary action is **“Try it with sample data”** and the adjacent
copy says that three sample classes open next.

In plain words: the product keeps the displayed class-seat count aligned with
a small language school's bookings; it is for school operations staff; click
**Try it with sample data** first. One click opens `/demo?demo=1`, three
realistic sample classes, and the persistent “Demo — sample data, nothing is
saved” banner with Reset demo and Start for real.

The cold page returned 200, made only three same-origin requests (HTML, hashed
JS, and hashed CSS), and logged no console or page errors.

## Claims gate

`.factory/claims.json` exists with 22 entries. After `npm ci`, every exact
declared command ran independently in manifest order and passed.

| Claim | Exact command | Result |
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
| `durable-restart` | `npm run test:durable-restart` | PASS |
| `configured-smtp-delivery` | `npm run test:api -- claim_configured_smtp_queues_an_encrypted_offer` | PASS |
| `workspace-recovery` | `npm run test:api -- claim_workspace_recovery_by_staff_identity` | PASS |
| `oldest-waitlist-offer` | `npm run test:api -- claim_oldest_waitlist_entry_gets_a_24_hour_offer` | PASS |
| `zero-config-runtime` | `bash scripts/test-zero-config.sh` | PASS |
| `forwarded-ip-rate-limits` | `npm run test:api -- rate_limit_uses_forwarded_ip_and_returns_retry_after` | PASS |
| `durable-one-replica-topology` | `npm run test:deployment` | PASS |
| `operational-metrics-no-pii` | `npm run test:api -- regression_protected_operational_metrics_are_aggregated_and_contain_no_pii` | PASS |

The durable restart check built the release process and confirmed a real
school booking, decrypted contact, SQLite file, and generated keys survived a
restart using one directly mounted `/data` directory. The deployment fixture
rejected the prior non-durable topology and accepted only the current
one-replica `/data` contract. The landing page, privacy page, terms, and README
were cross-checked against the manifest; no uncovered user-facing claim was
found.

## Clean checkout and repository gates

- Checkout before QA: clean `main`; `HEAD` and `origin/main` both
  `f8b545ad0efc4b1972d3f3447958b7baf5a413f6`.
- `npm ci`: PASS — 170 packages, 0 vulnerabilities.
- `npm test`: PASS — 8 frontend tests, 6 Rust unit tests, 21 API/integration
  tests, and both deployment regression scripts.
- `npm run typecheck`: PASS.
- `npm run lint`: PASS — TypeScript, rustfmt, and Clippy with warnings denied.
- `npm run build`: PASS — `dist/` and the optimized Rust API binary produced.
- `CI=1 npm run test:e2e -- --retries=0`: PASS — 27/27.
- `npm run test:cold-claim`: PASS in 199 seconds against its 600-second limit.

The production web build reports 73.85 kB gzip initial JavaScript, 79.59 kB
gzip lazy staff/auth JavaScript, and 4.62 kB gzip CSS. This is within the
200 kB initial-JS and 50 kB CSS budgets.

## Fresh live identity and persistence evidence

- `GET /health`: 200 with `status: ok`, `database: ready`, and exact build
  `f8b545ad0efc4b1972d3f3447958b7baf5a413f6`.
- The live HTML, initial JS, CSS, and lazy staff/auth JS are byte-identical to
  the local production build. SHA-256 values are respectively
  `4c4fa748c52e481bcfea12683cb9f7b69571a9ce97bc22b0f43583dc1239bec6`,
  `b29ad310064917f9980de3a61f3f5c47d0c190d1ddae152ce152b221cfd2a6b5`,
  `ae5111bbaa568fa3646c3c9a5e984ad370b8ced22102e2fc6cff6692029de50e`,
  and `89bcb4cb691b0fef9301481bc0c13931d08783271c9fa0f1d1e39676a5a611b6`.
- The repository's read-only production topology verifier passed. Fresh
  readback shows image `sf-class-capacity-truth:f8b545ad0efc`, one ready
  replica, `minReplicas=1`, `maxReplicas=1`, Azure Files storage
  `sf-class-capacity-truth-data`, and volume `data` mounted at `/data`. The only
  environment setting is `PORT=8080`.
- Startup logs for the active revision report SQLite DELETE journaling, one
  connection, and persisted generated cookie/contact keys. Sampled startup and
  cleanup logs contained no guardian data, credentials, or token values.

This fresh evidence resolves Verification 16's deployment-only failure. The
verifier did not restart or mutate the live service; persistence across a
process restart was exercised by the mandatory clean local `/data` claim.

## Live product and backend behavior

- A normal sample booking changed two open seats to one and showed a
  confirmation. Reset returned the seed to two open seats and a separate
  browser context retained its own seed state.
- A malformed email was blocked by native validation, the invalid field kept
  focus, and correcting it completed the booking. Direct malformed input
  returned 422 with a specific corrective message.
- Full and cutoff classes showed explanatory recovery actions and no booking
  button. Direct requests returned 409 `class_full` and 409 `booking_closed`.
- Repeating one idempotency key returned the original booking ID without
  consuming another seat.
- Three concurrent requests for two open seats returned 201, 201, and 409;
  the final state was eight confirmed and zero open, with no oversell.
- The live $99 action sent `POST` to the Sociobot product checkout endpoint,
  received 200, and navigated to `checkout.dodopayments.com`. No payment was
  submitted.
- Signed-out workspace and deep links loaded with route-specific titles and
  clear sign-in states. Invalid booking and offer links showed recovery text.

### Observed request allowances

- Anonymous demo/API allowance: **10 requests per forwarded client burst**.
  A 100-request same-client probe produced 10×200 and 90×429. A subsequent
  response included `Retry-After: 110`; a second client immediately received
  200.
- School/metrics allowance: **40 requests per forwarded client burst**. A
  60-request same-client probe produced 40×401 followed by 20×429; a sampled
  429 carried `Retry-After`, and another client retained a fresh allowance and
  received the expected 401 Bearer challenge.

## Privacy, security, auth, routes, and caching

- Before explicit sign-in or checkout, home, demo, privacy, terms, and
  signed-out workspace traffic was same-origin only. There were no analytics,
  advertising, third-party font, or third-party script requests.
- Demo cookies are `HttpOnly`, `Secure`, `SameSite=Strict`, path `/`, and have
  `Max-Age=86400`.
- The response supplies CSP as a header, including `frame-ancestors 'none'`,
  plus `X-Content-Type-Options: nosniff`, a strict referrer policy, and a
  restrictive permissions policy. There were no CSP console errors.
- An approved-origin preflight returned the production origin. An unapproved
  origin received no `Access-Control-Allow-Origin`.
- HTML, health, and API responses are no-cache. Hashed assets use
  `public, max-age=31536000, immutable`.
- Explicit sign-in used `sociobotcustomers.ciamlogin.com`, tenant
  `35c6fe40-0ec0-46b6-98c6-213ad4de6650`, client
  `25c704f4-465a-47af-80ab-2c489466b697`, callback
  `/auth/callback`, authorization code flow, and PKCE S256. Discovery returned
  the GUID issuer and Sociobot JWKS URL.
- All 17 discovered links returned 200 or were explicit `mailto:` links.
  `/missing-page` and `/404.html` return HTTP 404 with the designed recovery
  page. Sitemap, robots, favicon, Apple touch icon, and social card are live.

## Accessibility, mobile, and performance

- Factory `verify-url.sh`: PASS in 545 ms — descriptive title, `lang=en`, one
  h1, one main, no missing image alt, no unnamed buttons, and no console errors.
- Playwright Axe found zero serious/critical findings on home, demo, booking,
  privacy, terms, signed-out workspace, and 390 px dark/reduced-motion views.
- Keyboard checks passed for the skip link, route focus, and the labelled
  mobile menu. The focused skip link has a visible 3 px solid outline with a
  3 px offset. The menu target is 77.6 by 44.8 px and returns focus on Escape.
- At 390 px, normal and dark views have no page overflow. Home, demo,
  workspace, privacy, terms, and 404 reflow without overflow at 200% text.
  Reduced motion leaves all tested animation and transition durations at 0 s.
- Mobile Lighthouse on `/demo?demo=1`: performance 95, accessibility 100,
  best practices 100, SEO 100; FCP/LCP 1,332 ms, TBT 257 ms, CLS 0, and
  80,542 bytes transferred.

## Applicability and defects

- This product has no service worker, manifest, or offline claim. Offline
  reload is unavailable as documented; PWA update testing is not applicable.
- This is not a library or CLI, so package/consumer installation is not
  applicable.
- Production has no SMTP relay. The tested durable copy-offer path is the
  documented fallback and is not a defect.
- Local Docker/Podman is unavailable. The exact live container identity,
  runtime health, non-root Dockerfile contract, and production topology were
  verified by other gates.

Defects by severity: **P0 none; P1 none; P2 none; P3 none.**

## Release decision

**PASS.** Candidate
`f8b545ad0efc4b1972d3f3447958b7baf5a413f6` is accepted at
<https://class-capacity-truth.sociobot.in>.
