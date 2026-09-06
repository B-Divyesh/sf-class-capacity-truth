# Show the right number of class seats — verification 22

Verified 2026-09-06 UTC against
<https://class-capacity-truth.sociobot.in>.

- Implementation reviewed: `a419018d93ec151e2fc136179d290e41cdf5176a`
- Documentation checkout: `cf6f3ac0e84e94b269d04e2010a8739a557ed497`
- Controller milestone: `building-m1`
- Repository plan: M1–M4 are recorded as shipped. M5 remains planned and was
  not treated as a current promise.

## Verdict

**PASS — zero findings and zero untested claims.** The live health endpoint
reports the exact implementation candidate. The current public product works
from the one-click demo through the checked boundaries, and every declared
claim command passed from a clean checkout.

| Severity | Count |
| --- | ---: |
| P0 | 0 |
| P1 | 0 |
| P2 | 0 |
| P3 | 0 |
| Untested claims | 0 |

## First screen

Fresh 1440 px desktop and 390 px phone browser contexts both showed this
before scrolling:

- Job: **Show the right number of class seats.**
- Audience: **Language schools and tutoring centres.**
- First action: **Try it with sample data.**
- Immediate result: **See three sample classes next.**

This gives a school operator the task, audience, and first action in plain
words. The evidence is `verification-evidence-22/first-read.json` and the
two matching screenshots.

## Claims and clean checks

I cloned the repository into a new temporary directory, checked out
`a419018d93ec151e2fc136179d290e41cdf5176a`, confirmed it was clean, ran
`npm ci` (170 packages, zero reported vulnerabilities), and ran every exact
`test` value in `.factory/claims.json` separately. All 24 passed:

| Claim | Result |
| --- | --- |
| sample-booking-updates-seats | PASS |
| full-class-blocks-booking | PASS |
| cutoff-blocks-booking | PASS |
| demo-reset-isolated | PASS |
| school-capacity-flow | PASS |
| calendar-poll | PASS |
| concurrent-booking-does-not-oversell | PASS |
| released-seat-delivery | PASS |
| school-plan-price | PASS |
| no-third-party-tracking | PASS |
| contact-encryption-retention | PASS |
| entra-sign-in | PASS |
| staff-role-access | PASS |
| data-export-delete | PASS |
| demo-expiry-input-disposal | PASS |
| reconciliation-does-not-change-seats | PASS |
| durable-restart | PASS |
| configured-smtp-delivery | PASS |
| workspace-recovery | PASS |
| oldest-waitlist-offer | PASS |
| zero-config-runtime | PASS |
| forwarded-ip-rate-limits | PASS |
| durable-one-replica-topology | PASS |
| operational-metrics-no-pii | PASS |

The public landing, demo, workspace, legal pages, README, and claim locations
were cross-checked against this inventory. No unlisted public product claim
was found.

The remaining clean-checkout gates passed:

- `npm test`: 8 frontend tests, 7 Rust unit tests, 22 API integration tests,
  and 2 deployment regressions.
- `npm run typecheck`, `npm run lint`, and `npm run build`: PASS. `dist/` and
  the release API binary were produced.
- `CI=1 npm run test:e2e -- --retries=0 --reporter=line`: PASS, 28/28.

## Live demo and recovery paths

- One click loaded the Bright Path Languages sample with three realistic
  classes: two open seats, one full class, and one class past its cutoff.
- The **Demo — sample data, nothing is saved** label stayed visible while
  booking. A valid sample booking changed the count from two open seats to
  one. `live-demo-reset.json` proves Reset demo restored the two-seat seed.
- Empty name and invalid email left two invalid fields and made zero booking
  POSTs (`live-invalid.json`).
- The full and cutoff classes have no booking button. Direct requests returned
  `409 class_full` and `409 booking_closed` respectively
  (`live-boundaries.json`).
- Start for real removed the demo and opened `/app`, whose signed-out heading
  is **Sign in to manage class capacity** (`start-real.json`). No real
  workspace data was created by this demo check.

## Accessibility, routes, privacy, and performance

- `verify-url.sh` passed: a title, `lang=en`, one h1, a main landmark, image
  alternatives, labelled buttons, and no browser errors.
- The fresh live browser sweep found zero console or page errors and zero
  serious or critical Axe issues on home, demo, app, privacy, and terms. It
  verified the skip link, keyboard phone menu, its 131.17 by 44.80 px target,
  dark treatment, and reduced motion.
- Public and demo traffic was same-origin only. Microsoft sign-in reaches the
  correct Sociobot CIAM tenant with code flow and S256 PKCE. No service worker
  is registered; offline reload is unavailable as documented and is not a
  public promise.
- At 390 px and 200% text, home, demo, privacy, terms, and the designed
  unknown route have one h1, route-specific titles, and no horizontal
  overflow (`live-mobile-routes.json`). The unknown route returned its
  deliberate HTTP 404, which is expected.
- Fresh mobile Lighthouse: Performance 100, Accessibility 100, Best Practices
  100, SEO 100; LCP 0.2 s, CLS 0, TBT 0 ms
  (`lighthouse-mobile.json`).

## Backend, isolation, and limits

`GET /health` returned HTTP 200 with:

```json
{"status":"ok","build":"a419018d93ec151e2fc136179d290e41cdf5176a","database":"ready"}
```

- The product-only topology verifier confirms one replica, `PORT=8080`, and
  the owned Azure Files `/data` mount for SQLite and generated keys.
- The clean `demo-reset-isolated` and `durable-restart` claims passed. They
  prove separate demo tenants and committed capacity/contact persistence
  across a release-process restart without accessing real school data.
- A fresh forwarded IP received ten `200` demo-session responses and then
  `429` with `Retry-After: 4` (`demo-rate-429-headers.txt`).
- A fresh forwarded IP received forty protected `/metrics` challenges, then
  `429` with `Retry-After: 1` (`metrics-rate-429-headers.txt`).

## Earlier findings

I inspected the earlier reviews and verification reports, including their
minor findings. Review 1's 34 findings remain closed: the demo exit, exact
price proof, CIAM path, 404, sitemap, mobile menu, copy terms, and public
claim registration all have current browser or claim evidence. Review 2's 25
findings remain closed: price unit, checkout wording, privacy boundaries,
email fallback, concurrency, tracking, demo destruction, and plain wording
are covered by the current inventory and surfaces. Review 3's two live
SQLite/readiness failures are closed by the current health check and working
one-click demo; the cancellation-safe transaction regressions pass in the
22-test API suite. Earlier build-identity, replica, `/data`, metrics,
mobile-header, 200%-text, and 404 findings also pass the checks above.

## External dependencies

These are outside the current product verification and are not presented as
completed product capabilities:

- A full protected workspace session needs an authorised school staff account
  in the shared Microsoft CIAM tenant.
- Hosted Sociobot/Dodo checkout opened correctly in its recorded test and
  live action path. Payment, renewal, cancellation, refund, and entitlement
  completion were not performed.
- Production has no approved SMTP relay. The current tested path provides a
  saved offer URL for staff to copy; live email delivery needs that relay.

## Evidence

Fresh evidence is in `.factory/verification-evidence-22/`. No product code,
cloud setting, deployment, or real school record was changed by this
verification.
