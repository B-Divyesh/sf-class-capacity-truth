# Independent verification 9 — FAIL (2026-08-29)

Candidate `93500402cf97c5874bb37883ed92f72ea5f59396` was verified from a
clean checkout and at <https://class-capacity-truth.sociobot.in>.

**FAIL — do not accept real school data.** All 21 claims commands, the full
unit/API/browser suite, type/lint checks, production build, cold-start claim,
local durable restart, first-read/demo gate, accessibility, privacy, Entra,
checkout, and Lighthouse checks pass. The exact candidate is live, but active
revision `sf-class-capacity-truth--0000040` has `minReplicas=1`,
`maxReplicas=3`, only `PORT=8080`, no volume mount, and no volumes. Startup
logs report `database_config="generated-default"` and
`durable_backup="disabled"`.

Fresh QA traffic scaled the revision to two Ready replicas. A fixed valid demo
cookie then split across independently keyed databases: three concurrent
booking requests returned one 201 and two 401 `demo_cookie_missing`; 19
successful fixed-cookie session reads returned the original workspace nine
times and ten newly seeded workspaces. A fresh client also received 20 accepts
for an advertised 10-request allowance because each replica owned a separate
rate-limit bucket.

Required operator repair: redeploy the exact candidate through
`scripts/deploy-container.sh`; read back one replica, the `cct-data` mount at
`/mnt/cct`, `DATA_DIR=/mnt/cct/keys`, and
`DURABLE_BACKUP_PATH=/mnt/cct/snapshots/class-capacity-truth.db`; run
`scripts/prove-production-durability.sh`; then repeat independent live QA.

The complete commands, claim matrix, build/live identity hashes, request and
header evidence, rate allowances, functional cases, Lighthouse measurements,
and P0 defect are in `.factory/verification-9.md`.

---

# Previous repair 8 handoff — PASS (superseded)

Historical record only. The verification-9 result above supersedes this
earlier deployment state.

Repaired candidate `11a728e6b2f481506753caef919347958512c124` from
`.factory/verification-8.md` for work order `class-capacity-truth-repair-8`.

Live URL: <https://class-capacity-truth.sociobot.in>

Verified: 2026-08-29 UTC

## What changed

- Reproduced the P0 live state on revision `sf-class-capacity-truth--0000039`:
  image `11a728e6b2f4`, `minReplicas/maxReplicas: 1/3`, only `PORT=8080`, no
  mount, and no volume. The topology regression now starts with that exact
  verification-8 fixture, proves the readback guard rejects it, and proves
  `scripts/deploy-container.sh` replaces it with the durable contract.
- Built the repair in Azure Container Registry and deployed it through the
  checked-in durable deployment path. The effective template was read back
  before the production restart drill.
- Replaced the 390 px two-column link grid with a labelled disclosure menu.
  It is collapsed by default, uses native button semantics and
  `aria-expanded`/`aria-controls`, supports Enter and Space, closes with
  Escape, returns focus to its trigger, closes after navigation, and keeps a
  44 px target without horizontal overflow.
- Added exact unit and Playwright regression coverage for the menu, including
  an axe scan while it is open. Updated the claim fixture and copy audit.

## Durable production evidence

The first repair deployment used image
`sociobotregistry.azurecr.io/sf-class-capacity-truth:73da3ecbc936` (digest
`sha256:042bf231abfe35fd6e2e932e7bdf2fefe5146652c4b3b12f0f8347ece19f69ed`)
from source commit `73da3ecbc9362f14dcf43d7af7e2be1bf3e673d6`.

The post-drill effective template was:

```text
revision: sf-class-capacity-truth--d-c-1788009788-27750
latest ready revision: sf-class-capacity-truth--d-c-1788009788-27750
minReplicas/maxReplicas: 1/1
volume: cct-data (AzureFile) -> /mnt/cct
DATA_DIR=/mnt/cct/keys
DURABLE_BACKUP_PATH=/mnt/cct/snapshots/class-capacity-truth.db
replicas: one Ready/Running, restart count 0
TEST_AUTH_TOKEN: absent
```

`scripts/prove-production-durability.sh` passed with revisions:

```text
auth: sf-class-capacity-truth--d-a-1788009788-27750
restart: sf-class-capacity-truth--d-r-1788009788-27750
cleanup: sf-class-capacity-truth--d-c-1788009788-27750
```

The drill created a synthetic workspace, published a capacity-two class,
booked one seat, forced a new revision, and then observed `confirmed=1` and
`openSeats=1`. It also decrypted the exact synthetic guardian name and email.
It deleted the workspace, confirmed the public class returned 404, detached
the test token, and removed the temporary Container App secret.

## Complete verification

- `npm ci`: 170 packages installed; zero vulnerabilities.
- Every one of the 21 commands in `.factory/claims.json`: passed exactly as
  listed.
- `npm test`: 7 frontend, 5 Rust unit, 18 API/integration, and 2 deployment
  regression tests passed.
- `npm run typecheck` and `npm run lint`: passed, including rustfmt and Clippy
  with warnings denied.
- `npm run build`: passed and produced `dist/` plus the optimized Rust binary.
  Initial JavaScript is 70.86 KB gzip, CSS is 4.43 KB gzip, and the lazy MSAL
  chunk is 79.59 KB gzip.
- `npm run test:e2e`: 25/25 Chromium tests passed. Coverage includes all
  browser claims, desktop, 390 px, 200% text, keyboard, dark/reduced-motion,
  errors, privacy, route focus, and axe.
- `npm run test:durable-restart`: passed with a persisted real-school booking,
  snapshot, and generated keys. `scripts/test-zero-config.sh`: passed with only
  `PORT` plus the static path. `npm run test:cold-claim`: passed in 106 seconds
  against the 600-second limit.
- Local `/opt/fleet/lib/verify-url.sh`: HTTP 200, one H1, `lang=en`, main
  landmark, no missing alt text, no unlabelled buttons, and no console errors.
  Local Lighthouse mobile: 100 Performance, 100 Accessibility, 100 Best
  Practices, 100 SEO; FCP 1.2 s, LCP 1.4 s, TBT 0 ms, CLS 0.
- Live `/opt/fleet/lib/verify-url.sh`: the same semantic and console checks
  passed. Live Lighthouse mobile: 100/100/100/100; FCP 1.3 s, LCP 1.3 s,
  TBT 0 ms, CLS 0.
- Live 390 px reduced-motion check: menu opened and closed from the keyboard,
  restored trigger focus, had no overflow or browser errors, and produced zero
  serious/critical axe findings. A one-click sample booking changed two open
  seats to one with no third-party request or console error.
- Live response policy: HTML is `no-cache`; CSP is a response header with
  `frame-ancestors 'none'`; `nosniff`, strict referrer policy, and restrictive
  permissions policy are present. An unapproved origin received no CORS allow
  header; `https://hello.sociobot.in` was allowed.
- Live identity: the only sign-in action reached the required Sociobot CIAM
  tenant and client ID with the production callback, authorization-code flow,
  PKCE S256, and state. No account credentials were used.
- Live load smoke: 100 requests completed; 10 were accepted and 90 returned
  429 with `Retry-After`.

Evidence is in `.factory/qa-artifacts/repair-8-local/` and
`.factory/qa-artifacts/repair-8-live/`.

## Run and deploy

```bash
npm ci
npm test
npm run typecheck && npm run lint && npm run build
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

This is a backend web service, not a package or PWA. Package-consumer and
service-worker offline/update checks do not apply. The server presents a
clear retry path when network requests fail. AI is not part of the capacity
allocation job and remains absent.

## Known gaps and next steps

At the time of repair 8, no release-blocking gap remained. The production deployment has no approved
SMTP relay, so staff receive the tested durable copyable offer instead of a
sent email. M5 remains planned in `.factory/plan.md` and is outside this repair.
