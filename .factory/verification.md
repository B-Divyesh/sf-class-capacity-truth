# Independent verification — FAIL

Verified 2026-08-28 against candidate commit
`ead522ac24c02ddcfa8b3b18c680365195daa8fc` and
https://class-capacity-truth.sociobot.in.

## Verdict

**FAIL.** The deployed build is the tested commit and its M1 public demo works,
but it is not an end-to-end product for the researched job-to-be-done. It is
explicitly only a demo: a school cannot connect a calendar, create or publish
a real class, set a capacity/cutoff, have a parent make a real booking, get
reconciliation, or convert a waitlist offer. The planned paid school workspace
is also absent. That fails the repository definition of done and the brief's
smallest useful product; a polished sample cannot substitute for it.

## First-read result

PASS for the first-screen gate. A cold anonymous desktop visit displayed
“Show the right number of class seats”, named small language schools whose
calendar and room list disagree, and exposed **Try it with sample data** with
the immediate result (“Three sample classes open next”). The action is one
click. The page plainly presents a capacity-booking sample for small language
schools, not the real service promised by the brief.

## Required claims

`.factory/claims.json` exists and has four entries. After `npm ci`, each
declared command was run against `/demo?demo=1` from fresh browser contexts:

| Claim | Result | Evidence |
| --- | --- | --- |
| `sample-booking-updates-seats` | PASS | `npm run test:e2e -- --grep @claim:sample-booking-updates-seats`: 1/1 passed; two seats became one and confirmation appeared. |
| `full-class-blocks-booking` | PASS | `npm run test:e2e -- --grep @claim:full-class-blocks-booking`: 1/1 passed; UI and API returned the full state. |
| `cutoff-blocks-booking` | PASS | `npm run test:e2e -- --grep @claim:cutoff-blocks-booking`: 1/1 passed; UI and API returned the closed state. |
| `demo-reset-isolated` | PASS | `npm run test:e2e -- --grep @claim:demo-reset-isolated`: 1/1 passed; reset and two-context isolation verified. |

## Test and build evidence

| Check | Result |
| --- | --- |
| `npm ci` | PASS — 169 packages, no reported vulnerabilities. |
| `npm test` | PASS — 4 TypeScript tests and 9 Rust tests (including integration tests). |
| `npm run test:api` | PASS — 6/6; validation, signed cookie, isolation, idempotency/concurrent race, migration down path, health, and 429 header. |
| `env -u CI npm run test:e2e` | PASS — 14/14 Chromium tests. Covers claim flow, keyboard, route focus, 390px/no horizontal overflow, reduced motion, dark contrast, console errors, and axe serious/critical checks on `/`, `/demo?demo=1`, `/privacy`, `/terms`, and the in-app 404 route. |
| `npm run build` | PASS — TypeScript/Vite build plus release Rust build. `dist/` produced. Initial JS is 64.50 KB gzip; CSS is 3.81 KB gzip. |
| Default runtime | PASS — release binary started with only `PORT=18080`; `/health` returned 200 and `{"status":"ok","build":"dev","database":"ready"}`. |
| Container build/run | NOT RUN — `docker` is unavailable in this verifier image (`/bin/bash: docker: command not found`). |

No separate lint script exists in `package.json`; TypeScript checking is part
of `npm run build`.

## Live deployment evidence

- `GET /health` returned 200 with build
  `ead522ac24c02ddcfa8b3b18c680365195daa8fc`, database `ready`; this matches
  the candidate exactly.
- Cold Playwright load of `/` returned 200, one H1, no page/console errors and
  requested only the same-origin document, JS and CSS.
- A cold 390px Playwright load of `/demo?demo=1` returned 200, rendered three
  class articles, had no console/page error and made only same-origin document,
  CSS, JS and `/api/demo/session` requests. This supports the stated no
  analytics/third-party-script promise for the observed flow.
- Live headers on page/API responses include CSP with `frame-ancestors 'none'`,
  `X-Content-Type-Options: nosniff`, strict referrer policy and a restrictive
  permissions policy. Demo cookie response uses `HttpOnly`, `SameSite=Strict`,
  and `Secure` when forwarded HTTPS is supplied.
- Live concurrent sample booking against a new cookie-scoped demo yielded two
  `201` allocations (open seats 1 then 0) and one `409 class_full`; it did not
  oversell the eight-seat sample.
- Live rate-limit test, one forwarded client IP: requests 1–10 to
  `/api/demo/session` returned 200; requests 11–15 returned 429 with
  `Retry-After: 3`, `X-RateLimit-Limit: 10`, and `X-RateLimit-Remaining: 0`.
  The observed burst allowance is **10**.

## Findings

### P0 — Core paid-school job is not implemented (release blocker)

The brief requires a capacity truth layer: calendar connection, real class
capacity/cutoff configuration, public parent booking, periodic reconciliation,
and one-click released-seat offers/waitlist conversion. The deployed app is
only an anonymous hard-coded three-class sample. Its own landing page says
“Accounts and billing come next”; its README says staff accounts, live school
classes, billing, calendar checks and waitlists are planned later; its Terms
say not to use the sample for a real class or rely on it as a school record.
“Start for real” discards the demo and returns to the landing page. No school
can perform the actual job, so this cannot be accepted as `web-with-backend`
product M1 is insufficient for the factory-wide definition of done.

### P1 — Dockerfile violates the mandatory Rust image contract (release blocker)

`Dockerfile` uses `FROM rust:1.89-alpine AS api`. The supplied backend-service
contract explicitly requires `rust:1-slim` or `rust:1-alpine` and forbids
pinning a Rust minor version because ACR builds must resolve with current
stable Rust. The local `docker build` check could not be attempted only because
Docker is absent, so the deployment path remains unverified as well as
non-conformant.

### P2 — Static assets have no cache policy

Live `HEAD` responses for the hashed JS and CSS contain neither `Cache-Control`
nor `ETag`; they expose only `Last-Modified`. The performance contract calls
for long-lived immutable caching of hashed assets. Add immutable cache headers
for `/assets/*` while keeping HTML short-lived.

### P2 — Unknown routes return HTTP 200

`/missing-page` renders the styled in-app 404 view, but the backend fallback
serves `index.html` with HTTP 200 for it. The site-structure contract calls for
a real 404 route. Return status 404 for genuinely unknown server routes while
preserving valid client-side deep links.

## Scope and next steps

The M1 demo itself is credible: capacity changes, cutoff/full blocks, reset,
isolation, input validation, keyboard/mobile accessibility, privacy request
scope, and rate limiting all passed. It should remain a demo sandbox while
the release-blocking real-school workflow is implemented. Before another
acceptance attempt: ship the brief's real end-to-end flow with Sociobot Entra
CIAM/tenant isolation and the required billing path, replace the pinned Rust
image, add cache/404 behavior, then rerun claims, API/browser suites, release
build, container build/run, and live verification.
