# Adversarial first-read review 2 — FAIL

Reviewed 2026-09-01 from repository commit
`b94d6cc70fd2e613f3fb8093d0b101c0fd330c19` and fresh browser contexts at
<https://class-capacity-truth.sociobot.in>. Live `/health` reported build
`b5ade8e07d3ba4f8adbe1b77fa51a40f34205938`; the commits after that build add
only factory verification documents and evidence, so the reviewed application
source matches the live application.

## Verdict

**FAIL.** There are 25 findings: 10 major and 15 minor. There are no blocking
runtime findings: all 23 declared claim commands pass, the demo works and is
isolated, and the route/accessibility checks pass. PASS still requires zero
findings and no unlisted claim. The current copy contains false test-coverage
documentation, incomplete price wording, and unlisted billing, privacy,
concurrency, availability, and infrastructure claims.

## Cold first read

Fresh contexts were opened at 390 × 844 and 1440 × 900 before scrolling.

- What it does: keeps the number of bookable class seats aligned with the
  capacity a school sets.
- For whom: small language schools.
- What to click first: **Try it with sample data**.

All three questions are answerable from the first screen. The exact copy is
“Show the right number of class seats,” “For small language schools,” and “Try
it with sample data.” The adjacent result text is less precise; F-2-13 records
that issue.

## Findings

### Major

#### F-2-1 — The README falsely says Playwright verifies every claim

- Quote/location: “The Playwright suite starts the compiled Axum service with a
  clean temporary database and verifies every claim in `.factory/claims.json`.”
  — README, **Test and build**.
- Verification: the 29-test Playwright suite covers browser claims, but
  `calendar-poll`, retention, roles, restart durability, SMTP, topology, and
  other claims run through separate Rust or shell commands. Running Playwright
  alone does not execute them.
- Why this matters: a maintainer can publish after running only the browser
  suite while believing every product promise was checked.
- Fix: use “The Playwright suite checks browser flows against a clean temporary
  database. Run every command in `.factory/claims.json` to verify all claims.”

#### F-2-2 — Public price copy omits the per-school billing unit

- Quotes/locations: “The school plan costs $99 each month.” on the landing page
  and Terms; “The school plan costs $99 each month through Sociobot checkout.”
  in the README. The signed-out workspace instead says “$99 per school each
  month,” which matches the brief.
- Why this matters: a buyer must not infer whether the price is per school, per
  location, or per staff account.
- Fix: use “The plan costs $99 per school each month.” everywhere, and update
  `school-plan-price` so its recorded fixture and assertions include the
  per-school unit.

#### F-2-3 — Merchant-of-record, cancellation, and refund promises are unlisted

- Quote/location: “Sociobot is the merchant of record and handles checkout,
  cancellation, and refunds.” — `/terms`, **School plan**.
- Verification: `school-plan-price` proves a recorded USD 99 monthly checkout
  and navigation to a hosted Dodo URL. It does not prove merchant-of-record
  status, cancellation, or refunds.
- Why this matters: these are contractual billing promises.
- Fix: add separately testable claims against recorded Sociobot billing
  responses for the merchant, cancellation, and refund paths. If those paths
  cannot be proved, replace the sentence with “Checkout opens on Sociobot” and
  link the applicable billing terms.

#### F-2-4 — The privacy page makes unlisted legal-role claims

- Quote/location: “The school is the data controller. Sociobot processes
  guardian names and email addresses to manage seats and create requested
  offers.” — `/privacy`, **School booking data**.
- Verification: no claims entry identifies or tests the controller/processor
  relationship. The contact and school-flow claims test storage and product
  behavior, not the stated legal roles.
- Why this matters: schools may rely on this sentence when assigning privacy
  responsibilities.
- Fix: confirm the legal allocation and record it in a testable contract check,
  or remove the role labels and describe only the tested data flow.

#### F-2-5 — The email-relay disclosure promises more than its test proves

- Quote/location: “A configured email relay receives offer contact data only
  when the school enables email delivery.” — `/privacy`, **Who receives it**.
- Verification: `configured-smtp-delivery` proves only that an encrypted
  pending outbox row is created when email is configured. It does not exercise
  an SMTP receiver or prove the “only when” negative path.
- Why this matters: this is a data-recipient and consent-boundary promise.
- Fix: test a local SMTP receiver in configured and unconfigured modes, then
  add the exact claim. Otherwise use the tested sentence: “When email delivery
  is configured, the service queues an encrypted offer email.”

#### F-2-6 — “Transactions protect bookings” is an unlisted core claim

- Quote/location: “Transactions protect bookings.” — README, **Architecture
  and deployment**.
- Verification: `capacity_cutoff_idempotency_reset_and_concurrent_race` does
  exercise a two-request last-seat race, but it is not listed in
  `.factory/claims.json` and is not one of the declared claim commands.
- Why this matters: oversell protection is the product’s core reason to exist;
  a vague implementation assertion is not an inventoried outcome.
- Fix: rewrite as “Concurrent booking requests cannot oversell a class,” add a
  `concurrent-booking-does-not-oversell` claim, and map the existing race test
  to that claim.

#### F-2-7 — The 99.9% API target is an unlisted quantitative claim

- Quote/location: “Check calendar connections when lag exceeds ten minutes,
  and review monthly API availability against the 99.9% target.” — README,
  **Operations metrics**. The signed-in operations page also says “Review
  monthly availability against the 99.9% API target.”
- Verification: no claim records or measures monthly availability, and the
  metrics endpoint does not expose monthly availability.
- Why this matters: a numeric service target can be read as an operational
  commitment even though the product cannot calculate or verify it.
- Fix: remove “99.9%” until an externally measured SLO and claim test exist.
  The immediate rewrite is “Review API availability each month.”

#### F-2-8 — The third-party asset promise is absent from the claim inventory

- Quote/location: “The product loads no third-party fonts or scripts and sends
  no advertising or analytics requests.” — README, **Privacy and licence**.
- Verification: `no-third-party-tracking` inventories only the advertising and
  analytics sentence, although its same-origin request log currently also
  supports the font/script observation.
- Why this matters: the first half is a separate supply-chain and privacy
  promise that the claim inventory does not name.
- Fix: expand the claim to “Public and demo pages load no third-party fonts,
  scripts, advertising trackers, or analytics,” list both locations, and keep
  the existing whole-flow request assertion.

#### F-2-9 — “Explicit staff actions” is an unlisted privacy boundary

- Quote/location: “Entra sign-in and Sociobot checkout are explicit staff
  actions.” — README, **Privacy and licence**.
- Verification: sign-in and checkout have separate functional claims, but no
  claims entry says third-party navigation occurs only after an explicit staff
  action.
- Why this matters: the sentence asks readers to rely on when data can leave the
  product origin.
- Fix: add this boundary to `no-third-party-tracking` and test that neither
  external origin is requested before its named button is activated. Use
  “Microsoft sign-in” in the rewrite.

#### F-2-10 — The repository’s infrastructure-scope guarantee is unlisted

- Quote/location: “This repository does not change DNS, billing, or cloud
  infrastructure.” — README, after **Operations metrics**.
- Verification: deployment fixtures verify the owned Container App shape, but
  no claims entry inventories this broad negative statement or rejects every
  DNS, billing, and out-of-scope cloud command.
- Why this matters: this is a security and change-scope guarantee, not a casual
  implementation detail.
- Fix: add a claim whose command records every mocked cloud operation and fails
  on DNS, billing, or non-product targets, or remove the sentence from public
  product documentation.

### Minor

#### F-2-11 — Demo destruction on “Start for real” is tested but unlisted

- Quote/location: “Start for real removes that browser’s demo.” — `/privacy`,
  **How long it stays**.
- Verification: the untagged “Start for real discards demo data…” Playwright
  test passes, but `demo-reset-isolated` mentions only separation and reset.
- Why this matters: the privacy promise cannot be found by auditing
  `.factory/claims.json`.
- Fix: extend `demo-reset-isolated` to say that **Start for real** destroys the
  current demo, and keep the existing exit assertion under that claim tag.

#### F-2-12 — The landing page narrows the audience inconsistently

- Quotes/locations: “For small language schools” and “For small schools…” on
  the first screen; the README and brief include “tutoring centres.”
- Why this matters: a tutoring centre can conclude that the product is not for
  it during the five-second first read.
- Fix: use eyebrow “For language schools and tutoring centres” and lede “Match
  booking counts to the class capacity your staff set.”

#### F-2-13 — “Three sample classes open next” is ambiguous

- Quote/location: “Three sample classes open next.” — beside the primary
  landing action.
- Why this matters: “open” also means available in this product, while the next
  screen intentionally contains one open, one full, and one closed class.
- Fix: “See three sample classes next.”

#### F-2-14 — The footer ends with a decorative design label

- Quote/location: “Version 0.1.0 · Abacus visual system.” — landing footer.
- Why this matters: “Abacus visual system” gives a visitor no product, privacy,
  price, or action information. It is brand-lore copy under the plain-words
  rule.
- Fix: keep “Version 0.1.0.” and remove the decorative phrase.

#### F-2-15 — “Persistent class” is implementation language

- Quote/location: “Create a persistent class, publish its booking link,
  compare calendar bookings, and record released-seat offers.” — landing,
  **School workspace**.
- Why this matters: “persistent” describes storage behavior rather than a task
  the school performs.
- Fix: “Create a class, publish its booking link, compare calendar bookings,
  and record released-seat offers.”

#### F-2-16 — “Signed, temporary workspace” is unexplained jargon

- Quote/location: “Each browser gets a signed, temporary workspace that
  expires after 24 hours.” — README opening.
- Why this matters: “signed” does not tell a visitor what is signed or what the
  signature changes.
- Fix: “Each browser gets its own temporary workspace for 24 hours.”

#### F-2-17 — The README has a subject–verb error

- Quote/location: “Name and email input is validated but not retained.” —
  README opening.
- Why this matters: the sentence reads as unfinished or machine-written.
- Fix: “The demo checks each name and email, then discards both.”

#### F-2-18 — The SMTP settings sentence uses an unexplained acronym

- Quote/location: “Optional SMTP variables are `SMTP_RELAY`, `SMTP_USERNAME`,
  `SMTP_PASSWORD`, and `SMTP_FROM`.” — README, **Run locally**.
- Why this matters: the settings are listed before their purpose is stated in
  that paragraph.
- Fix: “These optional settings configure email delivery: `SMTP_RELAY`,
  `SMTP_USERNAME`, `SMTP_PASSWORD`, and `SMTP_FROM`.”

#### F-2-19 — “Durable, copyable offer” is implementation jargon

- Quote/location: “Without them, the workspace creates a durable, copyable
  offer and states that no email was sent.” — README, **Run locally**.
- Why this matters: “durable” does not tell staff what they can do or what will
  remain after a reload.
- Fix: “Without them, staff can copy the saved offer URL and see that no email
  was sent.”

#### F-2-20 — The test instructions needlessly expose Axum

- Quote/location: “The Playwright suite starts the compiled Axum service…” —
  README, **Test and build**.
- Why this matters: the framework name does not help someone choose or run the
  test command.
- Fix: say “compiled API service.” The framework already belongs in the
  architecture list.

#### F-2-21 — “Single-instance school ledger” is unexplained and inconsistent

- Quote/location: “Rust, Axum, SQLx, and SQLite for both the isolated demo and
  the single-instance school ledger.” — README, **Architecture and
  deployment**.
- Why this matters: the rest of the product calls this a “school workspace,”
  not a ledger.
- Fix: “Rust, Axum, SQLx, and SQLite store the isolated demo and school
  workspaces.”

#### F-2-22 — “Prometheus response” is unexplained jargon

- Quote/location: “The Prometheus response lists request counts, server
  errors, and response times.” — README, **Operations metrics**.
- Why this matters: the useful fact is the content; the unexplained format name
  interrupts it.
- Fix: “The metrics response lists request counts, server errors, and response
  times.”

#### F-2-23 — A deployment sentence exceeds the 22-word limit

- Quote/location (26 words): “Build and deploy the checked-out commit with its
  full identity; the deployment command refuses an unbound tag and fails unless
  live `/health` returns the same SHA.” — README, **Architecture and
  deployment**.
- Why this matters: it combines the operator action, tag guard, and health
  guard in one sentence.
- Fix: “Build and deploy the checked-out commit with its full commit ID. The
  deployment command rejects unbound tags and requires `/health` to return that
  ID.”

#### F-2-24 — “SHA” is unexplained jargon in the same deployment sentence

- Quote/location: “…unless live `/health` returns the same SHA.” — README,
  **Architecture and deployment**.
- Why this matters: the document otherwise talks about a commit and does not
  define SHA.
- Fix: use “full commit ID,” as in the F-2-23 rewrite.

#### F-2-25 — The README reintroduces the Entra term

- Quote/location: “Entra sign-in and Sociobot checkout are explicit staff
  actions.” — README, **Privacy and licence**.
- Why this matters: the workflow consistently says “Microsoft sign-in,” the
  plain term introduced after review 1. This sentence switches names for the
  same action.
- Fix: “Microsoft sign-in and Sociobot checkout open only after a staff member
  selects them.” Also inventory that privacy boundary as required by F-2-9.

## Landing copy audit

Counts are whitespace-separated. The table includes headings, labels, actions,
and footer copy so context-free headings and button labels are audited as well
as grammatical sentences. Decorative bead numerals are excluded.

| Copy | Words | Result |
| --- | ---: | --- |
| Skip to main content | 4 | Pass |
| Class Capacity Truth | 3 | Pass |
| Open menu | 2 | Pass; result-naming control |
| Demo | 1 | Pass |
| How it works | 3 | Pass |
| School workspace | 2 | Pass |
| Privacy | 1 | Pass |
| For small language schools | 4 | F-2-12 |
| Show the right number of class seats | 7 | Pass |
| For small schools that need booking counts to match the class capacity they set. | 14 | F-2-12 |
| Try it with sample data | 5 | Pass; result-naming action |
| Three sample classes open next. | 5 | F-2-13 |
| The demo stays separate and resets. | 6 | Pass; `demo-reset-isolated` |
| No advertising trackers or analytics scripts. | 6 | Pass; `no-third-party-tracking` |
| The school plan costs $99 each month. | 7 | F-2-2 |
| 2 seats open | 3 | Pass |
| Level check: upper primary | 4 | Pass |
| 8 seats − 6 booked = 2 open | 8 | Pass |
| Live seat preview | 3 | Pass |
| Count seats before taking a booking | 6 | Pass |
| Book a sample seat and see the open count change. | 10 | Pass; `sample-booking-updates-seats` |
| Level check: upper primary | 4 | Pass |
| 2 open | 2 | Pass |
| Friday conversation group | 3 | Pass |
| Full | 1 | Pass; `full-class-blocks-booking` |
| Saturday assessment | 2 | Pass |
| Booking closed | 2 | Pass; `cutoff-blocks-booking` |
| How the sample works | 4 | Pass |
| Follow one seat from open to booked | 7 | Pass |
| Choose a class | 3 | Pass |
| Compare an open class with full and closed examples. | 9 | Pass |
| Book one seat | 3 | Pass |
| Enter a sample guardian name and example.org email. | 8 | Pass |
| See the count move | 4 | Pass |
| The class changes from two open seats to one. | 9 | Pass; `sample-booking-updates-seats` |
| Class capacity only | 3 | Pass |
| Keep school records in your existing system | 7 | Pass |
| Use your existing school records for grades, attendance, tuition, and learning history. | 12 | Pass |
| Read how sample data is handled | 6 | Pass |
| School workspace | 2 | Pass |
| Set a real class capacity | 5 | Pass |
| Create a persistent class, publish its booking link, compare calendar bookings, and record released-seat offers. | 15 | F-2-15 |
| Open school workspace | 3 | Pass; result-naming action |
| Open Sociobot checkout | 3 | Pass; result-naming action |
| Class Capacity Truth | 3 | Pass |
| Seat counts for small schools. | 5 | Pass |
| Privacy | 1 | Pass |
| Terms | 1 | Pass |
| Built by Param Factory (external site) | 6 | Pass |
| Version 0.1.0 · Abacus visual system. | 6 | F-2-14 |

No landing sentence exceeds 22 words and no banned marketing word appears.

## README copy audit

Code blocks are commands rather than sentences and are excluded. Headings are
included because they must make sense out of context. Markdown link labels and
literal paths count as one whitespace-separated word.

| Copy | Words | Result |
| --- | ---: | --- |
| Keep class seat counts accurate | 5 | Pass |
| Class Capacity Truth helps small language schools and tutoring centres keep class seat counts accurate. | 15 | Pass |
| Staff connect a calendar feed (iCalendar), publish guardian booking links, and create one timed offer after cancelling a booking. | 19 | Pass |
| Staff can copy the offer link into the school’s usual email or messaging service. | 14 | Pass |
| If email delivery is configured, the service queues an encrypted offer email. | 12 | Pass; `configured-smtp-delivery` |
| The school plan costs $99 each month through Sociobot checkout. | 10 | F-2-2 |
| Try the deployed demo at class-capacity-truth.sociobot.in/demo?demo=1. | 6 | Pass |
| Each browser gets a signed, temporary workspace that expires after 24 hours. | 12 | F-2-16 |
| Name and email input is validated but not retained. | 9 | F-2-17 |
| Real school workflow | 3 | Pass |
| Open /app and sign in with your school’s Sociobot Microsoft account. | 11 | Pass; `entra-sign-in` |
| The server assigns owner, operator, or viewer permissions from each staff member’s Microsoft sign-in. | 14 | Pass; `staff-role-access` |
| A workspace can be recovered on another device after sign-in. | 10 | Pass; `workspace-recovery` |
| Calendar feeds are encrypted and checked every five minutes. | 9 | Pass; `calendar-poll` |
| A disagreement is visible as Attention and never changes confirmed seats automatically. | 12 | Pass; `reconciliation-does-not-change-seats` |
| Guardians can book while seats remain or consent to the waitlist. | 11 | Pass; `school-capacity-flow` |
| Staff select the exact booking to cancel. | 7 | Pass; `school-capacity-flow` |
| The server creates a 24-hour offer for the oldest waiting guardian. | 11 | Pass; `oldest-waitlist-offer` |
| The saved receipt shows the offer link and whether email was sent. | 12 | Pass; `released-seat-delivery` |
| Without email delivery, staff use Copy offer and send the URL through the school’s usual email or messaging service. | 19 | Pass; `released-seat-delivery` |
| Owners can export or delete the workspace. | 7 | Pass; `data-export-delete` |
| Contact fields are encrypted and scrubbed after 90 days. | 9 | Pass; `contact-encryption-retention` |
| Run locally | 2 | Pass |
| Requires Node 22+, npm 10+, and stable Rust. | 8 | Pass |
| Open http://localhost:8080/demo?demo=1. | 2 | Pass |
| The service starts with only PORT (and defaults to 8080); DATA_DIR defaults to /data in the container. | 17 | Pass; `zero-config-runtime` |
| When no cookie-signing key is supplied, the service creates a secure random key and stores it in the data directory. | 20 | Pass; `zero-config-runtime` |
| A separate contact-encryption key is generated and persisted the same way. | 11 | Pass; `zero-config-runtime` |
| Optional SMTP variables are SMTP_RELAY, SMTP_USERNAME, SMTP_PASSWORD, and SMTP_FROM. | 9 | F-2-18 |
| Without them, the workspace creates a durable, copyable offer and states that no email was sent. | 16 | F-2-19 |
| Test and build | 3 | Pass |
| npm test runs the TypeScript unit suite and Rust tests. | 10 | Pass |
| The Playwright suite starts the compiled Axum service with a clean temporary database and verifies every claim in .factory/claims.json. | 19 | F-2-1, F-2-20 |
| npm run build produces dist/ and a release API binary. | 10 | Pass; verified in this review |
| Architecture and deployment | 3 | Pass |
| React 19, Vite, strict TypeScript, and hand-authored CSS for the web app. | 12 | Pass; technology inventory |
| Rust, Axum, SQLx, and SQLite for both the isolated demo and the single-instance school ledger. | 15 | F-2-21 |
| Production mounts the work-order Azure Files share at /data; SQLite and generated keys live there. | 15 | Pass; `durable-one-replica-topology` and `durable-restart` |
| Production is fixed at one replica. | 6 | Pass; `durable-one-replica-topology` |
| The API validates Microsoft sign-in tokens and enforces staff roles. | 10 | Pass; `entra-sign-in` and `staff-role-access` |
| It encrypts contact and calendar fields. | 6 | Pass; `contact-encryption-retention` and `calendar-poll` |
| Transactions protect bookings. | 3 | F-2-6 |
| The server also stores offer receipts, can queue email, and limits requests by client IP. | 15 | Pass; `released-seat-delivery`, `configured-smtp-delivery`, and `forwarded-ip-rate-limits` |
| One container serves both the API and built web assets on PORT. | 12 | Pass; runtime architecture |
| The deployment contract fixes the app at one replica and mounts Azure Files at /data. | 15 | Pass; `durable-one-replica-topology` |
| Rate limits apply once per forwarded client IP. | 8 | Pass; `forwarded-ip-rate-limits` |
| Operations metrics | 2 | Pass |
| Signed-in owners and operators can open /app/operations. | 7 | Pass; `operational-metrics-no-pii` |
| Authorised school staff can fetch the same totals from GET /api/metrics or GET /api/workspaces/metrics. | 14 | Pass; `operational-metrics-no-pii` |
| Requests need their Microsoft sign-in token and workspace key. | 9 | Pass; `operational-metrics-no-pii` |
| The Prometheus response lists request counts, server errors, and response times. | 11 | F-2-22 |
| It also lists calendar delay, unresolved differences, and accepted seat offers. | 11 | Pass; `operational-metrics-no-pii` |
| It never contains guardian, class, school, or token values. | 9 | Pass; `operational-metrics-no-pii` |
| Treat any server error or unresolved public discrepancy as an investigation. | 11 | Pass; instruction |
| Check calendar connections when lag exceeds ten minutes, and review monthly API availability against the 99.9% target. | 17 | F-2-7 |
| The factory deploys the container. | 5 | Pass |
| This repository does not change DNS, billing, or cloud infrastructure. | 10 | F-2-10 |
| See .factory/plan.md for the milestone architecture and .factory/design.md for the modular classroom abacus visual system. | 15 | Pass |
| Every release sets deploy.data_dir to /data in the container work order. | 11 | Pass; `durable-one-replica-topology` |
| The topology script then checks the image, one-replica limit, and Azure Files mount. | 13 | Pass; `durable-one-replica-topology` |
| Build and deploy the checked-out commit with its full identity; the deployment command refuses an unbound tag and fails unless live /health returns the same SHA. | 26 | F-2-23, F-2-24 |
| Privacy and licence | 3 | Pass |
| The product loads no third-party fonts or scripts and sends no advertising or analytics requests. | 15 | F-2-8 |
| Entra sign-in and Sociobot checkout are explicit staff actions. | 9 | F-2-9, F-2-25 |
| See /privacy and the exact sandbox contract in .factory/demo.md. | 9 | Pass |
| Released source is available under the MIT License. | 8 | Pass |

No other README sentence exceeds 22 words and no banned marketing adjective
appears.

## Declared claims

Every command was run independently from the clean repository state after
`npm ci`. All 23 passed.

| Claim | Command | Result |
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
| `entra-sign-in` | `npm run test:e2e -- --grep @claim:entra-sign-in` | PASS |
| `staff-role-access` | `npm run test:api -- claim_staff_roles_enforce_owner_actions` | PASS |
| `data-export-delete` | `npm run test:e2e -- --grep @claim:data-export-delete` | PASS |
| `demo-expiry-input-disposal` | `npm run test:api -- claim_demo_expiry_and_input_disposal` | PASS |
| `reconciliation-does-not-change-seats` | `npm run test:api -- claim_reconciliation_never_mutates_confirmed_seats` | PASS |
| `durable-restart` | `npm run test:durable-restart` | PASS |
| `configured-smtp-delivery` | `npm run test:api -- claim_configured_smtp_queues_an_encrypted_offer` | PASS |
| `workspace-recovery` | `npm run test:api -- claim_workspace_recovery_by_staff_identity` | PASS |
| `oldest-waitlist-offer` | `npm run test:api -- claim_oldest_waitlist_entry_gets_a_24_hour_offer` | PASS |
| `zero-config-runtime` | `bash scripts/test-zero-config.sh` | PASS |
| `forwarded-ip-rate-limits` | `npm run test:api -- rate_limit_uses_forwarded_ip_and_returns_retry_after` | PASS |
| `durable-one-replica-topology` | `npm run test:deployment` | PASS |
| `operational-metrics-no-pii` | `npm run test:api -- regression_protected_operational_metrics_are_aggregated_and_contain_no_pii` | PASS |

F-2-3 through F-2-11 are claim-like sentences that are absent from this
inventory or materially broader than the listed claim. They remain untested
claims even though every listed command passes.

## Demo and sandbox behavior

- The first-screen action opens `/demo?demo=1` in one click.
- The first demo screen already shows three realistic Bright Path Languages
  classes: two open seats, a full class, and a class past its cutoff.
- The persistent banner says “Demo — sample data, nothing is saved” and exposes
  **Reset demo** and **Start for real**.
- A live booking changed the available sample from two open seats to one. Reset
  restored two. A second fresh browser context also showed two, so one demo did
  not alter another.
- **Start for real** destroyed the demo session, reached `/app`, and focused the
  visible “Sign in to manage class capacity” h1 at 390 px.
- The request log for landing, demo, booking, reset, privacy, terms, and signed-
  out workspace contained only
  `https://class-capacity-truth.sociobot.in` before an explicit sign-in or
  checkout action.
- Code inspection confirms separate `demo_tenants`/`class_sessions`/`bookings`
  tables and `workspaces`/`real_classes`/`real_bookings` tables. Demo booking
  input is replaced by `[demo input not retained]`; reset and exit delete only
  the cookie-derived demo tenant. No organisation parameter is accepted.
- The site registers no service worker and makes no offline claim. Offline
  reload was unavailable, consistent with the published scope.

## Structure, accessibility, and links

- `/`, `/demo`, `/privacy`, `/terms`, `/app`, every stable `/app/*` route, and
  the direct 404 have route-specific titles, descriptions, canonicals, Open
  Graph/Twitter metadata, favicon, one h1, `lang=en`, and the shared
  header/footer with Privacy and Terms.
- The title patterns are correct. The home title is “Class Capacity Truth —
  Show the right seat count”; utility routes use “Route — Class Capacity
  Truth.”
- Direct deep links returned 200. An unknown URL returned an HTTP 404 with the
  abacus shell and a recovery link. `sitemap.xml` lists every stable route.
- Forward navigation and browser Back both focused the new visible h1. The skip
  link, mobile menu keyboard behavior, 44 px menu target, 390 px reflow, 200%
  text checks, and reduced-motion checks passed.
- Live Playwright Axe checks found zero serious or critical issues on home,
  demo, privacy, terms, app, and the 404. The live verifier found no console or
  page errors, no missing image alt, and no unlabelled button.
- All discovered same-origin links returned 200 except the intentional current
  unknown URL, which returned 404. Dynamic sample booking links returned 200;
  the external Param Factory link returned 200; `mailto:` links are explicit.
- Initial JavaScript transferred 73,621 bytes compressed. The full production
  build reports 73.80 kB gzip initial JavaScript and 79.59 kB gzip lazy
  JavaScript.
- The modular classroom-abacus rails, paper panels, palette, typography, and
  404 are visibly product-specific rather than a generic SaaS template.

## Earlier review and handoff verification

Every earlier review, polish file, `handoff-m1.md`, and the cumulative
`handoff.md` was read. Each review-1 identifier was checked in live behavior
and source, not accepted from the polish status alone.

| Earlier finding | Current verification | Status |
| --- | --- | --- |
| F-1-1 | Live mobile **Start for real** reaches `/app`; the final h1 is visible and focused; returning to demo reseeds two seats. | Fixed |
| F-1-2 | Hero now says booking counts match the capacity the school sets; no room-list promise remains. | Fixed |
| F-1-3 | The recorded checkout fixture contains USD, 9900 cents, and monthly interval; its declared test passes. | Fixed |
| F-1-4 | `entra-sign-in` and `staff-role-access` are separate entries and both commands pass. | Fixed |
| F-1-5 | Landing now promises the observable count change, covered by `sample-booking-updates-seats`. | Fixed |
| F-1-6 | The student-record-system claim is gone; the page directs schools to keep other records in their existing system. | Fixed |
| F-1-7 | The live HTTP 404 has the shared shell, legal links, description, canonical, OG/Twitter tags, and favicon. | Fixed |
| F-1-8 | The live sitemap lists all stable workspace routes. | Fixed |
| F-1-9 | `calendar-poll.where` now says “School calendar connection and README.” | Fixed |
| F-1-10 | The visible mobile control says **Open menu**/**Close menu** and its accessible name matches. | Fixed |
| F-1-11 | “The product, now” is replaced by “Live seat preview.” | Fixed |
| F-1-12 | “A narrow tool” is replaced by “Class capacity only.” | Fixed |
| F-1-13 | Landing capacity copy consistently uses “seat”; no “place” remains. | Fixed |
| F-1-14 | “Level check: upper primary” is stable across hero, preview, demo, source seed, and tests. | Fixed |
| F-1-15 | README h1 is “Keep class seat counts accurate.” | Fixed |
| F-1-16 | The README opening no longer calls the product a capacity ledger. | Fixed |
| F-1-17 | The opening defines “calendar feed (iCalendar).” | Fixed |
| F-1-18 | The README now names the school’s usual email or messaging service. | Fixed |
| F-1-19 | Opening copy says configured email delivery queues an encrypted offer email. | Fixed |
| F-1-20 | The workflow instruction says “Sociobot Microsoft account.” | Fixed |
| F-1-21 | Role copy now explains permissions from Microsoft sign-in. | Fixed |
| F-1-22 | The workflow calls it a saved receipt and explains what it shows. | Fixed |
| F-1-23 | The README says a secure random key is created and stored. | Fixed |
| F-1-24 | The former 29-word architecture list is split into short sentences. | Fixed |
| F-1-25 | Replica/mount and forwarded-IP statements are separate sentences. | Fixed |
| F-1-26 | Metrics access is split into endpoint and credential sentences. | Fixed |
| F-1-27 | Metrics content is split into two short sentences. | Fixed |
| F-1-28 | The unlisted in-memory counter-lifetime promise is absent. | Fixed |
| F-1-29 | Release configuration and topology checks are separate sentences. | Fixed |
| F-1-30 | The untested “non-root” public statement is absent. | Fixed |
| F-1-31 | The public deployment-script claim about credentials/shared infrastructure is absent. F-2-10 concerns a different, broader repository statement. | Fixed |
| F-1-32 | “The sample is fictional” is absent. | Fixed |
| F-1-33 | The unlisted original-art provenance claim is absent. F-2-14 separately flags its decorative replacement. | Fixed |
| F-1-34 | Visitor-facing copy consistently uses “guardian.” | Fixed |

The M1 handoff’s deferred staff, durable workspace, calendar, billing, and
waitlist work is present. Historical handoff blockers for protected metrics,
deep links, demo timing, durable one-replica storage, and exact deployed build
identity were also rechecked through live routes, the 29-test suite, deployment
fixtures, restart claim, and `/health`; none has regressed.

## Additional verification

- `npm ci` — PASS, 170 packages, zero reported vulnerabilities.
- `npm test` — PASS: 8 frontend tests, 6 Rust unit tests, 21 Rust API tests,
  and both deployment regressions.
- `CI=1 npm run test:e2e -- --retries=0 --reporter=line` — PASS, 29/29.
- `npm run build` — PASS; `dist/` and the release API binary were produced.
- `/opt/fleet/lib/verify-url.sh` — PASS after supplying a fresh evidence
  directory; load 681 ms, no console errors, one h1, `lang=en`, main landmark,
  image alts present, and labelled buttons.

## Missed leverage

No missed-leverage finding is added. The brief calls for calendar comparison,
public booking, waitlist conversion, and data control; those flows exist. JSON
export exists. An AI step would add uncertainty to a seat ledger and is not an
obvious user expectation, so the Sociobot model gateway is not warranted here.

## What would make this perfect

Resolve all 25 findings, especially the false test instruction and every
unlisted legal/privacy/operational promise. Then rerun every declared claim
command, the full browser suite, the live demo isolation flow, and this entire
copy audit from a fresh context. A perfect next review has no rewritten copy
left to suggest and no sentence broader than its named claim test.
