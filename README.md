# Class Capacity Truth

Class Capacity Truth is a capacity ledger for small language schools and
tutoring centres. Staff connect an iCalendar feed, publish parent booking
links, and record one expiring offer when a named booking is cancelled. A
configured SMTP relay sends it; without one, the workspace says it was not
sent. The
school plan costs $99 each month through Sociobot checkout.

Try the deployed demo at
[class-capacity-truth.sociobot.in/demo?demo=1](https://class-capacity-truth.sociobot.in/demo?demo=1).
The sample is fictional. Each browser gets a signed, temporary workspace that
expires after 24 hours. Name and email input is validated but not retained.

## Real school workflow

Open `/app` and sign in with the shared Sociobot Microsoft Entra tenant.
Owners, operators, and viewers are authorized on the server by stable Entra
identity. A workspace can be recovered on another device after sign-in.
Calendar feeds are encrypted and checked every five minutes. A disagreement
is visible as **Attention** and never changes confirmed seats automatically.

Parents can book while seats remain or consent to the waitlist. Staff select
the exact booking to cancel. The server records a 24-hour offer for the oldest
waiting guardian and retries delivery only when SMTP is configured. Owners can export or
delete the workspace. Contact fields are encrypted and scrubbed after 90 days.

## Run locally

Requires Node 22+, npm 10+, and stable Rust.

```bash
npm ci
npm run build:web
DATA_DIR="$PWD/.data" FRONTEND_DIST="$PWD/dist" cargo run --manifest-path services/api/Cargo.toml
```

Open `http://localhost:8080/demo?demo=1`. The service needs no environment
variables. `PORT` defaults to `8080`; `DATA_DIR` defaults to `/data` in the
container. A cookie-signing key is generated with a CSPRNG and persisted in the
data directory when none is supplied. A separate contact-encryption key is
generated and persisted the same way. Optional SMTP variables are
`SMTP_RELAY`, `SMTP_USERNAME`, `SMTP_PASSWORD`, and `SMTP_FROM`. Without them,
the workspace explicitly says offers are recorded but not sent.

## Test and build

```bash
npm test
npm run test:api
npm run test:e2e
npm run build
docker build --build-arg BUILD_SHA=local -t class-capacity-truth .
```

`npm test` runs the TypeScript unit suite and Rust tests. The Playwright suite
starts the compiled Axum service with a clean temporary database and verifies
every claim in [.factory/claims.json](.factory/claims.json). `npm run build`
produces `dist/` and a release API binary.

## Architecture and deployment

- React 19, Vite, strict TypeScript, and hand-authored CSS for the web app.
- Rust, Axum, SQLx, and SQLite for both the isolated demo and the
  single-instance school ledger. Production runs SQLite on local disk and
  atomically checkpoints each successful change to durable Azure Files storage.
  Startup restores that checkpoint. Production is fixed at one replica.
- Entra JWT discovery/JWKS validation, owner/operator/viewer authorization,
  encrypted contact and calendar fields, retention cleanup, transaction-checked
  bookings, an email outbox, and forwarded-IP rate limits.
- One non-root container serves both the API and built web assets on `PORT`.

The factory deploys the container. This repository does not change DNS,
billing, or cloud infrastructure. See [.factory/plan.md](.factory/plan.md) for
the milestone architecture and [.factory/design.md](.factory/design.md) for the
modular classroom abacus visual system.

## Privacy and licence

The product loads no third-party fonts or scripts and sends no advertising or
analytics requests. Entra sign-in and Sociobot checkout are explicit staff
actions. See [/privacy](https://class-capacity-truth.sociobot.in/privacy)
and the exact sandbox contract in [.factory/demo.md](.factory/demo.md).

Released source is available under the [MIT License](LICENSE).
