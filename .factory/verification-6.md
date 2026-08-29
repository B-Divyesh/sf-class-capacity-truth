# Independent verification 6 — FAIL

Verified 2026-08-29 UTC against candidate commit
`00edf2a9a366bb0eda3e5eebce4e88e3377f2fa3` and
<https://class-capacity-truth.sociobot.in>.

## Verdict

**FAIL — do not accept real school data.** The candidate source, demo, claims,
accessibility, security headers, and performance checks are healthy. The
deployed revision is not the durable one-replica SQLite topology that this
product requires. This is a deployment-only regression, established from a
fresh Azure control-plane read and the live container startup log.

## Release-blocking defect

### P0 — live school data and encryption keys are ephemeral; deployment can scale to three replicas

The active revision is `sf-class-capacity-truth--0000025`. Azure reported its
image as `sociobotregistry.azurecr.io/sf-class-capacity-truth:00edf2a9a366`,
so the live build identity agrees with `/health`, which returned the full
candidate SHA and `database: ready`.

The same fresh Azure read reported only `PORT=8080`, `minReplicas: 1`,
**`maxReplicas: 3`**, `volumeMounts: null`, and `volumes: null`. This directly
contradicts the checked-in `infra/containerapp-topology.yaml`, which requires
exactly one replica, the `cct-data` Azure Files mount, `DATA_DIR=/mnt/cct/keys`,
and `DURABLE_BACKUP_PATH=/mnt/cct/snapshots/class-capacity-truth.db`.

The only current replica's startup log confirms the consequence:

```text
database_config=generated-default
durable_backup=disabled
cookie_signing_key=generated-and-persisted
contact_encryption_key=generated-and-persisted
```

Those generated files and SQLite database are on disposable container storage.
A replacement loses real workspaces, bookings, waitlist offers, receipts, and
the keys needed to decrypt contacts. Scaling to the configured second or third
replica also creates independent databases, keys, and per-replica limiters.
The current replica count was one at inspection time; that does not mitigate
the restart or scale-out failure. I did not restart production merely to
demonstrate an already proven destructive configuration.

Required repair: deploy the `cct-data` Azure Files environment storage and all
checked-in mount/path values, fix `maxReplicas` at one while SQLite is local,
then prove a booked real-school record survives an actual revision restart.

## Mandatory first-read and demo gate

**PASS.** A cold live visit says “Show the right number of class seats,” names
small language schools whose calendar and room list disagree, and places
**Try it with sample data** on the first screen with “Three sample classes open
next.” The one-click link opens `/demo?demo=1` with realistic available, full,
and cutoff sample classes plus the persistent reset/start-for-real banner.

## Claims gate

`.factory/claims.json` exists and has 21 entries. After clean `npm ci`, every
listed command completed successfully (fail-fast sequence):

| Claims | Result |
| --- | --- |
| sample booking, full class, cutoff, demo reset | PASS |
| school capacity flow, calendar five-minute poll, released-seat delivery | PASS |
| $99 plan, no tracking, contact encryption/retention, staff roles | PASS |
| export/delete, demo expiry/input disposal, non-mutating reconciliation | PASS |
| durable restart, configured SMTP, cross-device recovery, oldest 24-hour offer | PASS |
| zero-config runtime, forwarded-IP rate limit, durable one-replica topology contract | PASS locally/source contract |

The final topology claim proves the checked-in contract, not the active Azure
revision. It therefore does not clear P0 above.

## Clean-checkout quality evidence

| Check | Result |
| --- | --- |
| `npm ci` | PASS — 170 packages, zero reported vulnerabilities. |
| `npm test` | PASS — 6 Vitest, 5 Rust unit, 18 Rust API/integration tests. |
| `npm run typecheck` | PASS. |
| `npm run lint` | PASS — TypeScript, rustfmt, Clippy with `-D warnings`. |
| `npm run build` | PASS — Vite `dist/` and release API binary. |
| `env -u CI npm run test:e2e` | PASS — 24/24 Chromium tests. |
| `npm run test:cold-claim` | PASS — clean-target sample claim completed within its 600-second contract. |
| Docker production image | NOT RUN — Docker/Podman/Buildah/Nerdctl are unavailable in this verifier. |

The browser/API suite covers capacity boundaries, duplicate/concurrent booking
protection, cutoff/full recovery, form validation, calendar discrepancy without
seat mutation, a published parent page, waitlist conversion, offer copy/reload/
acceptance, export/delete, and responsive keyboard flows.

## Fresh production evidence

- `/health`: HTTP 200, full candidate SHA, `database: ready`.
- Deployed primary JS SHA-256 exactly matches local `dist`; its cache policy is
  `public, max-age=31536000, immutable`. HTML/API responses are no-cache.
- A fresh live demo booking changed the open sample from two seats to one,
  reset to two, and full/cutoff sample routes blocked booking.
- Demo requests stayed same-origin and produced no console/page errors.
  The demo cookie is `HttpOnly`, `Secure`, `SameSite=Strict`, `Max-Age=86400`.
- CSP is a response header with `frame-ancestors 'none'`; nosniff, strict
  referrer, and restrictive permissions policy are present.
- Live anonymous limiter: one forwarded IP received 10 `/api/demo/session`
  responses, then HTTP 429 with `Retry-After: 5` and limit 10. The provided
  100-request concurrency smoke yielded 10 accepted / 90 rate-limited.
  Protected school routes exposed allowance 40; request 41 returned 429 with
  `Retry-After` (a just-refilled request 42 returned its expected 401).
- The signed-out workspace redirects only to the required
  `sociobotcustomers.ciamlogin.com` tenant, with the correct client ID,
  production callback, authorization code flow, and PKCE S256.

## Accessibility and performance

- Fresh live axe scans at 390px, dark mode, and reduced motion found zero
  serious or critical violations on `/`, `/demo`, `/privacy`, `/terms`, `/app`,
  and the HTTP 404 route. Each has one H1, a main landmark, and zero horizontal
  overflow. Keyboard Tab reaches a 44px skip link with a 3px visible focus
  outline; the mobile 404 recovery link is also 44px high.
- Lighthouse mobile live landing: performance 100, accessibility 100, best
  practices 100, SEO 100; FCP 1.3 s, LCP 1.3 s, TBT 0 ms, CLS 0.
- Initial JS is 230,555 bytes raw / 70,531 bytes gzip; CSS is 18,879 bytes raw
  / 4,357 bytes gzip. No third-party font or script is loaded.

## Scope notes

This is a web service, not a library/CLI or PWA, so consumer-package and
service-worker/offline-update checks do not apply. Completion of an Entra sign
in or a paid hosted checkout was not attempted without an authorized account
or payment; the live redirect and local recorded integrations were verified.

## Next step

Repair the active Container App configuration, not product code alone. Re-run
the Azure topology read and real restart persistence proof before accepting the
candidate.
