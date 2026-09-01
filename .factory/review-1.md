# Adversarial first-read review 1 — FAIL

Reviewed 2026-09-01 at commit `4dbddf60e2b922c2ef5a0f9889ccc577c8633ac5`.
Live URL: <https://class-capacity-truth.sociobot.in>

## Verdict

**FAIL.** There are 34 findings: 1 blocking, 3 major, and 30 minor. All 22
declared claim commands pass, but PASS requires zero findings and no untested
claim. The blocking finding is a broken **Start for real** route from the demo.

The live `/health` response reports application build
`f8b545ad0efc4b1972d3f3447958b7baf5a413f6`, the immediate parent of the review
commit. The review commit adds verification documents only; the live product
files match the reviewed application source.

## Cold first read

Fresh browser contexts were used at 390 × 844 and 1440 × 900 before repository
copy was read.

- What it does: checks a class capacity against bookings and shows how many
  seats remain.
- For whom: small language schools.
- What to click first: **Try it with sample data**.

All three questions are answerable from the first screen. The determining copy
is “Show the right number of class seats”, “For small language schools”, and
“Try it with sample data”. The adjacent text “Three sample classes open next”
also states the immediate result.

## Findings

### Blocking

#### F-1-1 — “Start for real” does not reach the real-start section

- Location: persistent demo banner, **Start for real**.
- Observed: on a 390 px viewport the control changes the URL to
  `/#school-plan`, but the page remains at the product preview. The target was
  1,450.5 px below the viewport and focus moved to the off-screen home h1.
- Why this matters: the required exit from the sandbox does not show the first
  real action or put keyboard focus at that destination. This is broken routing
  on a primary demo control.
- Fix: navigate to `/app`, or wait for the home route to render before scrolling
  to and focusing `#school-plan`. Add a browser test that clicks **Start for
  real** and checks the visible target, URL, focus, and discarded demo session.

### Major

#### F-1-2 — The first screen promises a room-list comparison that the product does not provide

- Quote: “For schools whose booking calendar and room list disagree about
  places.”
- Location: landing hero.
- Observed: the workspace accepts a manually entered class capacity and an
  iCalendar URL. There is no room-list input, import, connection, or route in
  the live product or code.
- Why this matters: a first-time visitor can reasonably expect the tool to
  reconcile two sources. The implemented job is calendar bookings versus a
  capacity entered in this product.
- Fix: use “For small schools that need booking counts to match the class
  capacity they set.” Alternatively, implement and test a real room-list
  source.

#### F-1-3 — The $99 claim test does not confirm the charged price

- Quote: “The school plan costs $99 each month.”
- Location: landing facts and `.factory/claims.json` entry
  `school-plan-price`.
- Observed: the tagged test confirms the visible sentence, a POST request, and
  navigation to a mocked Dodo URL. It does not assert that the selected
  Sociobot product is billed at USD 99 monthly.
- Why this matters: a visitor can rely on the amount and billing interval, but
  the claim test only proves the copy and redirect behavior.
- Fix: test a recorded Sociobot product/checkout response that includes USD 99
  and a monthly interval, then assert the UI matches it. Otherwise change the
  claim to “Opens Sociobot checkout” and remove the price promise.

#### F-1-4 — The Entra claim test covers roles but not the claimed sign-in

- Quote: “School staff use Sociobot Entra sign-in and server-side roles.”
- Location: `.factory/claims.json` entry `staff-role-access` and README.
- Observed: `claim_staff_roles_enforce_owner_actions` exercises recorded test
  identities and role enforcement. It does not assert the Sociobot CIAM host,
  client, callback, or PKCE flow. A separate untagged live smoke does that.
- Why this matters: one claim combines two outcomes, but its one declared test
  proves only one of them.
- Fix: split this into `entra-sign-in` and `staff-role-access`. Give the sign-in
  claim a tagged browser test that confirms the CIAM host, client, callback,
  response type, and PKCE method.

### Minor

#### F-1-5 — “Same seat check” is an unlisted implementation claim

- Quote: “The sample uses the same seat check for the number and the booking
  result.”
- Location: landing product preview.
- Why this matters: the sentence promises shared implementation logic. No
  claim entry or tagged test asserts that property.
- Fix: replace it with the tested outcome, “Book a sample seat and see the open
  count change,” or add a claim and test for the shared check.

#### F-1-6 — The student-record boundary is an unlisted claim

- Quotes: “This is not a student record system.” and “The sample does not
  manage grades, attendance, tuition, or learning history.”
- Location: landing boundary section.
- Why this matters: schools can rely on this privacy and scope boundary, but
  `.factory/claims.json` does not inventory it.
- Fix: add a scope/privacy claim with a test that checks the accepted fields and
  export schema, or use one mapped sentence such as “This tool manages class
  capacity only.”

#### F-1-7 — The real 404 omits the site shell and route metadata

- Location: any unknown URL, including `/definitely-missing-review-1`.
- Observed: the HTTP 404 has one h1 and a recovery link, but no header, footer,
  Privacy link, Terms link, meta description, canonical link, Open Graph tags,
  or favicon.
- Why this matters: it does not meet the same-route skeleton and metadata
  contract as the rest of the site.
- Fix: give `404.html` the shared header/footer and complete metadata while
  retaining a real 404 response.

#### F-1-8 — The sitemap omits shipped static routes

- Location: `/sitemap.xml`.
- Observed: it lists `/`, `/demo`, `/app`, `/privacy`, and `/terms`, but omits
  `/app/reconciliation`, `/app/waitlist`, `/app/settings`,
  `/app/settings/billing`, `/app/settings/data`, and `/app/operations`.
- Why this matters: the site-structure contract requires every stable route.
- Fix: list every stable public route and keep dynamic class, offer, and auth
  callback URLs out of the sitemap.

#### F-1-9 — A claim location is inaccurate

- Quote: `calendar-poll` says its claim appears in “School calendar connection
  and privacy page”.
- Observed: the privacy page does not mention an iCalendar feed or five-minute
  checks. The sentence appears in the workspace and README.
- Why this matters: the claim inventory cannot be used to locate all published
  instances.
- Fix: change `where` to “School calendar connection and README”, or add the
  claim to the privacy page if it belongs there.

#### F-1-10 — The mobile menu button is not a result-naming verb

- Quote: “Menu”.
- Location: landing header at 390 px.
- Why this matters: the visible button names an object, not what pressing it
  does. Its accessible name is correctly “Open main menu”, so the visible copy
  should match.
- Fix: use “Open menu” and “Close menu”.

#### F-1-11 — “The product, now” is a decorative label

- Quote/location: “The product, now”, above the landing preview.
- Why this matters: it does not name the section or help a scanning visitor.
- Fix: use “Live seat preview”.

#### F-1-12 — “A narrow tool” is a decorative label

- Quote/location: “A narrow tool”, above the scope boundary.
- Why this matters: the phrase could describe many products and adds no usable
  information.
- Fix: use “What this tool stores” or “What this tool does not manage”.

#### F-1-13 — “Seat” and “place” name the same concept

- Quotes: “room list disagree about places”, “8 places − 6 booked = 2 open”,
  and “Count places before taking a booking”.
- Location: landing hero and preview.
- Why this matters: the product otherwise calls this unit a seat, including in
  its headline and terminology table.
- Fix: use “seat” everywhere: “capacity they set”, “8 seats − 6 booked = 2
  open”, and “Count seats before taking a booking”.

#### F-1-14 — The same sample class has two names

- Quotes: “Upper primary level check” and “Level check: upper primary”.
- Location: landing hero/preview and demo.
- Why this matters: the preview is presented as the real sample, so the label
  should remain stable after the click.
- Fix: use one name in the hero, preview, loading state, API seed, and tests.

#### F-1-15 — The README heading names the product, not the job

- Quote/location: `# Class Capacity Truth`.
- Why this matters: as a standalone heading it does not explain the utility.
- Fix: use “Keep class seat counts accurate” and place the product name in the
  first sentence or repository description.

#### F-1-16 — “Capacity ledger” is unexplained jargon

- Quote: “Class Capacity Truth is a capacity ledger for small language schools
  and tutoring centres.”
- Location: README opening.
- Fix: “Class Capacity Truth helps small language schools and tutoring centres
  keep class seat counts accurate.”

#### F-1-17 — The first README workflow sentence starts with an undefined format

- Quote: “Staff connect an iCalendar feed, publish parent booking links, and
  create one expiring offer when a named booking is cancelled.”
- Location: README opening.
- Fix: “Staff connect a calendar feed (iCalendar), publish guardian booking
  links, and create one timed offer after cancelling a booking.”

#### F-1-18 — “Approved school channel” is vague

- Quote: “Staff can copy its one-click URL into an approved school channel.”
- Location: README opening.
- Fix: “Staff can copy the offer link into the school’s usual email or
  messaging service.”

#### F-1-19 — “SMTP relay” is unexplained jargon

- Quote: “A configured SMTP relay queues an encrypted offer for delivery
  instead.”
- Location: README opening.
- Fix: “If email delivery is configured, the service queues an encrypted offer
  email.”

#### F-1-20 — The sign-in instruction exposes identity-platform terminology

- Quote: “Open `/app` and sign in with the shared Sociobot Microsoft Entra
  tenant.”
- Location: README, Real school workflow.
- Fix: “Open `/app` and sign in with your school’s Sociobot Microsoft account.”

#### F-1-21 — “Stable Entra identity” is unexplained jargon

- Quote: “Owners, operators, and viewers are authorized on the server by stable
  Entra identity.”
- Location: README, Real school workflow.
- Fix: “The server assigns owner, operator, or viewer permissions from each
  staff member’s Microsoft sign-in.”

#### F-1-22 — “Durable receipt” is unexplained jargon

- Quote: “Its durable receipt includes the offer URL and delivery state.”
- Location: README, Real school workflow.
- Fix: “The saved receipt shows the offer link and whether email was sent.”

#### F-1-23 — “CSPRNG” is unexplained jargon

- Quote: “A cookie-signing key is generated with a CSPRNG and persisted in the
  data directory when none is supplied.”
- Location: README, Run locally.
- Fix: “When no cookie-signing key is supplied, the service creates a secure
  random key and stores it in the data directory.”

#### F-1-24 — The architecture list is 29 words and acronym-heavy

- Quote: “Entra JWT discovery/JWKS validation, owner/operator/viewer
  authorization, encrypted contact and calendar fields, retention cleanup,
  transaction-checked bookings, encrypted offer tokens, durable delivery
  receipts, an optional email outbox, and forwarded-IP rate limits.”
- Location: README, Architecture and deployment.
- Fix: “The API validates Microsoft sign-in tokens and enforces staff roles. It
  encrypts contact and calendar fields. Transactions protect bookings. The
  server also stores offer receipts, can queue email, and limits requests by
  client IP.”

#### F-1-25 — The deployment sentence is 24 words

- Quote: “The checked-in deployment contract fixes the app at one replica and
  mounts Azure Files at `/data`; limits therefore apply once per forwarded
  client IP.”
- Location: README, Architecture and deployment.
- Fix: “The deployment contract fixes the app at one replica and mounts Azure
  Files at `/data`. Rate limits apply once per forwarded client IP.”

#### F-1-26 — The metrics-access sentence is 25 words and jargon-heavy

- Quote: “The same aggregate, no-PII data is available to an authorised school
  member at `GET /api/metrics` (or `/api/workspaces/metrics`) with their Entra
  bearer token and workspace key.”
- Location: README, Operations metrics.
- Fix: “Authorised school staff can fetch the same totals from `GET
  /api/metrics` or `GET /api/workspaces/metrics`. Requests need their Microsoft
  sign-in token and workspace key.”

#### F-1-27 — The metrics-content sentence is 23 words and jargon-heavy

- Quote: “The response is Prometheus text and contains fixed-route request,
  server-error, and latency totals plus calendar job lag, unresolved
  discrepancies, and released-seat offer conversion.”
- Location: README, Operations metrics.
- Fix: “The Prometheus response lists request counts, server errors, and
  response times. It also lists calendar delay, unresolved differences, and
  accepted seat offers.”

#### F-1-28 — The counter-lifetime sentence is 27 words and unlisted

- Quote: “The service keeps these counters in memory, so a restart starts a
  fresh operational interval; durable booking and reconciliation records
  remain the source for the workspace gauges.”
- Location: README, Operations metrics.
- Why this matters: it exceeds the sentence cap, uses “operational interval”
  and “gauges”, and promises restart behavior absent from `claims.json`.
- Fix: “A restart clears these in-memory counters. Durable booking and
  reconciliation records still supply the workspace totals.” Add a tagged
  restart test if this behavior remains published.

#### F-1-29 — The release-verification sentence is 28 words

- Quote: “Every production release uses the container work order with
  `deploy.data_dir` set to `/data`, then verifies the immutable image,
  one-replica limit, and the Azure Files `/data` mount with
  `scripts/verify-container-topology.sh`.”
- Location: README, Architecture and deployment.
- Fix: “Every release sets `deploy.data_dir` to `/data` in the container work
  order. The topology script then checks the image, one-replica limit, and
  Azure Files mount.”

#### F-1-30 — The non-root container statement is an unlisted security claim

- Quote: “One non-root container serves both the API and built web assets on
  `PORT`.”
- Location: README, Architecture and deployment.
- Why this matters: this is a concrete hardening claim without a claims entry
  or runtime UID assertion.
- Fix: add a claim that inspects the built image/runtime UID, or remove
  “non-root” from the sentence.

#### F-1-31 — The deployment script’s access boundary is an unlisted claim

- Quote: “It does not read storage credentials or modify shared
  infrastructure.”
- Location: README, Architecture and deployment.
- Why this matters: this is a security and change-scope promise not named in
  `.factory/claims.json`.
- Fix: add a tagged test that records every mocked Azure command and rejects
  credential reads or non-product mutations, or remove the promise.

#### F-1-32 — “The sample is fictional” is an unlisted privacy claim

- Quote/location: “The sample is fictional.” in the README.
- Why this matters: visitors are told sample people and data are not real, but
  no claim entry asserts the seeded names, addresses, and classes are fixtures.
- Fix: add this to the demo-isolation claim and assert the complete seed uses
  documented fixture identities and `example.org` addresses.

#### F-1-33 — The landing’s original-art statement is unlisted

- Quote: “Original abacus art drawn for this product.”
- Location: landing footer.
- Why this matters: provenance is a factual claim. It is documented in the
  design file but not represented in the claim inventory.
- Fix: either add a provenance claim backed by an asset/source audit or change
  the footer to the non-claim “Abacus visual system”.

#### F-1-34 — “Parent” and “guardian” are used for the same person

- Quotes: “publish parent booking links”, “Parents can book”, “oldest waiting
  guardian”, and “guardian details”.
- Location: README and landing instructions.
- Why this matters: the terminology table selects “guardian”, but the public
  copy alternates between two labels.
- Fix: use “guardian” throughout unless the product supports distinct parent
  and guardian roles.

## Landing copy audit

Counts use whitespace-separated words. Numeric bead labels are controls in the
illustration rather than sentences; all other landing copy, headings, labels,
and controls are included.

| Copy | Words | Flag |
| --- | ---: | --- |
| Skip to main content | 4 | — |
| Class Capacity Truth | 3 | — |
| Menu | 1 | F-1-10 |
| Close | 1 | — |
| Demo | 1 | — |
| How it works | 3 | — |
| School workspace | 2 | — |
| Privacy | 1 | — |
| Class Capacity Truth — Show the right seat count | 9 | — |
| For small language schools | 4 | — |
| Show the right number of class seats | 8 | — |
| For schools whose booking calendar and room list disagree about places. | 10 | F-1-2, F-1-13 |
| Try it with sample data | 6 | — |
| Three sample classes open next. | 5 | — |
| The demo stays separate and resets. | 6 | — |
| No advertising trackers or analytics scripts. | 6 | — |
| The school plan costs $99 each month. | 7 | F-1-3 |
| 2 seats open | 3 | — |
| Upper primary level check | 4 | F-1-14 |
| 8 places − 6 booked = 2 open | 8 | F-1-13 |
| The product, now | 3 | F-1-11 |
| Count places before taking a booking | 6 | F-1-13 |
| The sample uses the same seat check for the number and the booking result. | 14 | F-1-5 |
| Upper primary level check | 4 | F-1-14 |
| 2 open | 2 | — |
| Friday conversation group | 3 | — |
| Full | 1 | — |
| Saturday assessment | 2 | — |
| Booking closed | 2 | — |
| How the sample works | 4 | — |
| Follow one seat from open to booked | 7 | — |
| Choose a class | 3 | — |
| Compare an open class with full and closed examples. | 9 | — |
| Book one seat | 3 | — |
| Enter a fictional guardian name and example.org email. | 8 | — |
| See the count move | 4 | — |
| The class changes from two open seats to one. | 9 | — |
| A narrow tool | 3 | F-1-12 |
| It counts seats, not students | 5 | F-1-6 |
| This is not a student record system. | 7 | F-1-6 |
| The sample does not manage grades, attendance, tuition, or learning history. | 10 | F-1-6 |
| Read how sample data is handled | 6 | — |
| School workspace | 2 | — |
| Set a real class capacity | 5 | — |
| Create a persistent class, publish its booking link, compare calendar bookings, and record released-seat offers. | 14 | — |
| Open school workspace | 3 | — |
| Open the $99 monthly Sociobot checkout | 6 | F-1-3 |
| (external checkout) | 2 | — |
| Class Capacity Truth | 3 | — |
| Seat counts for small schools. | 5 | — |
| Privacy | 1 | — |
| Terms | 1 | — |
| Built by Param Factory | 4 | — |
| (external site) | 2 | — |
| Version 0.1.0 · Original abacus art drawn for this product. | 10 | F-1-33 |

No landing sentence exceeds 22 words and no banned marketing word appears.

## README copy audit

Code blocks are commands rather than sentences and are excluded. Markdown link
destinations are counted by their visible labels. Headings are included because
they must make sense out of context.

| # | Copy | Words | Flag |
| ---: | --- | ---: | --- |
| 1 | Class Capacity Truth | 3 | F-1-15 |
| 2 | Class Capacity Truth is a capacity ledger for small language schools and tutoring centres. | 14 | F-1-16 |
| 3 | Staff connect an iCalendar feed, publish parent booking links, and create one expiring offer when a named booking is cancelled. | 20 | F-1-17, F-1-34 |
| 4 | Staff can copy its one-click URL into an approved school channel. | 11 | F-1-18 |
| 5 | A configured SMTP relay queues an encrypted offer for delivery instead. | 11 | F-1-19 |
| 6 | The school plan costs $99 each month through Sociobot checkout. | 10 | F-1-3 |
| 7 | Try the deployed demo at class-capacity-truth.sociobot.in/demo?demo=1. | 6 | — |
| 8 | The sample is fictional. | 4 | F-1-32 |
| 9 | Each browser gets a signed, temporary workspace that expires after 24 hours. | 12 | — |
| 10 | Name and email input is validated but not retained. | 9 | — |
| 11 | Real school workflow | 3 | — |
| 12 | Open `/app` and sign in with the shared Sociobot Microsoft Entra tenant. | 12 | F-1-20 |
| 13 | Owners, operators, and viewers are authorized on the server by stable Entra identity. | 13 | F-1-21 |
| 14 | A workspace can be recovered on another device after sign-in. | 10 | — |
| 15 | Calendar feeds are encrypted and checked every five minutes. | 9 | F-1-9 |
| 16 | A disagreement is visible as Attention and never changes confirmed seats automatically. | 12 | — |
| 17 | Parents can book while seats remain or consent to the waitlist. | 11 | F-1-34 |
| 18 | Staff select the exact booking to cancel. | 7 | — |
| 19 | The server creates a 24-hour offer for the oldest waiting guardian. | 11 | — |
| 20 | Its durable receipt includes the offer URL and delivery state. | 10 | F-1-22 |
| 21 | Without SMTP, staff use Copy offer and send the URL through their approved channel. | 14 | F-1-18, F-1-19 |
| 22 | Owners can export or delete the workspace. | 7 | — |
| 23 | Contact fields are encrypted and scrubbed after 90 days. | 9 | — |
| 24 | Run locally | 2 | — |
| 25 | Requires Node 22+, npm 10+, and stable Rust. | 8 | — |
| 26 | Open http://localhost:8080/demo?demo=1. | 2 | — |
| 27 | The service starts with only `PORT` (and defaults to `8080`); `DATA_DIR` defaults to `/data` in the container. | 17 | — |
| 28 | A cookie-signing key is generated with a CSPRNG and persisted in the data directory when none is supplied. | 18 | F-1-23 |
| 29 | A separate contact-encryption key is generated and persisted the same way. | 11 | — |
| 30 | Optional SMTP variables are `SMTP_RELAY`, `SMTP_USERNAME`, `SMTP_PASSWORD`, and `SMTP_FROM`. | 9 | — |
| 31 | Without them, the workspace creates a durable, copyable offer and states that no email was sent. | 16 | — |
| 32 | Test and build | 3 | — |
| 33 | `npm test` runs the TypeScript unit suite and Rust tests. | 10 | — |
| 34 | The Playwright suite starts the compiled Axum service with a clean temporary database and verifies every claim in `.factory/claims.json`. | 19 | — |
| 35 | `npm run build` produces `dist/` and a release API binary. | 10 | — |
| 36 | Architecture and deployment | 3 | — |
| 37 | React 19, Vite, strict TypeScript, and hand-authored CSS for the web app. | 12 | — |
| 38 | Rust, Axum, SQLx, and SQLite for both the isolated demo and the single-instance school ledger. | 15 | — |
| 39 | Production mounts the work-order Azure Files share at `/data`; SQLite and generated keys live there. | 15 | — |
| 40 | Production is fixed at one replica. | 6 | — |
| 41 | Entra JWT discovery/JWKS validation, owner/operator/viewer authorization, encrypted contact and calendar fields, retention cleanup, transaction-checked bookings, encrypted offer tokens, durable delivery receipts, an optional email outbox, and forwarded-IP rate limits. | 29 | F-1-24 |
| 42 | One non-root container serves both the API and built web assets on `PORT`. | 13 | F-1-30 |
| 43 | The checked-in deployment contract fixes the app at one replica and mounts Azure Files at `/data`; limits therefore apply once per forwarded client IP. | 24 | F-1-25 |
| 44 | Operations metrics | 2 | — |
| 45 | Signed-in owners and operators can open `/app/operations`. | 7 | — |
| 46 | The same aggregate, no-PII data is available to an authorised school member at `GET /api/metrics` (or `/api/workspaces/metrics`) with their Entra bearer token and workspace key. | 25 | F-1-26 |
| 47 | The response is Prometheus text and contains fixed-route request, server-error, and latency totals plus calendar job lag, unresolved discrepancies, and released-seat offer conversion. | 23 | F-1-27 |
| 48 | It never contains guardian, class, school, or token values. | 9 | — |
| 49 | Treat any server error or unresolved public discrepancy as an investigation. | 11 | — |
| 50 | Check calendar connections when lag exceeds ten minutes, and review monthly API availability against the 99.9% target. | 17 | — |
| 51 | The service keeps these counters in memory, so a restart starts a fresh operational interval; durable booking and reconciliation records remain the source for the workspace gauges. | 27 | F-1-28 |
| 52 | The factory deploys the container. | 5 | — |
| 53 | This repository does not change DNS, billing, or cloud infrastructure. | 10 | — |
| 54 | See `.factory/plan.md` for the milestone architecture and `.factory/design.md` for the modular classroom abacus visual system. | 15 | — |
| 55 | Every production release uses the container work order with `deploy.data_dir` set to `/data`, then verifies the immutable image, one-replica limit, and the Azure Files `/data` mount with `scripts/verify-container-topology.sh`. | 28 | F-1-29 |
| 56 | The checked-in guarded script can apply the same product template when its image and full `EXPECTED_BUILD_SHA` are supplied. | 18 | — |
| 57 | It does not read storage credentials or modify shared infrastructure. | 10 | F-1-31 |
| 58 | Privacy and licence | 3 | — |
| 59 | The product loads no third-party fonts or scripts and sends no advertising or analytics requests. | 15 | — |
| 60 | Entra sign-in and Sociobot checkout are explicit staff actions. | 9 | — |
| 61 | See `/privacy` and the exact sandbox contract in `.factory/demo.md`. | 9 | — |
| 62 | Released source is available under the MIT License. | 8 | — |

README average sentence/heading length is 11.6 words. Six sentences exceed the
22-word hard cap. No banned marketing word appears.

## Demo and sandbox checks

- One click from the landing page reached `/demo?demo=1`.
- The first loaded demo screen showed three realistic classes: one open, one
  full, and one past its cutoff.
- “Demo — sample data, nothing is saved”, **Reset demo**, and **Start for real**
  remained visible.
- Booking changed the open class from two seats to one. Reset restored two.
- A second fresh browser context still showed two seats.
- The whole observed landing/demo/booking/reset flow contacted only
  `https://class-capacity-truth.sociobot.in`.
- The backend tests confirmed demo inputs are replaced before storage, expire
  after 24 hours, and cannot select a real organisation.
- The only failure in this area is F-1-1: the **Start for real** destination.

## Claim results

Every command was run independently from the clean review worktree after
`npm ci`. All declared commands exited 0. F-1-3 and F-1-4 are assertion-scope
gaps, not command failures.

| Claim | Result |
| --- | --- |
| `sample-booking-updates-seats` | PASS |
| `full-class-blocks-booking` | PASS |
| `cutoff-blocks-booking` | PASS |
| `demo-reset-isolated` | PASS |
| `school-capacity-flow` | PASS |
| `calendar-poll` | PASS |
| `released-seat-delivery` | PASS |
| `school-plan-price` | PASS, insufficient assertion (F-1-3) |
| `no-third-party-tracking` | PASS |
| `contact-encryption-retention` | PASS |
| `staff-role-access` | PASS, only the role half is asserted (F-1-4) |
| `data-export-delete` | PASS |
| `demo-expiry-input-disposal` | PASS |
| `reconciliation-does-not-change-seats` | PASS |
| `durable-restart` | PASS |
| `configured-smtp-delivery` | PASS |
| `workspace-recovery` | PASS |
| `oldest-waitlist-offer` | PASS |
| `zero-config-runtime` | PASS |
| `forwarded-ip-rate-limits` | PASS |
| `durable-one-replica-topology` | PASS locally and by read-only live target check |
| `operational-metrics-no-pii` | PASS |

## Earlier findings rechecked

No earlier `.factory/review-*.md` or `.factory/polish-*.md` files exist. Both
handoff files were read in full. The defects recorded in their historical FAIL
sections were rechecked as follows.

| Earlier defect | Live and code confirmation | Result |
| --- | --- | --- |
| Verification 12/13/15/16: ephemeral or multi-replica SQLite topology | Read-only `scripts/verify-container-topology.sh` confirmed exactly one `sf-class-capacity-truth` replica and its Azure Files `/data` mount. `durable-restart` passed. | Fixed |
| Verification 15: demo requests crossed replicas and lost class IDs | A cold live click, booking, reset, and second context all retained the expected seed. The one-replica check passed. | Fixed |
| Verification 15: rate allowance was multiplied | The live topology is one replica; the forwarded-IP regression test passed. | Fixed |
| Verification 15: top-level `/metrics` lacked a limiter | `/metrics`, `/api/metrics`, and `/api/workspaces/metrics` return 401 signed out. `regression_top_level_metrics_uses_forwarded_ip_limiter` passed. | Fixed |
| Verification 14: protected metrics were absent | The three live metrics paths now return 401 rather than 404. The aggregate/no-PII claim test passed. | Fixed |
| Verification 14: workspace deep links returned 404 | All seven listed `/app/...` routes returned 200 and showed route-specific titles and h1s. | Fixed |
| Verification 14: the 390 px check was flaky | The full 27-test Playwright run passed with retries disabled; the demo waits for three articles. | Fixed |
| Repair 14: skip-link focus race | The live keyboard smoke focused `main`; the full E2E check passed. | Fixed |
| Verification 16: demo CLS 0.122 | The loading state contains three data-shaped rails, and the no-growth regression passed in the no-retry suite. A fresh Lighthouse attempt crashed its browser tab, so no new Lighthouse score is claimed. | Fixed in code/test; live score not newly measured |

No historical finding was observed as regressed. F-1-1 through F-1-34 are new
findings from this full review.

## Structure, accessibility, and link checks

- Home, demo, app, all stable workspace deep links, auth callback, privacy,
  terms, booking, and offer routes had one h1, one main, `lang="en"`, a
  route-specific title, description, canonical URL, Open Graph title, favicon,
  header, footer, Privacy, and Terms.
- The real 404 returned HTTP 404 and reflowed at 390 px, but has the omissions
  in F-1-7.
- Every crawled anchor returned 200; the two `mailto:` links were treated as
  explicit exceptions.
- Playwright Axe reported zero violations of any severity on home, demo, app,
  privacy, terms, and 404 at 390 px.
- The live browser smoke found no home/demo/app console or page errors, no
  mobile overflow, a 44 px menu target, zero-duration motion under reduced
  motion, and only same-origin requests before explicit sign-in or checkout.
- The visual identity is distinct: chalkboard rails, paper panels, numbered
  abacus beads, a restrained teal/cream/yellow palette, and no generic gradient
  hero or feature-card grid.

## Missed leverage

No additional AI feature is justified. Capacity, cutoff, reconciliation, and
waitlist order are deterministic rules; model output would reduce confidence.
The brief-implied high-value additions are already present: calendar import,
workspace export/delete, released-seat offers, and cross-device workspace
recovery. No missed-leverage finding is recorded.

## Verification summary

- `npm ci`: passed, 170 packages, 0 vulnerabilities.
- All 22 `.factory/claims.json` commands: passed independently.
- `npm test`: passed (8 frontend tests, 6 Rust unit tests, 21 API/integration
  tests, and 2 deployment regressions).
- `npm run typecheck`: passed.
- `npm run lint`: passed.
- `npm run build`: passed; `dist/` produced. Initial JS is 73.85 kB gzip and
  the lazy staff/auth chunk is 79.59 kB gzip.
- `CI=1 npm run test:e2e -- --retries=0`: 27/27 passed.
- Live browser smoke: passed its own assertions; no serious/critical Axe issue,
  console error, page error, overflow, third-party request, or motion-policy
  breach was observed on its covered routes.
- Read-only production topology check: passed for
  `sf-class-capacity-truth` only.
- Fresh live Lighthouse attempt: browser tab crashed before a result; no score
  is reported and no finding relies on that run.

## What would make this perfect

Resolve F-1-1 through F-1-34, especially the demo exit and claim-test scope.
Then repeat the cold mobile/desktop read, the full copy and claim cross-check,
all 22 independent claim commands, the no-retry browser suite, live route/link
crawl, full-severity Axe scan, and a successful live Lighthouse run. PASS is
appropriate only if that rerun produces no finding.
