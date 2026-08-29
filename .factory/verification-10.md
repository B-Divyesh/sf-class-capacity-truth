# Independent verification 10 — FAIL

Verified on 2026-08-29 UTC against candidate commit
`d9f625677a1cc2ebe76670cc11365dc6340fcb29` and
<https://class-capacity-truth.sociobot.in>.

## Verdict

**FAIL — do not accept real school data.** The source candidate is healthy in a
clean local checkout and the live web assets identify and match the requested
commit. The active production Container App is nevertheless an ephemeral,
multi-replica-eligible SQLite service. A capacity ledger cannot truthfully
reconcile seats while its database and encryption keys disappear on a restart
or could diverge across replicas.

## Release-blocking defect

### P0 — live candidate has no durable storage and permits three replicas

Fresh Azure control-plane readback for the only active revision,
`sf-class-capacity-truth--0000041`, returned:

```text
image: sociobotregistry.azurecr.io/sf-class-capacity-truth:d9f625677a1c
active / latest ready revision: sf-class-capacity-truth--0000041 (100% traffic)
minReplicas/maxReplicas: 1/3
replicas currently running: 1
container env: PORT=8080 only
container volumeMounts: null
template volumes: null
```

The process startup log independently confirms the consequence:

```text
database_config="generated-default"
durable_backup="disabled"
cookie_signing_key="generated-and-persisted"
contact_encryption_key="generated-and-persisted"
```

There is no `DATA_DIR=/mnt/cct/keys`, no
`DURABLE_BACKUP_PATH=/mnt/cct/snapshots/class-capacity-truth.db`, no `cct-data`
Azure Files mount, and no one-replica limit. This violates the product's
durable-one-replica claim and the backend-service runtime contract. A restart
loses the local SQLite ledger and generated encryption/cookie keys; a second
replica can use a different ledger and keys. Either outcome can display an
incorrect seat count, lose a booking, or make protected contacts unreadable.

Required repair: deploy the exact candidate through `scripts/deploy-container.sh`,
then read back **maxReplicas=1**, the `cct-data` Azure Files mount at `/mnt/cct`,
and the two required durable environment paths. Repeat a production
revision-restart durability drill before release.

## Candidate and live identity

- Clean checkout started at the requested SHA with no modifications.
- Live `GET /health` returned HTTP 200:
  `{"status":"ok","build":"d9f625677a1cc2ebe76670cc11365dc6340fcb29","database":"ready"}`.
- Local and live SHA-256 values were identical for the entry JavaScript
  (`index-B5gvj-kr.js`) and CSS (`index-5CBMGoWt.css`).
- The document is `no-cache`; hashed assets send
  `Cache-Control: public, max-age=31536000, immutable`.

## Mandatory first-read and demo gate

**PASS.** In a fresh cold browser visit, the first screen plainly says it
shows the right number of class seats, names small language schools whose
calendar and room list disagree, and offers **Try it with sample data** with
the next result stated as “Three sample classes open next.” One click opens
`/demo?demo=1`, immediately displays realistic open/full/cutoff classes, and
shows the persistent “Demo — sample data, nothing is saved” banner with Reset
demo and Start for real.

## Claims gate

`.factory/claims.json` exists with 21 entries. After `npm ci`, every declared
command was run individually from the demo/test entry point and passed locally.
The topology fixture test passes, but it does not override the contradictory
live Azure readback above.

| Claim ID | Exact declared command | Result |
| --- | --- | --- |
| sample-booking-updates-seats | `npm run test:e2e -- --grep @claim:sample-booking-updates-seats` | PASS |
| full-class-blocks-booking | `npm run test:e2e -- --grep @claim:full-class-blocks-booking` | PASS |
| cutoff-blocks-booking | `npm run test:e2e -- --grep @claim:cutoff-blocks-booking` | PASS |
| demo-reset-isolated | `npm run test:e2e -- --grep @claim:demo-reset-isolated` | PASS |
| school-capacity-flow | `npm run test:e2e -- --grep @claim:school-capacity-flow` | PASS |
| calendar-poll | `npm run test:api -- claim_calendar_feed_is_encrypted_and_polled_every_five_minutes` | PASS |
| released-seat-delivery | `npm run test:e2e -- --grep @claim:released-seat-delivery` | PASS |
| school-plan-price | `npm run test:e2e -- --grep @claim:school-plan-price` | PASS |
| no-third-party-tracking | `npm run test:e2e -- --grep @claim:no-third-party-tracking` | PASS |
| contact-encryption-retention | `npm run test:api -- claim_contact_encryption_and_retention` | PASS |
| staff-role-access | `npm run test:api -- claim_staff_roles_enforce_owner_actions` | PASS |
| data-export-delete | `npm run test:e2e -- --grep @claim:data-export-delete` | PASS |
| demo-expiry-input-disposal | `npm run test:api -- claim_demo_expiry_and_input_disposal` | PASS |
| reconciliation-does-not-change-seats | `npm run test:api -- claim_reconciliation_never_mutates_confirmed_seats` | PASS |
| durable-restart | `npm run test:durable-restart` | PASS locally |
| configured-smtp-delivery | `npm run test:api -- claim_configured_smtp_queues_an_encrypted_offer` | PASS |
| workspace-recovery | `npm run test:api -- claim_workspace_recovery_by_staff_identity` | PASS |
| oldest-waitlist-offer | `npm run test:api -- claim_oldest_waitlist_entry_gets_a_24_hour_offer` | PASS |
| zero-config-runtime | `bash scripts/test-zero-config.sh` | PASS |
| forwarded-ip-rate-limits | `npm run test:api -- rate_limit_uses_forwarded_ip_and_returns_retry_after` | PASS locally |
| durable-one-replica-topology | `npm run test:deployment` | PASS fixture; **FAIL live** |

The local durable-restart test created a real school booking, restarted the
release process using mounted snapshot storage, and recovered both the seat
count and encrypted contact. That is sound local evidence, but production has
not supplied that storage.

## Quality, functional, and accessibility checks

- `npm test`: PASS — 7 frontend tests, 5 Rust unit tests, 18 API tests, and 2
  deployment regression scripts.
- `npm run typecheck`, `npm run lint`, and exact `npm run build`: PASS. `dist/`
  was produced and the optimized Rust binary built.
- `npm run test:e2e`: PASS — 25/25 Chromium tests.
- Live demo booking changed two open seats to one; Reset demo restored two.
  Full and cutoff sample classes correctly removed booking actions. A malformed
  email received browser validation and, after correction, booking succeeded.
- Desktop and 390 px mobile passed manual keyboard/mobile checks. The mobile
  menu is 77.56 x 44.80 px, opens with Enter, closes with Escape, restores
  focus, and the demo has no horizontal overflow under reduced motion.
- Live axe scan of the mobile demo had zero serious/critical findings. The
  supplied `verify-url.sh` passed live: descriptive title, `lang=en`, one H1,
  main landmark, no missing image alt text or unnamed buttons, and no console
  or page errors.
- Initial entry JS is 70,769 bytes gzip and CSS is 4,444 bytes gzip, within
  the static budget. The lazy staff auth chunk is not first-load.

## Privacy, auth, headers, rate limits, and concurrency

- A Playwright request log across home, demo booking/reset, privacy, and
  signed-out workspace contained only same-origin product requests. No
  advertising, analytics, third-party font, or script request occurred before
  explicit sign-in.
- Demo sessions send `HttpOnly; Secure; SameSite=Strict; Max-Age=86400` cookies.
  Response headers include CSP with `frame-ancestors 'none'`, `nosniff`, strict
  referrer policy, and restrictive permissions policy. An unapproved CORS
  origin received no allow-origin response.
- The explicit sign-in path used only the required Sociobot CIAM authority
  `sociobotcustomers.ciamlogin.com/35c6fe40-0ec0-46b6-98c6-213ad4de6650`,
  client ID `25c704f4-465a-47af-80ab-2c489466b697`, authorization-code PKCE
  (`S256`), and the production callback. No credentials were entered.
- A single forwarded client was allowed 10 demo requests in a 15-request
  burst; five received HTTP 429 with `Retry-After: 4` and
  `X-RateLimit-Limit: 10`. A 100-request/25-way concurrent smoke admitted 10
  and rate-limited 90. A school API route admitted 40 requests before a 429
  with `Retry-After`. This currently behaves as one process only; the P0
  multi-replica configuration makes that guarantee unsafe after scale-out.

## Applicability

This is a web-with-backend service, not a library, CLI, or PWA, so consumer
package and service-worker/offline-update checks do not apply. AI is not
needed for deterministic seat allocation. No product code was modified during
this verification.
