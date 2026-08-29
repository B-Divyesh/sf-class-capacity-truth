# Verification 6 handoff — FAIL

Candidate `00edf2a9a366bb0eda3e5eebce4e88e3377f2fa3` at
<https://class-capacity-truth.sociobot.in> is **not releasable for real school
data**. Full evidence is in [verification-6.md](verification-6.md).

## What was verified

- Clean dependency install, all 21 exact claim commands, unit/API tests,
  typecheck, lint, release build, full 24-test browser suite, and cold-start
  sample claim all passed.
- The live landing clears the cold first-read/demo gate. Demo capacity,
  reset, full/cutoff states, same-origin privacy behavior, headers, cookies,
  keyboard focus, mobile/reduced-motion reflow, axe, and Lighthouse passed.
- Live API rate limits are enforced: 10 anonymous demo creations and 40 school
  route requests per forwarded IP, with 429 and `Retry-After` after allowance.
- The live health build SHA and primary JS asset match the candidate.

## Release-blocking finding

Fresh Azure inspection of active revision `sf-class-capacity-truth--0000025`
found `maxReplicas: 3`, only `PORT`, and no volumes or mounts. Startup logs
say `database_config=generated-default` and `durable_backup=disabled`.
Therefore SQLite and generated encryption/cookie keys are disposable and can
split across replicas. This violates the core seat-inventory durability
contract and the repository's one-replica Azure Files topology contract.

## Required operator action

Deploy the checked-in Azure Files mount (`cct-data`), `DATA_DIR`, durable
snapshot path, and exact one-replica cap to the active revision. Then prove a
real-school booking survives an actual revision restart and request a new QA
pass. Docker image build could not be run here because no local container
builder is installed.
