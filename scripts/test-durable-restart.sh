#!/usr/bin/env bash
set -euo pipefail

# End-to-end durable restart proof with the release binary. It deliberately
# creates a real-school record (not a demo tenant) in the same direct /data
# layout used by the work-order Azure Files mount, then starts a new process
# from that mounted directory and verifies both capacity and encrypted contact
# recovery through the public/staff APIs.
BUILD_SHA=durable-restart-test npm run build >/dev/null
runtime_dir="$(mktemp -d)"
base_url="http://127.0.0.1:18089"
token="durable-restart-local-test-token"
log_file="$runtime_dir/server.log"
cleanup() {
  if [[ -n "${server_pid:-}" ]]; then kill "$server_pid" 2>/dev/null || true; wait "$server_pid" 2>/dev/null || true; fi
  rm -rf "$runtime_dir"
}
trap cleanup EXIT

start_server() {
  DATA_DIR="$runtime_dir/mounted" \
  FRONTEND_DIST="$PWD/dist" PORT=18089 TEST_AUTH_TOKEN="$token" BUILD_SHA=durable-restart-test \
  ./services/api/target/release/class-capacity-truth-api >>"$log_file" 2>&1 &
  server_pid=$!
  for _ in $(seq 1 100); do
    if curl --silent --fail "$base_url/health" | jq -e '.database == "ready" and .build == "durable-restart-test"' >/dev/null; then return 0; fi
    sleep 0.1
  done
  cat "$log_file" >&2
  return 1
}

request() {
  curl --silent --show-error --fail-with-body "$@"
}

start_server
workspace="$(request -X POST "$base_url/api/workspaces" \
  -H "Authorization: Bearer $token" -H 'Content-Type: application/json' \
  --data '{"schoolName":"Revision Restart QA School"}')"
workspace_key="$(jq -r '.accessKey' <<<"$workspace")"
now="$(date +%s)"
created_class="$(request -X POST "$base_url/api/workspaces/classes" \
  -H "Authorization: Bearer $token" -H "X-Workspace-Key: $workspace_key" -H 'Content-Type: application/json' \
  --data "{\"name\":\"Revision restart QA class\",\"startsAt\":$((now + 172800)),\"bookingCutoff\":$((now + 86400)),\"timezone\":\"Etc/UTC\",\"capacity\":2}")"
class_id="$(jq -r '.id' <<<"$created_class")"
public_id="$(jq -r '.publicId' <<<"$created_class")"
request -X POST "$base_url/api/workspaces/classes/$class_id/publish" \
  -H "Authorization: Bearer $token" -H "X-Workspace-Key: $workspace_key" >/dev/null
request -X POST "$base_url/api/classes/$public_id/book" \
  -H 'Content-Type: application/json' -H 'Idempotency-Key: durable-restart-booking-01' \
  --data '{"guardianName":"Revision Parent","guardianEmail":"revision.parent@example.org"}' \
  | jq -e '.confirmed == 1 and .openSeats == 1' >/dev/null

kill "$server_pid"
wait "$server_pid" || true
unset server_pid
start_server
request "$base_url/api/classes/$public_id" | jq -e '.confirmed == 1 and .openSeats == 1' >/dev/null
request "$base_url/api/workspaces/classes/$class_id/bookings" \
  -H "Authorization: Bearer $token" -H "X-Workspace-Key: $workspace_key" \
  | jq -e 'length == 1 and .[0].guardianName == "Revision Parent" and .[0].guardianEmail == "revision.parent@example.org"' >/dev/null
test -s "$runtime_dir/mounted/class-capacity-truth-state-v3.db"
test -s "$runtime_dir/mounted/contact-data.key"
test -s "$runtime_dir/mounted/demo-cookie.key"
printf 'real-school booking survived release-process restart from direct mounted /data state\n'
