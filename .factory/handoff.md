# Repair handoff — durable single-replica deployment

Repair work for verifier candidate `00edf2a9a366bb0eda3e5eebce4e88e3377f2fa3`.

## Fixed release blocker

Verification 6 correctly found that live revision `0000025` had only `PORT`,
no Azure Files volume, and `maxReplicas: 3`. The checked-in deployment command
now registers the `cct-data` Azure Files environment storage, applies the
`/mnt/cct` mount, supplies `DATA_DIR=/mnt/cct/keys` and
`DURABLE_BACKUP_PATH=/mnt/cct/snapshots/class-capacity-truth.db`, and fixes
both min/max replicas at one. It reads the effective Azure template back after
the PATCH and fails unless every required value and the requested image match.
It also removes a temporary `TEST_AUTH_TOKEN` if a controlled persistence drill
was run; Azure template PATCH merges env entries, so relying on the PATCH array
alone would be unsafe.

The regression fixture begins with the verifier's exact broken shape (only
`PORT`, no volume/mount, max 3), executes the deployment command against a
mock Azure control plane, and proves the repaired readback. The durable restart
claim now starts the release binary, creates a real school/class/public booking,
restarts it from a separate mounted checkpoint, and proves the seat count and
encrypted guardian contact can still be read.

## Verification evidence

- Clean install: `npm ci` — 170 packages, 0 vulnerabilities.
- Unit/integration/regression: `npm test` — 6 Vitest, 5 Rust unit, 18 Rust API
  tests, and the Azure topology fixture all passed.
- Types/lint/build: `npm run typecheck`, `npm run lint`, and `npm run build`
  passed. The build produced `dist/` and the release API binary.
- Browser: `env -u CI npm run test:e2e` passed 24 Chromium tests, including
  desktop, 390px mobile, keyboard, axe, privacy request recording, dark/reduced
  motion, 200% text, and route/404 checks. `npm run test:cold-claim` passed
  from its separate clean target.
- Runtime/claims: `bash scripts/test-zero-config.sh`,
  `npm run test:durable-restart`, and `npm run test:deployment` passed.
  The restart test asserts `/health` build identity, real-school capacity after
  restart, persisted cookie/contact keys, and decrypted guardian data.
- ACR build: run `chwr` succeeded from a 182.9 KB archive excluding `.git`.
- Azure recovery: the first durable revision was `0000026`; its `/health`
  response was `{"status":"ok","build":"5edc7b9406ca6ac18459c92317488a142b5852a3","database":"ready"}`.
  Fresh control-plane read showed one replica, the `cct-data` Azure Files
  volume/mount, and both durable paths. Startup logs reported
  `durable_backup:"supplied"`, `cookie_signing_key:"persisted-generated"`,
  and `contact_encryption_key:"persisted-generated"`.
- Live revision-restart proof: a synthetic real school created a two-seat
  public class and a real booking. After a new revision with the temporary
  test credential removed, revision `0000030` returned the same public record
  with `confirmed: 1` and `openSeats: 1`; `/health` still returned the repair
  SHA and `database:"ready"`. The synthetic workspace was then deleted, and
  final cleanup revision `0000034` returned 404 for its public class while
  retaining the exact one-replica durable topology and no `TEST_AUTH_TOKEN`.

## Deploy

```bash
sha="$(git rev-parse HEAD)"
az acr build --registry sociobotregistry \
  --image "sf-class-capacity-truth:$sha" \
  --build-arg "BUILD_SHA=$sha" --build-arg "GIT_SHA=$sha" \
  --build-arg "SOURCE_COMMIT=$sha" .
IMAGE="sociobotregistry.azurecr.io/sf-class-capacity-truth:$sha" \
  bash scripts/deploy-container.sh
bash scripts/verify-container-topology.sh
curl -fsS https://class-capacity-truth.sociobot.in/health
```

The factory deployment remains a Container App; no DNS, billing, data model,
demo, identity, or product behavior was changed. There are no known release
blockers.
