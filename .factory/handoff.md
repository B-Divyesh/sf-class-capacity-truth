# Independent QA handoff — class-capacity-truth-verify-3

## Result: FAIL

Candidate `b75b14a70e947fa49548bc2531ff2a3f5c7a551b` was independently
verified on 2026-08-29 UTC at https://class-capacity-truth.sociobot.in. The live
health build and all frontend asset hashes match the candidate, so the prior
deployment mismatch is resolved. **Do not release or onboard a school.**

Release blockers:

- the first exact `.factory/claims.json` command failed from the clean clone
  because the 120-second Playwright web-server timeout expired during the cold
  Rust compile (it passed after warmup, but the contract makes the initial
  failure blocking);
- the live container has only `PORT`, no volume, generated-default SQLite and
  generated keys on revision-local storage, while scaling permits up to three
  replicas;
- the visible Sociobot $99/month checkout returns HTTP 404;
- live startup reports `smtp: local-capture`, so promised released-seat email
  does not leave the container;
- claims omit/test only part of material public promises, including delivery,
  24-hour demo expiry/input disposal, and non-mutating reconciliation; and
- every checked route horizontally overflows at 390 px with 200% text; three
  inline links are only 19 px high against the 44 px target rule.

Positive evidence: first-read and one-click demo pass; `npm ci`, `npm test`,
`npm run typecheck`, `npm run lint`, `npm run build`, and the warmed full
Playwright suite (21/21) pass; live booking/full/cutoff/reset/isolation/error
recovery and concurrent no-oversell pass; Entra targets the required CIAM
tenant; privacy request logging is same-origin until explicit sign-in; rate
limits return 429 plus `Retry-After`; exact build/asset identity, security and
cache headers, route semantics, axe, normal 390 px, reduced motion, and mobile
Lighthouse 100/100/100/100 pass.

Full evidence, exact observed allowances, commands, defects, and screenshots
are in [verification-3.md](verification-3.md) and
`verification-evidence/`. Product code was not modified.

---

# Previous repair handoff — class-capacity-truth-repair-2

## Result

Repository repair and container deployment completed on 2026-08-29 UTC. The live
revision is `sf-class-capacity-truth--0000009`, built from
`3fce0cc1cc4870855e9952cf288326ae4430fd9c`. `/health` returns that full SHA and
`database: ready`. The deployment remains one non-root container on `PORT=8080`
with one replica.

Two factory-owned production integrations still need operator configuration:
the Sociobot billing catalogue has no `class-capacity-truth` product, so its
required hosted checkout currently returns 404; and no SMTP relay is supplied
to the Container App, so live offer mail is captured in the encrypted outbox
instead of leaving the service. The source now contains and tests the complete
billing verification/entitlement and SMTP delivery/retry paths. No Dodo or mail
provider secret was embedded or bypassed.

## Verifier findings repaired

- The public waitlist endpoint now returns JSON for 201, requires an idempotency
  key, and the visible form announces success or a useful error. The claim test
  uses the form; it no longer inserts with a hidden `fetch`.
- A school connects a private HTTPS iCalendar feed. Its URL is encrypted. The
  background worker and manual check poll the feed, map event summaries to
  classes, store checks, and schedule the next poll for five minutes.
- Cancelling a named booking atomically selects the oldest eligible waitlist
  entry, creates one 24-hour offer, and writes an encrypted-recipient outbox
  record. SMTP delivery has retry state and never exposes the offer URL in the
  staff UI.
- Staff sign in through the shared Sociobot Entra CIAM SPA. The client uses PKCE
  and session storage. The API discovers issuer/JWKS and validates RS256,
  audience, tenant, issuer, expiry/not-before, then keys access by `oid`.
  Owner/operator/viewer checks are server-side, and a workspace is recoverable
  on another signed-in device. The production redirect URI reaches the Entra
  sign-in page.
- New workspaces receive a 14-day trial. Writes require trial/active/grace
  entitlement. The $99 monthly plan and hosted Sociobot checkout are shown;
  owners can restore a purchase token through the Sociobot verification API.
- Guardian names, emails, waitlist contacts, and calendar URLs use
  XChaCha20-Poly1305 at rest. The key is generated with a CSPRNG and persisted.
  A 90-day cleanup scrubs contact fields. Owners can export JSON or confirm full
  workspace deletion. Privacy copy now states controller/processor roles,
  recipients, retention, regional rights, and access/deletion choices.
- Staff choose a named confirmed booking and confirm before cancellation. The
  old "cancel oldest family" route is gone.
- `datetime-local` values are interpreted in the selected school IANA time
  zone, independent of the browser zone. Display uses the same zone.
- The claims registry has 12 observable claims covering demo isolation,
  calendar polling, offer queuing, price, tracking, encryption/retention,
  roles, export/deletion, and the full visible school flow.
- Initial focus leaves the skip link first. Route changes still focus and
  announce the H1. Skip/footer targets are at least 44 px. `/app` is in the
  sitemap and the landing copy audit matches current text.
- Rust formatting and strict Clippy findings are fixed and enforced by
  `npm run lint`.

The passing demo, concurrency protection, 404 behavior, security/cache headers,
rate limiting, responsive design, and original classroom-abacus identity were
preserved.

## Exact verification evidence

From a clean dependency install:

```bash
npm ci
npm test
npm run lint
npm run build
npm run test:e2e
jq -r '.[].test' .factory/claims.json | sort -u
./scripts/load-smoke.sh http://127.0.0.1:4174
```

- `npm ci`: 170 packages installed, 171 audited, 0 vulnerabilities.
- `npm test`: 6/6 Vitest, 4/4 Rust unit, and 8/8 API/integration tests passed.
- `npm run lint`: strict TypeScript, `cargo fmt --check`, and Clippy with
  `-D warnings` passed.
- Every distinct command in `.factory/claims.json` was run independently and
  passed. The full Chromium run passed 21/21.
- Browser coverage includes 1440×900, 390×844 dark/reduced-motion, 200% text,
  keyboard order, route focus, dialog confirmation, request logging, loading
  and error recovery, all public/legal/404 routes, and axe on those routes.
- The 100-request policy smoke returned 10 accepted and 90 rate-limited
  responses; 429 responses included `Retry-After`.
- A zero-configuration release binary started under `env -i PORT=18083`,
  generated/persisted its defaults, and returned a ready health response.
- Production build: initial JS 69.63 KB gzip; CSS 3.98 KB gzip. The Entra library
  is a lazy 79.59 KB gzip chunk and is not part of the cold landing load.
- Served-release Lighthouse: 100 performance, 100 accessibility, 100 best
  practices, 100 SEO; LCP 1.3 s, CLS 0, TBT 0 ms.
- Live Lighthouse: 100/100/100/100; LCP 1.2 s, CLS 0, TBT 0 ms.
- Live browser smoke: first Tab focused `Skip to main content`; sample booking
  completed; desktop and 390 px app axe scans found zero violations; 390 px had
  no horizontal overflow; no console errors were recorded.
- Live initial JavaScript SHA-256 exactly matches local production output:
  `4afbf40b23f10ee6d9d189aebec51648e82eb128c40185309ba577dcf1f96793`.
- HTML is `no-cache`; hashed assets are one-year immutable. Live CSP includes
  only self, Entra, and Sociobot API connections and sends `frame-ancestors`
  as a response header. Unknown routes retain the styled HTTP 404.
- Entra discovery returned 200, and an authorize request using the production
  callback returned the Microsoft sign-in page rather than a redirect error.
- Offline/update and package-consumer checks are not applicable: this remains a
  connected `web-with-backend` container and makes no offline or package claim.

Evidence is committed under `.factory/qa-artifacts/repair-local/` and
`.factory/qa-artifacts/repair-live/`.

## Deployment

- ACR run `chqk` built and pushed
  `sociobotregistry.azurecr.io/sf-class-capacity-truth:3fce0cc1cc48`, digest
  `sha256:ac99e154a1149f7c2a188d1e283cf43d598c84cb1bb9ef5c0966971c61cbc314`.
- Container App `sf-class-capacity-truth`, resource group `sociobot`, serves the
  custom domain with revision `--0000009`, min/max replicas `1/1`.
- Live health: `{"status":"ok","build":"3fce0cc1cc4870855e9952cf288326ae4430fd9c","database":"ready"}`.
- An attempted Azure Files mount was rejected before traffic because SMB file
  locking is incompatible with SQLite. Both failed revisions were deactivated,
  their never-used share was removed, and no production data was deleted.

## Needs operator action

1. Register the recurring `$99/month` product slug `class-capacity-truth` in the
   Sociobot billing service. As of handoff,
   `https://api.sociobot.in/api/v1/products/class-capacity-truth/checkout`
   returns 404. The product calls only Sociobot checkout/verify endpoints.
2. Supply `SMTP_RELAY` and, when required, `SMTP_USERNAME`, `SMTP_PASSWORD`, and
   `SMTP_FROM` to the Container App. Until then, outbox records use the explicit
   local-capture fallback and are not delivered externally.
3. Provide a datastore that supports durable SQLite locking, or migrate the
   service to managed PostgreSQL before onboarding a real school. The current
   `/data` is revision-local because Azure Files SMB could not safely host this
   SQLite workload. Keep max replicas at one until that migration is complete.

No DNS, payment-provider, or email-provider credential was created or changed.
