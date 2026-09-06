# Show the right number of class seats — review 3

Reviewed 2026-09-06 UTC at <https://class-capacity-truth.sociobot.in>.

- Implementation candidate: `739d42da50fff5452ce4704a21b212fc597ebfb6`
- Documentation checkout: `1d892aa488ee6a9e791e727033214aea6d1f000c`
- Controller milestone: `building-m1`
- Repository plan: M1–M4 are presented as shipped; M5 remains planned and was
  not required or presented as working.

The commits after the implementation candidate change only `.factory`
reports and evidence. No product file differs between the live candidate and
this documentation checkout.

## Verdict

**FAIL — 2 findings and 0 untested claims.**

| Severity | Findings |
| --- | ---: |
| P0 | 1 |
| P1 | 1 |
| P2 | 0 |
| P3 | 0 |
| Untested claims | 0 |

The code and all declared claim commands pass from a clean checkout. The live
service does not. Its first action cannot load any sample classes because the
single SQLite connection is stuck inside an earlier transaction. The health
endpoint still reports the database as ready.

## First screen before scrolling

Fresh 1440×900 desktop and 390×844 phone contexts were opened before any
scrolling or repository copy review.

- Job: **Show the right number of class seats.**
- Audience: **Language schools and tutoring centres.**
- First action: **Try it with sample data.**
- Stated result: **See three sample classes next.**

All four items were visible above the fold in both contexts. In plain words,
the product is for school staff who need family-facing seat counts to match
the class capacity staff set.

## Findings

### P0 — the one-click sample and core booking proof are unavailable live

The primary **Try it with sample data** action opens `/demo?demo=1`, but the
page shows **The sample did not load** and no class cards. Fresh direct calls
to `GET /api/demo/session` repeatedly returned HTTP 503:

```json
{"code":"demo_unavailable","message":"The sample could not load. Try again."}
```

This was reproduced in fresh desktop and phone browser contexts and with
multiple forwarded client addresses from 00:15 through 00:20 UTC. The browser
logged the failed 503 resource. **Try loading again** did not recover the
sample. The error state keeps the demo label, Reset demo, and Start for real,
and Start for real still opens `/app`; those recovery controls do not restore
the promised sample.

The owned service log gives the immediate cause on every attempt:

```text
failed to initialize demo: error returned from database: (code: 1)
cannot start a transaction within a transaction
```

This breaks the required one-click sandbox and prevents live exercise of
`sample-booking-updates-seats`, `full-class-blocks-booking`,
`cutoff-blocks-booking`, and `demo-reset-isolated`. It also prevents a visitor
from seeing the realistic populated output promised beside the first action.

The implementation uses raw `BEGIN IMMEDIATE`, followed later by explicit
commit or rollback, on a one-connection pool. That pattern is not safe if a
request task is cancelled between those calls: the pooled connection can be
returned with an open transaction. This is consistent with the persistent
live error, but the review does not claim that request cancellation was the
only possible trigger.

Required repair: use SQLx transaction ownership or another cancellation-safe
transaction guard, discard or roll back a poisoned connection, and add a test
that aborts a demo request during its transaction. Prove that the next request
loads three classes without restarting the process. Then deploy and repeat
the live book, invalid, full, cutoff, reset, isolation, and Start for real
paths.

### P1 — health says the database is ready while transaction-backed work fails

During the same failure window, `GET /health` returned HTTP 200 and exactly:

```json
{"status":"ok","build":"739d42da50fff5452ce4704a21b212fc597ebfb6","database":"ready"}
```

The health check can read the database but does not detect that new
transactions fail immediately. A deployment monitor therefore sees a healthy
service while the public first action is unusable.

Required repair: make readiness detect the transaction state needed by normal
writes, without retaining test data. Add a regression that poisons or leaves
open a transaction and proves readiness fails until the connection is safely
recovered.

## Declared claims

A new local clone at documentation SHA `1d892aa` was installed with `npm ci`.
Every exact `test` value in `.factory/claims.json` ran separately. All 24
commands exited successfully, so the untested-claim count is zero.

| Claim | Clean command | Live disposition |
| --- | --- | --- |
| `sample-booking-updates-seats` | PASS | Blocked by P0; sample does not load |
| `full-class-blocks-booking` | PASS | Blocked by P0; sample does not load |
| `cutoff-blocks-booking` | PASS | Blocked by P0; sample does not load |
| `demo-reset-isolated` | PASS | Blocked by P0; sample does not load |
| `school-capacity-flow` | PASS | PASS in isolated signed-in fixture; live CIAM completion is external |
| `calendar-poll` | PASS | PASS in recorded calendar fixture |
| `concurrent-booking-does-not-oversell` | PASS | PASS in API concurrency test |
| `released-seat-delivery` | PASS | PASS in no-SMTP recovery fixture |
| `school-plan-price` | PASS | PASS against recorded USD 99 monthly checkout response |
| `no-third-party-tracking` | PASS | PASS live before explicit sign-in or checkout |
| `contact-encryption-retention` | PASS | PASS at exact retention boundary |
| `entra-sign-in` | PASS | PASS for CIAM host, client, callback, code flow, and S256 PKCE |
| `staff-role-access` | PASS | PASS for server-side role denial |
| `data-export-delete` | PASS | PASS in signed-in fixture |
| `demo-expiry-input-disposal` | PASS | PASS at exact 24-hour boundary |
| `reconciliation-does-not-change-seats` | PASS | PASS in API fixture |
| `durable-restart` | PASS | PASS with a separate data directory and release-process restart |
| `configured-smtp-delivery` | PASS | PASS in configured fixture; live SMTP is external and absent |
| `workspace-recovery` | PASS | PASS by stable staff identity fixture |
| `oldest-waitlist-offer` | PASS | PASS with exact 86,400-second expiry |
| `zero-config-runtime` | PASS | PASS with only PATH, PORT, and frontend path |
| `forwarded-ip-rate-limits` | PASS | PASS locally and live |
| `durable-one-replica-topology` | PASS | PASS in fixture and owned live topology verifier |
| `operational-metrics-no-pii` | PASS | PASS in protected API fixture |

The landing page, legal pages, workspace copy, README, demo contract, and
claim locations were cross-checked. No unlisted or broader claim was found.
The four live demo claims are tested but currently contradicted by P0; they
are not counted as untested.

## Clean-checkout quality gates

| Check | Result |
| --- | --- |
| `npm ci` | PASS — 170 packages, zero reported vulnerabilities |
| `npm test` | PASS — 8 frontend, 6 Rust unit, 21 API, and 2 deployment regression tests |
| `npm run typecheck` | PASS |
| `npm run lint` | PASS — TypeScript, rustfmt, and Clippy with warnings denied |
| `npm run build` | PASS — produced `dist/` and the release API binary |
| `CI=1 npm run test:e2e -- --retries=0 --reporter=line` | PASS — 28/28 |
| `/opt/fleet/lib/verify-url.sh` | PASS — title, language, h1, main, alternatives, labels, and home console |

The built public JavaScript is 73.72 KB gzip, the lazy staff/auth chunk is
79.59 KB gzip, and CSS is 4.62 KB gzip.

## Live routes, accessibility, privacy, and performance

The non-demo live surfaces passed 66 independent browser assertions:

- Fresh phone and desktop views showed the job, audience, action, and stated
  result before scrolling.
- Home, signed-out workspace routes, Privacy, Terms, and the deliberate 404
  had their route-specific titles, one h1, and one main landmark.
- The deliberate unknown route returned HTTP 404. It retained the site shell,
  metadata, legal links, 200% phone reflow, and a 44 px recovery target. The
  expected browser 404 resource message is not treated as a defect.
- Fresh Playwright Axe scans found no serious or critical issue on every
  checked non-demo route. The demo error state also had zero serious or
  critical issues, one h1, one main, and no phone overflow.
- Keyboard checks reached the skip link first, focused main, opened the phone
  menu with Enter, closed it with Escape, and restored focus. The phone menu
  exceeded 44×44 CSS pixels.
- Dark and reduced-motion phone treatment had no overflow. All checked
  animation and transition durations were zero.
- Public, legal, and signed-out routes stayed same-origin before an explicit
  sign-in or checkout action. No tracker, analytics, third-party font, or
  third-party script was observed.
- No service worker is registered. The product makes no offline or update
  promise, so offline/update behavior is not a finding.
- Sitemap, robots, favicon, touch icon, and social card returned 200. The
  sitemap lists all current stable routes.

Fresh mobile Lighthouse on the home page scored 100 for performance,
accessibility, best practices, and SEO. LCP was 1.23 seconds, CLS was 0, total
blocking time was 0 ms, and transfer was 79,840 bytes.

The demo failure itself produces the expected 503 console message and is
covered by P0; it is not hidden by the otherwise passing console sweep.

## Backend, isolation, persistence, and limits

- Live health reports the exact implementation candidate. Finding P1 explains
  why its database status is not sufficient.
- `scripts/verify-container-topology.sh` passed against the owned service: one
  replica and the owned Azure Files data volume mounted at `/data`.
- The clean `durable-restart` claim preserved committed capacity, decrypted
  contact, SQLite, and generated keys across a release-process restart.
- The API isolation suite passed
  `demo_cookie_isolates_bookings_and_rejects_tenant_input`. A second live
  browser received its own cookie, but live sample contents could not be
  compared because P0 prevents seeding either workspace.
- A fresh live demo probe reached the handler ten times, then returned 429
  with `Retry-After: 5`, limit 10, and remaining 0. The first ten responses
  were 503 because of P0.
- A fresh 42-request `/metrics` probe returned 40 authentication challenges
  and 2 rate-limit responses. The 429 included `Retry-After: 1`, limit 40,
  and remaining 0.
- An invalid bearer token received 401 with `WWW-Authenticate: Bearer`.
- `/api/runtime` reports `emailDelivery: not_configured`, matching the visible
  saved-offer fallback and the external dependency below.

## Earlier finding disposition

All earlier reports were read, including every minor copy finding.

### Review 1

| Finding | Current disposition |
| --- | --- |
| F-1-1 | Fixed: Start for real opens `/app` and focuses its h1, including from the live demo error state. |
| F-1-2 | Fixed: first-screen copy refers to capacity staff set, not a room-list connection. |
| F-1-3 | Fixed: the price claim asserts USD 99, 9900 cents, monthly, and per school. |
| F-1-4 | Fixed: CIAM sign-in and role enforcement are separate passing claims. |
| F-1-5 | Fixed: copy states the tested count change. Live execution is now blocked by P0. |
| F-1-6 | Fixed: the unsupported student-record claim is absent; the page directs schools to existing records. |
| F-1-7 | Fixed live: 404 has shell, legal links, metadata, one h1, and recovery. |
| F-1-8 | Fixed live: sitemap lists every stable current route. |
| F-1-9 | Fixed: calendar claim location names the workspace and README. |
| F-1-10 | Fixed live: Open menu/Close menu is keyboard-operable and at least 44 px. |
| F-1-11 | Fixed: section label is **Live seat preview**. |
| F-1-12 | Fixed: section label is **Class capacity only**. |
| F-1-13 | Fixed: visitor copy consistently uses seat. |
| F-1-14 | Fixed: sample name is consistently **Level check: upper primary**. |
| F-1-15 | Fixed: README title names the job. |
| F-1-16 | Fixed: unexplained capacity-ledger wording is absent from the opening. |
| F-1-17 | Fixed: README introduces the calendar feed as iCalendar. |
| F-1-18 | Fixed: README names the school’s usual email or messaging service. |
| F-1-19 | Fixed: README explains configured email delivery before listing settings. |
| F-1-20 | Fixed: sign-in copy uses plain Microsoft account wording. |
| F-1-21 | Fixed: role copy uses plain Microsoft sign-in wording. |
| F-1-22 | Fixed: offer copy says saved receipt and whether email was sent. |
| F-1-23 | Fixed: key copy says secure random key. |
| F-1-24 | Fixed: the architecture list is split into short sentences. |
| F-1-25 | Fixed: replica, mount, and limit statements are split. |
| F-1-26 | Fixed: metrics access is split into short sentences. |
| F-1-27 | Fixed: metrics contents are split into short sentences. |
| F-1-28 | Fixed: the unsupported counter-lifetime promise remains absent. |
| F-1-29 | Fixed: release verification is split into short sentences. |
| F-1-30 | Fixed: the untested non-root public promise remains absent. |
| F-1-31 | Fixed: the broad infrastructure-access promise remains absent. |
| F-1-32 | Fixed: the standalone fictional-sample promise remains absent from public copy. |
| F-1-33 | Fixed: the unsupported art-provenance footer claim and decorative label remain absent. |
| F-1-34 | Fixed: visitor wording consistently uses guardian. |

### Review 2

| Finding | Current disposition |
| --- | --- |
| F-2-1 | Fixed: README distinguishes browser tests from the full claim command list. |
| F-2-2 | Fixed: every public price states $99 per school each month. |
| F-2-3 | Fixed: unsupported merchant, refund, and cancellation promises remain absent. |
| F-2-4 | Fixed: unsupported controller/processor labels remain absent. |
| F-2-5 | Fixed: privacy copy describes only the tested encrypted queued-email behavior. |
| F-2-6 | Fixed: concurrent last-seat protection has a passing claim. |
| F-2-7 | Fixed: the unmeasured 99.9% promise remains absent. |
| F-2-8 | Fixed: the privacy claim covers fonts, scripts, advertising, and analytics. |
| F-2-9 | Fixed: the same claim proves external sign-in and checkout require explicit actions. |
| F-2-10 | Fixed: the broad repository infrastructure guarantee remains absent from public docs. |
| F-2-11 | Fixed in source and clean tests: demo exit destruction is part of the reset claim. Live seeding is blocked by P0. |
| F-2-12 | Fixed live: first screen names language schools and tutoring centres. |
| F-2-13 | Fixed live: action text says **See three sample classes next.** |
| F-2-14 | Fixed: decorative footer wording remains absent. |
| F-2-15 | Fixed: workspace copy says **Create a class**. |
| F-2-16 | Fixed: README uses plain 24-hour workspace wording. |
| F-2-17 | Fixed: README sample-input sentence has a subject and verb. |
| F-2-18 | Fixed: email settings are introduced by purpose. |
| F-2-19 | Fixed: offer wording describes a saved URL and no-email result. |
| F-2-20 | Fixed: test instructions say compiled API service. |
| F-2-21 | Fixed: README uses school workspaces. |
| F-2-22 | Fixed: README says metrics response. |
| F-2-23 | Fixed: deployment guidance stays within the sentence limit. |
| F-2-24 | Fixed: deployment guidance says full commit ID. |
| F-2-25 | Fixed: visitor copy says Microsoft sign-in. |

### Verification reports

| Earlier report | Current disposition |
| --- | --- |
| Verification 1 | Real-school workflow, Rust image tag, cache headers, and real 404 are present and locally tested. |
| Verification 2 | Waitlist response, real calendar polling, saved offer path, CIAM/roles, privacy controls, selected cancellation, time zone, claims, keyboard, touch, sitemap, and copy audit pass current tests. |
| Verifications 3–4 | Hosted checkout and saved-offer fallback are implemented. Purchase completion and SMTP delivery remain external dependencies, not claimed completions. |
| Verifications 3 and 5–16 | Current owned topology verifier passes one replica and Azure Files at `/data`; restart claim passes. The new wedged transaction is P0 and shows durability alone does not guarantee availability. |
| Verification 5 | Claim inventory, one-client limits, and styled 404 pass. |
| Verification 7 | Plan records M1–M4 shipped and M5 planned; this report separately records controller stage `building-m1`. |
| Verification 8 | Phone header follows the recorded keyboard-menu rule and exceeds 44 px. |
| Verification 14 | Protected metrics and direct workspace routes exist; all 28 browser tests pass without retries. |
| Verification 15 | Live per-client allowances and `/metrics` limiting pass with Retry-After. |
| Verification 16 | Fresh home CLS is 0. |
| Verifications 18 and 20 | Live health reports the exact candidate `739d42d`; later commits are report-only. |
| Verifications 17, 19, and 21 | Their prior PASS evidence was checked. Review 3 supersedes it because the live demo is now persistently unavailable. |

## External dependencies and milestone limits

- **Microsoft sign-in:** request shape is covered by the passing CIAM claim. A
  full live staff session still requires an authorised school account.
- **Hosted billing:** the recorded $99-per-school monthly checkout claim
  passes. No payment, renewal, cancellation, refund, or entitlement was
  completed in this review.
- **Email delivery:** production reports `not_configured`. The saved offer for
  staff to copy is the current honest path. Optional delivery needs an approved
  SMTP relay.
- **M5:** planned growth and optional AI drafting were not required. AI would
  not improve deterministic seat allocation. No missed-leverage finding was
  added for the current scope.

This is not a library, CLI, desktop app, or offline PWA. Consumer-install and
service-worker update tests do not apply.

## Evidence

Evidence is in `.factory/review-3-evidence/`. It includes first-read desktop
and phone screenshots, the live demo error screenshot and failed browser
record, 66 passing non-demo live assertions, fresh Lighthouse output, factory
URL verification, and clean command summaries. No product code, deployment
setting, or cloud resource was changed.
