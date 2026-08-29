#!/usr/bin/env bash
set -euo pipefail

# Regression for verification-12 P0. The fixture is the exact defective active
# control-plane shape read from production by the independent verifier:
# candidate 28fcd19f33b513f4a3b365be90bda7ec457340c7, revision 0000043, only
# PORT, no Azure Files mount, and maxReplicas 3. It first proves that the
# readback guard rejects that shape, then proves the checked-in deploy command
# registers Azure Files, replaces the stale template, waits for the revision
# that receives traffic, and verifies the repair's full runtime identity.
repo_root="$(cd "$(dirname "$0")/.." && pwd)"
fixture_dir="$(mktemp -d)"
cleanup() { rm -rf "$fixture_dir"; }
trap cleanup EXIT
mkdir -p "$fixture_dir/bin"

cat >"$fixture_dir/state.json" <<'JSON'
{
  "id": "/subscriptions/test/resourceGroups/sociobot/providers/Microsoft.App/containerApps/sf-class-capacity-truth",
  "properties": {
      "latestRevisionName": "sf-class-capacity-truth--0000043",
      "latestReadyRevisionName": "sf-class-capacity-truth--0000043",
      "provisioningState": "Succeeded",
      "template": {
      "containers": [{"name":"app","image":"sociobotregistry.azurecr.io/sf-class-capacity-truth:28fcd19f33b5","env":[{"name":"PORT","value":"8080"}]}],
      "scale": {"minReplicas":1,"maxReplicas":3}
    }
  }
}
JSON

cat >"$fixture_dir/bin/az" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
state="${AZ_FIXTURE_STATE:?}"
log="${AZ_FIXTURE_LOG:?}"
printf '%s\n' "$*" >>"$log"
case "$*" in
  "storage account keys list"*) printf 'fixture-storage-key\n' ;;
  "storage share create"*) : ;;
  "containerapp env storage set"*) : ;;
  "containerapp revision show"*) printf 'Healthy\n' ;;
  "containerapp show"*)
    if [[ " $* " == *" --query id "* ]]; then
      jq -r '.id' "$state"
    elif [[ " $* " == *" --query properties.latestRevisionName "* ]]; then
      jq -r '.properties.latestRevisionName' "$state"
    elif [[ " $* " == *" --query properties.latestReadyRevisionName "* ]]; then
      jq -r '.properties.latestReadyRevisionName' "$state"
    elif [[ " $* " == *" --query properties.provisioningState "* ]]; then
      jq -r '.properties.provisioningState' "$state"
    else
      cat "$state"
    fi
    ;;
  "rest --method PATCH"*)
    body=""
    for argument in "$@"; do
      [[ "$argument" == @* ]] && body="${argument#@}"
    done
    test -n "$body"
    jq --slurpfile patch "$body" '
      .properties.template = $patch[0].properties.template |
      .properties.latestRevisionName = "sf-class-capacity-truth--" + $patch[0].properties.template.revisionSuffix |
      .properties.latestReadyRevisionName = "sf-class-capacity-truth--" + $patch[0].properties.template.revisionSuffix
    ' "$state" >"$state.next"
    mv "$state.next" "$state"
    ;;
  *) echo "unexpected az command: $*" >&2; exit 64 ;;
esac
SH
chmod +x "$fixture_dir/bin/az"

cat >"$fixture_dir/bin/curl" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
state="${AZ_FIXTURE_STATE:?}"
image="$(jq -r '.properties.template.containers[0].image' "$state")"
tag="${image##*:}"
build="${AZ_FIXTURE_BUILD_SHA:-$tag}"
printf '{"status":"ok","database":"ready","build":"%s"}\n' "$build"
SH
chmod +x "$fixture_dir/bin/curl"

# First reproduce the failing revision without changing it.
jq -e '
  .properties.latestRevisionName == "sf-class-capacity-truth--0000043" and
  .properties.template.containers[0].image == "sociobotregistry.azurecr.io/sf-class-capacity-truth:28fcd19f33b5" and
  .properties.template.scale.maxReplicas == 3 and
  .properties.template.containers[0].env == [{name:"PORT", value:"8080"}] and
  (.properties.template.containers[0].volumeMounts | not) and
  (.properties.template.volumes | not)
' "$fixture_dir/state.json" >/dev/null

# The production verifier must reject this exact stale template before the
# deployment repair is attempted. A raw jq assertion alone would not prove
# that the same readback guard used by the deploy command catches the defect.
if PATH="$fixture_dir/bin:$PATH" \
  AZ_FIXTURE_STATE="$fixture_dir/state.json" \
  AZ_FIXTURE_LOG="$fixture_dir/az.log" \
  "$repo_root/scripts/verify-container-topology.sh" >/dev/null 2>&1; then
  echo "the verifier accepted the known-unsafe one-PORT/no-volume/max-3 template" >&2
  exit 1
fi

# A durable template is still not a successful release if ingress serves a
# different build. Exercise the full SHA guard independently before the
# positive deployment below; this is intentionally a copy so the latter starts
# from verification-12's exact unsafe production shape.
cp "$fixture_dir/state.json" "$fixture_dir/identity-mismatch-state.json"
if PATH="$fixture_dir/bin:$PATH" \
  AZ_FIXTURE_STATE="$fixture_dir/identity-mismatch-state.json" \
  AZ_FIXTURE_LOG="$fixture_dir/identity-mismatch-az.log" \
  IMAGE="sociobotregistry.azurecr.io/sf-class-capacity-truth:identity-mismatch" \
  EXPECTED_BUILD_SHA="expected-full-build-sha" \
  BASE_URL="https://fixture.invalid" \
  REVISION_SUFFIX="d-identity-mismatch-20260829" \
  AZ_FIXTURE_BUILD_SHA="different-full-build-sha" \
  "$repo_root/scripts/deploy-container.sh" >/dev/null 2>&1; then
  echo "the deployment accepted a traffic-serving process with the wrong build identity" >&2
  exit 1
fi

PATH="$fixture_dir/bin:$PATH" \
AZ_FIXTURE_STATE="$fixture_dir/state.json" \
AZ_FIXTURE_LOG="$fixture_dir/az.log" \
IMAGE="sociobotregistry.azurecr.io/sf-class-capacity-truth:deployment-regression" \
EXPECTED_BUILD_SHA="deployment-regression-full-sha" \
BASE_URL="https://fixture.invalid" \
REVISION_SUFFIX="d-regression-20260829" \
AZ_FIXTURE_BUILD_SHA="deployment-regression-full-sha" \
"$repo_root/scripts/deploy-container.sh" >/dev/null

jq -e '
  .properties.template.containers[0].image == "sociobotregistry.azurecr.io/sf-class-capacity-truth:deployment-regression" and
  .properties.template.revisionSuffix == "d-regression-20260829" and
  .properties.template.scale == {minReplicas: 1, maxReplicas: 1} and
  (.properties.template.volumes | any(.name == "cct-data" and .storageType == "AzureFile" and .storageName == "cct-data")) and
  (.properties.template.containers[0].volumeMounts | any(.volumeName == "cct-data" and .mountPath == "/mnt/cct")) and
  (.properties.template.containers[0].env | any(.name == "DATA_DIR" and .value == "/mnt/cct/keys")) and
  (.properties.template.containers[0].env | any(.name == "DURABLE_BACKUP_PATH" and .value == "/mnt/cct/snapshots/class-capacity-truth.db"))
' "$fixture_dir/state.json" >/dev/null
grep -q 'containerapp env storage set' "$fixture_dir/az.log"
grep -q 'containerapp revision show' "$fixture_dir/az.log"
printf 'deployment topology regression passed\n'
