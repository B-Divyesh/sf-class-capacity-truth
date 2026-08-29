# Repair handoff — PASS

Repaired verifier candidate `023bc90148efd22542aa1fb99c81588686e7aac4` from `.factory/verification-7.md`.

Live URL: <https://class-capacity-truth.sociobot.in>

Verified: 2026-08-29 UTC

## What changed

- Reproduced P0 exactly: revision `0000036` had only `PORT`, no Azure Files mount, and `maxReplicas: 3`. The fixture now rejects that unsafe template first.
- The deploy command creates a unique revision suffix, retries only Azure's transient in-progress conflict, then reads the effective template back. Copying the active suffix had let ARM accept an update without creating the requested revision.
- A separately tested traffic-readiness guard waits for `latestReadyRevisionName == latestRevisionName`; this fixes the drill race where a temporary credential could reach the old revision and receive 401.
- Aligned `.factory/plan.md`, `.factory/design.md`, and `services/api/README.md` with shipped SQLite/Azure Files topology and M1–M4 status.

## Effective production template

```text
revision: sf-class-capacity-truth--d-1788002511-20759
image: sociobotregistry.azurecr.io/sf-class-capacity-truth:6cd085986e3a01875b14026bcbad41d7abbbe013
minReplicas/maxReplicas: 1/1
volume: cct-data (AzureFile) -> /mnt/cct
DATA_DIR=/mnt/cct/keys
DURABLE_BACKUP_PATH=/mnt/cct/snapshots/class-capacity-truth.db
```

Readback reports `Provisioned`, `Healthy`, `RunningAtMaxScale`, one replica, and no `TEST_AUTH_TOKEN`. `/health` reports build `6cd085986e3a01875b14026bcbad41d7abbbe013`.

## Production persistence proof

`RESOURCE_GROUP=sociobot bash scripts/prove-production-durability.sh` passed on the final image. It created a synthetic workspace, class, and booking; forced a new revision; verified public `confirmed=1, openSeats=1` and the exact decrypted synthetic guardian contact; then deleted the workspace, confirmed its class returned 404, removed the temporary credential/secret, and reached this clean revision:

```text
auth=sf-class-capacity-truth--d-a-1788001363-14778
restart=sf-class-capacity-truth--d-r-1788001363-14778
cleanup=sf-class-capacity-truth--d-c-1788001363-14778
```

## Verification

- `npm ci`: 170 packages, 0 vulnerabilities.
- `npm test`: 6 frontend, 5 Rust unit, 18 API/integration, topology and traffic-readiness regressions passed.
- `npm run typecheck`, `npm run lint`, `npm run build`: passed. CSS 4.35 kB gzip; initial JS chunks 70.63 and 79.59 kB gzip.
- `npm run test:e2e`: 24/24 Chromium passed, including claims, desktop, 390px, 200% text, keyboard, dark/reduced motion, privacy, and axe.
- `npm run test:durable-restart`, `bash scripts/test-zero-config.sh`, and `npm run test:cold-claim` passed; cold claim was 111 seconds (limit 600).
- Final live browser smoke: semantic desktop page, skip-link and route-focus keyboard behavior, same-origin requests, no console/page errors, zero axe serious/critical issues, and no 390px overflow.
- Live response policy has CSP `frame-ancestors 'none'`, no-cache HTML, nosniff, strict referrer policy, and restrictive permissions policy. Live CIAM sign-in uses the required host, client ID, callback, code flow, and PKCE S256.

This is a web service, not a package or PWA: package-consumer and offline-update checks do not apply, and no product claim promises offline operation.

## Run/deploy

```bash
npm ci && npm test
npm run typecheck && npm run lint && npm run build
npm run test:e2e
IMAGE=sociobotregistry.azurecr.io/sf-class-capacity-truth:<immutable-tag> bash scripts/deploy-container.sh
RESOURCE_GROUP=sociobot bash scripts/prove-production-durability.sh
```

No known release-blocking gaps remain. The drill removes its synthetic data and does not alter DNS, billing, or real-school data.
