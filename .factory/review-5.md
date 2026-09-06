# Show the right number of class seats — review 5

Reviewed 2026-09-06 UTC against
<https://class-capacity-truth.sociobot.in>.

- Implementation reviewed: `e449d5725a6bd2e69e2982d60ade5b5001f90093`
- Documentation base: `aad07f5a24a539ee6c6f983a2771d465f5dd992c`
- Controller milestone: `building-m1`
- Repository plan: M1–M4 are recorded as shipped; M5 remains planned.

## Verdict

**PASS — zero findings and zero untested claims.**

The live service reports the exact implementation candidate. The later
documentation commit contains reports, evidence, and handoff updates only.
The current public product and all declared claims pass this strict review.

| Severity | Count |
| --- | ---: |
| P0 | 0 |
| P1 | 0 |
| P2 | 0 |
| P3 | 0 |
| Untested claims | 0 |

## First screen before scrolling

Fresh 1440×900 desktop and 390×844 phone browsers showed the required facts
before scrolling:

- Job: **Show the right number of class seats.**
- Audience: **Language schools and tutoring centres.**
- First action: **Try it with sample data.**
- Next result: **See three sample classes next.**

Both views had the correct route title, one h1, and one main landmark.
Evidence: `review-5-evidence/live-review.json`, `first-read-desktop.png`, and
`first-read-phone.png`.

## Demo, boundaries, and recovery

The one-click sample opened three realistic Bright Path Languages classes: an
open class, a full class, and a class past its cutoff. The persistent **Demo —
sample data, nothing is saved** label stayed visible during the flow.

- Empty name and malformed email left two invalid fields and made no booking
  request.
- A valid booking changed two open seats to one.
- Reset demo restored two open seats.
- Direct full and cutoff attempts returned the expected HTTP 409 results.
- A second fresh browser received its own two-seat seed.
- Start for real discarded the sample and opened the signed-out workspace.
- Every non-GET demo request remained under `/api/demo`; no real-school write
  occurred.

The sample, reset, isolation, normal, invalid, full, cutoff, and exit paths all
passed in the live browser and in their declared claim commands.

## Declared claims

A detached clean checkout at the implementation SHA received `npm ci`: 170
packages installed and npm reported zero vulnerabilities. Every exact `test`
value in `.factory/claims.json` then ran separately. All 24 passed.

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

The landing page, workspace routes, legal pages, README, demo contract, copy
audit, and claim locations were cross-checked. No false, incomplete, missing,
or untested public claim was found. Per-claim logs and exact commands are in
`review-5-evidence/claim-results.tsv` and `claim-*.log`.

## Clean-checkout quality gates

| Check | Result |
| --- | --- |
| `npm test` | PASS — 8 frontend, 7 Rust unit, 22 API, and 2 deployment regressions |
| `npm run typecheck` | PASS |
| `npm run lint` | PASS — TypeScript, rustfmt, and Clippy with warnings denied |
| `npm run build` | PASS — `dist/` and the release API binary were produced |
| `CI=1 npm run test:e2e -- --retries=0 --reporter=line` | PASS — 28/28 |

The built entry JavaScript is 73.74 kB gzip, the lazy staff/auth chunk is
79.59 kB gzip, and CSS is 4.62 kB gzip. Evidence is in
`review-5-evidence/quality-results.tsv` and `quality-*.log`.

## Accessibility, routes, privacy, and performance

- The URL verifier passed with `lang=en`, the correct title, one h1, a main
  landmark, complete image alternatives and control labels, and no console
  errors.
- Playwright and Axe checked home, demo, every stable signed-out workspace
  route, Privacy, Terms, and the unknown route. There were no serious or
  critical violations, page errors, or product console errors.
- The skip link was first in keyboard order and moved focus to main. The phone
  menu opened with Enter, closed with Escape, restored focus, and measured at
  least 44 px high.
- All checked routes reflowed at 390 px with 200% text and no horizontal
  overflow. Dark mode and reduced motion passed; computed animation and
  transition durations were zero.
- Public and demo browsing remained same-origin before an explicit external
  action. No service worker is registered, matching the absence of an offline
  promise.
- The link crawl checked 24 links with no unexpected failure. Mail actions
  were intentional. The unknown route deliberately returned HTTP 404 with one
  h1, factual copy, legal links, and a working home action.
- A normal fresh browser opened hosted Dodo checkout only after the named
  `$99-per-school` action. No purchase was made and no checkout URL is stored
  in this report.
- Fresh mobile Lighthouse scored **100 Performance, 100 Accessibility, 100
  Best Practices, and 100 SEO**. LCP was 1.2 s, CLS was 0, and total blocking
  time was 0 ms.

## Backend, isolation, persistence, and limits

- `GET /health` returned HTTP 200, `database: ready`, and the exact
  implementation SHA.
- The product-only topology verifier confirmed one replica, `PORT=8080`, and
  the owned Azure Files `sf-class-capacity-truth-data` mount at `/data`.
- The clean restart claim preserved a real-school seat change, decrypted
  contact, SQLite file, and generated keys across a release-process restart.
- API tests passed for tenant scoping, stable staff recovery, role denial,
  concurrent last-seat allocation, calendar reconciliation, retention, and
  transaction recovery.
- A fresh live demo client received ten HTTP 200 responses followed by five
  HTTP 429 responses with `Retry-After`.
- A fresh protected-metrics client received forty authentication challenges,
  then HTTP 429 with `Retry-After: 1`. A separate invalid token received HTTP
  401 with `WWW-Authenticate: Bearer`.
- `/api/runtime` reports `emailDelivery: not_configured`, matching the visible
  copyable-offer fallback.

## Earlier finding disposition

All earlier review and verification findings, including minor findings, were
inspected against the current implementation and live runtime.

| Earlier finding group | Current disposition |
| --- | --- |
| Review 1 and Review 2 demo exit, claim coverage, price, sign-in, privacy, copy, mobile, and 404 findings | Closed. All 24 claims, the live first read, copy audit, route sweep, and link crawl pass. |
| Verifications 1–4 missing real-school capacity, calendar, waitlist, identity, roles, billing handoff, privacy controls, cancellation identity, and time-zone behavior | Closed by the exact real-school, calendar, waitlist, CIAM, role, price, retention, and school-capacity claims. The hosted checkout action also opened live. |
| Verifications 3–16 ephemeral or multi-replica SQLite, missing mount, multiplied limits, metrics, deep-link, mobile target/reflow, and layout-shift findings | Closed. Live topology, limits, metrics authentication, routes, 200% phone reflow, and Lighthouse all pass. |
| Review 3 cancelled-transaction outage and false readiness | Closed. Health is ready, the one-click demo works repeatedly, and transaction-recovery regressions pass. |
| Review 4 P3-1 and F-1-33/F-2-14 404 metaphor/decorative label | Closed. Live 404 copy is factual, its footer is plain, and phone recovery passes. |
| Review 4 P3-2 and F-1-22/F-2-19 waitlist jargon | Closed. The current route says **saved offer receipts**, and the persisted receipt outcome passes. |
| Verification 18 and 20 candidate identity mismatch | Closed. Live health reports the exact implementation candidate. |

No earlier finding remains open or has recurred.

## Milestone and external dependencies

The controller stage is `building-m1`. The repository separately records M1
through M4 as shipped and M5 as planned. M5 was not required or presented as a
current capability.

External dependencies are not counted as completed product capabilities:

- A full protected workspace session requires an authorised school staff
  account in the shared Microsoft CIAM tenant. The live tenant, client,
  callback, authorization-code flow, and S256 PKCE redirect passed.
- Hosted Sociobot/Dodo checkout opens. Payment, renewal, cancellation, refund,
  and entitlement completion were not performed.
- Production SMTP is not configured. The tested saved-offer URL fallback is
  available; automatic email delivery requires an approved relay.

## Evidence

Fresh evidence is in `.factory/review-5-evidence/`. No product code,
deployment, cloud setting, or real school record was changed by this review.
