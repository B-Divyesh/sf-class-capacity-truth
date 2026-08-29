# Independent verification 3 — FAIL

Verified on 2026-08-29 UTC against candidate commit
`b75b14a70e947fa49548bc2531ff2a3f5c7a551b` and
https://class-capacity-truth.sociobot.in.

## Verdict

**FAIL. Do not release or onboard a school.** The live deployment now matches
the candidate and the public demo is useful, accessible at normal text size,
fast, isolated, and concurrency-safe. The actual paid product is not operable:
its checkout returns 404, released-seat mail is only captured inside the
container, and its SQLite database plus encryption/cookie keys live on an
unmounted revision filesystem. A restart can lose school data, and scaling can
split data and keys across replicas. In addition, the mandatory first claim
command failed from the clean clone, the claims registry omits material public
promises, and every tested route fails 200% text reflow at 390 px.

## Mandatory first-read gate

**PASS.** A fresh 1440×900 browser context showed, in the first viewport:

- what: “Show the right number of class seats”;
- for whom: “For small language schools” whose booking calendar and room list
  disagree; and
- first action: “Try it with sample data”.

The one-click action opened `/demo?demo=1`, showed three populated class states,
and displayed “Demo — sample data, nothing is saved” with **Reset demo** and
**Start for real**. The cold page returned 200 and had no console/page error.
Evidence: `verification-evidence/live-first-read-desktop.png` and
`verification-evidence/live-demo-one-click.png`.

## Mandatory claims gate

`.factory/claims.json` exists with 12 entries. After `npm ci`, every entry's
exact command was run separately, including both entries that deliberately
repeat the same Rust test command.

| Claim | First required execution | Evidence |
| --- | --- | --- |
| `sample-booking-updates-seats` | **FAIL** | Playwright stopped before running the test because its 120-second web-server timeout expired while compiling the backend from the clean cache. |
| `full-class-blocks-booking` | PASS | 1/1 passed after the remaining backend compile completed. |
| `cutoff-blocks-booking` | PASS | 1/1 passed. |
| `demo-reset-isolated` | PASS | 1/1 passed. |
| `school-capacity-flow` | PASS | 1/1 passed. |
| `calendar-poll` | PASS | 1/1 passed. |
| `released-seat-delivery` | PASS | 1/1 passed, but it proves an outbox row is queued, not that production mail is delivered. |
| `school-plan-price` | PASS | 1/1 passed, but it checks the link target rather than whether checkout works. Live checkout is 404. |
| `no-third-party-tracking` | PASS | 1/1 passed. |
| `contact-encryption-retention` | PASS | Exact Rust test passed, 1/1. |
| `staff-role-access` | PASS with coverage gap | The same Rust test passed, 1/1; it checks database roles but does not complete a valid CIAM JWT sign-in. |
| `data-export-delete` | PASS | 1/1 passed. |

The failed first execution is release-blocking under the supplied claims
contract even though the same command passed 1/1 on a warm-cache rerun and the
full Playwright suite later passed 21/21. The clean-start harness must allow the
backend to compile before its readiness timeout.

The registry also does not cover several statements a visitor can rely on:

- README and privacy: demo input is discarded and the demo expires after 24
  hours. The reset/isolation claim does not inspect retained values or advance
  the retention clock.
- Landing/app: the $99 plan includes “released-seat email delivery”. The
  registered claim and its test stop at an internal queued outbox record.
- README/app: a calendar discrepancy never changes confirmed seats
  automatically. No claim entry tests that promise.

The two Rust claim entries also point to one untagged test rather than one
`@claim:<id>` test per claim. These are release-blocking registry/coverage gaps
under the supplied claims contract.

## Clean-checkout tests and build

| Check | Result |
| --- | --- |
| Initial identity/status | PASS — exact candidate SHA; clean tracked tree. |
| `npm ci` | PASS — 170 packages installed, 171 audited, 0 vulnerabilities. |
| `npm test` | PASS — 6 Vitest, 4 Rust unit, 8 Rust API/integration tests. |
| `npm run typecheck` | PASS. |
| `npm run lint` | PASS — TypeScript, Rust format, strict Clippy. |
| `npm run build` | PASS — `dist/` plus optimized Rust binary; cold release build completed in 6m02s. |
| `npm run test:e2e` | PASS — 21/21 Chromium tests after compilation was warm. |
| Zero-env runtime | PASS — release binary started with only `PORT=18083`, generated/persisted defaults, and `/health` returned `status: ok`, `build: dev`, `database: ready`. |
| Container build | NOT RUN — no Docker/Podman/Buildah/Nerdctl is installed. Dockerfile inspection confirms multi-stage build, `rust:1-alpine`, non-root runtime, `PORT`, and `BUILD_SHA=dev`. |

Production output is 227,346 bytes raw / 69.63 KB gzip initial app JavaScript,
16,861 bytes raw / 3.98 KB gzip CSS, and a lazy 79.59 KB gzip Entra chunk. The
initial network transfer measured 73 KiB.

## Live deployment identity and platform boundary

- `/health` returned 200 with build
  `b75b14a70e947fa49548bc2531ff2a3f5c7a551b` and `database: ready`.
- Local and live production hashes matched byte-for-byte for main JS
  (`4afbf40b...f96793`), lazy auth JS (`206415a0...51a5`), and CSS
  (`4b6be03e...761b`).
- Azure reports ready revision `sf-class-capacity-truth--0000010`, image tag
  `b75b14a70e94`, 100% traffic, and one healthy replica.
- The deployment has only the `PORT` environment variable, no volume mounts,
  and `minReplicas: 1` / `maxReplicas: 3`.
- Live startup logs say `database_config: generated-default`, both keys
  `generated-and-persisted`, and `smtp: local-capture`.

### P0 — production data is neither durable nor replica-safe

With no `DATABASE_URL` and no mounted `/data`, the service writes SQLite, the
contact-encryption key, and the demo-cookie signing key to the container's
revision filesystem. Replacement loses the school ledger and the key needed to
decrypt contacts. Scaling toward the configured maximum of three creates
independent databases and keys, so requests can observe different schools or
reject another replica's cookies. This fails the brief's real capacity-truth
job and privacy requirements. The candidate itself only adds README wording
that durable storage is a prerequisite; it does not provide it.

### P0 — the advertised $99 subscription cannot be purchased

The visible **Start the $99 monthly plan** / **Open Sociobot checkout** link is
`https://api.sociobot.in/api/v1/products/class-capacity-truth/checkout`.
Fresh GET returned 404 with `{"error":"enabled factory product","status":404}`.
The internal link crawl found this as the only broken actionable link. A
stranger cannot complete the brief's paid adoption flow.

### P0 — released-seat email is not delivered in production

The live container has no SMTP variables, and startup confirms
`smtp: local-capture`. Cancellation can create an encrypted outbox record, but
no message leaves the service. The UI promises that the $99 plan includes
released-seat email delivery, while the brief requires sending a one-click
offer. The current deployment cannot complete waitlist conversion.

## Functional and resilience evidence

The independent live demo flow passed:

- one click entered a realistic three-class sandbox;
- an open class moved from two seats to one after booking;
- full and cutoff class API submissions both returned 409 with `class_full`
  and `booking_closed` respectively;
- after one setup booking left one seat, two simultaneous live requests
  produced exactly one 201 and one 409, preventing oversell;
- two browser contexts remained isolated; reset restored two open seats;
- the demo cookie was `HttpOnly`, `Secure`, `SameSite=Strict`, with a 24-hour
  expiry, and **Start for real** removed it;
- missing idempotency returned 400 with a recovery instruction; invalid name
  returned 422 with a specific correction;
- aborting the first demo API load showed “The sample did not load” and **Try
  loading again**; retry restored all three classes.

The local browser claim flow also created and published a class, checked a
recorded iCalendar fixture, booked and waitlisted through visible forms,
selected the named booking to cancel, queued an offer, accepted it, exported
data, and deleted the workspace. The Rust test independently passed encrypted
contact inspection, role denial, 91-day scrubbing, idempotency, and a concurrent
seat race. These local successes do not remedy missing production persistence,
billing, and email.

## Authentication, privacy, headers, and rate limits

- Signed-out `/app` offers only **Sign in with Sociobot**. Clicking it reached
  `sociobotcustomers.ciamlogin.com` for tenant
  `35c6fe40-0ec0-46b6-98c6-213ad4de6650`, client
  `25c704f4-465a-47af-80ab-2c489466b697`, production callback
  `/auth/callback`, scopes `openid profile email offline_access`, and PKCE S256.
- CIAM discovery returned 200. The source reads issuer/JWKS discovery, caches
  keys for one hour, requires RS256/audience/tenant/issuer/time claims, and keys
  users by `oid`. A live invalid token returned 401 plus
  `WWW-Authenticate: Bearer`.
- During landing, demo, privacy, legal, and signed-out workspace flows, all 33
  recorded requests were same-origin. No analytics, tracker, CDN font, or
  third-party script loaded. Microsoft origins appeared only after explicit
  sign-in.
- Responses include CSP with `frame-ancestors 'none'`, `nosniff`, strict
  referrer policy, and restrictive permissions policy. HTML/API are
  `no-cache, max-age=0`; hashed assets are one-year immutable.
- Anonymous burst allowance observed: 10 accepted, then 429 with
  `Retry-After`. Anonymous long-window allowance: 30 initial plus one token
  replenished during the paced run; request 32 returned 429 with
  `Retry-After: 103`.
- School/API and `/workspaces/billing/verify` allowance observed: 40 requests
  reached authentication, request 41 returned 429 with `Retry-After: 2`.
  Health is exempt: 45/45 returned 200.
- No obvious committed secret or raw Azure/Sociobot key was found.

## Accessibility, responsive behavior, and performance

- Factory `verify-url.sh` passed: HTTPS 200, title, `lang=en`, one H1, main,
  alt/button checks, 554 ms load, and no console error. Evidence:
  `verification-evidence/verify.json`.
- Independent axe scans found zero violations of any impact on `/`, demo,
  signed-out `/app`, privacy, terms, and the styled HTTP 404.
- First Tab focused the skip link. Enter operated the booking link, route
  navigation focused the new H1, and focus outlines are visible.
- At 390×844 in dark mode with reduced motion, demo and signed-out app had no
  normal-size overflow, no running animation, no console/page error, and no
  serious/critical axe issue.
- Live mobile Lighthouse: performance 100, accessibility 100, best practices
  100, SEO 100; FCP/LCP 1.3 s, TBT 10 ms, CLS 0. Evidence:
  `verification-evidence/lighthouse-live.json`.
- Titles, `lang=en`, one H1, and one main were correct on every public route;
  the unknown path returned a styled HTTP 404.

### P1 — all routes fail 200% text reflow at 390 px

At a 390 px viewport with root text set to 200%, `/demo`, `/app`, `/privacy`,
and `/terms` grew to 681 px document width; `/` grew to 996 px. Main headings
extended beyond the viewport. The source's `body { min-width: 20rem; }` scales
to 640 CSS px with the root size. This creates two-dimensional scrolling and
fails the supplied 200% text requirement. Evidence:
`verification-evidence/live-app-mobile-text-200.png`.

### P2 — three mobile inline links miss the 44 px target rule

At normal 390 px size, “Read how sample data is handled”,
`privacy@sociobot.in`, and `support@sociobot.in` measured 19 px high. All other
measured controls met 44×44 px. The attached accessibility/design contract
requires every interactive target to be at least 44 px.

## Applicability notes

This is not a library or CLI, so package-consumer checks do not apply. It is not
a PWA and makes no offline claim; service-worker update/offline reload checks do
not apply.

## Required next actions

1. Provision a durable, replica-safe production database/key strategy before
   accepting school data; keep one replica until storage is safe and prove
   restart/scale persistence and tenant isolation.
2. Register and verify the recurring Sociobot product so the public checkout
   returns a usable hosted flow.
3. Configure a real transactional email relay and verify one credentialed
   cancellation-to-inbox-to-one-click-accept flow.
4. Make every claims command pass from a cold clone, give each claim its own
   matching test, and register/test all public promises.
5. Fix 200% reflow and the three undersized touch targets, then rerun mobile,
   keyboard, axe, claims, build, live identity, and deployment-boundary checks.

