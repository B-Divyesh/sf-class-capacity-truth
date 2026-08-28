# Class Capacity Truth — venture plan

Status: M1 remains shipped. The 2026-08-28 release repair delivered the release-blocking M2/M3 core and M4 data controls while preserving the demo. External Entra callback, Sociobot product registration, and SMTP configuration still require factory operator configuration. Every builder reads this file, .factory/design.md, the brief, and earlier milestone handoffs before changing scope.

## PRD

### Customer and situation

The paying customer is the operations lead at a small language school or tutoring centre that sells places in group level checks, assessments, or classes. Their booking calendar and real room list disagree often enough that staff keep a second spreadsheet, manually confirm bookings, and telephone families when a place reopens. Families see an unreliable number when they decide whether to book.

### Promise

Show each family the seat count the school can stand behind, then turn a cancelled place into a fair, one-click offer.

### The three jobs

1. **Set and publish trustworthy capacity.** An operator defines a class, capacity, booking cutoff, and public link. A guardian can book while seats remain; the system never creates an extra confirmed seat.
2. **Reconcile the calendar before families see a wrong count.** The operator connects one calendar and sees which imported bookings changed the local seat ledger, which records need attention, and why.
3. **Fill released seats from a consented waitlist.** A guardian joins a queue; after a cancellation, the next eligible guardian receives one expiring offer with a visible outcome for staff.

### Monetisation

The product is a Dodo-backed subscription sold only through the Sociobot billing API. The initial named paid tier is **School — $99/month per school**: one school workspace, public booking pages, calendar reconciliation, waitlist offers, and transaction logs. The public demo is free, isolated, and is not a tier. A time-limited factory pilot, if offered, has the same capabilities and an explicit end date; it never silently converts.

M2 registers recurring school-monthly-99 through the factory billing workflow. The product uses hosted Sociobot checkout, shows price, renewal cadence, cancellation path, merchant of record, and legal links before checkout, and never embeds Dodo, another payment form, or a payment secret. A multi-campus price is a new documented tier, never an unannounced per-seat charge.

### Deliberately out of scope

This is not a student information system. It will not manage grades, attendance, curriculum, student records, payroll, classroom rosters beyond a session booking, marketing campaigns, a general calendar, or tuition payments. It does not promise an all-in-one school suite. The booking flow asks only for guardian contact information; it does not require a child's full name, date of birth, or learning history.

### Success measure

For a pilot school, the target is zero incorrect public seat displays for 90 days and at least 25% of released places filled from the waitlist within 24 hours. M4 records both measures with reconciliation evidence; no public copy claims these outcomes before they are measured.

## Evidence and wedge

| Signal | What it establishes |
| --- | --- |
| [HN scheduling discussion](https://hn.algolia.com/api/v1/items/43933966), 2025-05-09 | Group bookings and reminders are paid pain points, while generic schedulers feel rigid and excessive for small businesses. |
| [Cal.com issue #28914](https://github.com/calcom/cal.diy/issues/28914), 2026-04-16 | An English language school saw one or two places available although all places were available, directly limiting level-check bookings. |

The same revenue-impacting failure appears in a general scheduling complaint and at a language school. Calendly and Cal.com have group events; large SIS products are broad and costly. The switchable wedge is narrower: a local seat ledger is the source of capacity truth, calendar data is reconciled into it with visible exceptions, and the same ledger drives a fair waitlist. A school can adopt it for one class type without replacing its scheduler.

## Architecture

### Stack and deployment

- **Web:** React 19, Vite, strict TypeScript, and hand-authored CSS tokens. React is justified by dense multi-step booking, tables, optimistic updates, and dashboard state; there is no visual framework. Public routes target under 150 KB gzip initial JavaScript; staff views load on demand.
- **Service:** Rust 2021, Axum, Tokio, SQLx, Serde, tracing, and PostgreSQL in production. Rust makes reconciliation and concurrent allocation deterministic. A SQLite database at /data/class-capacity-truth.db is the no-environment local/default runtime so the container starts on PORT=8080 with no other variables; production supplies optional DATABASE_URL and never shares SQLite across replicas.
- **Repository:** root Vite app; services/api Rust service; packages/contracts request/response schemas. The API serves /api; the deployment serves hashed dist assets and falls back to the client router. Container Apps is the intended deployment target. This repo never changes infrastructure.
- **Build:** root npm test and npm run build are mandatory. M1 adds cargo test --manifest-path services/api/Cargo.toml and cargo build --release --manifest-path services/api/Cargo.toml to CI. The service Dockerfile is multi-stage and non-root, exposes 8080, accepts BUILD_SHA=dev, GIT_SHA=dev, and SOURCE_COMMIT=dev, and never reads .git.

### Routes and titles

The History API router updates title, focuses the destination h1, and announces the page in a polite live region. The consistent header contains the wordmark home link, at most Demo, How it works, Privacy, and one contextual sign-in/action, plus a skip link.

| Route | Title | First milestone |
| --- | --- | --- |
| / | Class Capacity Truth — Show the right seat count | M1 |
| /demo?demo=1 | Demo — Class Capacity Truth | M1 |
| /book/:publicClassId | Book a class — Class Capacity Truth | M1 |
| /app | Classes — Class Capacity Truth | M2 |
| /app/classes/:classId | Class capacity — Class Capacity Truth | M2 |
| /app/reconciliation | Calendar checks — Class Capacity Truth | M3 |
| /app/waitlist | Waitlist offers — Class Capacity Truth | M3 |
| /app/settings | Settings — Class Capacity Truth | M2 |
| /app/settings/billing | Billing — Class Capacity Truth | M2 |
| /privacy | Privacy — Class Capacity Truth | M1 |
| /terms | Terms — Class Capacity Truth | M1 |
| /404 | Page not found — Class Capacity Truth | M1 |

M1 ships canonical URLs, descriptions, Open Graph/Twitter metadata, original abacus-derived 1200×630 social art, SVG favicon, apple touch icon, robots.txt, sitemap.xml, staticwebapp.config.json, security headers, and a real styled 404. The initial CSP is same-origin only. A later connection source is added only when a shipped Entra, Sociobot, calendar, or transactional-email request needs it. There are no runtime CDNs or third-party fonts/scripts.

### Tenancy and data model

Every durable record carries organization_id; repositories accept authorized organisation context rather than a browser-provided tenant ID. PostgreSQL row-level security is enabled before production data is accepted. Tests prove two organisations cannot read one another's records or public links.

| Entity | Purpose and ownership |
| --- | --- |
| organizations | One paying school workspace; owns all operational data and the entitlement. |
| users / memberships | Entra oid and owner, operator, or viewer role; no local passwords. |
| class_templates / class_sessions | Time-zone-aware capacity, cutoff, public opaque ID, and materialized session ledger. |
| seat_events | Immutable idempotent hold, confirmation, cancellation, and reconciliation adjustment events. |
| bookings | Minimum guardian name, email, consent, party size, and status; sensitive fields encrypted at rest. |
| calendar_connections / external_events | Encrypted OAuth reference, provider cursor, mapping, checksum, status, and reconciled time. |
| waitlist_entries / seat_offers | Consented ordered queue, eligibility, one-time token hash, expiry, and acceptance result. |
| deliveries | Transactional email intent and outcome; message content is not analytics. |
| entitlements | Verified Sociobot billing subscription state and audited source event. |
| audit_events | Actor, action, opaque affected IDs, reason, and timestamp for capacity changes and exports. |

Public class IDs and offer tokens are high-entropy random values and never expose sequential IDs. A booking transaction locks the session row, checks cutoff and confirmed/held availability, writes an idempotency-keyed seat event, and commits atomically. A concurrent request receives either a seat or an intelligible full result. Calendar reconciliation can flag a conflict but cannot reduce confirmed capacity below local confirmed seats; an operator resolves that exception with a recorded reason.

Guardian email/name are encrypted with a per-environment key from Key Vault in production and a CSPRNG-generated persisted key in the zero-config local runtime. Emails are normalized for matching and never logged. Defaults: incomplete booking data deleted after 30 days; completed booking contact data 90 days after class end; expired offers 30 days after expiry; audit events 24 months. M4 delivers per-organisation CSV/JSON export and verified delete. The privacy notice explains these defaults, controller/processor roles, and access/delete requests. There is no analytics SDK, pixel, or ad cookie; aggregate operational counters have no guardian identity.

There is no file upload in the initial product. A future import must use encrypted object storage, type allowlist, malware scanning, and expiry; it is not silently added to bookings.

### Auth, access, and billing

M2 uses Sociobot Microsoft Entra External ID for staff accounts only. The frontend uses @azure/msal-browser with PKCE, loginRedirect, scopes openid profile email, and sessionStorage cache. Landing, demo, public booking, and offer routes remain public. The production callback is https://class-capacity-truth.sociobot.in/auth/callback; the factory must register it on the shared SPA before release.

The API loads discovery at startup and caches issuer/JWKS for one hour. It accepts only RS256 tokens with expected audience, tenant ID, discovery issuer, valid exp/nbf, and stable oid; failure returns 401 plus WWW-Authenticate: Bearer. Defaults are factory values; ENTRA_TENANT_ID, ENTRA_TENANT_SUBDOMAIN, and ENTRA_CLIENT_ID may override them. Owners manage staff; operators edit classes; viewers inspect reconciliation. Every write is authorized server-side.

M2 registers school-monthly-99 using the factory Sociobot/Dodo subscription contract. An owner reaches hosted Sociobot checkout and returns to billing settings. The API, not the client, records verified subscription events and derives an entitlement. The factory confirms the subscription webhook/verification contract before implementation. The product contains no Dodo API, card data, checkout SDK, or payment secret. Billing failure becomes a read-only grace state with export available; it never deletes data.

### Calendar, email, jobs, and AI

M3 supports Google Calendar first through a provider adapter. It requests read-only access to the chosen calendar, encrypts refresh tokens, uses webhook/delta sync where available, and polls every five minutes as fallback. The UI tells operators exactly what is read and how to revoke it. A Microsoft 365 adapter is a future implementation of the same interface. Missing connector configuration disables that control plainly; it never prevents a zero-config service start.

A DB-backed worker in the API process uses a lease and SKIP LOCKED so only one replica reconciles or sends an offer. It:

- reconciles due calendars every five minutes and creates explicit discrepancies;
- expires holds and unclaimed offers every minute;
- creates the next eligible waitlist offer after a released seat;
- sends consented transactional email with idempotent retry;
- deletes 24-hour demo workspaces and expired retained data; and
- reconciles subscription events while preserving audit history.

Transactional email uses a factory-managed SMTP relay adapter in production and captured mail in tests. It sends booking receipts, waitlist offers, and essential notices only: no tracking pixels. Waitlist entry includes explicit email consent and a control path.

AI is deliberately absent from capacity and reconciliation. M5 may add optional **Draft offer message** only if pilots demonstrate a need. The operator previews non-identifying class details, explicitly runs it, may discard it, and has normal templates without AI. It calls only the Sociobot gateway https://api.sociobot.in/v1/responses from the server with FACTORY_SOCIOBOT_KEY, daily spend cap, IP/user limits, and canned demo/test responses. It sends no guardian name, email, or child data. No AI marketing claim is made.

### Reliability, security, and operations

Every API endpoint except /health is rate limited using the first X-Forwarded-For hop with socket-IP fallback. Baseline is 20 requests/second, burst 40, per client; public demo creation/booking is 10/minute and 30/hour per IP/class; auth is 5 attempts/15 minutes; write/billing routes are 10/minute per actor. Limits return 429 plus Retry-After; tests prove it. Bookings also have an idempotency key and row lock. CORS is deployment allowlisted, cookie-backed demo requests have CSRF protection, inputs validate at the edge, queries are parameterized, and logs redact tokens, email, authorization, and bodies.

Health returns non-secret build SHA and dependency state. Structured JSON logs have request ID, route, status, duration, and opaque organisation ID. Protected metrics measure requests/errors/latency, job lag, discrepancies, and offer conversion. Initial targets: 99.9% monthly API availability; 99% of calendar changes reconciled in 10 minutes; no unresolved public capacity discrepancy. A failing dependency shows a conservative unavailable state rather than guessing.

Production PostgreSQL receives encrypted daily backups and point-in-time recovery for 30 days. Restore is rehearsed before the first pilot. Development SQLite is a local fallback only, never described as production backup. M4 gives owners export/delete and a backup/restore runbook.

## Design system

The source-of-truth visual thesis, tokens, provenance, and component inventory are .factory/design.md and .factory/component-inventory.md. The product is a **modular classroom abacus**, not generic SaaS: every availability state is a visible, labelled bead on a rail. M1 establishes src/styles/tokens.css; builders use tokens rather than new hex values or framework defaults.

It has light and dark treatments, system font stacks with no network request, an 8px rhythm, 44px targets, 180–220ms transform/opacity motion, and instant/opacity reduced-motion behaviour. Key screens are landing/demo, parent booking, staff capacity board, calendar checks, and waitlist/settings. Empty, loading, offline, full, expired-offer, authorization, and calendar-error states are product features. At 390px rails retain text summaries, tables become labelled stacks, and fixed controls honour safe areas. Semantic landmarks, visible focus, linked labels, live results, contrast at least 4.5:1, and keyboard operation are acceptance criteria for every milestone.

## Milestones

Each is one 3–4 hour builder session and passes review → polish → PASS before the next. Every milestone keeps ?demo=1 working, updates landing copy honestly, writes .factory/handoff-mN.md, updates this status table, commits small changes, and does not replace a real integration with a mock outside the isolated demo.

| Milestone | Status | Outcome |
| --- | --- | --- |
| M1 | Shipped 2026-08-28 | Public landing and isolated capacity-booking demo prove the core seat ledger interaction. |
| M2 | Planned | A school can create real classes with Entra accounts, persistence, and paid entitlement. |
| M3 | Planned | Google Calendar reconciliation and waitlist offers complete jobs two and three. |
| M4 | Planned | Operators can run, audit, export, and notify safely. |
| M5 | Planned | Growth paths make adoption and controlled integrations easier. |

### M1 — prove capacity in one click

**Routes/screens:** /, /demo?demo=1, /book/:publicClassId, /privacy, /terms, /404. The landing order is header; plain-words first screen with **“Show the right number of class seats”**; one sentence for language schools/tutoring centres; **“Try it with sample data”**; three tested facts; live preview; three steps; privacy/non-goal explanation; honest “School plan coming soon” slot; footer. The demo opens the booking task, not another marketing page.

**Implementation:** create a rate-limited unauthenticated demo endpoint which makes a random cookie-scoped demo tenant with 24-hour TTL and fictional Bright Path Languages seed data: an eight-seat class with six confirmed, a six-of-six full class, and a class past cutoff. The parent flow shows a labelled seat rail, accepts guardian name/email and one seat, allocates through a transaction, gives confirmation, blocks full/cutoff bookings, and resets. The persistent banner is “Demo — sample data, nothing is saved” with **Reset demo** and **Start for real**. Demo requests derive tenancy only from their signed cookie and cannot select/read a real organisation. Reset destroys/reseeds only its tenant; a fresh demo never sees another visitor’s booking. .factory/demo.md is the exact sandbox contract.

**Claims:** the required M1 .factory/claims.json contains exactly sample-booking-updates-seats, full-class-blocks-booking, cutoff-blocks-booking, and demo-reset-isolated. No copy can make another reliability statement unless its claim and test are added first.

**Tests:** unit tests for capacity, cutoff/time-zone boundaries, idempotency, seed reset; API tests for cookie isolation, tenancy rejection, validation, and 429/Retry-After; Playwright tagged tests for each claim from fresh contexts, keyboard booking, 390px mobile, route focus, reduced motion, and demo outgoing requests limited to same-origin; axe on every route; concurrent booking race; 100 rps public-route smoke. Create .factory/copy-audit.md with landing sentence counts, banned-word flags, and the terminology table before handoff. Run root/API tests, browser tests, and build.

**DoD:** a cold visitor starts realistic sample data in one click and completes the available booking without an account; the count changes; full/cutoff states explain the next step; the demo has no production data path; metadata, legal pages, mobile, a11y, CSP, and performance budgets pass. Handoff includes claim evidence, Lighthouse scores, bundle size, and operator action.

### M2 — real workspace, identity, and subscription

**Routes/screens:** /auth/callback, /app, /app/classes/:classId, /app/settings, /app/settings/billing, protected class create/edit, real public /book/:publicClassId, unchanged demo.

**Implementation:** add Entra CIAM frontend/server validation; owner/operator/viewer memberships; Postgres tenancy/RLS; create/edit time-zone-aware classes, capacity, cutoff, public link, and conservative status. Real bookings use the same transaction-tested ledger, without demo crossover. Register/verify school-monthly-99, owner-only hosted checkout, entitlement, renewal/cancel UI, and read-only grace. The real link is unlisted and opaque.

**Claims to add:** real-class-publishes-seat-count, concurrent-bookings-never-oversell, school-plan-price, demo-never-reads-real-data.

**Tests:** reversible migrations and RLS two-tenant tests; valid/invalid/aud/tenant/issuer JWT fixtures; role tests; Sociobot subscription webhook/entitlement/grace fixtures; checkout-return fixture; concurrent booking race; rate limits; 100 rps protected API smoke; all M1 claims.

**DoD:** an owner signs in, creates a class, publishes its link, takes a real booking, inspects capacity, and subscribes without help. No user can access another school. Billing loss does not lose bookings or block export. The handoff confirms Entra callback registration or names it as operator action.

### M3 — reconcile one calendar and convert the waitlist

**Routes/screens:** /app/reconciliation, /app/waitlist, selected-calendar connection, public waitlist join, signed one-click offer routes.

**Implementation:** Google Calendar adapter with least-privilege selected-calendar consent, encrypted tokens, webhook/delta sync and five-minute poll fallback, mapping view, deterministic status mapping, and exception queue. Add consented waitlist join, ordered eligibility, offer after cancellation, configurable 1–48-hour expiry (default 24), signed opaque accept link, atomic claim, expiry, resend/skip reason, and audit. Demo captures mail locally and never sends it.

**Claims to add:** calendar-change-updates-reconciliation, calendar-conflict-never-lowers-confirmed-seats, released-seat-offers-next-waitlisted-family, expired-offer-releases-seat.

**Tests:** adapter fixture/delta/replay; encrypted-token redaction; job lease/retry; cancellation-to-offer integration; one-click-offer race; captured delivery; queue fairness; Playwright demo traces. External calendar/SMTP are mocked in normal tests; one credentialed staging smoke needs operator approval.

**DoD:** an operator links one calendar, understands/resolves a discrepancy, and sees a change reconciled within five minutes. A cancelled confirmed seat creates exactly one fair offer whose acceptance cannot oversell.

### M4 — operate safely and prove the result

**Routes/screens:** /app/operations, /app/settings/data, role/member management, export and deletion confirmations.

**Implementation:** operational health, mismatch alerts, retry/backoff/dead-letter resolution, audit history, owner member management, CSV/JSON export, scoped guardian/organisation deletion, retention job, redacted support logs, aggregate pilot outcome report, complete legal pages, email suppression/unsubscribe controls, and backup/restore runbook with a non-production restore drill.

**Claims to add:** owner-exports-organization-data, deletion-removes-guardian-data, waitlist-offer-delivery-is-auditable, operations-alert-shows-unresolved-discrepancy.

**Tests:** export schema and isolation; deletion/retention clock; unauthorized operations endpoints; job/retry/dead-letter; structured-log redaction; restore drill; claim/browser/a11y/mobile/performance suite; 100 rps smoke and metrics baseline.

**DoD:** an owner can answer “what happened to this seat?”, export/delete the right data, and resolve a failing job without a database. Pilot measures are observable but not marketed until measured.

### M5 — adopt and extend without becoming a SIS

**Routes/screens:** invite/share link, integration catalogue, connection status, optional template/draft offer, and installable booking-link guidance.

**Implementation:** safe shareable class-link handoff, CSV session import with dry-run (not student import), Microsoft 365 adapter only if M3 pilots need it, calendar-health nudges, and an accessible embeddable booking link. Only after pilot evidence, add the optional Sociobot-gateway message draft with no PII and a non-AI template fallback. Never add cross-site tracking.

**Claims to add:** csv-dry-run-shows-valid-and-invalid-sessions, shared-booking-link-opens-correct-class, and only if shipped, draft-offer-uses-no-guardian-data.

**Tests:** malformed CSV/property tests, import rollback, shared-link tenancy, CSP/embed, integration retry, and recorded gateway fixture proving the optional draft excludes guardian data. Retain all previous claims and the full mobile/a11y/performance suite.

**DoD:** a school can bring in sessions and share a booking entry point without copying data manually, while the product remains a narrow capacity layer. Any AI is explicit, bounded, reviewable, and usable without it.

## Risks and experiments

| Risk | Why it matters | Experiment / decision gate |
| --- | --- | --- |
| Schools do not trust a second count. | The wedge fails if staff keep the spreadsheet. | M1 usability: five operations leads inspect three seeded differences; proceed when four can identify truth and next action unaided. |
| Calendar APIs cannot represent relevant bookings. | Partial imports could make capacity less trustworthy. | M3 connects two pilot Google calendars read-only, compares 30 days with spreadsheet, documents mappings; public count stays conservative while a difference is unresolved. |
| Waitlist email does not convert. | The $99 story depends on recovered places. | M3/M4 measures claim within 24 hours against current call process; compare 24-hour and shorter offers; target 25% only after sufficient sample. |
| Guardian privacy burden is too high. | A data incident breaks trust. | M2 review verifies minimum fields, retention, export/delete, processor list, and no PII logs before real bookings; legal review selects pilot region terms. |
| Entra callback or billing plan registration blocks M2. | Shared factory configuration is outside the repo. | Request callback registration and test school-monthly-99 before M2; use fixtures until confirmed and name operator action in handoff. |
| Concurrent bookings and calendar changes race. | Oversell violates the promise. | M1/M2 run property plus 100-race tests against Postgres; M3 replays calendar events out of order. Do not market real capacity until both pass. |
| $99 is too high for one small site. | The problem may be valued but not purchased. | Price five discovery calls against saved staff time and one recovered assessment group; use transparent expiring pilots, not an unbounded free plan. |
| Staff prefer calls to automatic offers. | Automation could lower conversion. | M3 pilot compares operator-approved email with calls and keeps a “call next” queue view with human control. |
