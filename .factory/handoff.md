# Independent QA handoff — FAIL

Candidate: `11a728e6b2f481506753caef919347958512c124`

Live URL: <https://class-capacity-truth.sociobot.in>

Verified: 2026-08-29 UTC

## Verdict

**FAIL — do not accept real school data.** The candidate artifact and local
quality gates pass, but the active deployment is an ephemeral, multi-replica
SQLite service. See [.factory/verification-8.md](verification-8.md) for the
complete evidence.

## Release blocker

### P0 — active candidate revision has no durable storage and scaled to two replicas

`/health` identifies the exact candidate build. Fresh Azure readback found
active revision `sf-class-capacity-truth--0000039`, image
`sociobotregistry.azurecr.io/sf-class-capacity-truth:11a728e6b2f4`,
`minReplicas/maxReplicas: 1/3`, only `PORT=8080`, and no volumes or mounts.
Startup reports `durable_backup="disabled"` and generated local signing and
encryption keys.

During QA the revision had two Ready/Running replicas. Each has its own
disposable database and keys, so requests can see conflicting seat counts and
a replacement can lose all real-school data. The registered `cct-data` Azure
Files storage exists but is not attached.

Required repair: deploy this immutable candidate image through
`scripts/deploy-container.sh`, read back one replica plus the `/mnt/cct` mount
and both durable path variables, then run
`scripts/prove-production-durability.sh`. Do not run the drill before the
effective template is safe.

### P3 — mobile navigation differs from the design contract

At 390 px the four navigation links wrap into two rows instead of collapsing
to the labelled menu specified in `.factory/design.md`. It remains readable
and usable.

## Verification summary

- All 21 commands in `.factory/claims.json` passed from the clean checkout.
  The durable-topology command passes its fixture, but the claim fails against
  the live control plane.
- `npm ci`, `npm test`, `npm run typecheck`, `npm run lint`, `npm run build`,
  `npm run test:e2e` (24/24), `npm run test:cold-claim`, durable restart,
  zero-config startup, and the 100-request live load smoke passed.
- The live HTML/primary JS/CSS exactly match local `dist/`; `/health` reports
  the full candidate SHA.
- First-read/demo, normal/full/cutoff booking behavior, invalid-input recovery,
  same-origin privacy, CORS/CSP/cache headers, Entra PKCE sign-in, Sociobot
  checkout, keyboard, 390 px, 200% text, reduced motion, and link crawl passed.
- Axe found zero serious/critical issues. Lighthouse mobile scored 100 in
  Performance, Accessibility, Best Practices, and SEO; LCP was 1.3 s and CLS
  was 0. Initial JS was 70,505 bytes gzip and CSS 4,343 bytes gzip.
- Live allowance observed: demo request 11 returned 429 with `Retry-After: 5`;
  staff request 41 returned 429 with `Retry-After: 1`; the 100-request smoke
  accepted 10 and rate-limited 90.
- Docker image build was not run because no Docker-compatible runtime is
  installed in the verifier container.

No product code or infrastructure was modified during verification.
