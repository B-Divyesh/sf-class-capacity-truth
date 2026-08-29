# Repair 9 handoff — PASS (2026-08-29)

Work order: `class-capacity-truth-repair-9`

Verifier report: `.factory/verification-9.md` at report commit
`b632f995e0c7087b8b6e542c216bea6b3cfb0af7`, covering candidate
`93500402cf97c5874bb37883ed92f72ea5f59396`.

Repair implementation and first immutable deployment:
`38019c2e70baae6e2b0076ae5a25a990556ff47e`.

Live URL: <https://class-capacity-truth.sociobot.in>

## Release blocker repaired

The verifier found one P0: production revision `0000040` used SQLite with
`minReplicas=1`, `maxReplicas=3`, only `PORT=8080`, and no Azure Files mount.
The registered `cct-data` storage still existed. Production deployment drift,
not application logic or lost storage, was the root cause.

- Reproduced the exact unsafe Azure readback before repair: image
  `93500402cf97`, revision `0000040`, scale `1/3`, one `PORT` variable, null
  mounts, and null volumes.
- Updated the deployment claim and regression fixture to that exact
  verification-9 candidate/revision. The test proves the production readback
  guard rejects it, then proves `scripts/deploy-container.sh` registers and
  applies the required durable template.
- Built image `sociobotregistry.azurecr.io/sf-class-capacity-truth:38019c2e70ba`
  in ACR. Digest:
  `sha256:31ed470b2010f93affbd290d6240832ecfabf1675576383ba81bebff204cb708`.
- Deployed only through the checked-in durable deployment path. DNS, billing,
  and unrelated infrastructure were not changed.

## Durable production evidence

After deployment and the production restart drill, Azure readback returned:

```text
latest/ready revision: sf-class-capacity-truth--d-c-1788020003-24635
active revisions: 1, Healthy, 100% traffic
minReplicas/maxReplicas: 1/1
replicas: 1 Ready/Running, restart count 0
volume: cct-data (AzureFile) -> /mnt/cct
DATA_DIR=/mnt/cct/keys
DURABLE_BACKUP_PATH=/mnt/cct/snapshots/class-capacity-truth.db
TEST_AUTH_TOKEN: absent
temporary drill secret: absent
```

Startup logs report `durable_backup="supplied"`,
`cookie_signing_key="persisted-generated"`, and
`contact_encryption_key="persisted-generated"`.

`scripts/prove-production-durability.sh` passed with these revisions:

```text
auth: sf-class-capacity-truth--d-a-1788020003-24635
restart: sf-class-capacity-truth--d-r-1788020003-24635
cleanup: sf-class-capacity-truth--d-c-1788020003-24635
```

The drill created a synthetic school, published a capacity-two class, booked
one seat, forced a new revision, and recovered `confirmed=1` and `openSeats=1`.
It also decrypted the exact synthetic guardian name and email. The drill then
deleted the workspace, proved its public class returned 404, detached the test
token, and removed the temporary secret.

The verifier's public split-brain scenario was repeated after repair. Twenty
concurrent session reads used one fixed valid demo cookie and one forwarded
client IP. Ten returned HTTP 200 for the same original workspace and ten
returned HTTP 429. There were zero 401 responses, zero replacement cookies,
one workspace observed, and `Retry-After` on every 429. A separate 100-request
live smoke admitted 10 and rate-limited 90.

## Complete local verification

- `npm ci`: 170 packages installed; zero vulnerabilities.
- Every command in all 21 `.factory/claims.json` entries passed individually.
- `npm test`: 7 frontend tests, 5 Rust unit tests, 18 API/integration tests,
  and both deployment regression scripts passed.
- `npm run typecheck` and `npm run lint`: passed, including rustfmt and Clippy
  with warnings denied.
- `npm run build`: passed and produced `dist/` plus the optimized Rust binary.
  Initial JavaScript is 70.86 KB gzip, CSS is 4.43 KB gzip, and the staff-only
  lazy MSAL chunk is 79.59 KB gzip.
- ACR container build `ch148` passed using the multi-stage, non-root Dockerfile.
- `npm run test:e2e`: 25/25 Chromium tests passed. Coverage includes desktop,
  390 px, 200% text, keyboard-only booking, route focus, dark mode, reduced
  motion, errors, privacy requests, and axe.
- `npm run test:durable-restart` passed. `bash scripts/test-zero-config.sh`
  passed with the minimal runtime environment and generated persisted keys.
- `npm run test:cold-claim` passed from an empty Cargo target in 102 seconds,
  below its 600-second limit.
- Local load smoke: 100 requests completed; 10 accepted and 90 returned 429
  with `Retry-After`.
- Local `/opt/fleet/lib/verify-url.sh`: HTTP 200, descriptive title, `lang=en`,
  one H1, main landmark, complete alt/button names, and no console errors.
- Local Lighthouse mobile: 100 Performance, 100 Accessibility, 100 Best
  Practices, and 100 SEO; FCP 1.2 s, LCP 1.4 s, TBT 0 ms, CLS 0.

The plain-words copy and `.factory/copy-audit.md` were unchanged because the
repair changes no product copy. The existing audit has no over-22-word or
banned-word flags.

## Complete live verification

- `/health` returned the deployed full SHA and `database="ready"`.
- Root HTML and the primary hashed JavaScript were byte-for-byte identical to
  the local production build. Root HTML is `no-cache`; hashed assets are
  `public, max-age=31536000, immutable`.
- Live `/opt/fleet/lib/verify-url.sh` passed with no console errors. Live
  Lighthouse mobile scored 100/100/100/100; FCP 1.2 s, LCP 1.2 s, TBT 0 ms,
  CLS 0.
- Fresh live axe scans found zero serious/critical findings on home, demo,
  privacy, terms, signed-out workspace, 404, and the open mobile menu.
- Keyboard-only live QA put focus on the skip link, restored route-heading
  focus, completed a sample booking, and reset the demo.
- At 390×844 in dark/reduced-motion mode, the menu measured 77.56×44.80 px,
  opened with Enter, closed with Escape, restored trigger focus, and had no
  overflow. The demo retained all three classes at 200% text without overflow.
- Ordinary home, demo, privacy, terms, and signed-out workspace flows made
  only same-origin requests. No analytics, advertising, CDN font, or external
  script loaded before an explicit identity or checkout action.
- CSP is a response header with `frame-ancestors 'none'`; `nosniff`, strict
  referrer policy, and restrictive permissions policy are present. Unknown
  routes return the styled HTTP 404. An unapproved CORS origin received no
  allow header; `https://hello.sociobot.in` was allowed.
- Explicit sign-in reached only the configured Sociobot CIAM authority with
  tenant `35c6fe40-0ec0-46b6-98c6-213ad4de6650`, client
  `25c704f4-465a-47af-80ab-2c489466b697`, the production callback,
  authorization-code flow, PKCE S256, state, nonce, and required scopes. No
  credentials were entered.
- Explicit checkout made POST
  `https://api.sociobot.in/api/v1/products/class-capacity-truth/checkout`,
  received HTTP 200, and opened an HTTPS `checkout.dodopayments.com` session.
  No payment was attempted.

Evidence is in `.factory/qa-artifacts/repair-9-local/` and
`.factory/qa-artifacts/repair-9-live/`.

## Run and deploy

```bash
npm ci
npm test
npm run typecheck
npm run lint
npm run build
npm run test:e2e
npm run test:durable-restart
bash scripts/test-zero-config.sh
npm run test:cold-claim

az acr build --registry sociobotregistry \
  --image sf-class-capacity-truth:<immutable-commit-tag> \
  --file Dockerfile \
  --build-arg BUILD_SHA=<full-commit-sha> \
  --build-arg GIT_SHA=<full-commit-sha> \
  --build-arg SOURCE_COMMIT=<full-commit-sha> .
IMAGE=sociobotregistry.azurecr.io/sf-class-capacity-truth:<immutable-commit-tag> \
  bash scripts/deploy-container.sh
RESOURCE_GROUP=sociobot bash scripts/prove-production-durability.sh
```

## Applicability and known gaps

This remains a `web-with-backend` container product. It is not a library, CLI,
or PWA, so package-consumer and service-worker offline/update checks do not
apply. The browser shows a retry path for network failure. AI is not useful to
the deterministic capacity ledger and remains absent.

No release-blocking gap remains. Production has no approved SMTP relay, so
staff receive the tested durable copyable offer instead of sent email. M5
remains planned and is outside this repair.
