# Independent verification 4 — FAIL

Verified 2026-08-29 UTC against candidate commit
`0ae1dfb7f00be2f54650fa14276e3eb820ca77fa` and
<https://class-capacity-truth.sociobot.in>.

## Verdict

**FAIL. Do not release or onboard a school.** The deployed application is the
candidate, its demo and local test suite are in good condition, and the
accessibility/privacy/rate-limit checks below pass. Two production-only
dependencies still prevent the product from doing the paid job in the brief:
the advertised checkout does not exist, and released-seat offers cannot be
sent.

## Release-blocking defects

### P0 — the advertised $99/month plan cannot be purchased

Fresh `GET https://api.sociobot.in/api/v1/products/class-capacity-truth/checkout`
returned **HTTP 404** with:

```json
{"error":"enabled factory product","status":404}
```

The landing page and signed-out workspace both direct a prospective school to
this URL for the $99 monthly plan. This prevents the subscription adoption
flow required by the researched brief.

### P0 — released-seat offers are not delivered in production

Fresh live `GET /api/runtime` returned **200** with
`{"emailDelivery":"not_configured"}`. The workspace accurately says that
an offer is recorded but not sent, but the brief's smallest useful product
requires it to send a one-click released-seat offer. Without a configured
transactional relay, a school cannot convert its waitlist and the real
end-to-end job is incomplete.

There were no additional P1 or P2 findings in this verification.

## Mandatory first-read and demo gates

**PASS.** A cold 1440x900 Chromium visit returned 200. Its first screen said:

- what: “Show the right number of class seats”;
- for whom: small language schools whose calendar and room list disagree; and
- what to click first: **Try it with sample data**.

The single click reached `/demo?demo=1` and immediately presented three
realistic sample class states plus the persistent “Demo — sample data, nothing
is saved” banner, **Reset demo**, and **Start for real**.

## Claims gate

`.factory/claims.json` exists and contains 15 claims. From this clean checkout
I ran `npm ci`, then every listed command separately through the documented
demo/test entry points. All passed. The dedicated `npm run test:cold-claim`
harness additionally cleared the Rust target and completed the first browser
claim in 206 seconds, within its configured 600-second allowance. A direct
warm rerun was also 1/1.

| Claim group | Result |
| --- | --- |
| `sample-booking-updates-seats`, `full-class-blocks-booking`, `cutoff-blocks-booking`, `demo-reset-isolated` | PASS — each exact Playwright command 1/1. |
| `school-capacity-flow`, `calendar-poll`, `released-seat-delivery`, `school-plan-price`, `no-third-party-tracking`, `data-export-delete` | PASS — each exact Playwright command 1/1. |
| `contact-encryption-retention`, `staff-role-access`, `demo-expiry-input-disposal`, `reconciliation-does-not-change-seats`, `durable-restart` | PASS — each exact Rust API command 1/1. |

The release failure is not a missing registry or failing claim test. The
`school-plan-price` claim proves the price/link text, not that the external
checkout completes; `released-seat-delivery` correctly proves the visible
`not_configured` boundary rather than an actual email delivery.

## Clean-checkout quality gates

| Check | Result |
| --- | --- |
| `npm ci` | PASS — 170 packages installed; 0 vulnerabilities reported. |
| `npm test` | PASS — 6 Vitest, 4 Rust unit, 13 Rust API/integration tests. |
| `npm run typecheck` | PASS. |
| `npm run lint` | PASS — TypeScript, Rust format, Clippy with `-D warnings`. |
| `npm run build` | PASS — Vite production output and release Rust binary. |
| `npm run test:e2e` | PASS — 24/24 Chromium tests in 41.1 seconds. |
| `npm run test:cold-claim` | PASS — clean-cache first browser claim in 206 seconds (600-second limit). |
| Production output | PASS — initial app JS 228,262 bytes raw / 69,555 bytes gzip; CSS 17,860 bytes raw / 4,191 bytes gzip. |
| Container image build | NOT RUN — Docker, Podman, Buildah, and Nerdctl are unavailable in this verifier. Dockerfile was inspected; it is multi-stage, non-root, accepts `BUILD_SHA`, and has a `PORT` health check. |

The release binary also started with only `PORT=18083`; it generated and
persisted default keys without printing values and `/health` returned
`{"status":"ok","build":"dev","database":"ready"}`. Its supplied
100-request smoke returned 10 accepted and 90 rate-limited requests.

## Live deployment, functional, privacy, and security evidence

- Live `/health` returned 200 with build
  `0ae1dfb7f00be2f54650fa14276e3eb820ca77fa` and `database: ready`.
  Fresh SHA-256 hashes for the deployed main JS and CSS exactly matched the
  locally built candidate assets.
- In a fresh live demo context, a booking returned 201 and changed the open
  count from two to one; reset restored the seed count. The full and cutoff
  sample states showed their blocking UI. Invalid guardian input returned 422
  with a specific correction, and a missing idempotency key returned 400 with
  “Reload the form, then book again.”
- A bogus workspace bearer token returned 401 with `WWW-Authenticate: Bearer`.
  Explicit sign-in redirected only to
  `sociobotcustomers.ciamlogin.com`, with the specified tenant ID, client ID
  `25c704f4-465a-47af-80ab-2c489466b697`, `/auth/callback`, authorization
  code + PKCE S256, and `openid profile email` scopes.
- A Playwright request capture through landing, demo, privacy, terms, and
  signed-out workspace recorded 18 requests, all same-origin, with no console
  or page error. Microsoft is contacted only after the explicit sign-in action.
- HTML/API responses carried CSP with `frame-ancestors 'none'`, `nosniff`,
  strict referrer policy, and restrictive permissions policy. HTML/API used
  `no-cache, max-age=0`; the hashed JS used
  `public, max-age=31536000, immutable`.
- Live anonymous rate smoke observed 10 successful demo-session requests and
  35 HTTP 429 responses; the first 429 supplied `Retry-After: 5`,
  `X-RateLimit-Limit: 10`, and `X-RateLimit-Remaining: 0`.

## Accessibility and responsive evidence

- Fresh axe scans on `/`, `/demo?demo=1`, `/app`, `/privacy`, `/terms`, and
  the HTTP 404 page found zero serious or critical violations. Every route had
  exactly one H1; expected 404-network logging was the sole console error on
  the missing page.
- The first keyboard Tab focused “Skip to main content”.
- At 390x844 in dark mode, with reduced motion and 200% root text, `/`, demo,
  workspace, privacy, and terms each had `scrollWidth == clientWidth == 390`
  and zero running animations.

## Required operator actions before re-verification

1. Register and enable the recurring Sociobot product
   `class-capacity-truth` at $99/month so its public checkout returns a usable
   hosted flow.
2. Configure a transactional SMTP relay and prove a credentialed
   cancellation-to-inbox-to-one-click-accept flow in production.
3. Re-run the live checkout, delivery, claims, and smoke checks after both
   dependencies are available.
