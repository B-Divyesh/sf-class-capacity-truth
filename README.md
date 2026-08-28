# Class Capacity Truth

Class Capacity Truth demonstrates a class seat count for a small language
school. M1 ships a public, isolated booking demo with three sample
classes: open, full, and past cutoff. Staff accounts, live school classes,
billing, calendar checks, and waitlists are planned for later milestones.

Try the deployed demo at
[class-capacity-truth.sociobot.in/demo?demo=1](https://class-capacity-truth.sociobot.in/demo?demo=1).
The sample is fictional. Each browser gets a signed, temporary workspace that
expires after 24 hours. Name and email input is validated but not retained.

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
data directory when none is supplied.

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
- Rust, Axum, SQLx, and SQLite for the temporary M1 demo ledger.
- Signed HttpOnly demo cookies, transaction-checked bookings, reversible SQL
  migration, expiry cleanup, and forwarded-IP rate limits.
- One non-root container serves both the API and built web assets on `PORT`.

The factory deploys the container. This repository does not change DNS,
billing, or cloud infrastructure. See [.factory/plan.md](.factory/plan.md) for
the milestone architecture and [.factory/design.md](.factory/design.md) for the
modular classroom abacus visual system.

## Privacy and licence

The product loads no third-party fonts or scripts and sends no advertising or
analytics requests. See [/privacy](https://class-capacity-truth.sociobot.in/privacy)
and the exact sandbox contract in [.factory/demo.md](.factory/demo.md).

Released source is available under the [MIT License](LICENSE).
