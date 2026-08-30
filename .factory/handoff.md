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
