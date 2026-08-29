#!/usr/bin/env bash
set -euo pipefail

# Regression for the production-drill 401: Container Apps publishes a new
# latestRevisionName before ingress switches latestReadyRevisionName. A health
# probe against the old revision is not evidence that the temporary test token
# exists on the new one.
repo_root="$(cd "$(dirname "$0")/.." && pwd)"
fixture_dir="$(mktemp -d)"
cleanup() { rm -rf "$fixture_dir"; }
trap cleanup EXIT
mkdir -p "$fixture_dir/bin"

cat >"$fixture_dir/bin/az" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
log="${AZ_FIXTURE_LOG:?}"
ready_count_file="${AZ_READY_COUNT:?}"
revision_count_file="${AZ_REVISION_COUNT:?}"
printf '%s\n' "$*" >>"$log"
case "$*" in
  *"--query properties.latestRevisionName"*) printf 'sf-class-capacity-truth--new\n' ;;
  *"--query properties.latestReadyRevisionName"*)
    count="$(cat "$ready_count_file")"
    count=$((count + 1))
    printf '%s\n' "$count" >"$ready_count_file"
    if [[ "$count" -eq 1 ]]; then
      printf 'sf-class-capacity-truth--old\n'
    else
      printf 'sf-class-capacity-truth--new\n'
    fi
    ;;
  *"revision show"*"--query properties.healthState"*)
    count="$(cat "$revision_count_file")"
    count=$((count + 1))
    printf '%s\n' "$count" >"$revision_count_file"
    if [[ "$count" -eq 1 ]]; then
      echo "ERROR: The containerapp 'sf-class-capacity-truth' does not exist" >&2
      exit 3
    fi
    printf 'Healthy\n'
    ;;
  *"--query properties.provisioningState"*) printf 'Succeeded\n' ;;
  *) echo "unexpected az command: $*" >&2; exit 64 ;;
esac
SH

cat >"$fixture_dir/bin/curl" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
printf 'curl\n' >>"${AZ_FIXTURE_LOG:?}"
printf '{"status":"ok","database":"ready"}\n'
SH

cat >"$fixture_dir/bin/sleep" <<'SH'
#!/usr/bin/env bash
exit 0
SH
chmod +x "$fixture_dir/bin/az" "$fixture_dir/bin/curl" "$fixture_dir/bin/sleep"
printf '0\n' >"$fixture_dir/ready-count"
printf '0\n' >"$fixture_dir/revision-count"

result="$(PATH="$fixture_dir/bin:$PATH" \
  AZ_FIXTURE_LOG="$fixture_dir/az.log" \
  AZ_READY_COUNT="$fixture_dir/ready-count" \
  AZ_REVISION_COUNT="$fixture_dir/revision-count" \
  MAX_ATTEMPTS=3 SLEEP_SECONDS=0 \
  "$repo_root/scripts/wait-for-containerapp-revision.sh" sf-class-capacity-truth--old)"

[[ "$result" == "sf-class-capacity-truth--new" ]]
[[ "$(cat "$fixture_dir/ready-count")" == "3" ]]
[[ "$(cat "$fixture_dir/revision-count")" == "2" ]]
[[ "$(grep -c '^curl$' "$fixture_dir/az.log")" == "1" ]]
printf 'revision traffic-readiness regression passed\n'
