# Independent verification 5 — FAIL

Verified 2026-08-29 UTC against candidate commit
`029c619bf3bba1c156f650f15cc14e49ef733146` and
<https://class-capacity-truth.sociobot.in>.

## Verdict

**FAIL. Do not release or accept real school data.** The source checkout is
healthy, every declared claim command passes, the live image and assets match
the candidate, checkout and CIAM are connected, and the public UI is strong.
The live deployment is nevertheless unsafe for the product's core job. It is
configured for up to three replicas with no volume, database URL, durable
snapshot, or stable keys. The required load test caused two replicas to run.
They immediately disagreed about signed demo sessions and persisted state.

## Release-blocking findings

### P0 — production has independent, ephemeral databases and keys

Fresh read-only Azure inspection of active revision
`sf-class-capacity-truth--0000022` found:

- image `sociobotregistry.azurecr.io/sf-class-capacity-truth:029c619bf3bb`;
- `minReplicas: 1`, **`maxReplicas: 3`**;
- **two healthy running replicas** after the required load checks;
- only `PORT=8080` in the container environment;
- `volumeMounts: null` and `volumes: null`.

Startup logs from both replicas independently reported
`database_config: generated-default`, `durable_backup: disabled`, and
`cookie_signing_key` / `contact_encryption_key` as
`generated-and-persisted`. Those files and SQLite databases are therefore on
each replica's disposable filesystem. This contradicts the handoff and README
claims that production is fixed to one replica and checkpoints successful
changes to Azure Files.

The fault was reproduced at the public API, not inferred only from settings.
Using one freshly issued signed demo cookie and one class ID for ten identical
idempotent booking requests produced **four 201 responses and six 401
`demo_cookie_missing` responses**, depending on which replica received the
request. Twenty repeated session reads with that same fixed cookie replaced
the cookie ten times. A real browser can therefore lose or switch sample state
between requests. More importantly, real school workspaces, bookings,
waitlists, receipts, and the keys needed to decrypt contacts can split across
replicas and disappear when a replica is replaced.

Required fix: deploy exactly one replica with the proven Azure Files mount,
persisted keys, and durable snapshot path, or move state and rate limits to a
replica-safe shared store. Then prove booking state and decryption across an
actual restart before re-verification.

### P1 — the live per-client allowance multiplies across replicas

The source configures the anonymous API for burst 10 and the school API for
burst 40. A local single-process smoke behaved exactly that way: 10 accepted /
90 rate-limited for the demo and 40 authentication responses / 60 HTTP 429 for
the protected API, with `Retry-After` on 429.

Once production had two replicas, one forwarded client IP received **20**
successful demo-session responses before 80 HTTP 429 responses even though
each response advertised `X-RateLimit-Limit: 10`. The protected endpoint
similarly returned 82 authentication responses and only 18 HTTP 429 responses
in a 100-request burst. A sequential 60-request protected run produced no 429.
The live allowance is therefore per replica, not per client as documented.
This must be fixed together with the P0 topology issue; every observed 429 did
include `Retry-After`.

### P1 — public claims are not completely registered or proven

All 15 registered commands pass, but the claims contract also requires every
public claim to be registered and its tagged test to prove the complete
observable statement. The following README/application promises have no
matching claim entry and tagged proof:

- “A configured SMTP relay can send it instead.”
- “A workspace can be recovered on another device after sign-in.”
- a 24-hour offer goes to the oldest waiting guardian;
- the service needs no environment variables; and
- the README architecture's forwarded-IP rate-limit promise.

The registered `calendar-poll` claim says an **encrypted** feed is checked
**every five minutes**, but its tagged browser test only connects a fixture and
performs an immediate manual check. It neither inspects ciphertext at rest nor
advances the clock to exercise the five-minute background poll. These are
release-blocking claim-registry/coverage gaps under the supplied contract.

### P2 — the standalone 404 fails the mobile zoom/touch baseline

At 390×844 with root text at 200%, the live HTTP 404 document had 72 CSS pixels
of horizontal overflow (`scrollWidth` 462 versus 390). Its only recovery link,
“Go to Class Capacity Truth,” measured 267×21 CSS pixels, below the required
44-pixel target height. Other tested routes had no overflow or undersized
targets. Evidence:
`.factory/qa-artifacts/verification-5-live/404-mobile-text-200.png`.

## Mandatory first-read and demo gate

**PASS.** A cold desktop and 390px mobile visit immediately stated:

- what: “Show the right number of class seats”;
- for whom: small language schools whose calendar and room list disagree; and
- first action: **Try it with sample data**, followed by “Three sample classes
  open next.”

The one-click action opened `/demo?demo=1`, displayed three populated class
states, and kept the “Demo — sample data, nothing is saved” banner with
**Reset demo** and **Start for real** visible.

## Mandatory claims gate

`.factory/claims.json` exists with 15 entries. After `npm ci`, each exact
listed command was executed separately. The first browser claim compiled the
backend from the clean cache and passed in 3.2 minutes; subsequent commands
used only their declared demo/test fixtures.

| Claim | Result |
| --- | --- |
| `sample-booking-updates-seats` | PASS — 1/1 |
| `full-class-blocks-booking` | PASS — 1/1 |
| `cutoff-blocks-booking` | PASS — 1/1 |
| `demo-reset-isolated` | PASS — 1/1 |
| `school-capacity-flow` | PASS — 1/1 |
| `calendar-poll` | PASS — 1/1; coverage gap noted above |
| `released-seat-delivery` | PASS — 1/1 |
| `school-plan-price` | PASS — 1/1 |
| `no-third-party-tracking` | PASS — 1/1 |
| `contact-encryption-retention` | PASS — 1/1 |
| `staff-role-access` | PASS — 1/1 |
| `data-export-delete` | PASS — 1/1 |
| `demo-expiry-input-disposal` | PASS — 1/1 |
| `reconciliation-does-not-change-seats` | PASS — 1/1 |
| `durable-restart` | PASS — 1/1 locally; contradicted by live deployment |

## Clean-checkout verification

| Check | Result |
| --- | --- |
| Candidate and tree | PASS — initial HEAD exactly matched; no pre-existing changes. |
| `npm ci` | PASS — 170 packages; 0 vulnerabilities. |
| `npm test` | PASS — 6 Vitest, 4 Rust unit, 13 Rust API/integration tests. |
| `npm run typecheck` | PASS. |
| `npm run lint` | PASS — TypeScript, rustfmt, and Clippy `-D warnings`. |
| `npm run build` | PASS — `dist/` and optimized Rust binary; cold release build 6m07s. |
| `env -u CI npm run test:e2e` | PASS — 23/23 Chromium tests. |
| Production bundle | PASS — initial JS 230.56 KB raw / 70.63 KB gzip; CSS 18.88 KB raw / 4.35 KB gzip; lazy CIAM chunk 79.59 KB gzip. |
| Zero-config API start | PASS — release binary started with only `PORT`, generated/persisted keys, and returned `database: ready`. |
| Local load smoke | PASS — demo 10/100 accepted, 90/100 429; protected API 40/100 then 429, all with `Retry-After`. |
| Container build | NOT RUN — Docker, Podman, Buildah, and Nerdctl are unavailable. Dockerfile inspection passed the multi-stage, floating Rust 1, non-root, `PORT`, and build-arg requirements. |

The local browser suite exercised the complete smallest useful flow: create a
school and capacity-one class, publish it, connect/check a recorded iCalendar
feed, book the final seat, block oversell, join the waitlist, cancel a selected
booking, persist/copy the released-seat offer, reload, accept it once, export,
and delete. It also covers the exact cutoff instant, invalid input, concurrent
seat allocation, and recovery states.

## Live functional and integration evidence

- `/health` returned 200, `database: ready`, and exact build
  `029c619bf3bba1c156f650f15cc14e49ef733146`.
- SHA-256 hashes of all three deployed hashed assets exactly matched local
  `dist/`; the active image tag also matches the candidate prefix.
- The live demo accepted corrected input, changed two open seats to one,
  reset to two, and suppressed booking controls for full and cutoff classes
  before the deployment scaled. Invalid email was blocked with the browser's
  specific correction. The later multi-replica failure is documented above.
- Live checkout made `POST` to the Sociobot product endpoint, received 200
  with CORS restricted to the product origin, and navigated to a newly created
  `checkout.dodopayments.com/session/...` URL. No card was submitted.
- Signed-out `/app` exposed only **Sign in with Sociobot**. Clicking it reached
  `sociobotcustomers.ciamlogin.com` with the required tenant and client IDs,
  production `/auth/callback`, authorization code flow, PKCE S256, and
  `openid profile email offline_access` scopes. A bogus bearer received 401
  plus `WWW-Authenticate: Bearer`.
- The demo's response cookie was `HttpOnly`, `Secure`, `SameSite=Strict`, and
  had `Max-Age=86400`.
- During landing and the complete demo flow, every observed request was
  same-origin; there were no analytics, trackers, remote scripts, fonts, page
  errors, or console errors. Microsoft and Sociobot billing were contacted
  only after their explicit staff actions.
- All crawled links returned 200 or were explicit `mailto:` links.
- HTML/API responses use `no-cache, max-age=0`; hashed assets use
  `public, max-age=31536000, immutable`. CSP includes header-only
  `frame-ancestors 'none'`; nosniff, strict referrer, and restrictive
  permissions policies are present. Unknown paths return HTTP 404.

## Accessibility, responsive, and performance evidence

- Factory `verify-url.sh` passed in 594 ms: title, `lang=en`, exactly one H1,
  main landmark, image/button names, and console checks were clean.
- Fresh live axe scans of `/`, demo, signed-out `/app`, privacy, terms, and the
  HTTP 404 found zero serious or critical findings in dark/reduced-motion
  mobile mode.
- Keyboard-only testing focused the skip link, activated the booking route,
  moved focus to the new H1, blocked invalid input, completed booking, and
  showed a designed 3px focus outline.
- At 390px and 200% text, every application route except the standalone 404
  had zero horizontal overflow and no target below 44px.
- Fresh mobile Lighthouse: performance 98, accessibility 100, best practices
  100, SEO 100; FCP 1.3s, LCP 1.3s, TBT 150ms, CLS 0.
- Evidence is in `.factory/qa-artifacts/verification-5-live/`.

## Applicability and next steps

This is not a library/CLI and not a PWA; package-consumer and service-worker
checks do not apply. AI would not improve the narrow deterministic seat ledger
and is appropriately absent.

Before re-verification: restore durable mounted storage and a one-replica cap
(or adopt truly shared state/limits), prove a real restart, repair the claim
registry/tests, and fix the standalone 404 at 200% text. Do not treat passing
local snapshot tests as evidence that production has the required mount.
