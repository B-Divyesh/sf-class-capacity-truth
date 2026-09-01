#!/usr/bin/env bash
set -euo pipefail

# Regressions for verifier findings 16 and 18. The first fixture is the exact
# defective active control-plane shape read from production by verification 16:
# candidate 283758f64e321a3037951b433f24bc79c0622ee6, revision 0000046, only
# PORT, no durable /data Azure Files mount, and maxReplicas 3. Verification 18
# then found an otherwise healthy live process still reporting
# 1612b35cb5141a1312e2be93dae26a0a51d59e5a instead of requested candidate
# 2c800aa84529f69f6819d4bf7bea08891832dfce. This test first rejects each exact
# failure, then proves the checked-in command deploys a matching full identity.
repo_root="$(cd "$(dirname "$0")/.." && pwd)"
grep -Fq 'BUILD_SHA="$BUILD_SHA" cargo build --release' "$repo_root/Dockerfile"
fixture_dir="$(mktemp -d)"
cleanup() { rm -rf "$fixture_dir"; }
trap cleanup EXIT
mkdir -p "$fixture_dir/bin"

cat >"$fixture_dir/state.json" <<'JSON'
{
  "id": "/subscriptions/test/resourceGroups/sociobot/providers/Microsoft.App/containerApps/sf-class-capacity-truth",
  "properties": {
      "latestRevisionName": "sf-class-capacity-truth--0000046",
      "latestReadyRevisionName": "sf-class-capacity-truth--0000046",
      "provisioningState": "Succeeded",
      "template": {
      "containers": [{"name":"app","image":"sociobotregistry.azurecr.io/sf-class-capacity-truth:283758f64e32","env":[{"name":"PORT","value":"8080"}]}],
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
  "containerapp revision show"*)
    if [[ " $* " == *" --query properties.active "* ]]; then printf 'true\n'; else printf 'Healthy\n'; fi
    ;;
  "containerapp revision deactivate"*) ;;
  "containerapp revision activate"*) ;;
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
  .properties.latestRevisionName == "sf-class-capacity-truth--0000046" and
  .properties.latestReadyRevisionName == "sf-class-capacity-truth--0000046" and
  .properties.template.containers[0].image == "sociobotregistry.azurecr.io/sf-class-capacity-truth:283758f64e32" and
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

# The deployment command must not accept an unbound image tag. Its public
# contract requires the exact full candidate SHA that /health must report.
if PATH="$fixture_dir/bin:$PATH" \
  AZ_FIXTURE_STATE="$fixture_dir/state.json" \
  AZ_FIXTURE_LOG="$fixture_dir/missing-identity-az.log" \
  IMAGE="sociobotregistry.azurecr.io/sf-class-capacity-truth:2c800aa84529" \
  BASE_URL="https://fixture.invalid" \
  REVISION_SUFFIX="d-missing-identity-20260901" \
  "$repo_root/scripts/deploy-container.sh" >/dev/null 2>&1; then
  echo "the deployment accepted an image without its exact source identity" >&2
  exit 1
fi
[[ ! -e "$fixture_dir/missing-identity-az.log" ]]

# Verification 18's exact failure: a candidate-shaped image is requested but
# the traffic-serving process still reports the earlier 1612b35 build. This
# must fail even though health and the database are otherwise ready. Use a copy
# so the positive deployment below still begins from the known-unsafe shape.
cp "$fixture_dir/state.json" "$fixture_dir/identity-mismatch-state.json"
if PATH="$fixture_dir/bin:$PATH" \
  AZ_FIXTURE_STATE="$fixture_dir/identity-mismatch-state.json" \
  AZ_FIXTURE_LOG="$fixture_dir/identity-mismatch-az.log" \
  IMAGE="sociobotregistry.azurecr.io/sf-class-capacity-truth:2c800aa84529" \
  EXPECTED_BUILD_SHA="2c800aa84529f69f6819d4bf7bea08891832dfce" \
  BASE_URL="https://fixture.invalid" \
  REVISION_SUFFIX="d-identity-mismatch-20260901" \
  AZ_FIXTURE_BUILD_SHA="1612b35cb5141a1312e2be93dae26a0a51d59e5a" \
  "$repo_root/scripts/deploy-container.sh" >/dev/null 2>&1; then
  echo "the deployment accepted Verification 18's stale traffic-serving build" >&2
  exit 1
fi

PATH="$fixture_dir/bin:$PATH" \
AZ_FIXTURE_STATE="$fixture_dir/state.json" \
AZ_FIXTURE_LOG="$fixture_dir/az.log" \
IMAGE="sociobotregistry.azurecr.io/sf-class-capacity-truth:4a8c9ef01234" \
EXPECTED_BUILD_SHA="4a8c9ef0123456789abcdef0123456789abcdef0" \
BASE_URL="https://fixture.invalid" \
REVISION_SUFFIX="d-regression-20260901" \
AZ_FIXTURE_BUILD_SHA="4a8c9ef0123456789abcdef0123456789abcdef0" \
"$repo_root/scripts/deploy-container.sh" >/dev/null

jq -e '
  .properties.template.containers[0].image == "sociobotregistry.azurecr.io/sf-class-capacity-truth:4a8c9ef01234" and
  .properties.template.revisionSuffix == "d-regression-20260901" and
  .properties.template.scale == {minReplicas: 1, maxReplicas: 1} and
  (.properties.template.volumes | any(.name == "data" and .storageType == "AzureFile" and .storageName == "sf-class-capacity-truth-data")) and
  (.properties.template.containers[0].volumeMounts | any(.volumeName == "data" and .mountPath == "/data")) and
  (.properties.template.containers[0].env == [{name:"PORT", value:"8080"}])
' "$fixture_dir/state.json" >/dev/null
grep -q 'containerapp revision show' "$fixture_dir/az.log"
grep -q 'containerapp revision deactivate.*sf-class-capacity-truth--0000046' "$fixture_dir/az.log"
deactivate_line="$(grep -n 'containerapp revision deactivate.*sf-class-capacity-truth--0000046' "$fixture_dir/az.log" | head -1 | cut -d: -f1)"
patch_line="$(grep -n 'rest --method PATCH' "$fixture_dir/az.log" | head -1 | cut -d: -f1)"
readiness_line="$(grep -n 'containerapp revision show.*--query properties.healthState' "$fixture_dir/az.log" | head -1 | cut -d: -f1)"
[[ "$deactivate_line" -lt "$patch_line" ]]
[[ "$deactivate_line" -lt "$readiness_line" ]]
printf 'deployment topology regression passed\n'
