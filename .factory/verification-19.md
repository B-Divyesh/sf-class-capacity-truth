# Independent verification 19 — PASS

Verified 2026-09-01 UTC from a clean checkout at candidate commit
`b5ade8e07d3ba4f8adbe1b77fa51a40f34205938` against
<https://class-capacity-truth.sociobot.in>.

## Release decision

**PASS — accept this candidate.** `GET /health` returned HTTP 200 and exactly
`{"status":"ok","build":"b5ade8e07d3ba4f8adbe1b77fa51a40f34205938","database":"ready"}`.
This resolves verification 18's only P1: the live runtime now identifies as
the requested candidate, not its earlier ancestor. The product deployment
also passed the product-scoped one-replica/Azure Files `/data` topology guard.

## First read and demo

**PASS.** On a cold desktop visit the first screen says **“Show the right
number of class seats”**, explains that it is **“For small schools that need
booking counts to match the class capacity they set,”** and makes **“Try it
with sample data”** the first action, with the immediate result “Three sample
classes open next.” In plain words: it lets small language schools show
families the current seat count; it is for school staff who manage group
bookings; click **Try it with sample data** first.

One click opened `/demo?demo=1` with the realistic Bright Path Languages
sample, three classes (open, full, and cutoff), and the persistent **Demo —
sample data, nothing is saved** banner with Reset demo and Start for real.
Evidence: `verification-evidence-19/home-desktop.png` and
`booking-success-desktop.png`.

## Claims gate

`.factory/claims.json` exists with 23 entries. After `npm ci`, every exact
declared command passed from the clean checkout:

- Browser commands: `sample-booking-updates-seats`,
  `full-class-blocks-booking`, `cutoff-blocks-booking`,
  `demo-reset-isolated`, `school-capacity-flow`, `released-seat-delivery`,
  `school-plan-price`, `no-third-party-tracking`, `entra-sign-in`, and
  `data-export-delete` all passed via their individual exact
  `npm run test:e2e -- --grep @claim:...` commands.
- API commands: `calendar-poll`, `contact-encryption-retention`,
  `staff-role-access`, `demo-expiry-input-disposal`,
  `reconciliation-does-not-change-seats`, `configured-smtp-delivery`,
  `workspace-recovery`, `oldest-waitlist-offer`,
  `forwarded-ip-rate-limits`, and `operational-metrics-no-pii` all passed via
  their individual exact `npm run test:api -- ...` commands.
- `npm run test:durable-restart` passed; its release process preserved the
  booked count, decrypted booking, SQLite file, and generated keys across a
  restart on direct `/data`.
- `bash scripts/test-zero-config.sh` passed.
- `npm run test:deployment` passed both topology and traffic-readiness
  regressions.

No declared claim test failed.

## Local quality gates

- `npm ci`: PASS — 170 packages installed; npm reported 0 vulnerabilities.
- `npm test`: PASS — 8 frontend tests, 6 Rust unit tests, 21 API/integration
  tests, and both deployment regression scripts.
- `npm run typecheck`: PASS.
- `npm run lint`: PASS — TypeScript, rustfmt, and Clippy with warnings denied.
- `npm run build`: PASS — Vite emitted `dist/` and Cargo built the release
  API. Initial JavaScript chunks gzip to 73,802 and 79,590 bytes; CSS is
  4,618 bytes gzip (each initial entry is below the 200 KB JS budget).

## End-to-end, privacy, and backend checks

- The declared browser/API coverage exercised normal booking, full and cutoff
  boundaries, isolated reset, class creation, calendar reconciliation that
  does not mutate confirmed seats, waitlist offer conversion, encrypted
  contact retention, owner/viewer role boundaries, export/delete, and durable
  restart persistence.
- A fresh live demo check cleared the guardian name and entered an invalid
  email. Native validation announced “Please fill out this field” and the
  email format problem; correcting both values then reached **Your sample seat
  is booked**.
- A direct live 100-request concurrency smoke used one forwarded IP:
  **10 accepted and 90 rate-limited**. A separate sequential check observed
  ten `200` responses, then request 11 `429` with `Retry-After: 4`,
  `X-RateLimit-Limit: 10`, and `X-RateLimit-Remaining: 0`. Thus the observed
  allowance is 10 requests per forwarded client before the 429 response.
- `scripts/verify-container-topology.sh` against only
  `sf-class-capacity-truth` passed: one replica and the owned Azure Files
  `/data` mount are configured. No unrelated resource, service, storage, or
  secret was read.
- The live browser request log across home, demo, privacy, terms, and signed-
  out workspace pages contained only
  `https://class-capacity-truth.sociobot.in`. No trackers, analytics,
  third-party font, or external script request was observed.
- The CIAM sign-in handoff uses
  `sociobotcustomers.ciamlogin.com`, tenant `35c6fe40-0ec0-46b6-98c6-213ad4de6650`,
  client `25c704f4-465a-47af-80ab-2c489466b697`, the product callback, and
  authorization-code PKCE S256.

## Live routes, headers, accessibility, and responsive behavior

- Browser-verified route titles and one h1: home, demo, privacy, terms,
  workspace sign-in, auth callback, and designed 404. `/`, `/demo?demo=1`,
  `/privacy`, `/terms`, `/app`, `/auth/callback`, `/robots.txt`, and
  `/sitemap.xml` returned 200; a missing route returned 404.
- `/opt/fleet/lib/verify-url.sh` passed: title, `lang=en`, one h1, a main
  landmark, no missing `img` alt attributes, no unnamed buttons, and no
  load-time console/page errors. Its report is
  `verification-evidence-19/verify.json`.
- Playwright Axe found zero serious/critical findings on home, demo, privacy,
  terms, workspace sign-in, and the 404 route. Keyboard verification focused
  the skip link then main; the mobile menu opens with Enter and closes with
  Escape.
- At 390 px in dark mode with reduced motion, there was no horizontal
  overflow; all animation and transition durations were `0s`; the menu target
  measured 131.17 × 44.80 CSS px. Screenshot:
  `verification-evidence-19/demo-mobile-dark-reduced.png`.
- HTML and API responses use no-cache; the hashed JS response uses
  `Cache-Control: public, max-age=31536000, immutable`. Responses carry
  `X-Content-Type-Options: nosniff`, strict referrer policy, restrictive
  permissions policy, and a response-header CSP including
  `frame-ancestors 'none'`.

This is a web-with-backend application, not a library, CLI, or PWA. Consumer
package and service-worker update checks do not apply; the browser found zero
service-worker registrations, consistent with the absence of an offline claim.

## Defects by severity

| Severity | Finding |
| --- | --- |
| P1 | None. |
| P2 | None. |
| P3 | None. |

## Evidence

Live screenshots, browser results, header captures, and route HTML are in
`.factory/verification-evidence-19/`. No application code or product cloud
configuration was changed by this verification.
