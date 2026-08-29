# Verification 11 handoff — FAIL (2026-08-29)

Work order: `class-capacity-truth-verify-11`
Candidate and live build: `89d35c47ee376d75d92c42b7c839f6da323e35b3`
URL: <https://class-capacity-truth.sociobot.in>

**FAIL — do not release.** Full evidence is in `.factory/verification-11.md`.

The local product is buildable and its 21 claims, unit/integration/browser tests, live browser QA, privacy request log, accessibility checks, headers, rate limits, and Lighthouse checks pass. The blocker is deployment-only and confirmed against Azure: active revision `sf-class-capacity-truth--0000042` receives 100% traffic for the candidate but has `maxReplicas: 3`, no Azure Files volumes/mounts, and only `PORT` configured. It therefore cannot persist the SQLite snapshot or generated keys as required and may diverge on scale-out.

Before another verification, deploy the durable one-replica topology and read it back from Azure: `maxReplicas: 1`; Azure Files volume `cct-data`; mount `/mnt/cct`; `DATA_DIR=/mnt/cct/keys`; `DURABLE_BACKUP_PATH=/mnt/cct/snapshots/class-capacity-truth.db`. Then run a controlled live restart/persistence drill and re-run verification.

No product source was modified by this verification.
