# Polish 2 handoff — PASS (2026-09-02)

Work order: `class-capacity-truth-polish-2`
Repair commits: `1a8ea791b2bc536caef11473aace2cb5e1af2b44` and
`aafbf033359d67053c7b9358b35451cf135f852a`

## Result

All 25 findings from `.factory/review-2.md` are resolved. The public copy now
uses the exact price, **$99 per school each month**, and removes unsupported
merchant, refund, legal-role, infrastructure, and availability promises. The
claim inventory now covers the demo exit, exact price/checkout, encrypted email
queue, concurrent last-seat booking, third-party assets, and explicit
Microsoft-sign-in/Sociobot-checkout actions. The classroom-abacus identity,
SQLite `/data` storage, one-replica deployment, and isolated `?demo=1` path
are preserved.

## Verification

- Clean clone at `aafbf03`: `npm ci`, then every one of the **24** commands in
  `.factory/claims.json` — PASS (exit status 0).
- `npm test` — PASS: 8 TypeScript tests, 6 Rust unit tests, 21 API tests, and
  both deployment fixtures.
- `npm run test:durable-restart`, `bash scripts/test-zero-config.sh`,
  `npm run lint`, `npm run build`, and full `CI=1 npm run test:e2e --
  --retries=0 --reporter=line` — PASS.
- Browser suite covers keyboard operation, route focus, 390px reflow, 200%
  text, dark/reduced motion, demo isolation/reset/exit, headers/404, request
  privacy, CIAM PKCE, and Axe serious/critical-zero checks.
- ACR run `ch1st` built
  `sociobotregistry.azurecr.io/sf-class-capacity-truth:aafbf033359d` from a
  `.git`-free 243.455 KiB archive.
- Product-only guarded deployment set revision `sf-class-capacity-truth--p2-aafbf03`,
  one replica, and Azure Files `sf-class-capacity-truth-data` at `/data`.
  Live `/health` returned `status: ok`, `database: ready`, and build
  `aafbf033359d67053c7b9358b35451cf135f852a`.
- Cold live checks passed for `/`, `/demo?demo=1`, `/privacy`, `/terms`, and
  `/app`; the unknown-route check returned HTTP 404. Live first-read/legal/demo
  assertions passed, including the revised audience, sample result, price,
  privacy copy, and three sample classes.
- Live browser evidence is in `.factory/evidence-polish-2/live/`:
  `browser-smoke.json`, `home-desktop.png`, `booking-success-desktop.png`, and
  `demo-mobile-dark-reduced.png`.

## Known gaps

None.

---

# Review 2 handoff — FAIL (2026-09-01)

Work order: `class-capacity-truth-review-2`

Reviewed source: `b94d6cc70fd2e613f3fb8093d0b101c0fd330c19`

Live application build: `b5ade8e07d3ba4f8adbe1b77fa51a40f34205938`

## Result

**FAIL — 25 findings remain (10 major, 15 minor).** The full report is
`.factory/review-2.md`. No product code, deployment, or cloud resource was
changed.

The first screen is clear at 390 px and desktop. The one-click demo loads three
realistic classes, booking changes two open seats to one, reset and fresh
contexts return to two, **Start for real** destroys the demo and focuses the
real workspace, and observed pre-auth traffic is same-origin. All 23 declared
claim commands pass independently.

The review still fails because the README incorrectly says Playwright verifies
every claim; price copy omits the per-school unit; billing, privacy,
concurrency, availability, third-party asset, explicit-action, and repository
scope promises are absent from or broader than `.factory/claims.json`; and 15
plain-language findings remain. Every F-1 finding was independently rechecked
and remains fixed.

## Verification

- `npm ci` — PASS, zero reported vulnerabilities.
- All 23 commands from `.factory/claims.json` — PASS independently.
- `npm test` — PASS: 8 frontend, 6 Rust unit, 21 Rust API, and 2 deployment
  regression tests.
- `CI=1 npm run test:e2e -- --retries=0 --reporter=line` — PASS, 29/29.
- `npm run build` — PASS; `dist/` and the release API binary were produced.
- Live browser/Axe sweep — no console or page errors and zero serious/critical
  findings on home, demo, privacy, terms, app, and 404.
- Live route/link/metadata crawl — stable routes and dynamic demo links pass;
  the designed unknown route returns HTTP 404.
- Factory URL verifier — PASS after creating its evidence directory; 681 ms
  load, title/lang/main/h1/alt/button checks pass.

## Left to do

Resolve every F-2 item in `.factory/review-2.md`, update the claim inventory and
tests for any promise that remains, and rerun the complete checklist. PASS
requires zero findings and no unlisted claim.

---

# Repair 17 handoff — PASS (2026-09-01)

Work order: `class-capacity-truth-repair-17`
Verifier report: commit `6608a343748a35ef7ddfda50059453e6adae8e0d`,
`.factory/verification-18.md`
Failed candidate: `2c800aa84529f69f6819d4bf7bea08891832dfce`
Repair commit and deployed candidate:
`382e397b8458ff6ff48e816c6e9df2f3633f37f7`

## Result

**PASS — the exact repair candidate is live.** The public health endpoint now
returns `{"status":"ok","build":"382e397b8458ff6ff48e816c6e9df2f3633f37f7","database":"ready"}`.
It is served by revision `sf-class-capacity-truth--r17-382e397` from image
`sociobotregistry.azurecr.io/sf-class-capacity-truth:382e397b8458`, with one
replica and the work-order Azure Files share
`sf-class-capacity-truth-data` mounted at `/data`.

## Reproduced failure and repair

Before changing source or cloud state, `GET /health` reproduced Verification
18 exactly: it returned healthy/ready with build
`1612b35cb5141a1312e2be93dae26a0a51d59e5a`, rather than requested candidate
`2c800aa84529f69f6819d4bf7bea08891832dfce`. The product-only topology
readback was already correct: one replica, only `PORT=8080`, and the product
Azure Files `/data` mount.

The release command previously allowed `EXPECTED_BUILD_SHA` to be omitted and
then accepted a tag-prefix health check. `scripts/deploy-container.sh` now:

- requires a lowercase, full 40-character `EXPECTED_BUILD_SHA`;
- requires the immutable image tag to equal that SHA's first 12 characters;
- requires traffic-serving `/health` to return the full SHA exactly.

`README.md` documents the complete ACR build and guarded deployment command.
The deployment fixture now rejects an image with no full identity before
making an Azure call, reproduces the literal Verification 18 requested
`2c800aa…`/served `1612b35…` mismatch, and proves a positive deployment only
when the full identity matches.

## Verification

- Clean install: `npm ci` — 170 packages, 0 vulnerabilities.
- `npm test` — PASS: 8 frontend tests, 6 Rust unit tests, 21 API integration
  tests, and both deployment/readiness fixtures.
- `npm run typecheck`, `npm run lint`, and `npm run build` — PASS. Production
  output: 73.80 kB gzip initial JavaScript, 79.59 kB gzip lazy JavaScript,
  and 4.62 kB gzip CSS.
- `CI=1 npm run test:e2e -- --retries=0 --reporter=line` — PASS, 29/29.
  It covers desktop and 390 px mobile, keyboard, 200% text, reduced motion,
  all claims, privacy, CIAM PKCE, route focus, and Axe serious/critical-zero
  checks on every public route.
- `npm run test:durable-restart`, `bash scripts/test-zero-config.sh`, and
  `npm run test:cold-claim` — PASS.
- ACR run `ch1r6` built the `.git`-free 242.461 KiB source archive. Image
  digest: `sha256:6a953eec3500d0c0776c1170029a73826aead189ece5b386ac3995bf620045da`.
- The guarded release command completed successfully, read back the exact
  product template, and checked full live build identity. It did not touch
  any resource outside `sf-class-capacity-truth*`.
- Live factory URL verification passed in 569 ms: title, `lang=en`, one h1,
  main landmark, image alt text, labelled buttons, and no console errors.
  Evidence: `.factory/evidence-repair-17/verify-url/verify.json`.
- `scripts/verify-live-browser.mjs` passed against production. It found no
  console/page errors, no serious/critical Axe violations, same-origin
  pre-sign-in traffic only, no undocumented service worker, correct CIAM
  tenant/client/callback/PKCE, and a 44.8 px mobile menu with no 390 px
  overflow. Evidence: `.factory/evidence-repair-17/live/browser-smoke.json`
  plus the associated desktop/mobile screenshots.
- Live headers confirm no-cache HTML/API, immutable hashed assets, response
  header CSP with `frame-ancestors 'none'`, `nosniff`, strict referrer policy,
  and permissions policy. An unknown route returns HTTP 404.
- Live forwarded-IP rate check: requests 1–10 to `/api/demo/session` returned
  200; request 11 returned 429 with `Retry-After: 4`.

## Tooling note

The standalone `@axe-core/cli` was invoked, but this worker's global
ChromeDriver supports Chrome 152 while the supplied Playwright Chromium is
145, so the CLI cannot create a matching WebDriver session. The preinstalled
Playwright Axe integration is the applicable alternative and passed locally
and live as recorded above. The Lighthouse launcher has the same fleet Chrome
discovery incompatibility; no fresh Lighthouse score is claimed. This repair
does not change frontend assets or runtime UI; Verification 17's preceding
live mobile Lighthouse result was 95/100/100/100.

## Known gaps

None for the product or release. The two standalone browser CLI limitations
above are worker-tool compatibility constraints, not product findings.

---

# Verification 18 handoff — FAIL (2026-09-01)

Work order: `class-capacity-truth-verify-18`
Candidate: `2c800aa84529f69f6819d4bf7bea08891832dfce`
Live URL: <https://class-capacity-truth.sociobot.in>

## Result

**FAIL — do not release this candidate.** All 23 declared claim commands,
the 29-test browser suite, cold claim test, `npm test`, typecheck, lint, and
production build pass from a clean checkout. The first screen is clear and its
one-click sample demo works. Fresh live privacy, accessibility, responsive,
rate-limit, header, caching, and normal sample-booking checks also pass.

The release blocker is deployment identity: live `/health` returns ready with
build `1612b35cb5141a1312e2be93dae26a0a51d59e5a`, not requested candidate
`2c800aa84529f69f6819d4bf7bea08891832dfce`. Git history shows the live build
is an ancestor of the candidate. Deploy this exact candidate and repeat the
health identity check.

Full evidence and commands: `.factory/verification-18.md`. Factory URL-verifier
screenshots and JSON are in `.factory/verification-evidence-18/`.

No product code or cloud resource was changed by this verification.

---

# Polish 1 handoff — PASS locally (2026-09-01)

Work order: `class-capacity-truth-polish-1`  
Repair commit: `f1b5523b527df482d9bd93ad719466e05f56ffc0`
Documentation handoff commit: `1612b35cb5141a1312e2be93dae26a0a51d59e5a`

## Result

All 34 adversarial review findings are addressed. The demo exit now discards its
isolated session and opens `/app` with focus on the real-start heading. The
first-screen promise now matches the implemented capacity workflow. Claims are
complete: price uses a recorded monthly USD 99 checkout fixture; CIAM/PKCE and
server roles are separate claims. Routing, static 404 metadata/shell, sitemap,
mobile menu wording, plain language, terminology, legal links, and footer
provenance copy were corrected without changing the classroom-abacus visual
system.

The complete finding-to-change map is `.factory/polish-1.md`. The one-line
catalog text is `.factory/catalog-description.txt`.

## Local verification

- Clean dependency install: `npm ci` (170 packages; npm reported 0 vulnerabilities).
- `npm test` passed: 8 TypeScript tests, 6 Rust unit tests, 21 API/integration
  tests, and both topology/readiness fixtures.
- `npm run test:api` passed all 21 API tests.
- `npm run test:e2e -- --retries=0 --reporter=line` passed **29/29** browser
  tests, including every browser claim, axe serious/critical-zero checks,
  same-origin privacy recording, 390px reflow, 200% text, dark/reduced motion,
  keyboard navigation, route focus, demo reset/isolation, and standalone 404.
- `npm run test:durable-restart`, `bash scripts/test-zero-config.sh`, and
  `npm run test:deployment` passed.
- `npm run typecheck`, `npm run lint`, and `npm run build` passed. The built
  initial JS is 73.80 kB gzip, lazy JS is 79.59 kB gzip, and CSS is 4.62 kB gzip.
- The product uses the existing Playwright Axe integration; no serious or
  critical accessibility violations were found on the tested routes.

## Deployment and live re-check

- ACR run `ch1pw` built image
  `sociobotregistry.azurecr.io/sf-class-capacity-truth:1612b35cb514` with digest
  `sha256:9b30d9e506cc72bead52cc057680843d1e5bb80297717a6b2c17405ee80a8780`.
- Product-scoped deployment created ready revision
  `sf-class-capacity-truth--polish1-1612b35`. Readback confirms that exact image,
  `minReplicas=1`, `maxReplicas=1`, Azure Files storage
  `sf-class-capacity-truth-data` mounted at `/data`, and only `PORT=8080`.
- Cold `/health` returned `status: ok`, `database: ready`, and build
  `1612b35cb5141a1312e2be93dae26a0a51d59e5a`.
- `scripts/verify-live-browser.mjs` passed live. It wrote
  `.factory/evidence-polish-1/live/browser-smoke.json` and screenshots. The
  report records no console/page errors, no serious/critical Axe issues, only
  same-origin pre-sign-in requests, one h1/main and `lang=en`, 390px no-overflow,
  reduced-motion 0s motion, and expected CIAM authorization-code PKCE S256.
- A cold 390px **Start for real** check reached `/app`; after render, the
  visible “Sign in to manage class capacity” heading had focus. Cold unknown
  route `/definitely-missing-review-1` returned HTTP 404 with header, footer,
  Privacy/Terms links, title, description, canonical, and social metadata.

## Known gaps

None.

---

# Review 1 handoff — FAIL (2026-09-01)

Work order: `class-capacity-truth-review-1`

The adversarial first-read report is `.factory/review-1.md`. No product code
was modified.

## Result

**FAIL — 34 findings.** One is blocking: the demo’s **Start for real** control
sets `/#school-plan` before the home target exists, leaving a 390 px visitor in
the preview with focus on the off-screen h1. Three major findings cover the
unsupported room-list wording and incomplete price/sign-in claim assertions.
The remaining findings cover copy clarity, terminology, claim inventory, 404
metadata/shell consistency, and sitemap coverage.

The cold first screen itself is clear, the one-click sample works, booking and
reset work, separate contexts remain isolated, and observed demo traffic is
same-origin. All 22 declared claim commands passed independently. The full
quality run also passed: `npm test`, typecheck, lint, build, and 27/27
Playwright tests with retries disabled. A read-only check of only
`sf-class-capacity-truth` confirmed one running replica and the Azure Files
`/data` mount. A fresh live Lighthouse attempt crashed its browser tab, so no
new score is claimed.

## Reproduce

```bash
npm ci
npm test
npm run typecheck
npm run lint
npm run build
CI=1 npm run test:e2e -- --retries=0
bash scripts/verify-container-topology.sh
```

For the blocking live defect, open `/demo?demo=1` at 390 px, activate **Start
for real**, and inspect the viewport and active element. The URL becomes
`/#school-plan`, while the visible section is the product preview and focus is
on the home h1.

## Left to do

Resolve every item in `.factory/review-1.md`; PASS requires zero findings and
no untested claim. Re-run the full checklist from a fresh state after repair.

---

# Verification 17 handoff — PASS (2026-09-01)

Work order: `class-capacity-truth-verify-17`

Candidate: `f8b545ad0efc4b1972d3f3447958b7baf5a413f6`

Live URL: <https://class-capacity-truth.sociobot.in>

Full report: `.factory/verification-17.md`

## Result

**PASS — accept the candidate.** Fresh `/health` returns the exact candidate
SHA and `database: ready`. All 22 claim commands, the full repository tests,
typecheck, lint, exact production build, 27/27 no-retry Playwright tests, and
the clean cold-claim check pass.

The mandatory first screen states the seat-count job, names small language
schools, and provides the one-click **Try it with sample data** demo. Fresh live
checks cover normal booking, invalid-input recovery, full/cutoff boundaries,
idempotency, reset/isolation, and a two-seat concurrency race without oversell.

Fresh read-only production evidence shows image tag `f8b545ad0efc`, one ready
replica, `minReplicas=1`, `maxReplicas=1`, and the product Azure Files share
mounted at `/data`. Startup logs report one SQLite connection with persisted
generated keys. This resolves the former deployment-only failure without
relying on its earlier report.

The observed forwarded-client allowances are 10 demo requests and 40
school/metrics requests; excess traffic returns 429 with `Retry-After`, while a
second client remains independent. The live checkout posts to the Sociobot
product endpoint and reaches hosted Dodo checkout. Sign-in uses the required
Sociobot CIAM tenant, client, callback, and PKCE S256.

Live browser checks found only same-origin pre-sign-in traffic, no console/page
errors, and zero serious/critical Axe findings. Desktop, 390 px mobile,
keyboard focus, 200% text, dark mode, and reduced motion pass. Mobile
Lighthouse is 95/100/100/100 with LCP 1.33 s, CLS 0, and 80.5 kB transferred.
HTML/API are no-cache, hashed assets are immutable, security headers and CORS
are correct, and local/live production assets match byte for byte.

Defects by severity: **P0 none; P1 none; P2 none; P3 none.** No product code or
infrastructure was changed. The verifier only added this handoff and
`.factory/verification-17.md`.

---

# Repair 16 handoff — PASS (2026-08-30)

Work order: `class-capacity-truth-repair-16`

Base verifier report: `.factory/verification-16.md` at
`5219b0382ccd2d8528ff44a91f4e8d8c74703204`

Failed candidate: `283758f64e321a3037951b433f24bc79c0622ee6`

## Repaired findings

1. **Durable one-replica state now follows the work-order contract.** The
   production topology, readback verifier, guarded product deployment command,
   API assertion, and exact deployment fixture now require one replica and the
   product Azure Files volume mounted at `/data`. SQLite and both generated
   files (`class-capacity-truth-state-v4.db`, `contact-data.key`, and
   `demo-cookie.key`) live directly in that mount. The product deployment
   guard no longer reads storage credentials or changes shared storage.
2. **The exact Verification 16 failure is a regression fixture.** The fixture
   starts as revision `sf-class-capacity-truth--0000046`, image
   `283758f64e32`, only `PORT=8080`, no volume or mount, and `maxReplicas=3`.
   It proves the readback verifier rejects that exact shape before a simulated
   deployment restores the `/data` mount, exactly one replica, and build
   identity.
3. **A direct mounted-state restart is covered.** `npm run test:durable-restart`
   creates a real workspace/class/booking in one mounted directory, starts a
   new release process, verifies the changed seat count plus decrypted guardian
   contact, and asserts the SQLite database and generated key files remain.
4. **The mobile demo no longer shifts its footer while loading.** Three inert,
   data-shaped sample rails reserve the loaded 390px result height while the
   API request is pending. The new Playwright regression delays the response
   and asserts the loaded result never grows beyond its loading region.

## Local verification

- Clean install: `npm ci` — 170 packages, 0 vulnerabilities.
- `npm test` passed: 8 frontend tests, 6 Rust unit tests, 21 API/integration
  tests, and both Container App topology/readiness fixtures.
- `npm run typecheck`, `npm run lint` (rustfmt + Clippy warnings denied), and
  `npm run build` passed. The build emits 73.85 kB gzip initial JavaScript,
  79.59 kB lazy JavaScript, and 4.62 kB gzip CSS.
- `CI=1 npm run test:e2e -- --retries=0` passed 27/27. It includes desktop,
  390px mobile, keyboard, text zoom, dark/reduced-motion, route/focus,
  same-origin privacy, and Playwright Axe serious/critical-zero coverage.
- All 22 manifest claim commands passed independently, including the exact
  topology fixture, zero-config startup, and direct `/data` restart proof.
- Factory `verify-url.sh` passed locally: 575ms load, one title/h1/main,
  `lang=en`, no console errors, no unlabelled buttons, and no missing image
  alts. Local `/`, `/demo`, `/privacy`, and `/terms` return 200; `/missing-page`
  returns 404. The response supplies CSP response-header `frame-ancestors`,
  `nosniff`, strict referrer policy, permissions policy, and no-cache HTML.
- Mobile Lighthouse on `/demo?demo=1`: performance 100, accessibility 100,
  best practices 100, SEO 100; FCP 1,353ms, LCP 1,505ms, TBT 0ms, CLS 0,
  and 83,937 bytes transferred.
- The product intentionally has no service worker or offline claim. Browser
  smoke confirmed zero registrations and a normal unavailable offline reload;
  its online demo requests were same-origin.

## Deployment evidence

- ACR build `ch1nc` built the `.git`-free source archive for application commit
  `2d3fac5b6e9e431a931aad4a206f9ed7aa933b50`. Image
  `sociobotregistry.azurecr.io/sf-class-capacity-truth:2d3fac5b6e9e` has digest
  `sha256:427da3799ea5ee1298819132621372eedb8c0e406b75217b8848c4a1d214d75f`.
  The guarded deployment created revision
  `sf-class-capacity-truth--r16-2d3fac5-0842`; `/health` returned that exact
  full source SHA and `database: ready`.
- The live persistence drill created synthetic school/class/booking state in
  `/data`, then restarted the sole database owner as revision
  `sf-class-capacity-truth--d-r-1788079326-20845`. Before public traffic moved,
  its revision-specific hostname returned the unchanged confirmed/open count
  and decrypted the exact synthetic guardian name and email. Traffic moved
  only after both checks passed. Cleanup removed the synthetic workspace and
  the one-time credential in revision
  `sf-class-capacity-truth--d-c-1788079326-20845`.
- Final readback after that drill showed one running replica, Single revision
  mode, 100% traffic on the cleanup revision, `minReplicas=1`,
  `maxReplicas=1`, volume `data` backed by
  `sf-class-capacity-truth-data`, mount `/data`, and only `PORT=8080` in the
  environment. Startup logs reported both generated keys as persisted and the
  SQLite journal as `DELETE`; no value or guardian data was logged.
- Live factory `verify-url.sh` passed in 550ms with one title/h1/main,
  `lang=en`, complete image alts, labelled buttons, and no console errors.
  Live mobile Lighthouse scored 100/100/100/100 with FCP/LCP 1,202ms, TBT 0,
  CLS 0, and 80,540 bytes transferred.
- The committed live browser smoke covers desktop booking, 390px dark mode,
  reduced motion, keyboard skip/menu, same-origin requests before explicit
  sign-in, offline behavior, and Axe on home, demo, privacy, terms, workspace,
  and mobile. It recorded zero errors or serious/critical findings, no
  overflow, a 77.6 by 44.8px menu target, zero-duration motion, and the exact
  Sociobot CIAM tenant/client/callback with authorization code plus PKCE S256.
- Live response policy retained response-header CSP with
  `frame-ancestors 'none'`, `nosniff`, strict referrer policy, permissions
  policy, no-cache HTML/API responses, immutable hashed assets, rejected
  unapproved CORS, and a real HTTP 404. Local and live index/initial-JS
  SHA-256 hashes match. The load smoke completed 100 concurrent requests: 10
  accepted and 90 returned 429 with `Retry-After`. Three concurrent requests
  for two sample seats returned 201, 201, and 409 without oversell.

Evidence is under `.factory/evidence-repair-16/`. The product has no service
worker or offline claim, and it is neither a package nor a CLI, so update and
consumer-package checks are not applicable.

## Known gaps / next steps

None. Future releases must use `scripts/deploy-container.sh`; a generic image
update can remove the work-order `/data` mount or create overlapping SQLite
owners. Run `scripts/prove-production-durability.sh` before accepting traffic.

---

# Verification 16 handoff — FAIL (2026-08-30)

Work order: `class-capacity-truth-verify-16`

Candidate: `283758f64e321a3037951b433f24bc79c0622ee6`

Live URL: <https://class-capacity-truth.sociobot.in>

Full report: `.factory/verification-16.md`

## Result

**FAIL — the exact candidate is live, but production again has ephemeral local
SQLite/key storage and permits three replicas.** Fresh Azure readback for
revision `sf-class-capacity-truth--0000046` shows image `283758f64e32`, 100%
traffic, `minReplicas=1`, `maxReplicas=3`, only `PORT=8080`, and no volume or
mount. Startup logs say `database_config="generated-default"` and
`durable_backup="disabled"`. The registered `cct-data` Azure Files storage is
not connected to the revision. The checked-in topology verifier exits 1.

The live demo also records CLS 0.122 against the required <0.1 budget. The
footer moves when asynchronous demo content replaces its loading state.

## What passed

- Mandatory cold first-read and one-click sample demo gate.
- All 22 `.factory/claims.json` commands after `npm ci`; the topology fixture
  passes locally but its production claim is contradicted by live readback.
- `npm test`, typecheck, lint, exact production build, and 26/26 Playwright
  tests with retries disabled.
- Exact live build identity and byte-identical HTML/initial JS/CSS.
- Normal, invalid/recovery, full, cutoff, reset, and concurrent booking paths;
  three requests for two seats produced 201/201/409 without oversell.
- Demo allowance 10 and school allowance 40, each followed by live 429 plus
  `Retry-After`; a second forwarded client kept an independent allowance.
- Same-origin pre-sign-in privacy, secure demo cookie, response headers,
  unapproved-CORS rejection, required Sociobot CIAM/PKCE flow, and live
  Sociobot-to-Dodo checkout navigation.
- Desktop and 390 px mobile, keyboard focus/menu, reduced motion, dark mode,
  route status/link crawl, zero Axe serious/critical findings, and no browser
  errors. Lighthouse: 91/100/100/100, LCP 1.29 s, CLS 0.122, 80.3 kB transfer.

## Required repair

Deploy through `scripts/deploy-container.sh` and read back exactly one replica,
`cct-data` mounted at `/mnt/cct`, the documented data/snapshot environment
paths, and the full build identity. Prove a real booking plus decrypted contact
survive a revision restart before traffic is accepted. Also reserve the demo's
loading/result space so live CLS is below 0.1.

No product code or infrastructure was modified during verification.

---

# Repair 15 handoff — PASS (2026-08-30)

Work order: `class-capacity-truth-repair-15`

Base verifier report: `.factory/verification-15.md` at
`b5eba0370c53036c3ccfbc2ac4304e0faffb1768`

Failed candidate: `cc5542bbec9b12fc8b5f61cd25e50824c563c6c9`

Repaired application source: `0120c8e9a2c8564f61a18cab980ca951226e036a`

Live URL: <https://class-capacity-truth.sociobot.in>

## Repaired findings

1. **Production is durable and single-replica again.** The checked-in guarded
   deployment restored `minReplicas=1`, `maxReplicas=1`, the `cct-data` Azure
   Files volume at `/mnt/cct`, `DATA_DIR=/mnt/cct/keys`, and
   `DURABLE_BACKUP_PATH=/mnt/cct/snapshots/class-capacity-truth.db`. The exact
   Verification 15 drift shape—revision `0000045`, image `cc5542bbec9b`, only
   `PORT`, no volume or mount, and `maxReplicas=3`—is now the deployment
   regression fixture. It must be rejected before the guarded path repairs it.
2. **The documented per-client allowance is no longer multiplied.** Azure
   readback shows one running replica. A live same-client demo probe produced
   exactly 10 successful requests and two `429` responses; a second forwarded
   client immediately received two successful requests. Twelve fresh browser
   contexts each completed Home → Try it with sample data → Book this sample
   class without the former “This sample link has ended” failure.
3. **Top-level `/metrics` is rate limited.** It now shares the school governor
   used by `/api/metrics` and `/api/workspaces/metrics`. The exact regression
   sends 60 requests from one forwarded client and asserts 40 authorization
   challenges followed by 20 `429` responses with `Retry-After` and zero
   remaining allowance. It also proves the API alias shares the exhausted
   bucket and a second client remains independent. The same 40/20 split passed
   live.
4. **Keyboard skip focus is deterministic.** The complete no-retry browser run
   exposed an existing native fragment-focus race. The skip action now focuses
   and scrolls `main#main` explicitly while retaining the real `#main` fallback.
   Its exact no-retry stress run passed 10/10.

The researched brief, artifact class (`web-with-backend`), capacity rules,
demo isolation, Entra flow, Sociobot-hosted billing, visual system, and all
previously passing behavior remain unchanged.

## Local verification

- Clean install: `npm ci` installed 170 packages with 0 vulnerabilities.
- Final gates: `npm test`, `npm run typecheck`, `npm run lint`, and
  `npm run build` passed. This includes 8 frontend tests, 6 Rust unit tests, 20
  API/integration tests, and both deployment regressions. Rustfmt and Clippy
  passed with warnings denied.
- Every command in `.factory/claims.json` passed independently in manifest
  order: 22/22 claims, including real-school flow, encryption/retention,
  durable restart, zero-config startup, forwarded-IP limits, and the repaired
  production-topology fixture.
- `CI=1 npm run test:e2e -- --retries=0`: 26/26 passed. It covers desktop,
  390px mobile, keyboard, 200% text, dark mode, reduced motion, route/history
  focus, same-origin privacy, real 404s, and Axe. The skip-focus stress command
  passed 10/10 with no retries.
- Production assets: 73.61 kB gzip initial JavaScript, 79.59 kB gzip lazy staff
  JavaScript, and 4.62 kB gzip CSS. `dist/` and the release API binary were
  produced.
- `scripts/load-smoke.sh` completed 100 concurrent same-client demo requests:
  exactly 10 accepted and 90 rate limited with `Retry-After`.
- Factory `verify-url.sh` passed locally with no console error, one title/h1/
  main, `lang=en`, complete image alt text, and labelled buttons. Local mobile
  Lighthouse scored 100 performance, 100 accessibility, 100 best practices,
  and 100 SEO; FCP/LCP 1,224 ms, TBT 8.5 ms, CLS 0, transfer 82,342 bytes.
- Docker/Podman are not installed in the worker. The required container build
  was instead exercised by the successful multi-stage ACR build below.

Local screenshots, verify output, and Lighthouse JSON are under
`.factory/evidence-repair-15/local/`.

## Deployment and durability evidence

- ACR build `ch1ey` built
  `sociobotregistry.azurecr.io/sf-class-capacity-truth:0120c8e9a2c8` from the
  committed repair with full build SHA
  `0120c8e9a2c8564f61a18cab980ca951226e036a`. Image digest:
  `sha256:ab6fd7444321912319727d4441df5b6c85dd118c51629f8381e869b719b0749b`.
- The guarded release created
  `sf-class-capacity-truth--r15-0120c8e-034735`. The required persistence drill
  then created auth revision `d-a-1788061698-22373`, booked a synthetic real
  class, created restart revision `d-r-1788061698-22373`, and read the same
  confirmed count and decrypted guardian contact after restart. It deleted the
  synthetic workspace and removed its one-time credential in cleanup revision
  `sf-class-capacity-truth--d-c-1788061698-22373`.
- Final Azure readback shows that cleanup revision healthy and ready with 100%
  traffic, the immutable repair image, exactly one running replica, `cct-data`
  mounted at `/mnt/cct`, and only the documented `PORT`, `DATA_DIR`, and
  `DURABLE_BACKUP_PATH` values. `/health` reports a ready database and the exact
  full repair SHA. Local and live `index.html` hashes match byte for byte.
- Startup logs identify a supplied durable backup and persisted generated
  signing/encryption keys without printing their values. No temporary drill
  secret or synthetic workspace remains.

## Live product evidence

- Twelve of twelve fresh browser contexts opened the ordinary sample booking
  form. One completed a booking from two open seats to one. There were no
  console or page errors. A live 390px dark/reduced-motion check had zero
  horizontal overflow, zero-duration animation/transition, a keyboard-opened
  labelled menu, and zero Axe violations.
- Live Axe checks found zero violations on home, demo, privacy, terms,
  signed-out workspace, operations, and the real 404. Each had one h1 and one
  main. Factory `verify-url.sh` also passed at desktop and 390px.
- Live mobile Lighthouse scored 100/100/100/100; FCP/LCP 1,201 ms, TBT 0,
  CLS 0, and transfer 79,713 bytes.
- Home/API responses are no-cache; hashed assets are immutable. CSP is a
  response header with `frame-ancestors 'none'`; `nosniff`, referrer, and
  permissions policies are present. Approved-origin preflight returns the
  production origin; an unapproved origin receives none. Unknown paths return
  HTTP 404. Every shipped route, sitemap, robot file, icon, and social card
  returned its expected status.
- A signed-out `/metrics` request returns 401, `WWW-Authenticate: Bearer`, and
  limiter headers. The live 60-request regression produced 40×401 and 20×429.
- Before explicit sign-in or checkout, home, demo, privacy, and workspace made
  12 requests, all same-origin, with no errors. No service worker is
  registered. The product makes no offline/update claim, so offline reload and
  service-worker update testing are not applicable.
- Live sign-in used the required Sociobot CIAM tenant, client
  `25c704f4-465a-47af-80ab-2c489466b697`, callback `/auth/callback`, authorization
  code response, and PKCE `S256`. Discovery supplied the GUID issuer and
  Sociobot JWKS URI. No credentials were entered.
- The live $99 action POSTed to the Sociobot billing API, received 200, and
  navigated to `checkout.dodopayments.com`. No payment was submitted.

Live screenshots and machine-readable browser, Axe, checkout, verify-url, and
Lighthouse evidence are under `.factory/evidence-repair-15/live/`.

## Applicability and next releases

- This is not a library or CLI, so package/consumer installation is not
  applicable.
- No operator action remains for this repair. Future releases must finish
  through `scripts/deploy-container.sh`; a generic Container Apps update alone
  can erase the SQLite durability topology and must not receive traffic.

## Reproduce

```bash
npm ci
npm test
npm run typecheck
npm run lint
npm run build
CI=1 npm run test:e2e -- --retries=0
npm run test:api -- regression_top_level_metrics_uses_forwarded_ip_limiter
npm run test:deployment
npm run test:durable-restart
```

---

# Verification 15 handoff — FAIL (2026-08-30)

Work order: `class-capacity-truth-verify-15`

Candidate: `cc5542bbec9b12fc8b5f61cd25e50824c563c6c9`

Live URL: <https://class-capacity-truth.sociobot.in>

Full report: `.factory/verification-15.md`

## Result

**FAIL — production has regressed to the previously rejected ephemeral
multi-replica topology.** The exact candidate image is live and healthy, all 22
claim commands pass locally, all repository gates pass, and the first screen
passes the cold-read/one-click entry test. Fresh Azure readback shows
`maxReplicas=3`, three ready replicas, no volume, and no volume mount.
`scripts/verify-container-topology.sh` exits 1.

The regression is user-visible. With one cookie jar, nine consecutive demo
reads returned eight different class IDs. In 12/12 fresh browser attempts,
clicking the available sample class ended at **“This sample link has ended”**
instead of the booking form. The per-replica 10-request limiter also accepted
20 same-IP requests before returning 429; the 429 correctly included
`Retry-After: 5`. Top-level `/metrics` has a separate P1 defect: 60 same-IP
requests returned 60×401 with no limiter headers or 429.

## Verification summary

- `npm ci`: 170 packages, 0 vulnerabilities.
- Every one of the 22 `.factory/claims.json` commands: PASS locally.
- `npm test`: PASS (8 frontend, 6 backend unit, 19 API/integration, 2
  deployment regressions).
- `npm run typecheck`, `npm run lint`, `npm run build`: PASS.
- `CI=1 npm run test:e2e -- --retries=0`: 26/26 PASS.
- Exact live identity: `/health` reports the full candidate SHA and ready DB;
  local/live HTML, JS, and CSS hashes match.
- Live privacy/auth/billing: same-origin before explicit action; required
  Sociobot CIAM tenant/client/callback and PKCE; hosted Sociobot/Dodo checkout.
- Live accessibility/performance: Axe zero violations on all checked routes;
  390 px/dark/reduced-motion/200% text checks pass; Lighthouse 100/100/100/100,
  LCP 1,276 ms, CLS 0, TBT 14 ms.
- Evidence: `.factory/evidence-15/`.

## Required repair

Deploy through the guarded topology path with exactly one replica, `cct-data`
mounted at `/mnt/cct`, and the documented key/snapshot paths. Read the live
configuration back before traffic, prove a real booking and decrypted contact
survive a revision restart, then repeat the ordinary demo and same-client
rate-limit tests. Put the top-level `/metrics` alias under a forwarded-IP
limiter or remove it.

No product code or infrastructure was modified during verification.

---

# Repair 14 handoff — PASS (2026-08-30)

Work order: `class-capacity-truth-repair-14`
Base verifier report: `.factory/verification-14.md` at
`ca228b938b4946146ac8e25df6c779991c78d2d1`
Failed candidate: `b8349a9ffdf7985edc0331faf6bd2b5a1db7fb44`
Live URL: <https://class-capacity-truth.sociobot.in>
Deployed application source: `2991e638b7619669716ff93514d23a43fbb9720e`

## Repaired findings

1. **Protected operational metrics now exist.** `GET /metrics`, `GET
   /api/metrics`, and `GET /api/workspaces/metrics` all require a valid
   Sociobot Entra bearer plus an owner/operator workspace key. The Prometheus
   response has fixed route labels only and exposes request/error/latency
   totals, calendar job lag, unresolved discrepancies, and released-seat offer
   conversion. It contains no school, class, guardian, email, token, or staff
   identity values. `/app/operations` presents the same metrics and the alert
   thresholds documented in the plan and README.
2. **Every shipped workspace route now survives direct navigation and reload.**
   The Axum service serves the application shell for
   `/app/classes/:id`, `/app/reconciliation`, `/app/waitlist`,
   `/app/settings`, `/app/settings/billing`, `/app/settings/data`, and
   `/app/operations`. The History API router gives each a route-specific title,
   focussed h1, live announcement, and a usable screen. The workspace now has
   labelled section navigation; class cards link to their permanent detail URL.
3. **The 390px test waits for real demo readiness.** It now waits for all three
   class articles before it asserts the loading marker is absent. The exact
   no-retry stress command passed all 10 repetitions.
4. **The skip link now focuses the main landmark.** All routes make their
   `main#main` programmatically focusable. The keyboard E2E test activates
   “Skip to main content” and asserts focus is on `main`, preventing a future
   regression where the link only changed the fragment.

The product class remains `web-with-backend`; the researched brief, demo,
Entra flow, hosted Sociobot billing path, SQLite/Azure Files topology, and all
previously passing capacity behavior are unchanged.

## Local evidence

- Clean install: `npm ci` — 170 packages, 0 vulnerabilities.
- Final quality gates: `npm test`, `npm run typecheck`, `npm run lint`, and
  `npm run build` all passed. The production build emits 73.54 kB gzip initial
  JavaScript, 79.59 kB gzip lazy staff/auth JavaScript, and 4.62 kB gzip CSS.
- Full no-retry browser suite: `CI= npm run test:e2e` — **26/26 passed**.
  This includes desktop/mobile, keyboard navigation, deep-link title/focus and
  history behavior, 200% text/reflow, dark/reduced-motion, same-origin privacy,
  and Playwright Axe checks. The new operations screen also has an Axe
  serious/critical-zero assertion.
- Every one of the 22 `claims.json` commands was run independently after the
  repair. The 21 pre-existing claims passed, and the new
  `operational-metrics-no-pii` claim passed through
  `regression_protected_operational_metrics_are_aggregated_and_contain_no_pii`.
- Exact flake regression: `CI= npm run test:e2e -- --grep 'demo remains usable
  at 390px' --repeat-each=10 --retries=0` — **10/10 passed**.
- Local release binary: `/health` returned ready; `/opt/fleet/lib/verify-url.sh`
  passed with no console errors, one title/h1/main, `lang=en`, and no unnamed
  buttons or missing image alt. Evidence is in
  `.factory/qa-artifacts/repair-14-local/verify-url/`.
- Local direct probes returned 200 for all seven repaired `/app/...` paths;
  unauthenticated `/metrics` and `/api/metrics` both returned 401. The 100-rps
  smoke completed with 10 accepted requests and 90 correctly rate-limited
  responses with `Retry-After`.
- Local mobile Lighthouse: performance 100, accessibility 100, best practices
  100, SEO 100; LCP 1,201.824 ms and CLS 0. Evidence:
  `.factory/qa-artifacts/repair-14-local/lighthouse-mobile.json`.
- `npx @axe-core/cli` was attempted twice against the local server, including
  with the installed Playwright Chromium path. Its Selenium launcher could not
  start a Chrome session in this container. The repository's Playwright
  `@axe-core/playwright` integration ran successfully instead (all relevant
  routes, including operations, have zero serious/critical violations).

## Final deployment and live evidence

- ACR build `ch1cq` built
  `sociobotregistry.azurecr.io/sf-class-capacity-truth:2991e638b7619669716ff93514d23a43fbb9720e`
  from the final source. Image digest:
  `sha256:c9e4dce24af545ba7e0264eac20e2938cf4ea431dc7362359866ad7fe9452774`.
- The guarded deployment created
  `sf-class-capacity-truth--r14-2991e63-20260830015944`. Azure readback shows
  the immutable image above, one ready replica, `minReplicas=1`,
  `maxReplicas=1`, and the `cct-data` Azure Files mount at `/mnt/cct`.
  `/health` returned `status: ok`, `database: ready`, and the exact full
  deployed SHA.
- Live direct navigation returned HTTP 200 for `/app`, class detail,
  reconciliation, waitlist, settings, billing, data, and operations. All
  three metrics URLs returned 401 plus `WWW-Authenticate: Bearer` when
  unauthenticated; the API integration claim proves an owner request receives
  only fixed-label, aggregated Prometheus metrics with no PII.
- Live desktop home/demo/operations and 390px demo browser checks had one h1,
  no console errors, no horizontal overflow, and zero Axe serious/critical
  violations. The keyboard smoke confirms the skip link focuses `main`.
  `verify-url.sh` also passed: title, `lang=en`, main landmark, image alts,
  and labelled buttons are present.
- Live policy probes confirmed HTML is no-cache; hashed assets remain
  immutable; CSP carries `frame-ancestors 'none'`; `nosniff`, referrer, and
  permissions headers are set. CORS returned the production origin only (and
  no `Access-Control-Allow-Origin` for `https://example.invalid`). A fresh
  forwarded IP received an initial demo session plus nine 200 demo calls, then
  429 responses with `Retry-After: 5`.
- A live demo browser session made four same-origin requests and no
  third-party requests before any explicit sign-in or checkout action.
- The signed-out live staff flow redirects to the required Sociobot CIAM
  tenant with client `25c704f4-465a-47af-80ab-2c489466b697`, callback
  `https://class-capacity-truth.sociobot.in/auth/callback`, and PKCE `S256`.
  No credentials were entered.
- Final mobile Lighthouse: performance 100, accessibility 100, best practices
  100, SEO 100; LCP 1,201.16 ms and CLS 0. Final live evidence is in
  `.factory/qa-artifacts/repair-14-live/`.

Offline/service-worker update checks remain not applicable: this product makes
no offline claim and registers no service worker. It is not a library or CLI,
so package-consumer testing is not applicable.

## Run and verify

```bash
npm ci
npm test
npm run typecheck
npm run lint
npm run build
CI= npm run test:e2e
```

For the specific repaired regressions:

```bash
npm run test:api -- regression_protected_operational_metrics_are_aggregated_and_contain_no_pii
CI= npm run test:e2e -- --grep 'release regression: shipped workspace routes load directly' --retries=0
CI= npm run test:e2e -- --grep 'demo remains usable at 390px' --repeat-each=10 --retries=0
```

## Deployment

Deployment is complete and live on the final application source above. No
operator action remains.

---

# Verification 14 handoff — FAIL (2026-08-30)

Candidate: `b8349a9ffdf7985edc0331faf6bd2b5a1db7fb44`

Live URL: <https://class-capacity-truth.sociobot.in>
Full report: `.factory/verification-14.md`

## Result

**FAIL — release blocked by two P1 contract gaps.** The earlier deployment-only
failure is resolved: `/health` reports the exact candidate SHA and a ready
database, local/live frontend hashes match, and Azure readback shows one ready
replica with `cct-data` mounted at `/mnt/cct`, persisted key/snapshot paths,
and 100% traffic.

The remaining blockers are:

1. No protected operational metrics implementation exists. `/metrics` and
   `/api/metrics` both return 404 even though the venture contract and shipped
   plan require metrics for requests/errors/latency, job lag, discrepancies,
   and offer conversion.
2. The shipped plan's product routes do not survive direct navigation.
   `/app/classes/example`, `/app/reconciliation`, `/app/waitlist`,
   `/app/settings`, `/app/settings/billing`, `/app/operations`, and
   `/app/settings/data` all return 404. Available controls are consolidated on
   `/app`, which does not satisfy the deep-link routing contract.

A P2 browser-test defect is also reproducible: the 390px reduced-motion test
checks for the absence of the async loading marker before waiting for demo data.
The standard full E2E run passed only after one retry; a 10-run no-retry stress
test failed twice. The actual live demo completed and remained usable.

## Verification summary

- All 21 commands in `.factory/claims.json` passed independently.
- `npm ci`, `npm test`, `npm run typecheck`, `npm run lint`, and
  `npm run build` passed. Build output: 70.86 kB gzip initial JS, 79.59 kB gzip
  lazy JS, and 4.43 kB gzip CSS.
- Cold first read and one-click sample demo passed. Valid booking, validation
  recovery, full/cutoff blocking, reset, context isolation, and a live
  last-seat race passed without oversell.
- Live rate allowance was 10 requests per forwarded IP: calls 11 and 12
  returned 429 with `Retry-After: 5`.
- Sign-in redirected to the required Sociobot CIAM tenant/client/callback with
  PKCE. The live $99 action reached the hosted Dodo checkout through the
  Sociobot API; no card was submitted.
- Playwright traffic stayed same-origin before explicit auth/checkout. Browser
  product flows had no console/page errors. Security headers, CORS behavior,
  HTML no-cache, and immutable asset caching passed.
- Axe found zero violations on landing, demo, app, privacy, terms, and 404.
  Keyboard focus, 390px dark/reduced-motion, 200% text, and 44px targets passed.
- Mobile Lighthouse: 98 performance, 100 accessibility, 100 best practices,
  100 SEO; LCP 1,422 ms, CLS 0, TBT 139 ms.

Evidence is in `.factory/evidence-14/`. No product code was modified.

---

# Repair 13 handoff — PASS (2026-08-29)

Work order: `class-capacity-truth-repair-13`
Base report: `.factory/verification-13.md` at
`041097c92500f8951bf61be08e92cd23ca5b0ffe`
Repaired executable source: `757c8bdc8e71a78b966135421f9c12db62c56337`
Live URL: <https://class-capacity-truth.sociobot.in>

## Release-blocking repair

Verification 13's only P0 was reproduced from a fresh Azure readback before
repair. Revision `sf-class-capacity-truth--0000044` served image
`791928864bd0`, allowed three replicas, had only `PORT=8080`, and had no volume
or mount. `scripts/verify-container-topology.sh` rejected that state.

The exact regression fixture now uses Verification 13's revision, image,
one-variable environment, absent volume/mount, and `maxReplicas=3`. It proves
that the release guard rejects the failed state, rejects a mismatched running
build, then applies and reads back the required durable template. The checked-in
topology now also records the mandatory `PORT=8080` value, covered by the Rust
claim test. The brief, web-with-backend class, and passing product behavior are
unchanged.

## Deployment and durability evidence

ACR build `ch19u` built the immutable image
`sociobotregistry.azurecr.io/sf-class-capacity-truth:757c8bdc8e71` from the
repaired source. Its digest is
`sha256:44f7b1dfb40caec91196f37e7b81caccd37fd1007411650f0f2da9d0e9cdf9c6`.
All three build identity arguments used the full repaired source SHA.

The guarded deploy first created revision
`sf-class-capacity-truth--r13-757c8bd-20260829`. A controlled persistence drill
then created a synthetic school and encrypted booking, rolled a revision,
verified the same confirmed seat and decrypted guardian contact, deleted the
workspace, and removed its one-time credential. Drill revisions were:

- auth: `sf-class-capacity-truth--d-a-1788047158-22872`;
- restart: `sf-class-capacity-truth--d-r-1788047158-22872`;
- cleanup: `sf-class-capacity-truth--d-c-1788047158-22872`.

Final readback showed the cleanup revision healthy and ready with 100% traffic,
`minReplicas=1`, `maxReplicas=1`, `cct-data` mounted at `/mnt/cct`,
`DATA_DIR=/mnt/cct/keys`, and
`DURABLE_BACKUP_PATH=/mnt/cct/snapshots/class-capacity-truth.db`. There was no
`TEST_AUTH_TOKEN` environment entry or `cct-persist-drill` secret. `/health`
returned the full repaired SHA with `database: ready`.

## Verification evidence

- `npm ci` passed with 0 vulnerabilities. `npm test` passed 7 Vitest tests, 5
  Rust unit tests, 18 Rust API/integration tests, and both deployment tests.
- `npm run typecheck`, `npm run lint`, and `npm run build` passed. `dist/` was
  produced. Initial JS is 70.86 kB gzip, the lazy app chunk is 79.59 kB gzip,
  and CSS is 4.43 kB gzip.
- `npm run test:e2e` passed 25 Chromium tests. Coverage includes all 21 claims,
  desktop, 390px, keyboard, route focus, dark mode, reduced motion, 200% text,
  same-origin privacy logging, and AxeBuilder checks.
- The clean-target browser claim passed in 104 seconds against its 600-second
  limit. Zero-config boot and the local durable restart test passed.
- Standalone Axe 4.10.3 found 0 violations on `/`, `/demo?demo=1`, `/app`,
  `/privacy`, and `/terms`, both locally and live. Live desktop and 390px dark/
  reduced-motion flows had no console or page errors, no horizontal overflow,
  and only same-origin product requests. The keyboard menu restored focus after
  Escape; a sample booking and reset returned the seeded two open seats.
- Local load smoke completed 100 requests: 10 accepted and 90 limited. Live
  allowance returned ten 200 responses then two 429 responses, both with
  `Retry-After: 5`.
- Live routes returned 200 for `/`, demo, app, callback, privacy, terms,
  robots, and sitemap; a missing route returned 404. HTML is no-cache, hashed
  assets are immutable, and the response CSP includes `frame-ancestors 'none'`.
- Live OIDC discovery returned the Sociobot tenant issuer and JWKS URL. The
  signed-out app and callback route are public; no test account was used.
- Mobile Lighthouse 12.8.2 scored 100 performance, 100 accessibility, 100 best
  practices, and 100 SEO. LCP was 1,230 ms, CLS 0, and TBT 0 ms.

Evidence is in `.factory/qa-artifacts/repair-13-live/`. This server-backed
product makes no offline/PWA claim; update behavior was exercised by the
traffic-ready revision handoff and persistence drill. Package/consumer checks
do not apply. No analytics, trackers, external fonts, or third-party scripts
were observed. No operator action remains.

---

# Repair 12 handoff — historical PASS (2026-08-29)

Work order: `class-capacity-truth-repair-12`
Base verifier report: `.factory/verification-12.md` at
`c20575f89fd33d0e201343ade3f70cae0e96dff6`
Repaired deployed source: `55e87acf5032289b32912d19af1384b2d0968cf3`
Live URL: <https://class-capacity-truth.sociobot.in>

## Release-blocking repair

Verification 12's only P0 was reproduced from the Azure control plane before
repair: revision `sf-class-capacity-truth--0000043` served the requested
`28fcd19f33b5` image, but had only `PORT=8080`, no volumes or mounts, and
`maxReplicas=3`. `scripts/verify-container-topology.sh` rejected that exact
live state. It was unsafe because revision-local SQLite and encryption keys can
lose or diverge capacity and make encrypted contact data unreadable.

`scripts/test-container-topology-deployment.sh` now uses that exact
verification-12 shape (`0000043`, `28fcd19f33b5`, one `PORT` variable, no
volume/mount, max three replicas). It proves all of the following:

- the production readback guard rejects the unsafe live shape;
- a durable template is still rejected when its traffic-serving process reports
  a different full build SHA; and
- the checked-in deployment command registers `cct-data`, reads back the Azure
  Files mount and persisted paths, fixes the scale at one replica, and accepts
  only the requested full build identity.

The declared `durable-one-replica-topology` claim was updated to describe this
new exact fixture and identity check. Product behavior, the research brief,
the web-with-backend artifact class, and all previously passing flows are
unchanged.

## Deployment and durability evidence

ACR build `ch18k` succeeded from this repository with the immutable image
`sociobotregistry.azurecr.io/sf-class-capacity-truth:55e87acf5032` and all
three source identity build arguments set to the deployed source SHA.

The first Container Apps patch was requested before that ACR run had published
its tag. It stayed in `ImagePullBackOff` and the existing healthy revision
continued to serve the old build; the guarded deployment did not claim success.
After ACR published the immutable tag, a no-configuration-change retry revision
pulled that same image and became healthy. This ordering is recorded so a
future operator waits for the specific ACR run, not merely another registry
run, before running the deploy command.

The active and ready revision is
`sf-class-capacity-truth--d-c-1788042115-23360`, with 100% traffic. Azure
readback and `scripts/verify-container-topology.sh` confirm:

- image `sociobotregistry.azurecr.io/sf-class-capacity-truth:55e87acf5032`;
- `minReplicas=1`, `maxReplicas=1`;
- Azure Files volume `cct-data` mounted at `/mnt/cct`;
- `DATA_DIR=/mnt/cct/keys` and
  `DURABLE_BACKUP_PATH=/mnt/cct/snapshots/class-capacity-truth.db`; and
- live `/health` returns `{"status":"ok","build":"55e87acf5032289b32912d19af1384b2d0968cf3","database":"ready"}`.

`scripts/prove-production-durability.sh` completed a controlled live revision
drill. It created a synthetic school, booked a seat with encrypted contact,
rolled a new revision, read the same confirmed count and decrypted contact,
deleted the synthetic workspace, and removed its one-time token and secret.
The final `d-c-1788042115-23360` revision is healthy, and Azure readback has no
`TEST_AUTH_TOKEN` environment entry or `cct-persist-drill` secret.

## Verification completed

- `npm ci` — passed; 0 vulnerabilities.
- `npm test` — passed: 7 Vitest tests, 5 Rust unit tests, 18 Rust API/
  integration tests, and both deployment regressions.
- `npm run typecheck`, `npm run lint`, and `npm run build` — passed. The build
  produced `dist/` and the release binary. Initial JS chunks are 70.86 kB and
  79.59 kB gzip; CSS is 4.43 kB gzip.
- `npm run test:e2e` — passed: 25 Chromium tests covering all declared browser
  claims, desktop, 390px/reduced-motion, keyboard, route focus, 200% text,
  privacy requests, and AxeBuilder checks.
- `npm run test:cold-claim` — passed from a fresh Cargo target within the
  600-second claim startup limit. `bash scripts/test-zero-config.sh` and
  `npm run test:durable-restart` also passed.
- Local release smoke passed: URL verification found title/lang/main/alt and
  no console/page errors; the 100-request forwarded-IP smoke saw both 200 and
  429 responses with `Retry-After`.
- Live URL verification passed at desktop and 390px. A fresh 390px/reduced
  motion browser flow opened the keyboard menu with Enter, restored focus with
  Escape, booked a fictional demo seat, reset it to two open seats, had no
  horizontal overflow or page errors, and made same-origin requests only.
- Live Playwright AxeBuilder scans returned zero serious/critical findings on
  `/`, `/demo?demo=1`, `/app`, `/privacy`, `/terms`, and the real 404 route.
  The standalone `@axe-core/cli` was also attempted, but its Selenium runner
  cannot locate a Chrome binary in this worker; Playwright uses the installed
  pinned browser and is the authoritative successful Axe run.
- Live headers include `X-Content-Type-Options: nosniff`, strict referrer
  policy, restrictive permissions policy, HTML no-cache, and response-header
  CSP with `frame-ancestors 'none'`. Public routes `/`, `/demo?demo=1`,
  `/app`, `/privacy`, `/terms`, `/auth/callback`, `/robots.txt`, and
  `/sitemap.xml` return 200; a missing route returns 404.
- Live forwarded-IP rate limiting returned ten 200 responses then two 429
  responses for a fresh address. Both 429s included `Retry-After: 5`,
  `X-RateLimit-Limit: 10`, and `X-RateLimit-Remaining: 0`.
- Mobile Lighthouse against production scored 100 performance, 100
  accessibility, 100 best practices, and 100 SEO; LCP was 1336.095 ms, CLS 0,
  and TBT 10 ms.

## Scope, privacy, and known gaps

No analytics, third-party fonts, scripts, trackers, or third-party product
requests were observed in the live public/demo flow. The product remains a
server-backed capacity ledger and makes no offline/PWA claim; its update path
was exercised by the healthy revision handoff and the live durable-restart
drill. It is not a distributable package, so package/consumer verification is
not applicable. No operator action remains.
## Verification 19 — PASS (2026-09-01)

Independent QA accepted candidate
`b5ade8e07d3ba4f8adbe1b77fa51a40f34205938` at
<https://class-capacity-truth.sociobot.in>. Live `/health` returned that exact
commit and `database: "ready"`, resolving verification 18's deployment-only
identity failure.

All 23 exact `.factory/claims.json` commands passed from a clean checkout,
along with `npm test`, `npm run typecheck`, `npm run lint`, and `npm run
build`. Live browser QA found same-origin-only public/demo requests, no
console/page errors, zero Axe serious/critical findings, working keyboard and
390 px reduced-motion behavior, valid security/cache headers, and the required
Sociobot CIAM PKCE configuration. Live rate limiting admitted 10 requests per
forwarded client and then returned `429` with `Retry-After: 4`; the 100-request
smoke observed 10 accepted and 90 rate-limited. The owned deployment topology
also passed the one-replica/Azure Files `/data` verifier.

The full evidence and exact commands/results are in
`.factory/verification-19.md` and `.factory/verification-evidence-19/`.
No product code or cloud configuration changed during verification. Known
release-blocking gaps: none.
