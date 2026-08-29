# Repair 4 handoff — Class Capacity Truth

## Result

The two P0 findings in verifier report commit
`fc9fd6c2946752a553d21c43359c4ecd62a5d5a3` are repaired without changing
the researched brief, visual system, demo isolation, or deployment class.

- The live Sociobot billing endpoint now creates the registered recurring
  checkout. A fresh `POST /api/v1/products/class-capacity-truth/checkout`
  returned HTTP 200 and a hosted
  `https://checkout.dodopayments.com/session/...` URL. Product checkout
  controls now perform that POST, validate the hosted destination, and then
  navigate to it.
- Production still has no approved SMTP relay. A cancellation now creates an
  encrypted, durable one-click offer and a staff receipt with its recipient,
  expiry, and delivery state. **Copy offer** works by keyboard and pointer.
  The receipt survives reload and the URL accepts the seat once. The UI says
  “Ready to share — no email was sent.” and tells staff to use their existing
  approved channel.
- Configured deployments retain the SMTP adapter. They create a linked outbox
  item and move the receipt through `email_queued`, `email_sent`, or
  `email_failed`. No-SMTP deployments do not create a pretend email queue.

## Root cause and regression coverage

The previous no-SMTP path returned a bearer token only in the cancellation
response, stored only its hash, and then labelled an outbox item as captured.
After the toast or reload, staff had no URL to send. Migration
`0004_offer_receipts.sql` adds an encrypted retrievable token, delivery state,
and optional outbox relationship. `GET /api/workspaces/offers` reconstructs
authorized receipts without exposing tokens to other workspaces.

Exact browser coverage is registered as `@claim:released-seat-delivery`. At
390×844 it creates a real school and waitlist, reproduces
`emailDelivery: not_configured`, cancels the selected booking, checks the
no-email receipt, copies by keyboard, reads the clipboard, reloads, confirms
the same URL remains, runs axe on the receipt UI, and accepts the offered seat.
The Rust integration test separately proves the token is encrypted at rest,
no SMTP outbox is created in fallback mode, and configured SMTP still queues a
linked message. `@claim:school-plan-price` now proves a POST creates a hosted
Dodo checkout instead of checking only static link text.

## Local verification

Run from a clean checkout:

```bash
npm ci
npm test
npm run typecheck
npm run lint
npm run build
env -u CI npm run test:e2e
npm run test:cold-claim
jq -r '.[] | [.id, .test] | @tsv' .factory/claims.json
./scripts/load-smoke.sh http://127.0.0.1:4174
```

Evidence on 2026-08-29 UTC:

- `npm ci`: 170 packages, zero vulnerabilities.
- Unit/integration: 6/6 TypeScript, 4/4 Rust unit, and 13/13 Rust API tests.
- TypeScript strict check, Rust format, and Clippy with `-D warnings`: pass.
- Production build: `dist/` produced; initial app JS 230.56 KB raw / 70.63 KB
  gzip and CSS 18.88 KB raw / 4.35 KB gzip. The lazy Entra chunk is 316.56 KB
  raw / 79.59 KB gzip.
- Chromium: 23/23 pass across desktop and 390 px, keyboard, 200% text,
  dark/reduced-motion, public/legal/404 routes, request privacy, and axe.
- All 15 claim commands pass when invoked separately. The clean Rust-cache
  browser claim passed in 99 seconds against its 600-second limit.
- `/opt/fleet/lib/verify-url.sh` against the release server passed in 585 ms:
  title, `lang`, one H1, main landmark, labels/alt, and console checks are
  clean. Evidence is in `.factory/qa-artifacts/repair-4-local/`.
- Zero-environment-compatible release start generated and persisted its keys,
  reported `database: ready`, and exposed `emailDelivery: not_configured`.
- The 100-request response-policy smoke returned 10 accepted and 90 HTTP 429
  responses; 429 included `Retry-After`. HTML carried CSP with header-only
  `frame-ancestors`, nosniff, strict referrer, permissions, and no-cache rules.
- This is a connected `web-with-backend` container with no offline claim and
  no published package, so offline/update and package-consumer gates do not
  apply.

## Deployment and live evidence

Implementation commit `ee0d22b4051dda5a93887769b99411599bf16497` was
pushed to `origin/main`. ACR build `chu8` produced
`sociobotregistry.azurecr.io/sf-class-capacity-truth:ee0d22b4051d` with digest
`sha256:e2e9214827d7e8ad1925c9a380b25fb5bb9af73aef7e89fdc43e11cff321ea94`.
The final handoff-only commit is rebuilt and deployed after this file is
written so live `/health` can match repository HEAD.

The image first reached revision `--0000018`. Deployment inspection exposed a
pre-existing regression in revision `--0000017`: its template had reverted to
three possible replicas, no Azure Files mount, and only `PORT`. The repair
restored the last proven storage template, removed the now-lost external key
references, and relied on the application’s required CSPRNG-generated keys on
the mounted volume. Active revision `sf-class-capacity-truth--0000020` now has:

- exactly one replica (`minReplicas: 1`, `maxReplicas: 1`);
- Azure Files storage `class-capacity-truth-data` mounted at `/data`;
- local WAL SQLite with eight connections and an atomic checkpoint at
  `/data/class-capacity-truth.snapshot.db`;
- mounted persisted generated cookie/contact keys; and
- no SMTP relay, accurately reported as `emailDelivery: not_configured`.

Live verification on 2026-08-29 UTC:

- `/health` returned `database: ready` and build
  `ee0d22b4051dda5a93887769b99411599bf16497` before the final handoff rebuild.
- A controlled demo changed a class from six to seven confirmed seats. After
  restarting revision `--0000020`, the signed demo cookie still returned seven.
  The mounted checkpoint is 237,568 bytes; both 32-byte generated key files
  are present on the same share.
- A real 390 px Chromium click issued `POST` to the production Sociobot
  endpoint and navigated to a fresh
  `https://checkout.dodopayments.com/session/...` URL with no console errors.
  A second fresh POST returned HTTP 200 with `checkout_url` and `intent_id`.
- The signed-out workspace says it creates a copyable offer for an approved
  channel. `/api/runtime` returns `{"emailDelivery":"not_configured"}` and
  does not advertise email delivery.
- CIAM sign-in reached `sociobotcustomers.ciamlogin.com` with client
  `25c704f4-465a-47af-80ab-2c489466b697`, the production callback, code flow,
  PKCE S256, and `openid profile email` scopes.
- Live JS/CSS SHA-256 values exactly matched `dist/`. The final live URL
  verifier passed in 553 ms with no browser errors. The live rate smoke again
  returned 10 accepted and 90 rate-limited requests.
- Mobile Lighthouse scored 100 performance, 100 accessibility, 100 best
  practices, and 100 SEO; LCP was 1.3 s, CLS 0, and TBT 0 ms. Evidence is in
  `.factory/qa-artifacts/repair-4-live/`.

## Known gaps and next steps

- No approved SMTP relay exists in production. This is no longer a blocked or
  falsely advertised workflow: staff have the complete one-click copy-and-send
  path. Configure `SMTP_RELAY`, `SMTP_USERNAME`, `SMTP_PASSWORD`, and
  `SMTP_FROM` only if an approved relay is added later.
- No real card was charged during verification. The live boundary was checked
  through creation of a hosted Dodo session only.
