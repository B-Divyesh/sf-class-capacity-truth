# Show the right number of class seats — verification 23

Verified 2026-09-06 UTC against <https://class-capacity-truth.sociobot.in>.

- Implementation reviewed: `e449d5725a6bd2e69e2982d60ade5b5001f90093`
- Documentation base: `8793edb0005ebccc1c74212eb6ba82df6b8fbfad`
- Controller milestone: `building-m1`
- Repository plan: M1–M4 are recorded as shipped. M5 remains planned and was
  not treated as a current public capability.

## Verdict

**PASS — zero findings and zero untested claims.** The live health endpoint
reports the exact implementation candidate, and the later documentation commit
contains evidence and handoff updates only. The candidate meets the current
product promises in a fresh browser and a fresh local checkout.

| Severity | Count |
| --- | ---: |
| P0 | 0 |
| P1 | 0 |
| P2 | 0 |
| P3 | 0 |
| Untested claims | 0 |

## First screen and one-click sample

Fresh 1440px desktop and 390px phone browsers showed the following before
scrolling:

- Job: **Show the right number of class seats.**
- Audience: **Language schools and tutoring centres.**
- First action: **Try it with sample data.**
- Result: **See three sample classes next.**

One click opened the isolated Bright Path Languages sample. It showed an open
class, a full class, and a past-cutoff class. The persistent **Demo — sample
data, nothing is saved** label remained visible while a valid booking changed
the count from two open seats to one. Reset restored the seed count, a second
fresh browser received its own seed, and Start for real discarded the sample
and focused the signed-out workspace. Empty name and invalid email made no
booking request; full and cutoff requests returned their expected `409`
responses. No non-demo write occurred in the sample flow.

Evidence: `verification-evidence-23/live/browser-check.json` and its desktop
and phone screenshots.

## Claims and local quality

A new clone at the implementation SHA was clean before verification. `npm ci`
installed the documented dependencies (170 packages, zero reported
vulnerabilities). Every exact command declared in `.factory/claims.json` was
run separately and passed: **24/24**. This includes sample isolation,
full/cutoff boundaries, real-school capacity flow, encrypted calendar polling,
concurrency, offer fallback, price/checkout handoff, privacy, staff roles,
export/delete, restart persistence, zero-config startup, deployment topology,
and forwarded-IP rate limiting.

| Check | Result |
| --- | --- |
| `npm test` | PASS — 8 frontend, 7 Rust unit, 22 API, 2 deployment regressions |
| `npm run typecheck` | PASS |
| `npm run lint` | PASS |
| `npm run build` | PASS — `dist/` and release API binary produced |
| `CI=1 npm run test:e2e -- --retries=0 --reporter=line` | PASS — 28/28 |

The built entry JavaScript chunks gzip to 73.74 kB and 79.59 kB; CSS gzips to
4.62 kB. `claim-results.tsv` lists each declared claim and command.

## Live structure, accessibility, privacy, and performance

- `verify-url.sh` passed: title, `lang=en`, one h1, main landmark, image
  alternatives, labelled controls, and no browser errors.
- The live Playwright + Axe sweep made 87 passing assertions on home, demo,
  signed-out workspace routes, legal pages, and the unknown route. It found
  zero console errors, page errors, serious Axe issues, or critical Axe issues.
- Keyboard testing confirmed the skip link, 44px phone menu, Enter/Escape menu
  operation, focus restoration, 200% text reflow, dark treatment, and reduced
  motion. Public and demo requests stayed same-origin until an explicit sign-in
  action. No service worker is registered, matching the absence of an offline
  promise.
- The live 404 deliberately returned HTTP 404. It has one h1, legal links,
  factual `404 error` / `This page was not found.` copy, and a working home
  action. This expected status is not a defect.
- All 24 crawled links succeeded, apart from intentional `mailto:` actions and
  the deliberate 404 anchor.
- Fresh mobile Lighthouse: **100 Performance, 100 Accessibility, 100 Best
  Practices, 100 SEO**; LCP 0.2s, CLS 0, TBT 0ms.

## Backend, isolation, and limits

`GET /health` returned HTTP 200 with the exact candidate build and
`database: ready`. The owned topology verifier confirmed one replica, `PORT`
at 8080, and the owned Azure Files `/data` mount for SQLite and generated
keys. Fresh forwarded-IP checks received ten demo-session 200 responses before
429 with `Retry-After: 3`; protected metrics likewise returned 429 with
`Retry-After: 2` after its allowance. The declared restart and tenant-isolation
claims passed from the clean checkout.

## Earlier findings

I re-read every earlier review and verification report, including their minor
findings. Their current dispositions are proved by the checks above:

| Earlier finding group | Current disposition |
| --- | --- |
| Review 1 and Review 2 public-claim, price, sign-in, privacy, copy, mobile, demo-exit, and 404 findings | Closed. The 24 exact claim commands, route sweep, link crawl, first-read evidence, and copy audit regression cover them. |
| Verification 2–16 capacity, email, billing, route, persistence, topology, metrics, rate-limit, and mobile findings | Closed. The local capacity/durability/deployment claims and current live health, topology, 429, route, and responsive checks pass. |
| Review 3 live SQLite readiness and cancelled-transaction findings | Closed. Current health is ready; the transaction recovery API regression and one-click demo flow pass. |
| Review 4 P3-1 404 metaphor/decorative label | Closed. Live 404 uses factual copy and its home recovery flow passes at phone width and 200% text. |
| Review 4 P3-2 and earlier F-1-22/F-2-19 waitlist jargon | Closed. The candidate uses **saved offer receipts**; the persisted-offer browser outcome passes. |

No earlier finding remains open or has recurred.

## External dependencies

These are separate from the current PASS and are not represented as completed
product capabilities:

- A real protected workspace session needs an authorised school staff account
  in the shared Microsoft CIAM tenant. The redirect/PKCE contract passed.
- Hosted Sociobot/Dodo checkout opens through the tested path. Payment,
  renewal, cancellation, refund, and entitlement completion were not run.
- Production SMTP is not configured. The tested saved-offer URL fallback is
  available; automatic email delivery requires an approved relay.

## Evidence

Fresh evidence is in `.factory/verification-evidence-23/`. No product code,
deployment, cloud setting, or real school record was changed by this
verification.
