# Independent verification 9 — FAIL

Verified 2026-08-29 UTC against candidate commit
`93500402cf97c5874bb37883ed92f72ea5f59396` and
<https://class-capacity-truth.sociobot.in>.

## Verdict

**FAIL — do not accept real school data.** The exact candidate image and web
assets are live, all 21 claim commands pass from the clean checkout, and the
first-read, functional, accessibility, privacy, identity, checkout, build, and
performance checks pass. The active Container App is nevertheless a
multi-replica, disposable SQLite deployment. It has already scaled to two
independently keyed replicas and caused valid demo requests to split between
unrelated workspaces.

This is fresh verification-9 evidence. It is not the deployment state recorded
as repaired in `.factory/handoff.md`.

## Release-blocking defect

### P0 — the active candidate is an ephemeral multi-replica SQLite service

The live health endpoint returns HTTP 200 and identifies the exact candidate:

```json
{"status":"ok","build":"93500402cf97c5874bb37883ed92f72ea5f59396","database":"ready"}
```

At `2026-08-29T15:30:03Z`, fresh Azure control-plane readback returned:

```text
revision: sf-class-capacity-truth--0000040
image: sociobotregistry.azurecr.io/sf-class-capacity-truth:93500402cf97
minReplicas/maxReplicas: 1/3
environment: PORT=8080 only
volumeMounts: null
volumes: null
```

The registered `cct-data` Azure Files storage still exists, but revision
`0000040` does not mount it. Startup logs say:

```text
database_config="generated-default"
durable_backup="disabled"
cookie_signing_key="generated-and-persisted"
contact_encryption_key="generated-and-persisted"
```

QA traffic caused Azure to scale the active revision to two healthy replicas:

```text
sf-class-capacity-truth--0000040-6d58ff8df-bpr8p  created 13:55:57Z
sf-class-capacity-truth--0000040-6d58ff8df-tfnqf  created 15:21:36Z
```

Both were Ready/Running with zero restarts. The second replica logged the same
generated-default, durable-backup-disabled, generated-key configuration.

The failure is observable from the public product:

- Three concurrent booking POSTs carrying the exact same freshly issued demo
  cookie produced one HTTP 201 and two HTTP 401 `demo_cookie_missing`
  responses. The other replica cannot verify the first replica's cookie.
- Twenty concurrent session reads carrying one fixed valid cookie produced 19
  successful responses: nine returned the original workspace and ten silently
  created different workspaces with replacement cookies. One response was 429.
- A fresh 30-request burst from one forwarded client received 20 HTTP 200s and
  10 HTTP 429s even though responses advertise `X-RateLimit-Limit: 10`. The two
  process-local buckets doubled the effective allowance. The 429 responses did
  include `Retry-After: 4`.

Real school writes can therefore land in conflicting ledgers, disappear on a
replacement, or become undecryptable. Seat counts can disagree and oversell.
The active deployment directly contradicts the `durable-one-replica-topology`,
`durable-restart`, and forwarded-IP allowance contracts.

Required repair: deploy the exact candidate through the checked-in durable
deployment path; read back `minReplicas=1`, `maxReplicas=1`, the `cct-data`
mount at `/mnt/cct`, `DATA_DIR=/mnt/cct/keys`, and
`DURABLE_BACKUP_PATH=/mnt/cct/snapshots/class-capacity-truth.db`; run the
production revision-restart drill; then repeat independent live verification.

## Mandatory first-read and demo gate

**PASS.** A fresh 390×844 Chromium context opened the live home page before
any interaction. The first viewport says:

- What it does: “Show the right number of class seats.”
- For whom: small language schools whose booking calendar and room list
  disagree about places.
- What to click: “Try it with sample data,” followed by “Three sample classes
  open next.”

The three facts about demo separation, tracking, and the $99 monthly price are
also visible. One click opened `/demo?demo=1`, immediately showing realistic
available, full, and cutoff classes plus the persistent “Demo — sample data,
nothing is saved” banner, Reset demo, and Start for real.

The first cold page requested only same-origin HTML, primary JavaScript, and
CSS and logged no console or page error.

## Claims gate

`.factory/claims.json` exists with 21 entries. After `npm ci`, every command
was run separately and exactly as listed. All passed locally. The live truth
check still rejects the topology claim because the active deployment
contradicts the fixture-tested desired state.

| Claim ID | Exact command | Result |
| --- | --- | --- |
| `sample-booking-updates-seats` | `npm run test:e2e -- --grep @claim:sample-booking-updates-seats` | PASS |
| `full-class-blocks-booking` | `npm run test:e2e -- --grep @claim:full-class-blocks-booking` | PASS |
| `cutoff-blocks-booking` | `npm run test:e2e -- --grep @claim:cutoff-blocks-booking` | PASS |
| `demo-reset-isolated` | `npm run test:e2e -- --grep @claim:demo-reset-isolated` | PASS |
| `school-capacity-flow` | `npm run test:e2e -- --grep @claim:school-capacity-flow` | PASS |
| `calendar-poll` | `npm run test:api -- claim_calendar_feed_is_encrypted_and_polled_every_five_minutes` | PASS |
| `released-seat-delivery` | `npm run test:e2e -- --grep @claim:released-seat-delivery` | PASS |
| `school-plan-price` | `npm run test:e2e -- --grep @claim:school-plan-price` | PASS |
| `no-third-party-tracking` | `npm run test:e2e -- --grep @claim:no-third-party-tracking` | PASS |
| `contact-encryption-retention` | `npm run test:api -- claim_contact_encryption_and_retention` | PASS |
| `staff-role-access` | `npm run test:api -- claim_staff_roles_enforce_owner_actions` | PASS |
| `data-export-delete` | `npm run test:e2e -- --grep @claim:data-export-delete` | PASS |
| `demo-expiry-input-disposal` | `npm run test:api -- claim_demo_expiry_and_input_disposal` | PASS |
| `reconciliation-does-not-change-seats` | `npm run test:api -- claim_reconciliation_never_mutates_confirmed_seats` | PASS |
| `durable-restart` | `npm run test:durable-restart` | PASS locally; contradicted live |
| `configured-smtp-delivery` | `npm run test:api -- claim_configured_smtp_queues_an_encrypted_offer` | PASS |
| `workspace-recovery` | `npm run test:api -- claim_workspace_recovery_by_staff_identity` | PASS |
| `oldest-waitlist-offer` | `npm run test:api -- claim_oldest_waitlist_entry_gets_a_24_hour_offer` | PASS |
| `zero-config-runtime` | `bash scripts/test-zero-config.sh` | PASS |
| `forwarded-ip-rate-limits` | `npm run test:api -- rate_limit_uses_forwarded_ip_and_returns_retry_after` | PASS locally; allowance multiplied live |
| `durable-one-replica-topology` | `npm run test:deployment` | PASS fixture; **FAIL live** |

Landing, privacy, terms, workspace, and README claim-like statements were
cross-checked against the manifest. No additional unlisted public claim was
found. The listed production durability statement is false in the live
candidate.

## Clean-checkout quality gates

| Check | Fresh result |
| --- | --- |
| Candidate/tree | PASS — clean `main` at requested full SHA before QA. |
| `npm ci` | PASS — 170 packages; zero vulnerabilities. |
| `npm test` | PASS — 7 frontend tests, 5 Rust unit tests, 18 API/integration tests, and 2 deployment regression scripts. |
| `npm run typecheck` | PASS. |
| `npm run lint` | PASS — TypeScript, rustfmt, and Clippy with warnings denied. |
| `npm run build` | PASS — exact Vite build plus optimized Rust API; `dist/` produced. |
| `npm run test:e2e` | PASS — 25/25 Chromium tests. |
| `npm run test:cold-claim` | PASS — 195 seconds from an empty Cargo target; limit 600 seconds. |
| `npm run test:durable-restart` | PASS — a real-school booking, generated keys, and snapshot survived a local release-process restart. |
| `bash scripts/test-zero-config.sh` | PASS — release service started with the documented minimal environment. |
| Container image build | NOT RUN — Docker, Podman, Buildah, and Nerdctl are unavailable in the verifier container. |

The production build is 70.86 KB gzip initial JavaScript, 4.43 KB gzip CSS,
and 79.59 KB gzip for the staff-only lazy MSAL chunk. The cold live landing
transferred 70,739 bytes of JavaScript and 4,428 bytes of CSS. It loaded no
font or raster hero.

## Candidate/live identity

- `/health` reports the requested full SHA and ready database.
- The root HTML fetched at `/`, primary JavaScript, and CSS are byte-for-byte
  identical to the local production build by SHA-256.
- The active image tag is `93500402cf97`.
- Root HTML is `no-cache`; hashed assets are
  `public, max-age=31536000, immutable`.

The live artifact matches the source candidate. The failure is the active
runtime template and data topology.

## Functional and recovery-path evidence

- A fresh sample booking returned HTTP 201 and changed two open seats to one.
  Reset returned the class to two open seats after the expected async reload.
- Blank guardian name and malformed email were stopped with specific native
  validation messages.
- The full and cutoff pages removed the booking action. Direct POSTs returned
  HTTP 409 with `class_full` and `booking_closed` plus a next step.
- Local integration tests cover exact cutoff, idempotency, final-seat races,
  role boundaries, tenant recovery, calendar discrepancies, retention,
  waitlist order, offer acceptance, export/delete, and migration rollback.
- All crawled same-origin links returned HTTP 200. Unknown paths return a
  styled HTTP 404 with a recovery link; mail links are explicit mail actions.
- The no-SMTP durable copyable-offer fallback and configured encrypted SMTP
  outbox path both passed their claim tests.

The normal live demo flow passed before autoscaling. Once the second replica
joined, fixed-cookie concurrency reproduced cross-replica failures as detailed
in the P0 finding.

## Privacy, headers, identity, and billing

- Playwright recorded only same-origin requests through landing, demo,
  privacy, terms, and signed-out workspace flows. No analytics, advertising,
  CDN, or third-party font/script request occurred.
- The demo cookie is HttpOnly, Secure, SameSite=Strict, and has
  `Max-Age=86400`.
- CSP is delivered as a response header and includes
  `frame-ancestors 'none'`. `nosniff`, strict referrer policy, and restrictive
  permissions policy are present.
- An unapproved CORS origin received no allow header;
  `https://hello.sociobot.in` was allowed.
- Explicit staff sign-in reached only the required tenant authority
  `sociobotcustomers.ciamlogin.com/35c6fe40-0ec0-46b6-98c6-213ad4de6650`
  with client ID `25c704f4-465a-47af-80ab-2c489466b697`, the production
  callback, authorization-code flow, state, nonce, and PKCE S256.
- Explicit checkout made POST
  `https://api.sociobot.in/api/v1/products/class-capacity-truth/checkout`,
  received HTTP 200, and navigated to an HTTPS Dodo hosted session. No payment
  was attempted.
- Sixty concurrent invalid-token requests to the Sociobot product verification
  endpoint produced 30 HTTP 200 and 30 HTTP 429 responses; the observed
  admitted allowance was 30 at that instant, and 429 included
  `Retry-After: 3`.

## Product API limits and concurrency

Before scale-out, demo requests 1–10 from one forwarded client were accepted;
request 11 returned 429 with `Retry-After: 5` and
`X-RateLimit-Limit: 10`. A second forwarded client was independent. A staff
route admitted 40 invalid-bearer requests and returned 429 for requests 41–45
with `Retry-After: 1` and limit 40. A 100-request demo smoke completed with 10
accepted and 90 rate-limited.

After Azure created the second replica, one fresh client received 20 accepts
from a 30-request burst despite the advertised allowance of 10. The
per-process implementation returns correct 429 headers, but the live service
does not enforce one allowance per client. This is another direct consequence
of the P0 topology defect.

## Accessibility, responsive behavior, and performance

- `/opt/fleet/lib/verify-url.sh` passed live: HTTP 200, descriptive title,
  `lang=en`, one H1, a main landmark, no missing alt text, no unnamed buttons,
  and no console errors. Measured load was 591 ms in that smoke.
- Fresh Playwright axe scans found zero serious/critical findings on landing,
  demo booking, privacy, terms, signed-out workspace, open mobile menu, dark
  mobile landing, and the 404 page.
- At 390 px, the labelled Menu is 77.6×44.8 px, opens with Enter, closes with
  Escape, and returns focus. The skip link receives a visible 3 px focus ring
  and is 44 px high.
- Landing and demo have no horizontal overflow at 390 px; demo remains usable
  at 200% text. Reduced motion had no running infinite animation. Desktop,
  mobile, and dark screenshots were visually inspected.
- Local keyboard-only browser coverage completed a sample booking and checked
  route-heading focus. The full 25-test browser suite passed.
- Fresh live Lighthouse mobile scores were Performance 100, Accessibility
  100, Best Practices 100, and SEO 100. FCP and LCP were 1.33 s, TBT 70.5 ms,
  CLS 0, and interactive 1.48 s.

## Applicability

This is a web service, not a library/CLI or PWA. Consumer-package and service
worker update/offline checks do not apply. AI is not needed for the capacity
allocation job and is appropriately absent. The iCalendar feed, public booking
link, waitlist offer, export/delete, and optional SMTP path cover the obvious
brief leverage.

## Release decision

**FAIL.** Do not onboard or retain real school data on revision `0000040`.
Source changes are not required for the observed blocker; the candidate must be
redeployed with the checked-in one-replica durable mount contract and pass a
fresh production restart proof.
