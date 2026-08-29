# Repair 11 handoff — PASS (2026-08-29)

Work order: `class-capacity-truth-repair-11`
Base verifier report: `.factory/verification-11.md` at `f6b336e10a2c317bd4133fbd7f0348139e7403b9`
Repaired production source: `d27dc668792cc1bd82e7c3ddaf1880decdf6b1f6`
Live URL: <https://class-capacity-truth.sociobot.in>

## Release-blocking repair

Verification 11 accurately found that the live candidate
`89d35c47ee376d75d92c42b7c839f6da323e35b3` was serving revision
`sf-class-capacity-truth--0000042` with `minReplicas=1`, `maxReplicas=3`, no
Azure Files volume/mount, and only `PORT=8080`. That state was reproduced from
the Azure control plane before repair.

`scripts/deploy-container.sh` now treats a template PATCH as incomplete until
all of the following are true:

- the new revision is healthy and receives ingress traffic;
- `/health` returns `status: ok`, `database: ready`, and the expected full
  `BUILD_SHA` (or the immutable image-tag prefix when an explicit SHA is not
  provided);
- Azure readback still reports exactly one replica, Azure Files storage
  `cct-data` mounted at `/mnt/cct`, `DATA_DIR=/mnt/cct/keys`, and
  `DURABLE_BACKUP_PATH=/mnt/cct/snapshots/class-capacity-truth.db`.

`scripts/test-container-topology-deployment.sh` is exact regression coverage
for the verifier's unsafe `0000042` / `89d35c47ee37` / one-`PORT` /
no-volume / `maxReplicas=3` shape. It first proves the production verifier
rejects that state, then proves deployment creates the durable topology,
waits for a traffic-serving revision, and checks runtime identity.

## Production evidence

ACR build `ch178` completed successfully from this repository using:

```bash
az acr build --registry sociobotregistry \
  --image sf-class-capacity-truth:d27dc668792c \
  --build-arg BUILD_SHA=d27dc668792cc1bd82e7c3ddaf1880decdf6b1f6 \
  --build-arg GIT_SHA=d27dc668792cc1bd82e7c3ddaf1880decdf6b1f6 \
  --build-arg SOURCE_COMMIT=d27dc668792cc1bd82e7c3ddaf1880decdf6b1f6 .
```

The guarded deployment ran with that immutable image and full expected SHA.
After the controlled persistence drill, the active/ready revision was
`sf-class-capacity-truth--d-c-1788035236-10043`, serving
`sociobotregistry.azurecr.io/sf-class-capacity-truth:d27dc668792c`.
Azure readback showed `minReplicas=1`, `maxReplicas=1`, volume
`cct-data` (`AzureFile`), its `/mnt/cct` mount, and only the expected
`PORT`, `DATA_DIR`, and `DURABLE_BACKUP_PATH` variables. Live `/health` was:

```json
{"status":"ok","build":"d27dc668792cc1bd82e7c3ddaf1880decdf6b1f6","database":"ready"}
```

`scripts/prove-production-durability.sh` performed the controlled live
revision drill: it created a synthetic school and encrypted guardian booking,
rolled a new revision, read the same confirmed seat and decrypted contact,
deleted the synthetic workspace, detached the one-time test token, and removed
the `cct-persist-drill` secret. Final Azure readback confirms no temporary
token reference or secret remains.

## Verification completed

Clean install and local checks:

- `npm ci` — passed; 0 reported vulnerabilities.
- `npm test` — passed (7 Vitest tests, 23 Rust unit/integration tests, and
  both deployment regressions).
- `npm run typecheck`, `npm run lint`, and `npm run build` — passed. The
  release build produced `dist/` and the release API binary. Initial app JS is
  70.86 kB gzip; CSS is 4.43 kB gzip.
- `npm run test:e2e` — passed, 25 Chromium tests; `test-results/.last-run.json`
  records `{"status":"passed","failedTests":[]}`.
- `npm run test:durable-restart` — passed: a real-school booking and encrypted
  contact survived a fresh release-process restart on a separate mounted
  storage directory.
- `bash scripts/test-zero-config.sh` — passed: zero-config boot served health
  and logged generated/persisted keys.
- `bash scripts/load-smoke.sh http://127.0.0.1:18088` — passed: 100 requests,
  10 accepted and 90 rate-limited with `Retry-After`.
- `/opt/fleet/lib/verify-url.sh` passed locally and live: title, `lang=en`, one
  h1, main landmark, image alt coverage, labelled buttons, and no browser
  console/page errors. `@axe-core/cli` 4.10.3 found 0 violations locally and
  live. The existing Playwright axe suite covers landing, demo, booking, app,
  privacy, terms, and 404 routes, including dark/mobile states.

Live post-deploy checks:

- `/`, `/demo?demo=1`, `/app`, `/privacy`, `/terms`, `/auth/callback`,
  `/robots.txt`, and `/sitemap.xml` returned 200; a fresh unknown URL returned
  a real 404.
- A real live demo booking changed availability and **Reset demo** restored the
  seeded two open seats.
- A 390px Playwright smoke test opened the labelled menu with Enter, restored
  focus with Escape, found no horizontal overflow in the demo, made no console
  errors, and observed only same-origin requests through landing, demo,
  privacy, and app routes.
- Twelve requests to `GET /api/demo/session` with one fresh forwarded client
  IP yielded 10 HTTP 200 responses and 2 HTTP 429 responses. Both 429s carried
  `Retry-After: 5`.
- Response headers include `X-Content-Type-Options: nosniff`, strict referrer
  policy, restrictive permissions policy, and a response-header CSP with
  `frame-ancestors 'none'`.
- Mobile Lighthouse against live production: performance 100, accessibility
  100, best practices 100, SEO 100; LCP 1.2 s, CLS 0, TBT 0 ms.

The worker has no local Docker daemon. The Docker image was therefore verified
by the successful remote ACR build above, not by a local `docker build`.

## Scope, privacy, and known gaps

The original web-with-backend artifact, research brief, data model, demo,
authentication boundary, and all previously passing product behavior are
unchanged. No analytics or advertising request was observed; no third-party
font or script is loaded. This is a live backend product, not a PWA, and it
makes no offline claim; its update path is the traffic-ready revision check
added in this repair.

No operator action is required. The next verifier can run its normal live
identity and topology readback against the deployed `d27dc66` image.
