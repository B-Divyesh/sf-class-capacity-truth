# Verification 15 — FAIL

Candidate: `cc5542bbec9b12fc8b5f61cd25e50824c563c6c9`

Live URL: <https://class-capacity-truth.sociobot.in>

Verified: 2026-08-30 03:10 UTC

Work order: `class-capacity-truth-verify-15`

## Decision

**FAIL — release blocked by live topology and durability drift.** The exact
candidate image is live and the clean candidate passes every local quality and
claim test. Production does not use that candidate's required deployment
topology, however: it allows three replicas, currently runs three ready
replicas, and has no durable volume or mount. Each replica therefore has its
own SQLite database and generated cookie/encryption keys.

This is already breaking the real demo. One cookie jar received eight
different first-class IDs over nine consecutive reads. In 12 of 12 fresh
browser attempts, a visitor could open the sample class list but clicking the
available class produced **“This sample link has ended”** instead of a booking
form. The same scale-out also multiplies per-process rate-limit allowances.

No product code or infrastructure was changed during verification.

## Required first gates

### Cold first read — PASS

At 1440×900 in a new browser context, before scrolling, the page says:

- what it does: **“Show the right number of class seats”**;
- for whom: **“For small language schools”** whose booking calendar and room
  list disagree about places; and
- what to click: **“Try it with sample data”**, next to “Three sample classes
  open next.”

The action is visible and opens three realistic Bright Path Languages classes
in one click with the persistent **“Demo — sample data, nothing is saved”**
banner, **Reset demo**, and **Start for real**. The first-read gate itself
passes. The next required product action is broken live by the deployment
defect described below.

Evidence: `evidence-15/live-first-read-desktop.png` and
`evidence-15/live-demo-broken-by-replicas.png`.

### Claims gate — PASS locally (22/22)

`.factory/claims.json` exists. From the initially clean exact candidate, `npm
ci` completed with 0 vulnerabilities, then every manifest command was run
independently in manifest order. Every command exited 0.

| Claim | Exact command | Result |
| --- | --- | --- |
| `sample-booking-updates-seats` | `npm run test:e2e -- --grep @claim:sample-booking-updates-seats` | PASS, 1/1 |
| `full-class-blocks-booking` | `npm run test:e2e -- --grep @claim:full-class-blocks-booking` | PASS, 1/1 |
| `cutoff-blocks-booking` | `npm run test:e2e -- --grep @claim:cutoff-blocks-booking` | PASS, 1/1 |
| `demo-reset-isolated` | `npm run test:e2e -- --grep @claim:demo-reset-isolated` | PASS, 1/1 |
| `school-capacity-flow` | `npm run test:e2e -- --grep @claim:school-capacity-flow` | PASS, 1/1 |
| `calendar-poll` | `npm run test:api -- claim_calendar_feed_is_encrypted_and_polled_every_five_minutes` | PASS, 1/1 |
| `released-seat-delivery` | `npm run test:e2e -- --grep @claim:released-seat-delivery` | PASS, 1/1 |
| `school-plan-price` | `npm run test:e2e -- --grep @claim:school-plan-price` | PASS, 1/1 |
| `no-third-party-tracking` | `npm run test:e2e -- --grep @claim:no-third-party-tracking` | PASS, 1/1 |
| `contact-encryption-retention` | `npm run test:api -- claim_contact_encryption_and_retention` | PASS, 1/1 |
| `staff-role-access` | `npm run test:api -- claim_staff_roles_enforce_owner_actions` | PASS, 1/1 |
| `data-export-delete` | `npm run test:e2e -- --grep @claim:data-export-delete` | PASS, 1/1 |
| `demo-expiry-input-disposal` | `npm run test:api -- claim_demo_expiry_and_input_disposal` | PASS, 1/1 |
| `reconciliation-does-not-change-seats` | `npm run test:api -- claim_reconciliation_never_mutates_confirmed_seats` | PASS, 1/1 |
| `durable-restart` | `npm run test:durable-restart` | PASS; release-process booking and decrypted contact survived restart |
| `configured-smtp-delivery` | `npm run test:api -- claim_configured_smtp_queues_an_encrypted_offer` | PASS, 1/1 |
| `workspace-recovery` | `npm run test:api -- claim_workspace_recovery_by_staff_identity` | PASS, 1/1 |
| `oldest-waitlist-offer` | `npm run test:api -- claim_oldest_waitlist_entry_gets_a_24_hour_offer` | PASS, 1/1 |
| `zero-config-runtime` | `bash scripts/test-zero-config.sh` | PASS |
| `forwarded-ip-rate-limits` | `npm run test:api -- rate_limit_uses_forwarded_ip_and_returns_retry_after` | PASS, 1/1 |
| `durable-one-replica-topology` | `npm run test:deployment` | PASS against the recorded control-plane fixture |
| `operational-metrics-no-pii` | `npm run test:api -- regression_protected_operational_metrics_are_aggregated_and_contain_no_pii` | PASS, 1/1 |

The deployment-topology claim test proves the deployment guard against a
fixture. It does not query production. Fresh production evidence contradicts
the claim, so the local PASS does not make the release acceptable.

## Release-blocking defects

### P0 — production is three ephemeral SQLite replicas, breaking the demo and durability

Fresh Azure readback for active revision
`sf-class-capacity-truth--0000045` showed:

```text
image:       sociobotregistry.azurecr.io/sf-class-capacity-truth:cc5542bbec9b
traffic:     100% to latest revision
minReplicas: 1
maxReplicas: 3
replicas:    3 ready/running
volumes:     null
mounts:      null
environment: PORT=8080 only
```

`scripts/verify-container-topology.sh` exited **1** against this live state.
The repository and README require exactly one replica plus the `cct-data`
Azure Files mount at `/mnt/cct` for SQLite snapshots and generated keys.

Direct user impact is deterministic at the time of verification:

- nine consecutive `GET /api/demo/session` calls from one forwarded IP and one
  cookie jar returned eight different sample class IDs;
- 12/12 new browser contexts followed Home → Try it with sample data → Book
  this sample class and all 12 reached **“This sample link has ended”**;
- the product could complete the live booking only when consecutive requests
  happened to hit a compatible replica earlier in the run;
- real-school SQLite writes, cookie signing keys, and contact-encryption keys
  are likewise replica-local and have no mounted restart boundary.

Required repair: deploy through the checked-in guarded deployment path, set
`minReplicas=1` and `maxReplicas=1`, restore `cct-data` at `/mnt/cct`, set the
documented durable data/snapshot paths, then prove a real booking and decrypted
contact survive a revision restart. Re-run the ordinary live demo flow from
fresh contexts after the topology readback passes.

Evidence: `evidence-15/containerapp.json`, `evidence-15/replicas.json`,
`evidence-15/revisions.json`, and
`evidence-15/live-demo-broken-by-replicas.png`.

### P0 — the documented single-client request allowance is multiplied by replicas

Locally, a 100-request concurrent smoke from one forwarded IP produced exactly
10 accepted responses and 90 `429` responses. Live, the same limiter advertises
`X-RateLimit-Limit: 10`, but:

- 12 fast sequential requests from one client all returned 200 and the
  remaining count jumped between independent buckets;
- a 45-request concurrent wave from one forwarded IP returned **20×200** and
  **25×429**;
- the `429` response correctly had `Retry-After: 5` and
  `X-RateLimit-Remaining: 0`;
- a second forwarded IP then received 12/12 successful requests despite the
  advertised allowance of 10.

The observed effective live allowance was at least 20 for one client while two
replicas were serving the wave; Azure then showed three replicas. This violates
the documented once-per-forwarded-client allowance and follows directly from
the unsupported multi-replica topology.

Required repair: the one-replica topology above is mandatory for the current
in-memory limiter and SQLite architecture. A future multi-replica design would
need shared persistence, shared keys, and a distributed limiter.

### P1 — top-level `/metrics` is not rate limited

The protected `/metrics` alias is mounted outside the governor layer. A fresh
same-IP burst of 60 requests returned **60×401**, with no
`X-RateLimit-Limit`, no `Retry-After`, and no `429`. The equivalent
`/api/metrics` route does expose the per-replica school limiter.

This independently violates the backend contract that every server endpoint
except `/health` is rate limited. Either remove the alias or put it under the
same forwarded-IP limiter and add a regression that exceeds the allowance.

## Clean local verification

- Initial worktree: clean; `HEAD` and `origin/main` were the exact candidate.
- `npm ci`: PASS; 170 packages, 0 vulnerabilities.
- `npm test`: PASS; 8 Vitest tests, 6 Rust unit tests, 19 Rust API/integration
  tests, and both deployment regression scripts.
- `npm run typecheck`: PASS.
- `npm run lint`: PASS, including rustfmt and clippy with warnings denied.
- `npm run build`: PASS; emitted `dist/` and the release API binary.
- Full browser suite with retries disabled: `CI=1 npm run test:e2e --
  --retries=0` — PASS, 26/26.
- Build output: 73.54 kB gzip initial JavaScript, 79.59 kB gzip lazy
  staff/auth JavaScript, and 4.62 kB gzip CSS.
- Local load smoke: exactly 10×200 and 90×429 from 100 concurrent same-IP demo
  requests. Startup logs recorded generated and persisted keys without values.
- A container-image build could not be repeated because Docker is not installed
  in this verifier image. The exact repository production build and zero-config
  release-process claim both passed.

## Independent live product evidence

### Candidate identity and health — PASS

- `/health` returned HTTP 200 with `status: ok`, `database: ready`, and the
  exact full candidate SHA.
- Azure image tag is `cc5542bbec9b` and all traffic targets that revision.
- Local and live SHA-256 hashes match byte-for-byte for `index.html`, the
  initial JS, and CSS.
- No runtime source file differs between deployed source `2991e638…` and the
  candidate; the candidate's later changes are verification documentation.

The deployment runs the candidate but with an unsafe runtime topology, so
identity does not cure the release failure.

### Core, boundary, and recovery behavior — mixed

When several calls happened to stay on one replica, a live open class moved
from two seats to one after a valid booking. A one-character guardian name sent
no request and exposed the browser's minimum-length message; correcting it
completed the booking. Two simultaneous requests for the last seat returned
one 201 and one 409, leaving no oversell. Direct attempts against full and
cutoff classes returned 409 with `class_full` and `booking_closed`; reset
restored two seats.

Those backend rules are sound per replica and are comprehensively covered by
the local suite. The normal routed demo is nevertheless unusable because the
next request reaches another local database, as the 12/12 failure above proves.

### Privacy, security, identity, and billing — PASS except rate findings

- Playwright recorded 15 requests through home, demo, privacy, and signed-out
  workspace; every origin was the product origin. There were no console/page
  errors. No third-party font, script, tracker, or analytics request occurred.
- HTML/API responses are no-cache; the hashed JS response is
  `public, max-age=31536000, immutable`.
- Response headers include `nosniff`, strict-origin referrer policy, restrictive
  permissions policy, and a response-header CSP with `frame-ancestors 'none'`.
  HTTP redirects to HTTPS. Approved-origin preflight receives that origin;
  an unapproved origin receives no `Access-Control-Allow-Origin`.
- Signed-out metrics return 401 plus `WWW-Authenticate: Bearer`.
- Explicit sign-in navigates only to
  `sociobotcustomers.ciamlogin.com/35c6fe40-0ec0-46b6-98c6-213ad4de6650/`,
  client `25c704f4-465a-47af-80ab-2c489466b697`, callback
  `/auth/callback`, `openid profile email`, response type `code`, and PKCE
  `S256`. Discovery returns the GUID-based issuer and Sociobot JWKS URI.
- The live $99 action sent POST to the Sociobot billing API, received 200, and
  reached a live `checkout.dodopayments.com` session titled “Sociobot |
  Checkout.” No payment was submitted.

### Accessibility, mobile, routes, and performance — PASS

- The factory `verify-url.sh` passed: title and `lang=en`, one h1, a main
  landmark, no missing alt, no unnamed button, and no console error.
- Independent Playwright Axe runs found zero violations at any impact level on
  home, demo, privacy, terms, signed-out app, operations, and the 404 page.
- At 390 px in dark mode with reduced motion, there was no horizontal overflow;
  seat animation and transition durations were `0s`; the labelled menu opened
  with Enter, closed with Escape, and restored focus.
- The skip link is first in the tab order with a 3 px high-contrast ring and 3
  px offset. Activating it moves focus to the first content heading inside
  `main`. All checked routes reflowed at 200% text with zero overflow.
- Every shipped workspace deep link returned 200 with a route-specific title,
  canonical, one h1, and one main. Internal links, robots, sitemap, favicon,
  touch icon, and social card returned 200. An unknown path returned a real 404.
- Fresh mobile Lighthouse: performance 100, accessibility 100, best practices
  100, SEO 100; FCP/LCP 1,276 ms, TBT 14 ms, CLS 0, total transfer 79,621 bytes.

Evidence is under `.factory/evidence-15/`.

## Applicability and limitations

- This is not a library or CLI, so consumer package installation is not
  applicable.
- It makes no offline/PWA claim and registers no service worker, so offline
  reload and service-worker update checks are not applicable.
- No live staff credentials were available. The live redirect/OIDC contract,
  signed-out challenges, and local signed-in role/workspace flow were tested;
  no credential was entered.
- No live persistence restart was performed because verification is
  non-mutating and the required mount is absent. Azure readback and live demo
  churn already prove that boundary is broken.

## Final disposition

The candidate is **not accepted**. Repair the live one-replica Azure Files
topology first, verify the deployed readback before sending traffic, prove a
booking survives a revision restart, and repeat the live demo/rate-limit
checks. Also rate-limit or remove the top-level `/metrics` alias.
