# Keep class seat counts accurate

Class Capacity Truth helps small language schools and tutoring centres keep
class seat counts accurate. Staff connect a calendar feed (iCalendar), publish
guardian booking links, and create one timed offer after cancelling a booking.
Staff can copy the offer link into the school’s usual email or messaging
service. If email delivery is configured, the service queues an encrypted offer
email. The plan costs $99 per school each month through Sociobot checkout.

Try the deployed demo at
[class-capacity-truth.sociobot.in/demo?demo=1](https://class-capacity-truth.sociobot.in/demo?demo=1).
Each browser gets its own temporary workspace for 24 hours. The demo checks
each name and email, then discards both.

## Real school workflow

Open `/app` and sign in with your school’s Sociobot Microsoft account. The
server assigns owner, operator, or viewer permissions from each staff member’s
Microsoft sign-in. A workspace can be recovered on another device after sign-in.
Calendar feeds are encrypted and checked every five minutes. A disagreement
is visible as **Attention** and never changes confirmed seats automatically.

Guardians can book while seats remain or consent to the waitlist. Staff select
the exact booking to cancel. The server creates a 24-hour offer for the oldest
waiting guardian. The saved receipt shows the offer link and whether email was
sent. Without email delivery, staff use **Copy offer** and send the URL through
the school’s usual email or messaging service. Owners can export or delete the
workspace. Contact fields
are encrypted and scrubbed after 90 days.

## Run locally

Requires Node 22+, npm 10+, and stable Rust.

```bash
npm ci
npm run build:web
DATA_DIR="$PWD/.data" FRONTEND_DIST="$PWD/dist" cargo run --manifest-path services/api/Cargo.toml
```

Open `http://localhost:8080/demo?demo=1`. The service starts with only `PORT`
(and defaults to `8080`); `DATA_DIR` defaults to `/data` in the
container. When no cookie-signing key is supplied, the service creates a secure
random key and stores it in the data directory. A separate contact-encryption key is
generated and persisted the same way. These optional settings configure email
delivery: `SMTP_RELAY`, `SMTP_USERNAME`, `SMTP_PASSWORD`, and `SMTP_FROM`.
Without them, staff can copy the saved offer URL and see that no email was
sent.

## Test and build

```bash
npm test
npm run test:api
npm run test:e2e
npm run build
docker build --build-arg BUILD_SHA=local -t class-capacity-truth .
```

`npm test` runs the TypeScript unit suite and Rust tests. The Playwright suite
starts the compiled API service with a clean temporary database and checks
browser flows. Run every command in [.factory/claims.json](.factory/claims.json)
to verify all claims. `npm run build` produces `dist/` and a release API binary.

## Architecture and deployment

- React 19, Vite, strict TypeScript, and hand-authored CSS for the web app.
- Rust, Axum, SQLx, and SQLite store the isolated demo and school workspaces.
  Production mounts the work-order Azure Files
  share at `/data`; SQLite and generated keys live there. Production is fixed
  at one replica.
- The API validates Microsoft sign-in tokens and enforces staff roles. It
  encrypts contact and calendar fields. Concurrent booking requests cannot
  oversell a class. The
  server also stores offer receipts, can queue email, and limits requests by
  client IP.
- One container serves both the API and built web assets on `PORT`. The
  deployment contract fixes the app at one replica and mounts Azure Files at
  `/data`. Rate limits apply once per forwarded client IP.

## Operations metrics

Signed-in owners and operators can open `/app/operations`. Authorised school
staff can fetch the same totals from `GET /api/metrics` or
`GET /api/workspaces/metrics`. Requests need their Microsoft sign-in token and
workspace key. The metrics response lists request counts, server errors, and
response times. It also lists calendar delay, unresolved differences, and
accepted seat offers. It never contains guardian, class, school, or token
values.

Treat any server error or unresolved public discrepancy as an investigation.
Check calendar connections when lag exceeds ten minutes. Review API
availability each month.

See [.factory/plan.md](.factory/plan.md) for
the milestone architecture and [.factory/design.md](.factory/design.md) for the
modular classroom abacus visual system.

Every release sets `deploy.data_dir` to `/data` in the container work order.
The topology script then checks the image, one-replica limit, and Azure Files
mount. Build and deploy the checked-out commit with its full commit ID. The
deployment command rejects unbound tags and requires `/health` to return that
ID.

```bash
release_sha="$(git rev-parse HEAD)"
release_tag="${release_sha:0:12}"
az acr build --registry sociobotregistry \
  --image "sf-class-capacity-truth:${release_tag}" \
  --file Dockerfile \
  --build-arg "BUILD_SHA=${release_sha}" \
  --build-arg "GIT_SHA=${release_sha}" \
  --build-arg "SOURCE_COMMIT=${release_sha}" .
IMAGE="sociobotregistry.azurecr.io/sf-class-capacity-truth:${release_tag}" \
EXPECTED_BUILD_SHA="$release_sha" \
bash scripts/deploy-container.sh
```

## Privacy and licence

Public and demo pages load no third-party fonts, scripts, advertising trackers,
or analytics. Microsoft sign-in and Sociobot checkout open only after a staff
member selects them. See [/privacy](https://class-capacity-truth.sociobot.in/privacy)
and the exact sandbox contract in [.factory/demo.md](.factory/demo.md).

Released source is available under the [MIT License](LICENSE).
