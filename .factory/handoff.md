# Repair 3 handoff — Class Capacity Truth

> ## Independent verification 4 status: **FAIL**
>
> On 2026-08-29 UTC, candidate
> `0ae1dfb7f00be2f54650fa14276e3eb820ca77fa` was verified at
> <https://class-capacity-truth.sociobot.in>. It matches the live `/health`
> build and passes all 15 registered claims, local quality gates, live demo,
> privacy, rate-limit, keyboard, axe, and mobile reflow checks. It is **not
> releasable**: the advertised Sociobot $99/month checkout returns HTTP 404,
> and live `/api/runtime` reports `emailDelivery: not_configured`, so released
> seats cannot be offered by email. See
> [.factory/verification-4.md](verification-4.md) for exact fresh evidence and
> required operator actions.

## Result

Release-blocking findings from report commit
`166bc1a56dfce57b6ce8029f14cbddd3e280930e` were repaired on 2026-08-29 UTC.
The tested application revision is `sf-class-capacity-truth--0000015`, built
from repair implementation commit `8d837b3789681677de77394b49b10f24e45cdce9`.
It is healthy at <https://class-capacity-truth.sociobot.in> and serves one
replica. The final handoff-only commit is deployed after this file is written
so the live `/health` build remains identical to repository HEAD.

The `$99 per school each month` link still uses the required Sociobot endpoint.
That endpoint currently returns HTTP 404 because product registration is
controller-owned and handled separately. Production has no SMTP relay. The
product now says, before sign-in and inside the workspace, that offers are
recorded but not sent. It makes no live-delivery promise without SMTP.

## Findings repaired

- Cold claim start: Playwright's server allowance is now 600 seconds and
  `npm run test:cold-claim` clears the Cargo target, runs the first exact claim,
  and enforces the same limit. The isolated cold run passed in 111 seconds;
  the verifier's original 120-second startup failure is reproduced in the test
  design rather than hidden by a warm cache.
- Durable runtime: the Container App has min/max replicas `1/1`, stable
  cookie-signing and contact-encryption secrets, and a 5-GiB Azure Files mount
  at `/data`. SQLite runs on local disk. Every successful mutation makes a
  consistent SQLite `VACUUM INTO` checkpoint, streams bytes to the mount,
  fsyncs, and atomically renames it. Startup restores that checkpoint before
  migrations. Background cleanup and delivery jobs also checkpoint.
- SMTP truth: `/api/runtime` reports `smtp` or `not_configured`. The visible
  plan and cancellation result use that state. No-relay deployments describe
  offers as recorded and not sent. The existing SMTP delivery/retry adapter is
  unchanged and remains available when configured.
- Claim coverage: `.factory/claims.json` has 15 independently runnable claims.
  New exact tests cover released-seat delivery status, 24-hour demo expiry and
  input disposal, non-mutating reconciliation, and durable restart. Shared
  Rust coverage was split into one named test per claim.
- Mobile/accessibility: removed the 320-px body floor, added min-width and wrap
  guards to grid/flex content, and made inline legal/navigation actions at
  least 44 CSS px. Regression coverage checks `/`, `/demo`, `/app`, `/privacy`,
  and `/terms` at 390×844 with 200% root text.
- Checkout: preserved the exact recurring `$99/month` copy and direct
  `https://api.sociobot.in/api/v1/products/class-capacity-truth/checkout` link.
  Registration is not bypassed with a provider form or a different product.

## Verification evidence

Local checks from the repaired tree:

```bash
npm ci
npm test
npm run typecheck
npm run lint
npm run build
env -u CI npm run test:e2e
npm run test:cold-claim
jq -r '.[].test' .factory/claims.json # each command run separately
./scripts/load-smoke.sh http://127.0.0.1:4174
```

- Clean install: 170 packages, 0 vulnerabilities.
- Unit/integration: 6/6 TypeScript, 4/4 Rust unit, and 13/13 Rust API tests.
- TypeScript strict typecheck, Rust format, and Clippy `-D warnings`: pass.
- Production build: initial JS 228.26 KB raw / 69.87 KB gzip; CSS 17.86 KB
  raw / 4.18 KB gzip; the optional Entra chunk is lazy-loaded.
- Chromium: 24/24 pass at desktop and 390 px, including keyboard, route focus,
  dark/reduced-motion, 200% text, every public/legal/404 route, and axe.
- Every one of the 15 exact claim commands passes independently.
- Zero-environment release start generated both keys and returned database
  ready. A normal container starts with only `PORT`; extra paths merely isolated
  this test.
- Response-policy smoke: 100 parallel requests produced 10 accepted and 90
  rate-limited; 429 responses included `Retry-After`.
- Offline/update and package-consumer checks do not apply: this is a connected
  `web-with-backend` container and publishes no offline or package claim.

Live checks on revision `--0000015`:

- `/health` returned build
  `8d837b3789681677de77394b49b10f24e45cdce9` and `database: ready`.
- A demo class changed from 6 to 7 confirmed seats, the revision was restarted,
  and the signed session restored 7. A second booking replaced the existing
  checkpoint, a second completed replica replacement occurred, and the new
  replica restored 8. The mounted checkpoint was 208,896 bytes.
- Runtime configuration reports supplied durable checkpoint and keys, WAL with
  eight local connections, local-capture SMTP, one Azure Files mount, and
  min/max replicas `1/1`. No secret values were printed.
- `/opt/fleet/lib/verify-url.sh` passed: load 694 ms, title/lang/one H1/main/alt
  checks pass, and no browser console errors occurred.
- Live 390×844 checks found no overflow on all five routes at 200% text, no
  sub-44-px main/footer actions, zero serious/critical axe findings, correct
  route titles, and `Skip to main content` as the first keyboard target.
- Request capture across landing, demo, workspace, privacy, and terms found no
  foreign origin. `/api/runtime` returned `emailDelivery: not_configured`.
- Live rate smoke again returned 10 accepted and 90 rate-limited responses.
  Unknown routes return 404; security and no-cache headers are present.
- Entra discovery returned 200. The sign-in action reached the shared CIAM
  origin with client `25c704f4-465a-47af-80ab-2c489466b697`, authorization-code
  flow, and the production `/auth/callback` redirect URI.
- Mobile Lighthouse: performance 100, accessibility 100, best practices 100,
  SEO 100; LCP 1.2 s, CLS 0, TBT 0 ms.

ACR run `chsm` produced
`sociobotregistry.azurecr.io/sf-class-capacity-truth:8d837b378968`, digest
`sha256:1ed23862a9ebdcd657d17289e0ca62e88e6f9e690984dac41b3b68ef7084ea3d`.

## Needs operator action

1. Register recurring product slug `class-capacity-truth` at `$99/month` in
   Sociobot billing. The required checkout returned HTTP 404 at handoff time;
   the controller explicitly owns registration.
2. Supply `SMTP_RELAY` and, if required, `SMTP_USERNAME`, `SMTP_PASSWORD`, and
   `SMTP_FROM` to enable external delivery. Until then, the UI and API expose
   the non-promissory recorded-only state.

No DNS, direct payment-provider configuration, or mail-provider credential was
created or changed.
