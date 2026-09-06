# Show the right number of class seats — review 4

Reviewed 2026-09-06 UTC against
<https://class-capacity-truth.sociobot.in>.

- Implementation reviewed: `a419018d93ec151e2fc136179d290e41cdf5176a`
- Documentation base: `6292a0708483a462933463b43655f559697efef3`
- Verification 22 documentation base: `cf6f3ac0e84e94b269d04e2010a8739a557ed497`
- Controller milestone: `building-m1`
- Repository plan: M1–M4 are recorded as shipped; M5 remains planned.

## Verdict

**FAIL — 2 P3 findings and 0 untested claims.**

The implementation and live product pass every functional, security,
accessibility, performance, demo, persistence, and rate-limit check in this
review. Two plain-word defects remain. PASS requires zero findings of every
severity, so these copy defects block strict acceptance.

| Severity | Count |
| --- | ---: |
| P0 | 0 |
| P1 | 0 |
| P2 | 0 |
| P3 | 2 |
| Untested claims | 0 |

## Findings

### P3-1 — the 404 page uses metaphor and a decorative design label

The live HTTP 404 is structurally correct, accessible, and useful, but it says
**“404 · bead off the rail”** and ends with **“Abacus visual system.”** The
first phrase is a metaphor and the second is a decorative label. Both violate
the attached plain-words contract. The latent SPA not-found component also
uses **“This page has no class”**, another metaphorical heading.

This is a recurrence of the wording covered by F-1-33 and F-2-14. The ordinary
app footer removed the decorative text, but the standalone 404 did not.

Evidence: `review-4-evidence/not-found.html`,
`review-4-evidence/phone-dark-reduced-200.png`, `404.html:30`, `404.html:35`,
and `src/App.tsx:549`.

Required change: use a factual 404 label and heading on both implementations,
remove the design-system footer phrase, and include the 404 copy in the copy
audit.

### P3-2 — the signed-in waitlist screen reintroduces “durable” jargon

The waitlist introduction says **“Review durable offer receipts before
contacting a waiting guardian.”** “Durable” describes storage behavior, not a
school operator's task. Earlier findings F-1-22 and F-2-19 required this idea
to be expressed as a saved receipt and a visible delivery result. The README
uses that plain wording, but the shipped signed-in screen does not.

Evidence: `src/App.tsx:445`; the exact phrase is absent from
`.factory/copy-audit.md`.

Required change: say **“Review saved offer receipts before contacting a
waiting guardian.”** Add the signed-in route copy to the copy audit.

## First screen before scrolling

Fresh 1440 px desktop and 390 px phone contexts showed all four required
items before scrolling:

- Job: **Show the right number of class seats**.
- Audience: **For language schools and tutoring centres**.
- First action: **Try it with sample data**.
- Next result: **See three sample classes next.**

Both views had the correct title, one h1, and one main landmark. Evidence:
`review-4-evidence/live-review.json`, `first-read-desktop.png`, and
`first-read-phone.png`.

## Demo, normal, invalid, boundary, and recovery paths

The live demo passed independently:

- One click loaded three realistic Bright Path Languages classes.
- **Demo — sample data, nothing is saved** remained visible through booking.
- Empty name plus malformed email produced two invalid controls and no booking
  request.
- A valid booking changed two open seats to one.
- Reset demo restored two open seats.
- The full and past-cutoff classes showed their recovery states. Direct
  booking attempts returned `409 class_full` and `409 booking_closed`.
- A fresh second browser still had two open seats.
- Start for real discarded the demo and reached **Sign in to manage class
  capacity**.
- Every non-GET request in the demo flow stayed under `/api/demo`; no real
  school endpoint was written.

Evidence: `review-4-evidence/live-review.json` and
`booking-success-desktop.png`.

## Declared claims

A detached clean checkout at the implementation SHA received `npm ci`. Every
exact `test` value in `.factory/claims.json` then ran separately. All 24
passed.

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

The landing page, demo, workspace copy, legal pages, README, and claim
locations were cross-checked. No false, missing, incomplete, or untested
public product claim was found. The two findings concern wording, not an
unregistered capability claim. Exact results and per-claim logs are in
`review-4-evidence/claim-results.tsv` and `claim-*.log`.

## Clean-checkout quality gates

| Check | Result |
| --- | --- |
| `npm ci` | PASS — 170 packages, zero reported vulnerabilities |
| `npm test` | PASS — 8 frontend, 7 Rust unit, 22 API, and 2 deployment regression tests |
| `npm run typecheck` | PASS |
| `npm run lint` | PASS |
| `npm run build` | PASS — produced `dist/` and the release API binary |
| `CI=1 npm run test:e2e -- --retries=0 --reporter=line` | PASS — 28/28 |
| `/opt/fleet/lib/verify-url.sh` | PASS — title, language, h1, main, alternatives, labels, and console |

The initial JavaScript is 73.74 kB gzip and CSS is 4.62 kB gzip. The lazy
staff/auth chunk is 79.59 kB gzip. Evidence is in
`review-4-evidence/quality-results.tsv`, `quality-*.log`,
`clean-checkout.txt`, and `verify-url/`.

## Accessibility, routes, privacy, links, and performance

- Home, demo, every stable signed-out workspace route, Privacy, Terms, and the
  deliberate 404 had the expected route title, one h1, one main landmark, and
  no serious or critical Axe issue.
- The unknown route correctly returned HTTP 404. The expected status is not a
  defect; only its copy is reported in P3-1.
- At 390 px with 200% text, every checked route had no horizontal overflow.
- The skip link was first, moved focus to main, and the phone menu opened with
  Enter, closed with Escape, and restored focus. Its target was 131 by 44.8 px.
- Dark mode and reduced motion passed. All checked animation and transition
  durations were zero.
- There were no console or page errors. Public and demo traffic stayed
  same-origin before explicit sign-in or checkout.
- No service worker is registered. The product makes no offline or update
  promise.
- The full route crawl found no broken link. Mail links were treated as
  intentional actions.
- Fresh mobile Lighthouse scored 100 performance, 100 accessibility, 100 best
  practices, and 100 SEO. LCP was 1.23 s, CLS was 0, total blocking time was
  0 ms, and transfer was 79,525 bytes.

Evidence: `review-4-evidence/live-review.json`, `link-crawl.json`,
`lighthouse-mobile.json`, `verify-url/`, and `home-headers.txt`.

## Backend, isolation, persistence, health, and limits

- `/health` returned HTTP 200 with `database: ready` and the exact
  implementation SHA.
- The owned topology verifier confirmed one replica and the owned Azure Files
  volume mounted at `/data` for SQLite and generated keys.
- The clean `durable-restart` claim preserved committed capacity and encrypted
  contact across a release-process restart.
- API tests proved tenant scoping, signed-cookie isolation, rejected
  browser-supplied tenant selection, role denial, transaction recovery, and
  concurrent last-seat protection. The live second-browser demo check also
  remained isolated.
- A fresh live demo client received ten HTTP 200 responses, then two HTTP 429
  responses with `Retry-After: 4`.
- A fresh protected metrics client reached the allowance and received HTTP
  429 with `Retry-After: 1`. A separate invalid token returned 401 with
  `WWW-Authenticate: Bearer`.
- `/api/runtime` reports `emailDelivery: not_configured`, matching the visible
  copyable-offer fallback.

Evidence: `review-4-evidence/health.json`, `topology.txt`, rate status and
header files, `invalid-auth-headers.txt`, and the relevant claim logs.

## Earlier finding disposition

All earlier review and verification reports were inspected, including minor
findings.

### Review 1

| Finding | Current disposition |
| --- | --- |
| F-1-1 | Fixed live: Start for real leaves demo and reaches the signed-out workspace. |
| F-1-2 | Fixed: the first screen refers to capacity staff set, not a room list. |
| F-1-3 | Fixed: price proof asserts USD 99, 9900 cents, monthly, and per school. |
| F-1-4 | Fixed: CIAM and staff roles have separate passing claims. |
| F-1-5 | Fixed live: the count visibly changes from two seats to one. |
| F-1-6 | Fixed: copy directs schools to existing records without an unsupported product claim. |
| F-1-7 | Fixed structurally: live 404 has shell, metadata, legal links, h1, and recovery. P3-1 covers its wording. |
| F-1-8 | Fixed: current sitemap lists every stable route. |
| F-1-9 | Fixed: the calendar claim names the workspace and README. |
| F-1-10 | Fixed live: phone menu labels, keyboard operation, and target size pass. |
| F-1-11 | Fixed: **Live seat preview** is factual. |
| F-1-12 | Fixed: **Class capacity only** is factual. |
| F-1-13 | Fixed: visitor copy consistently uses seat. |
| F-1-14 | Fixed: the sample is consistently **Level check: upper primary**. |
| F-1-15 | Fixed: README title names the job. |
| F-1-16 | Fixed: unexplained capacity-ledger wording is absent from the opening. |
| F-1-17 | Fixed: README defines the calendar feed as iCalendar. |
| F-1-18 | Fixed: README names the school's usual email or messaging service. |
| F-1-19 | Fixed: email delivery is explained before settings are named. |
| F-1-20 | Fixed: sign-in copy uses plain Microsoft account wording. |
| F-1-21 | Fixed: role copy uses Microsoft sign-in wording. |
| F-1-22 | Recurred: the README says saved receipt, but the waitlist screen says durable offer receipts. See P3-2. |
| F-1-23 | Fixed: key copy says secure random key. |
| F-1-24 | Fixed: the architecture description uses short sentences. |
| F-1-25 | Fixed: replica, mount, and rate-limit statements are separated. |
| F-1-26 | Fixed: metrics access is split into plain sentences. |
| F-1-27 | Fixed: metrics contents are split into plain sentences. |
| F-1-28 | Fixed: the unsupported counter-lifetime promise remains absent. |
| F-1-29 | Fixed: release verification is split into short sentences. |
| F-1-30 | Fixed: the untested non-root public promise remains absent. |
| F-1-31 | Fixed: the broad infrastructure-access promise remains absent. |
| F-1-32 | Fixed: the standalone fictional-sample promise remains absent. |
| F-1-33 | Recurred on the standalone 404 footer. See P3-1. |
| F-1-34 | Fixed: visitor wording consistently uses guardian. |

### Review 2

| Finding | Current disposition |
| --- | --- |
| F-2-1 | Fixed: README distinguishes browser tests from the full claim list. |
| F-2-2 | Fixed: public prices say $99 per school each month. |
| F-2-3 | Fixed: unsupported merchant, refund, and cancellation promises remain absent. |
| F-2-4 | Fixed: unsupported controller/processor labels remain absent. |
| F-2-5 | Fixed: email copy matches the tested configured and fallback paths. |
| F-2-6 | Fixed: concurrent oversell protection has a passing claim. |
| F-2-7 | Fixed: the unmeasured 99.9% promise remains absent. |
| F-2-8 | Fixed: tracking and third-party asset promises have a passing claim and live evidence. |
| F-2-9 | Fixed: sign-in and checkout occur only after explicit actions. |
| F-2-10 | Fixed: the broad repository infrastructure guarantee remains absent. |
| F-2-11 | Fixed live: Start for real destroys the demo. |
| F-2-12 | Fixed live: the first screen names both audience groups. |
| F-2-13 | Fixed live: the action states that three sample classes open next. |
| F-2-14 | Recurred on the standalone 404 footer. See P3-1. |
| F-2-15 | Fixed: workspace copy says **Create a class**. |
| F-2-16 | Fixed: README uses plain 24-hour workspace wording. |
| F-2-17 | Fixed: README sample-input sentence has a subject and verb. |
| F-2-18 | Fixed: email settings are introduced by purpose. |
| F-2-19 | Recurred on the waitlist screen. See P3-2. |
| F-2-20 | Fixed: test instructions say compiled API service. |
| F-2-21 | Fixed: README uses school workspaces. |
| F-2-22 | Fixed: README says metrics response. |
| F-2-23 | Fixed: deployment guidance stays within the sentence limit. |
| F-2-24 | Fixed: deployment guidance says full commit ID. |
| F-2-25 | Fixed: visitor copy says Microsoft sign-in. |

### Review 3 and earlier verification reports

- Review 3 P0 and P1 remain fixed. The one-click demo works repeatedly;
  health reports ready; and the cancelled/abandoned transaction regressions
  pass in the current API suite.
- Verification 1–4 feature gaps for real classes, calendar checks, waitlists,
  CIAM, roles, checkout, privacy controls, cancellation identity, time zones,
  and released-seat delivery are covered by the 24 passing claims. Live email
  correctly uses the documented no-SMTP fallback.
- Verification 5–16 durability, replica, mounted storage, multiplied rate
  allowance, metrics rate limit, deep-link, 200% text, 44 px target, phone
  header, and layout-shift findings pass the current topology, live browser,
  rate-limit, full browser, and Lighthouse checks.
- Verification 18 and 20 build-identity mismatches are closed: live health
  reports the exact implementation candidate.
- Verification 17, 19, 21, and 22 were PASS reports. Their positive claims
  were independently rerun here; this review's two copy findings supersede
  their zero-finding result for strict acceptance.

## Milestone and external dependencies

The controller stage is `building-m1`. The repository separately records M1
through M4 as shipped and M5 as planned. No M5 capability was required or
treated as available.

- Full protected-workspace use still needs an authorised staff account in the
  shared Microsoft CIAM tenant. The live tenant, client, callback, code flow,
  and S256 PKCE redirect passed.
- The live action reached hosted Sociobot/Dodo checkout. No purchase was made,
  so payment completion and entitlement activation remain external.
- Production has no approved SMTP relay. The current product stores a tested
  offer URL for staff to copy. Automatic email remains externally dependent.

No product code, deployment, cloud setting, or real school record was changed
by this review.

## Evidence

Fresh evidence is in `.factory/review-4-evidence/`.
