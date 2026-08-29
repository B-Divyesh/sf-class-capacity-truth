#!/usr/bin/env bash
set -euo pipefail

repo_dir="$(cd "$(dirname "$0")/.." && pwd)"
cold_target="$(mktemp -d /tmp/class-capacity-truth-cold.XXXXXX)"
cleanup() { rm -rf "$cold_target"; }
trap cleanup EXIT

cd "$repo_dir"
start_seconds="$(date +%s)"
CARGO_TARGET_DIR="$cold_target" \
PLAYWRIGHT_SERVER_TIMEOUT_MS=600000 \
npm run test:e2e -- --grep @claim:sample-booking-updates-seats
elapsed_seconds="$(( $(date +%s) - start_seconds ))"

if (( elapsed_seconds >= 600 )); then
  echo "cold claim exceeded its 600-second startup contract" >&2
  exit 1
fi
echo "cold claim passed in ${elapsed_seconds}s (limit: 600s)"
