# Show the right number of class seats — verification 21

Verified 2026-09-05–06 UTC against
<https://class-capacity-truth.sociobot.in>.

- Implementation reviewed: `739d42da50fff5452ce4704a21b212fc597ebfb6`
- Documentation checkout: `165dc1fa8541e192095b18a73a9ffec2c770f434`
- Controller milestone: `building-m1`
- Repository plan status: M1–M4 are present in this candidate; M5 remains
  planned and was not treated as shipped.

## Verdict

**PASS — zero findings and zero untested claims.** The live service reports the
exact implementation candidate, all 24 declared claims pass from a clean
checkout, and the current public paths pass desktop, phone, accessibility,
privacy, recovery, rate-limit, persistence, and route checks.

| Severity | Finding count |
| --- | ---: |
| P0 | 0 |
| P1 | 0 |
| P2 | 0 |
| P3 | 0 |
| Untested claims | 0 |

## First screen

Before scrolling in fresh 1440×900 and 390×844 browser contexts:

- Job: **Show the right number of class seats.**
- Audience: **Language schools and tutoring centres.**
- First action: **Try it with sample data.**
- Immediate result: **See three sample classes next.**

The job, audience, and action were visible in the first phone and desktop
viewports. In plain words, this lets school staff keep the number of seats
families see aligned with the capacity staff set.

## Claims

I cloned `main` into a new temporary directory, confirmed documentation commit
`165dc1fa8541e192095b18a73a9ffec2c770f434`, ran `npm ci`, and ran every exact
`test` value in `.factory/claims.json` separately. All 24 passed.

| Claim | Result |
| --- | --- |
| `sample-booking-updates-seats` | PASS |
| `full-class-blocks-booking` | PASS |
| `cutoff-blocks-booking` | PASS |
| `demo-reset-isolated` | PASS |
| `school-capacity-flow` | PASS |
| `calendar-poll` | PASS |
| `concurrent-booking-does-not-oversell` | PASS |
| `released-seat-delivery` | PASS |
| `school-plan-price` | PASS |
| `no-third-party-tracking` | PASS |
| `contact-encryption-retention` | PASS |
| `entra-sign-in` | PASS |
| `staff-role-access` | PASS |
| `data-export-delete` | PASS |
| `demo-expiry-input-disposal` | PASS |
| `reconciliation-does-not-change-seats` | PASS |
| `durable-restart` | PASS |
| `configured-smtp-delivery` | PASS |
| `workspace-recovery` | PASS |
| `oldest-waitlist-offer` | PASS |
| `zero-config-runtime` | PASS |
| `forwarded-ip-rate-limits` | PASS |
| `durable-one-replica-topology` | PASS |
| `operational-metrics-no-pii` | PASS |

The landing page, legal pages, workspace copy, README, demo contract, and claim
locations were cross-checked. No broader or missing public claim was found.
The exact command list and timings are in
`.factory/verification-evidence-21/claim-results.tsv`.

## Clean quality gates

| Check | Result |
| --- | --- |
| `npm ci` | PASS — 170 packages, 0 reported vulnerabilities |
| `npm test` | PASS — 8 frontend, 6 Rust unit, 21 API/integration tests, and 2 deployment regressions |
| `npm run typecheck` | PASS |
| `npm run lint` | PASS — TypeScript, rustfmt, and Clippy with warnings denied |
| `npm run build` | PASS — produced `dist/` and the release API binary |
| `CI=1 npm run test:e2e -- --retries=0 --reporter=line` | PASS — 28/28 |
| `npm run test:cold-claim` | PASS — 104 seconds |
| Mobile regression repeated 10 times with no retries | PASS — 10/10 |

The initial home JavaScript is 73,558 bytes gzip, the lazy staff/auth chunk is
79,396 bytes gzip, and CSS is 4,632 bytes gzip.

## Live demo and user paths

- One click opened the isolated Bright Path Languages demo with three
  realistic classes: two open seats, a full class, and a class past cutoff.
- The **Demo — sample data, nothing is saved** label remained visible on the
  list, booking form, and booking result.
- Empty name plus an invalid email activated native validation and sent no
  booking request. Correcting both fields booked one seat and changed the
  visible count from two to one.
- **Reset demo** restored two open seats. After another booking, **Start for
  real** called only the demo leave path, opened `/app`, focused its signed-out
  h1, and a new demo started again with two seats.
- The recorded non-GET requests were only demo book, reset, and leave paths.
  No real-workspace write occurred while the demo label was shown.
- A second fresh context retained its own seed. The API integration suite also
  passed `demo_cookie_isolates_bookings_and_rejects_tenant_input`.
- The full class showed zero open seats, had no booking button, and rejected a
  direct booking with 409 `class_full`.
- The cutoff class had no booking button and rejected a direct booking with
  409 `booking_closed`.
- An ended sample link explained what happened and linked back to a fresh demo.

## Accessibility, phone layout, routes, and privacy

- `/opt/fleet/lib/verify-url.sh` passed in 550 ms: title, `lang=en`, one h1,
  main landmark, image alternatives, labelled buttons, and no console errors.
- Fresh Playwright Axe scans found zero serious or critical issues on home,
  demo, app, privacy, terms, booking, mobile dark/reduced-motion, and 404
  surfaces. Browser logs contained no console or page errors.
- Keyboard checks reached the skip link first, moved focus to main, operated
  booking and the phone menu, and restored route focus. The phone menu measured
  131.17×44.80 CSS pixels.
- At 390 px, every checked route had no horizontal overflow at 200% root text.
  Reduced-motion mode left all animation and transition durations at `0s`.
- Home, demo, app, privacy, terms, every stable signed-out workspace deep link,
  and the auth callback returned 200 with route-specific titles and one h1.
- An unknown route deliberately returned HTTP 404. Its 390 px, 200% text view
  had no overflow, retained Privacy and Terms links, complete metadata, and an
  86 px-high recovery link. This expected 404 is not a defect.
- All discovered same-origin links, booking links, `robots.txt`, `sitemap.xml`,
  favicon, touch icon, and social card returned 200. The Sociobot footer link
  is explicitly labelled as external; it was not fetched because this work
  order forbids reading another service. Mail links were checked as explicit
  `mailto:` links.
- Pre-sign-in traffic across home, demo, app, privacy, and terms was same-origin
  only. There were no third-party fonts, scripts, trackers, or analytics.
  Microsoft sign-in and hosted checkout were reached only after their named
  controls were selected.
- No service worker is registered and offline reload is unavailable. The
  product makes no offline or update promise, so this is not a finding.

Fresh mobile Lighthouse: 100 performance, 100 accessibility, 100 best
practices, and 100 SEO; LCP 1.23 seconds, CLS 0, and total blocking time 0 ms.

## Backend and persistence

`GET /health` returned HTTP 200 and exactly:

```json
{"status":"ok","build":"739d42da50fff5452ce4704a21b212fc597ebfb6","database":"ready"}
```

- Product-only topology readback passed: one replica, `PORT=8080`, and the
  owned `sf-class-capacity-truth-data` Azure Files volume mounted at `/data`.
- The clean durable-restart claim passed in 196 seconds. A committed class,
  booking count, decrypted contact, SQLite file, and generated keys survived a
  release-process restart against a separate direct data directory.
- A live 100-request demo smoke returned 10 accepted and 90 rate-limited
  responses. A separate sequential probe returned ten 200 responses, then 429
  responses with `Retry-After: 5`, limit 10, and remaining 0.
- The earlier top-level metrics exemption is closed. A live 60-request burst
  to `/metrics` returned 40 authentication challenges and 20 rate-limit
  responses; the 429 included `Retry-After: 1` and limit 40.
- Health, HTML, and API responses use no-cache where applicable. Hashed assets
  are immutable. Responses include `nosniff`, strict referrer policy, a
  restrictive permissions policy, and CSP with header-only
  `frame-ancestors 'none'`.

## Earlier finding disposition

Every earlier finding was inspected, including the minor copy findings.

### Review 1

| Finding | Current proof |
| --- | --- |
| F-1-1 | Live Start for real discards demo state, opens `/app`, and focuses the signed-out h1. |
| F-1-2 | First screen now describes entered class capacity, not a room-list connection. |
| F-1-3 | The recorded checkout test asserts USD 99, 9900 cents, and monthly billing; live action reached the HTTPS Dodo host. |
| F-1-4 | CIAM and roles are separate claims; live redirect used the required tenant, client, callback, code flow, and S256 PKCE. |
| F-1-5 | Copy states the tested count change; the live count changed from two to one. |
| F-1-6 | The unsupported student-record claim is gone; current copy directs schools to their existing records. |
| F-1-7 | Live 404 has shell, legal links, metadata, favicon, one h1, and recovery. |
| F-1-8 | The sitemap lists all stable current routes. |
| F-1-9 | `calendar-poll` now names the workspace and README locations. |
| F-1-10 | Phone menu says Open menu/Close menu, works by keyboard, and meets 44 px. |
| F-1-11 | Label is **Live seat preview**. |
| F-1-12 | Label is **Class capacity only**. |
| F-1-13 | Visitor copy consistently uses seat. |
| F-1-14 | Sample name is consistently **Level check: upper primary**. |
| F-1-15 | README title names the job: **Keep class seat counts accurate**. |
| F-1-16 | Capacity-ledger jargon is absent from the opening. |
| F-1-17 | README defines the calendar feed as iCalendar. |
| F-1-18 | README uses the school’s usual email or messaging service. |
| F-1-19 | README introduces configured email delivery before its settings. |
| F-1-20 | Sign-in copy says the school’s Sociobot Microsoft account. |
| F-1-21 | Role copy uses plain Microsoft sign-in wording. |
| F-1-22 | Offer copy uses saved receipt and whether email was sent. |
| F-1-23 | Key copy says secure random key. |
| F-1-24 | The architecture sentence is split into plain sentences. |
| F-1-25 | Replica, mount, and rate-limit statements are split. |
| F-1-26 | Metrics access is split into short plain sentences. |
| F-1-27 | Metrics content is split into short plain sentences. |
| F-1-28 | The unsupported counter-lifetime promise remains removed. |
| F-1-29 | Release verification is split into two sentences. |
| F-1-30 | The untested non-root public promise remains removed. |
| F-1-31 | The broad infrastructure-access promise remains removed. |
| F-1-32 | The unsupported standalone fictional-sample sentence remains removed from public copy. |
| F-1-33 | The factual art-provenance footer claim is gone; the later decorative label is also gone. |
| F-1-34 | Visitor wording is standardised on guardian. |

### Review 2

| Finding | Current proof |
| --- | --- |
| F-2-1 | README says Playwright checks browser flows and directs readers to every claim command. |
| F-2-2 | Every public price says **$99 per school each month**. |
| F-2-3 | Unsupported merchant, refund, and cancellation promises remain removed. |
| F-2-4 | Unsupported controller/processor labels remain removed. |
| F-2-5 | Privacy now states only the tested encrypted queued-email behavior. |
| F-2-6 | Concurrent last-seat protection has its own passing claim. |
| F-2-7 | The unmeasured 99.9% promise remains removed. |
| F-2-8 | The privacy claim now covers fonts, scripts, advertising, and analytics. |
| F-2-9 | The same claim proves sign-in and checkout happen only after explicit actions. |
| F-2-10 | The broad repository infrastructure guarantee remains removed from public docs. |
| F-2-11 | Demo exit destruction is part of the passing demo-reset claim and passed live. |
| F-2-12 | The first screen names language schools and tutoring centres. |
| F-2-13 | Adjacent action text says **See three sample classes next.** |
| F-2-14 | Decorative footer design wording remains removed. |
| F-2-15 | Workspace copy says **Create a class**. |
| F-2-16 | README uses plain 24-hour workspace wording. |
| F-2-17 | README’s sample input sentence has a subject and verb. |
| F-2-18 | Email settings are introduced by their purpose. |
| F-2-19 | Offer wording describes a saved URL and no-email result. |
| F-2-20 | Test instructions say compiled API service, not framework jargon. |
| F-2-21 | README uses school workspaces, not single-instance ledger. |
| F-2-22 | README says metrics response, not Prometheus response. |
| F-2-23 | Deployment guidance stays below the 22-word sentence limit. |
| F-2-24 | Deployment guidance says full commit ID, not SHA. |
| F-2-25 | Visitor copy says Microsoft sign-in; the explicit-action claim passed. |

### Verification 2 through 20

- Verification 2’s waitlist, calendar, offer, identity, roles, privacy,
  cancellation selection, time-zone, claim coverage, keyboard, touch, and docs
  gaps are covered by the 24 passing claims, full test suite, and live checks.
- Verification 3 and 4’s billing and delivery gaps are closed by the live
  hosted-checkout handoff, saved-offer fallback, and configured-email claim.
  Actual purchase completion and optional SMTP are external dependencies below.
- Verification 3 and 5 through 16 repeatedly found unsafe multi-replica,
  ephemeral SQLite topology. The live owned topology now has one replica and
  Azure Files at `/data`; local restart persistence passed.
- Verification 3 and 5’s 200% text and 44 px findings pass on all current
  routes. Verification 5’s claim-registration and 404 findings also pass.
- Verification 7’s plan contradiction is resolved in the repository plan:
  M1–M4 are recorded as shipped and M5 as planned. This report separately
  records the controller stage as `building-m1` and does not require M5.
- Verification 8’s mobile header finding passes live with a labelled keyboard
  menu and a 44.80 px target.
- Verification 14’s missing metrics and deep-link findings pass live. Its flaky
  mobile test passed 10/10 with retries disabled.
- Verification 15’s multiplied rate allowance and ungoverned `/metrics` alias
  pass the current live 10-request and 40-request checks with `Retry-After`.
- Verification 16’s CLS finding passes with fresh CLS 0.
- Verification 18 and 20’s build-identity mismatches are closed: live health
  reports the exact requested implementation candidate. Verifications 17 and
  19 had no remaining product finding.

## External dependencies

These are not product findings and are not presented as completed checks:

- **Microsoft sign-in:** the live redirect and PKCE request are correct. A full
  protected-workspace session still needs an authorised school staff account.
- **Hosted billing:** the live action reached the HTTPS Dodo checkout host.
  No purchase, renewal, cancellation, refund, or entitlement was completed.
- **Email delivery:** production reports `emailDelivery: not_configured`.
  The current live path clearly provides a saved offer for staff to copy. The
  optional encrypted email queue passes its configured test but requires an
  approved SMTP relay before live delivery can be checked.

AI is not part of the current promise and would not improve deterministic seat
counting. No AI feature is required for this milestone. The original visual
assets are hand-authored and their provenance is recorded in
`.factory/design.md`.

## Evidence

Fresh live evidence is in `.factory/verification-evidence-21/`, including
browser screenshots and JSON, route/link metadata, 404, boundary responses,
health and rate-limit headers, Lighthouse output, and clean command summaries.
No product code, deployment setting, or cloud resource was changed.
