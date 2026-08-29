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
