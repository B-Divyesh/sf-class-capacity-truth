# Independent verification 18 — FAIL

Verified 2026-09-01 UTC from a clean checkout at candidate commit
`2c800aa84529f69f6819d4bf7bea08891832dfce` against
<https://class-capacity-truth.sociobot.in>.

## Release decision

**FAIL — do not accept this candidate yet.** The candidate builds and tests
successfully, but the live deployment does not report the requested candidate
build identity. `GET /health` returned 200 with `database: "ready"` and build
`1612b35cb5141a1312e2be93dae26a0a51d59e5a`, not
`2c800aa84529f69f6819d4bf7bea08891832dfce`.

Git ancestry confirms that the live build is an ancestor of the candidate,
rather than the candidate itself: `1612b35` is two commits behind HEAD
(`173b361`, then `2c800aa`). The application assets currently match the
candidate's locally built asset names, but this cannot replace the required
runtime build-identity confirmation. Deploy the exact candidate and repeat the
health check before release.

No product code or cloud resource was changed during this verification.

## First read and one-click demo

**PASS.** A cold anonymous desktop page states: “Show the right number of
class seats” and “For small schools that need booking counts to match the
class capacity they set.” It identifies small language schools as the audience
and presents **Try it with sample data** as the first action, with “Three
sample classes open next.” A single click opens `/demo?demo=1` with three
realistic classes and the persistent Demo banner, Reset demo, and Start for
real controls.

In neutral plain words: it lets small language-school staff show families the
current number of seats; it is for schools managing class bookings; click
**Try it with sample data** first.

## Claims gate

`.factory/claims.json` exists and contains 23 entries. After `npm ci`, every
exact declared command passed from the clean checkout. The completed claim IDs
were:

- `sample-booking-updates-seats`, `full-class-blocks-booking`,
  `cutoff-blocks-booking`, `demo-reset-isolated`, `school-capacity-flow`,
  `released-seat-delivery`, `school-plan-price`, `no-third-party-tracking`,
  `entra-sign-in`, and `data-export-delete` — each via its exact
  `npm run test:e2e -- --grep @claim:...` command.
- `calendar-poll`, `contact-encryption-retention`, `staff-role-access`,
  `demo-expiry-input-disposal`, `reconciliation-does-not-change-seats`,
  `configured-smtp-delivery`, `workspace-recovery`, `oldest-waitlist-offer`,
  and `operational-metrics-no-pii` — each via its exact `npm run test:api --`
  command.
- `durable-restart` (`npm run test:durable-restart`), `zero-config-runtime`
  (`bash scripts/test-zero-config.sh`), `forwarded-ip-rate-limits` (its exact
  API claim command), and `durable-one-replica-topology`
  (`npm run test:deployment`).

The complete browser suite also passed: `CI=1 npm run test:e2e -- --retries=0
--reporter=line` (29/29). `npm run test:cold-claim` passed from its isolated
cold build directory.

## Local quality gates

- `npm ci`: PASS — 170 packages, npm reported 0 vulnerabilities.
- `npm test`: PASS — 8 frontend tests, 6 Rust unit tests, 21 API/integration
  tests, and both deployment regression scripts.
- `npm run typecheck`: PASS.
- `npm run lint`: PASS — TypeScript, rustfmt, and Clippy with warnings denied.
- `npm run build`: PASS — emitted `dist/` and the release API binary.
- Built initial JS is 245,759 bytes raw (the local production build's gzip
  measurement is below the 200 KB initial-JS budget); CSS is 20,958 bytes raw.

## Live product checks

- `GET /`, `/demo?demo=1`, `/privacy`, and `/terms` returned 200 with their
  route-specific titles and one h1. An unknown route returned the designed 404
  page with HTTP 404.
- A fresh forwarded-client demo context loaded sample data, submitted the
  prefilled sample guardian details, received `201`, and showed “Your sample
  seat is booked.” The count changed from two open seats to one.
- Full and cutoff boundaries, invalid-input recovery, booking idempotency,
  concurrent capacity handling, real workspace flow, waitlist conversion,
  data controls, encryption/retention, and restart persistence are covered by
  the passing declared browser/API claims above.
- The live rate check used one distinct forwarded client. Ten requests to
  `/api/demo/session` were accepted; request 11 returned `429` with
  `Retry-After: 4`. This confirms the documented protection is active.
- Pre-sign-in requests across home, demo, privacy, terms, and the normal demo
  booking flow were same-origin only. There were no advertising or analytics
  requests. The code and passing CIAM claim use only
  `sociobotcustomers.ciamlogin.com` with the required Sociobot client and
  PKCE flow.
- `/health`, HTML, and API responses are no-cache. The hashed JS asset returned
  `Cache-Control: public, max-age=31536000, immutable`. Responses include CSP
  with response-header `frame-ancestors 'none'`, `X-Content-Type-Options:
  nosniff`, strict referrer policy, and restrictive permissions policy.

## Accessibility and responsive checks

- `/opt/fleet/lib/verify-url.sh` passed against the live URL. Its saved report
  is `.factory/verification-evidence-18/verify.json`: title present,
  `lang=en`, one h1, main landmark, no missing image alt text, no unnamed
  buttons, and no load-time console/page errors.
- Playwright Axe found zero serious or critical findings on home, demo,
  privacy, terms, and the 404 route.
- Keyboard check: Tab reaches the skip link and Enter moves focus to main.
- At 390 px with dark color treatment and reduced motion, the demo showed all
  three classes and Reset demo with no horizontal overflow.

## Defects by severity

| Severity | Finding | Evidence and required resolution |
| --- | --- | --- |
| P1 release blocker | Live build identity is not candidate `2c800aa84529f69f6819d4bf7bea08891832dfce`. | `/health` reports `1612b35cb5141a1312e2be93dae26a0a51d59e5a`; deploy the exact candidate and verify its health build identity. |
| P2 | None found. | — |
| P3 | None found. | — |

## Scope notes

This is a web-with-backend product, not a library, CLI, or PWA; package-consumer
and service-worker update checks do not apply. The deployed product is healthy,
but health identity prevents confirmation that this requested candidate is the
one running in production.
