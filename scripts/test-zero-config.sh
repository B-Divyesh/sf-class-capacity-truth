#!/usr/bin/env bash
set -euo pipefail

# Regression proof for the container's required zero-config runtime contract.
# PORT is the one factory-provided environment variable; DATA_DIR and keys are
# intentionally omitted so the application must generate/persist its defaults.
npm run build >/dev/null
log_file="$(mktemp)"
cleanup() {
  if [[ -n "${server_pid:-}" ]]; then kill "$server_pid" 2>/dev/null || true; wait "$server_pid" 2>/dev/null || true; fi
  rm -f "$log_file"
}
trap cleanup EXIT
env -i PATH="$PATH" PORT=18087 FRONTEND_DIST="$PWD/dist" ./services/api/target/release/class-capacity-truth-api >"$log_file" 2>&1 &
server_pid=$!
for _ in $(seq 1 50); do
  if curl --silent --fail http://127.0.0.1:18087/health | grep -q '"database":"ready"'; then
    grep -q 'generated-default' "$log_file"
    grep -q 'generated-and-persisted\|persisted-generated' "$log_file"
    exit 0
  fi
  sleep 0.1
done
cat "$log_file" >&2
exit 1
