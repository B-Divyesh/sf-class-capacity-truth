# Independent verification 2 — FAIL

Verified on 2026-08-28 against candidate commit
`450bd409fe2a7c81838b376b128fe0504516532f` and the deployed product at
https://class-capacity-truth.sociobot.in.

## Verdict

**FAIL. Do not release this candidate.** The deployment is the candidate and
the sample is polished, fast, accessible, isolated, and rate limited. The real
school path is not the researched product, however. It does not connect or
poll a calendar, does not deliver a released-seat offer, has a broken visible
waitlist submission, has no Sociobot Entra identity or roles, has no $99
subscription, and retains real guardian contact data in plaintext without a
retention/delete/export path. These are release blockers under the brief,
repository definition of done, and supplied backend/auth/privacy contracts.

## Mandatory first-read gate

**PASS.** In a fresh 1440×900 browser context, the first viewport says:

- what: “Show the right number of class seats”;
- for whom: “For small language schools” whose calendar and room list
  disagree; and
- first action: “Try it with sample data”, with “Three sample classes open
  next” beside it.

The one-click action opened `/demo?demo=1` with three populated classes. The
cold page returned 200, requested only the same-origin HTML, one JS file, and
one CSS file, and logged no console or page errors. Screenshot:
`.factory/qa-artifacts/live-cold-desktop.png`.

The explicit first-read gate passes even though the required $99 price/privacy
facts and paid tier are absent from the first screen and later page.

## Required claims

`.factory/claims.json` exists with five entries. After `npm ci`, every exact
declared command was run separately from the clean checkout through the demo
entry point:

| Claim | Result | Fresh evidence |
| --- | --- | --- |
| `sample-booking-updates-seats` | PASS | Exact grep command: 1/1 passed; 2 open became 1 open. |
| `full-class-blocks-booking` | PASS | Exact grep command: 1/1 passed; UI and API block the full class. |
| `cutoff-blocks-booking` | PASS | Exact grep command: 1/1 passed; UI and API block the past-cutoff class. |
| `demo-reset-isolated` | PASS | Exact grep command: 1/1 passed; reset and fresh-context isolation passed. |
| `school-capacity-flow` | PASS, test gap | Exact grep command: 1/1 passed, but the test bypasses the visible waitlist form with `page.evaluate(fetch(...))`; the customer form is broken live. |

The claim commands pass, but the fifth test does not prove the observable
end-to-end workflow its wording implies. Live evidence below reproduces the
missed defect.

## Clean-checkout tests and build

| Check | Result |
| --- | --- |
| Candidate identity | PASS — `git rev-parse HEAD` was exactly `450bd409fe2a7c81838b376b128fe0504516532f`. |
| `npm ci` | PASS — 168 packages added/169 audited; 0 vulnerabilities reported. |
| `npm test` | PASS — 4 Vitest tests, 3 Rust unit tests, and 7 Rust integration/API tests. |
| `npm run test:api` | PASS — 7/7. |
| `npm run test:e2e` | PASS — 17/17 Chromium tests. |
| `npx tsc -b --pretty false` | PASS. |
| `cargo fmt --manifest-path services/api/Cargo.toml --check` | **FAIL** — committed `db/mod.rs`, `routes/mod.rs`, and `tests/api.rs` produce extensive formatting diffs. |
| `cargo clippy --manifest-path services/api/Cargo.toml --all-targets --all-features -- -D warnings` | **FAIL** — `too_many_arguments` and two `result_large_err` errors. |
| `npm run build` | PASS — exact Vite/TypeScript and optimized Rust build; `dist/` produced. |
| Default runtime | PASS — release binary started with `env -i PORT=18081`, generated/persisted defaults, and `/health` returned `{"status":"ok","build":"dev","database":"ready"}`. Temporary `/data` test files were removed afterward. |
| Container build | NOT RUN — no Docker, Podman, Buildah, or Nerdctl is installed in this verifier. Source uses the required unpinned `rust:1-alpine`, non-root runtime, and build arg. |

Production output was 217,875 bytes raw / 66.77 KB gzip JavaScript and
16,179 bytes raw / 3.88 KB gzip CSS. It is within the supplied compressed
first-load budgets.

## Live deployment identity, headers, and performance

- `GET /health` returned 200 with build
  `450bd409fe2a7c81838b376b128fe0504516532f` and database `ready`.
- The live JS and clean local production JS have the same SHA-256:
  `c7597fa7894bebf94101d5dcbc2be582c16cdb2f6c7122ab96977fa25f3e68f2`.
- HTML uses `Cache-Control: no-cache, max-age=0`; hashed JS/CSS use
  `public, max-age=31536000, immutable`; an unknown route returns a styled
  HTTP 404.
- Responses include a same-origin CSP with `frame-ancestors 'none'`,
  `nosniff`, strict referrer policy, and a restrictive permissions policy.
- A fresh demo cookie was `HttpOnly; SameSite=Strict; Max-Age=86400; Secure`.
- The supplied `verify-url.sh` passed: title, `lang=en`, one H1, one main,
  no missing alt, no unlabeled button, and no load errors. Evidence:
  `.factory/qa-artifacts/verify-url/verify.json`.
- Fresh mobile Lighthouse: performance 97, accessibility 100, best practices
  100, SEO 100; FCP 1.4 s, LCP 1.4 s, TBT 190 ms, CLS 0; 72,143 transferred
  bytes and zero third-party bytes. Evidence:
  `.factory/qa-artifacts/lighthouse-home.json`.

## Live functional evidence

### What passed

- Workspace/class creation, publication, parent booking, and API persistence
  worked with example.org data.
- A capacity-one live concurrency test returned one 201 and one 409, so it
  did not oversell.
- Invalid capacity 0 returned 422 with a recovery message; an invalid
  waitlist consent returned 422; a reused offer returned 409.
- Reopening the workspace with its opaque key returned the persisted class;
  another workspace key received 404 for that class.
- Aborting the first demo load showed an error and “Try loading again”; retry
  recovered all three sample classes.
- Direct backend lifecycle returned: workspace 201, class 201, publish 200,
  waitlist 201, release 200, offer view 200, first accept 200, second accept
  409.

### P0 — The visible waitlist path silently fails

On a live full class, entering a guardian name/email and choosing **Join
waitlist** sends a successful 201 response with `Content-Length: 0`. The
client's shared `responseJson()` then tries `response.json()`. The error is
stored in state, but the waitlist branch renders no error element. The form
therefore remains unchanged with no success, no error, and no next step even
though the server inserted the guardian. Retrying can create duplicates.

Evidence: `.factory/qa-artifacts/live-waitlist-result.png`. The response was
201 with an empty body and the live page still showed the original waitlist
form with zero alerts. The passing claim test misses this by inserting the
waitlist entry with a direct fetch instead of operating the form.

### P0 — There is no calendar connection or recurring reconciliation

The control labelled **Connect one calendar** saves only a user-entered label.
The live API returned `{provider:"manual_calendar", enabled:true}`. The
database constrains the provider to `manual_calendar`; there is no OAuth
connector, external-event model, cursor/webhook, polling job, or five-minute
reconciliation. Staff must manually type a count on blur. This is the old
spreadsheet-style workaround, not the brief's smallest useful product.

### P0 — No released-seat offer is delivered

The UI says a waiting guardian agrees to receive an offer “by email” and the
success copy promises “We will email one expiring offer”. There is no email
transport, outbox, background delivery job, or provider integration. Releasing
a seat prints the secret offer URL into the operator's generic message area;
the operator would have to copy and send it manually. This is both a missing
core job and a false, unlisted product claim.

### P0 — Required identity, roles, and paid access are absent

Anyone can anonymously create a durable real-school workspace. Its sole owner
credential is `cct_owner_…` in `localStorage`; there is no sign-in, sign-out,
recovery, membership, owner/operator/viewer authorization, or multi-device
access. The required `@azure/msal-browser` package, CIAM authority
`sociobotcustomers.ciamlogin.com`, OIDC discovery/JWKS validation, `oid` user
key, and bearer-token validation are absent.

The $99/month school subscription and Sociobot billing calls are also absent.
There is no price, checkout, entitlement, renewal/cancellation, or grace
state. A stranger can neither securely adopt nor pay for the researched
product.

### P0 — Real guardian data lacks the promised privacy controls

Real booking and waitlist names/emails are stored as plaintext `TEXT` in
SQLite. Only demo tenants are cleaned up. Real tables have no expiry/retention
job, encryption, export, deletion, or access-request path. The privacy page is
titled “Privacy in the sample” and does not state the real workspace's
retention, controller/processor roles, regional rights, or data deletion.
This violates the brief's child/parent-data and regional-privacy constraints.

The observed browser request log was privacy-positive: landing, demo, app,
and legal flows used only `https://class-capacity-truth.sociobot.in`; no
tracker, analytics, CDN font, or external script request occurred. The CSP
also restricts runtime connections to self. That network behavior does not
remedy unsafe server-side retention.

### P1 — Staff can cancel the wrong family's place

**Release one confirmed seat** does not select a cancellation. The backend
queries the oldest confirmed booking and cancels it. The staff UI exposes no
booking list, guardian identifier, or confirmation. A real cancellation can
therefore remove an unrelated family's confirmed place.

### P1 — Class times change with the operator's browser time zone

The form has no school time-zone control. It parses `datetime-local` in the
browser's zone but always sends and displays `Europe/London`. In an
`America/New_York` context, entering `2030-06-10T10:00` produces epoch
`1907330400` and the product displays `Mon 10 Jun, 15:00`. A non-UK operator
cannot publish the intended class time or cutoff.

### P1 — Claims registry does not cover material product promises

The README/privacy page claim no analytics/advertising requests and that demo
name/email data is discarded, but neither is a claim entry. The real booking
page's email-delivery promise is unlisted and false. The supplied claims
contract makes unlisted claims release-blocking even when an independent
request-log check happens to support one of them.

### P2 — Keyboard/touch/documentation defects

- Route focus runs on the initial page load, so the H1 receives focus before
  keyboard input. The first Tab then lands on “Try it with sample data”,
  bypassing the skip link and header navigation in forward order. The CTA's
  focus ring itself is a visible 3 px outline and route-change H1 focus works.
- At 390 px, the skip link measured 40.8 px high and the Terms target 41.2 px
  wide, below the 44×44 target requirement.
- `sitemap.xml` omits the public `/app` route.
- `.factory/copy-audit.md` still describes the pre-repair landing copy and is
  not a complete audit of the current page.

## Accessibility and responsive evidence

- Axe found zero serious/critical findings (and zero findings of any impact)
  on `/`, `/demo?demo=1`, `/app`, `/privacy`, `/terms`, and the 404 view.
- Every checked page had `lang=en`, one H1, and one main landmark.
- Desktop landing/demo/app/legal loads had no console/page errors. Chromium
  emits the expected failed-resource console line for the intentional 404
  document response.
- At 390×844 in dark mode with reduced motion, demo and app had no horizontal
  overflow, no console error, no serious/critical axe finding, and seat motion
  computed to 0 s. Screenshots:
  `.factory/qa-artifacts/live-demo-mobile-dark-reduced.png` and
  `.factory/qa-artifacts/live-app-mobile.png`.
- With root text at 200%, `/app` had no horizontal overflow or clipped
  interactive control. Evidence:
  `.factory/qa-artifacts/live-app-text-200.png`.

## Rate limits and backend boundaries

Fresh live tests using one forwarded client IP observed:

- demo allowance: requests 1–10 returned 200; requests 11–13 returned 429
  with `Retry-After: 5`, `X-RateLimit-Limit: 10`;
- workspace allowance: requests 1–40 reached the handler (401 for the test's
  invalid key); requests 41–43 returned 429 with `Retry-After: 2` and
  `X-RateLimit-Limit: 40`; and
- health exemption: 45/45 requests returned 200.

The enforced allowances and `Retry-After` behavior pass the backend contract.

## Required next action

Keep the current demo. Before re-verification, implement the actual calendar
connector/poll, visible/idempotent waitlist success and transactional delivery,
specific booking cancellation, Sociobot Entra roles, Sociobot $99 subscription,
encrypted/retained/deletable real contact data, and correct school time zones.
Add honest claim entries/tests that exercise only visible customer paths, then
make formatting and strict Clippy clean and rerun the complete clean/live suite.
