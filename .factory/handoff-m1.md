# M1 handoff — isolated capacity booking

Shipped 2026-08-28 for work order `venture-class-capacity-truth-m1`.

## What shipped

- A public landing page with the modular classroom-abacus visual system, plain
  first-screen copy, live seat preview, product boundaries, legal routes, a
  styled 404, route titles, canonical metadata, and original SVG identity art.
- A one-click Bright Path Languages demo with open, full, and past-cutoff
  classes. A guardian can complete an available sample booking and see the
  count change from two open seats to one.
- A Rust/Axum service backed by SQLite and SQLx migrations. Demo workspaces use
  signed HttpOnly cookies, random tenant IDs, a 24-hour expiry, parameterized
  queries, transactional allocation, idempotency keys, and destructive reset.
  Demo name and email input is validated but replaced before database storage.
- The M1 Container App is pinned to one replica because its temporary SQLite
  ledger and generated signing key are replica-local. Do not raise this limit
  before M2 moves real workspaces to PostgreSQL and shared secret storage.
- Forwarded-IP rate limits on every API route except health. Excess requests
  receive `429` and `Retry-After`; the 100-request smoke accepted 10 requests
  and limited 90.
- Responsive 390 px layouts, keyboard operation, route focus and announcements,
  reduced-motion behavior, visible focus, and tested light and dark contrast.
- The four M1 claims in `.factory/claims.json`, each with exactly one tagged
  Playwright test against a fresh demo context.

## Scope decision

The plan was not changed. Its M1 implementation is an unauthenticated,
temporary demo with real SQLite persistence. Sociobot Entra CIAM staff
accounts, durable PostgreSQL school workspaces, real public school classes,
and the Sociobot/Dodo subscription belong to M2. Adding them here would have
silently crossed the plan's milestone boundary. The landing page says that
accounts and billing come next and does not offer a paid plan.

## Verification evidence

Run from a clean clone:

```bash
npm ci
npm test
npm run test:e2e
npm run build
```

Results on 2026-08-28:

- `npm ci`: 169 packages audited, zero vulnerabilities.
- `npm test`: 4 Vitest tests, 3 Rust unit tests, and 6 Rust API/integration
  tests passed. Coverage includes exact-cutoff behavior, idempotency, cookie
  isolation, validation, concurrent allocation, reset, forwarded-IP limits,
  health, and the reversible down migration.
- `npm run test:e2e`: 14 Chromium tests passed. All four claim tags passed,
  as did keyboard, route focus/title, reset navigation, 390 px, reduced motion,
  same-origin request, light/dark axe, and public-route console checks.
- `npm run build`: `dist/` produced; initial JS is 64.50 KB gzip and CSS is
  3.81 KB gzip. The optimized Rust service built successfully.
- `scripts/load-smoke.sh`: 100 concurrent requests completed with only 200 and
  429 outcomes; every 429 included `Retry-After`.
- Local mobile Lighthouse: landing 100 performance / 100 accessibility / 100
  best practices / 100 SEO, with 1.4 s LCP and 0 CLS. Demo 99 / 100 / 100 /
  100, with 1.4 s LCP and 0.06 CLS.
- Container image `1e1d63a1b7cd` was built by ACR and deployed to
  `https://class-capacity-truth.sociobot.in`, and checked cold with
  `/opt/fleet/lib/verify-url.sh`. `/health` reports the deployed build SHA and
  a ready database. The landing, demo, privacy, terms, 404, metadata assets,
  claim flow, and rate limit were checked on the public origin.

## M2 needs

1. Register `https://class-capacity-truth.sociobot.in/auth/callback` on the
   shared Sociobot Entra SPA, then add PKCE frontend login and strict API JWT
   validation using discovery and JWKS.
2. Add PostgreSQL migrations for organisations, users, memberships, classes,
   bookings, seat events, and entitlements. Enable and test tenant RLS before
   accepting real data; keep the current demo tables and cookie path isolated.
3. Register the test `school-monthly-99` subscription through the factory
   Sociobot billing workflow. Implement hosted checkout, verified entitlement,
   renewal/cancellation state, and read-only grace without a direct Dodo SDK.
4. Reuse the transaction-tested seat ledger for real opaque public class links,
   add create/edit screens, role checks, encrypted guardian fields, and the M2
   claims. Retain every M1 test.

## Known gaps and operator action

There is no known M1 product gap. M1 intentionally has no staff account, real
school data, calendar connection, email, waitlist, or paid checkout.

Before M2 can ship, an operator must confirm the Entra callback registration
and test subscription registration. Those external registrations were not
needed or attempted in M1.
