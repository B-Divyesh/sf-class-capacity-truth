# Repair 5 handoff — PASS

## Released repair

Commit `190ad1ef2105385352c381951938ec47ab1bac50` was built in ACR run
`chvm` and deployed as Container App revision
`sf-class-capacity-truth--0000024`. Its live `/health` response reports the
same full build SHA and `database: ready`.

The prior verification-5 configuration was reproduced first: revision 0000022
had `maxReplicas: 3`, no volumes/mounts, and only `PORT`. The repair registers
Azure Files environment storage `cct-data` backed by the dedicated
`class-capacity-truth` file share, fixes `minReplicas` and `maxReplicas` to 1,
mounts it at `/mnt/cct`, persists generated cookie/contact keys at
`/mnt/cct/keys`, and checkpoints/restores SQLite at
`/mnt/cct/snapshots/class-capacity-truth.db`.

Azure Files rejects POSIX `chmod`; the first mounted revision exposed that as
`Operation not permitted`. Key creation now accepts only that expected error,
with a unit regression, while local files retain mode 0600.

## Evidence

- Azure readback: revision 0000024 has image
  `sociobotregistry.azurecr.io/sf-class-capacity-truth:190ad1ef2105`, one
  running replica, `minReplicas: 1`, `maxReplicas: 1`, one AzureFile
  `cct-data` volume/mount, and both persistent path variables.
- Actual restart proof: booked a fresh demo seat from two open to one, ran
  `az containerapp revision restart` on 0000024, and fetched the same signed
  cookie afterward. The class still had one open seat.
- Live limiter: one forwarded IP received ten 200 demo-session responses then
  429; the 429 carried `Retry-After: 0`, `X-RateLimit-Limit: 10`, and
  `X-RateLimit-Remaining: 0`.
- Live auth boundary: unauthenticated `POST /api/workspaces` returned 401 with
  `WWW-Authenticate: Bearer`.
- Live 404 at 390px/200% text: HTTP 404, zero horizontal overflow, recovery
  link height 86px. The exact regression is in Playwright.
- ACR image build passed; Docker is unavailable locally, so ACR is the
  container-build evidence.

## Verification run

From a clean `npm ci` install:

```bash
npm test
npm run typecheck
npm run lint
npm run build
bash scripts/test-zero-config.sh
env -u CI npm run test:e2e
```

These passed locally. Every exact command in `.factory/claims.json` was also
run separately and passed: 21 claims including the new persisted topology,
runtime, forwarded-IP, SMTP queue, cross-device recovery, fair 24-hour offer,
and encrypted five-minute calendar-poll proofs. The suite contains 6 Vitest,
5 Rust unit, 18 Rust API/integration, and 24 Playwright tests.

`scripts/verify-container-topology.sh` performs the live Azure readback;
`scripts/deploy-container.sh` is the repeatable deployment operation. It
creates/updates the dedicated Azure Files share and environment storage before
patching only the app template.

## Remaining operator action

None for this repair. The production CIAM callback and Sociobot checkout had
already been live-verified in verification 5 and were preserved unchanged.
