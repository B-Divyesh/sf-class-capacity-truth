# Verification 5 handoff — FAIL

## Result

**FAIL — do not release or accept school data.** Independent verification was
performed on 2026-08-29 UTC against commit
`029c619bf3bba1c156f650f15cc14e49ef733146` and
<https://class-capacity-truth.sociobot.in>. The live health build, image tag,
and all hashed frontend assets match that candidate.

The code gates are healthy: all 15 exact claim commands, `npm test`, typecheck,
lint/Clippy, the exact production build, and 23/23 Playwright tests passed.
Fresh mobile Lighthouse scored 98/100/100/100. Checkout now creates a real
hosted Dodo session, the no-SMTP copyable offer path is tested, and sign-in uses
the required Sociobot CIAM tenant.

## Release blockers

- **P0 — live state is replica-local and ephemeral.** Active revision
  `sf-class-capacity-truth--0000022` has `maxReplicas: 3`, no volume mount, and
  only `PORT`. Required load created two running replicas. Both logged a
  generated default SQLite database, disabled durable backup, and independently
  generated signing/encryption keys. With one valid demo cookie, ten identical
  booking calls produced four 201 and six `401 demo_cookie_missing` responses.
  Real school data can split or disappear on replacement.
- **P1 — rate limits multiply by replica.** Locally, the anonymous burst is 10
  and protected burst is 40. With two live replicas, one forwarded client got
  20 anonymous successes before 429 and 82 protected authentication responses
  in a 100-request burst. 429 responses do carry `Retry-After`.
- **P1 — claim coverage is incomplete.** Public statements about configured
  SMTP delivery, cross-device workspace recovery, oldest/24-hour offer
  semantics, zero-env startup, and forwarded-IP limiting are not registered.
  The tagged calendar test does not prove encryption at rest or the automatic
  five-minute schedule in its claim.
- **P2 — standalone 404 accessibility.** At 390px and 200% text it overflows by
  72px and its only recovery link is 21px high, below the 44px target rule.

## Reproduce

```bash
npm ci
npm test
npm run typecheck
npm run lint
npm run build
env -u CI npm run test:e2e
# In another terminal, serve the built product:
DATA_DIR=/tmp/cct-qa-data FRONTEND_DIST="$PWD/dist" npm start
./scripts/load-smoke.sh http://127.0.0.1:8080
```

Read-only deployment inspection:

```bash
az containerapp show -g sociobot -n sf-class-capacity-truth
az containerapp replica list -g sociobot -n sf-class-capacity-truth \
  --revision sf-class-capacity-truth--0000022
```

Full evidence, claim-by-claim results, headers, hashes, CIAM parameters, and
remediation are in [.factory/verification-5.md](verification-5.md). Browser and
Lighthouse artifacts are in `.factory/qa-artifacts/verification-5-live/`.

## Required next step

Restore the one-replica limit plus Azure Files data/key/snapshot mounts (or use
a replica-safe shared database and limiter), then prove state through a real
restart. Afterward add the missing claim tests and repair the zoomed 404 before
requesting verification again. No product code was changed during this QA run.
