#!/usr/bin/env bash
set -euo pipefail

# Regression for verification-6 P0. The fixture is the exact defective
# control-plane shape reported by the independent verifier. It proves the
# checked-in deploy command registers Azure Files, replaces the stale template,
# and verifies the effective topology after the PATCH.
repo_root="$(cd "$(dirname "$0")/.." && pwd)"
fixture_dir="$(mktemp -d)"
cleanup() { rm -rf "$fixture_dir"; }
trap cleanup EXIT
mkdir -p "$fixture_dir/bin"

cat >"$fixture_dir/state.json" <<'JSON'
{
  "id": "/subscriptions/test/resourceGroups/sociobot/providers/Microsoft.App/containerApps/sf-class-capacity-truth",
  "properties": {
    "latestRevisionName": "sf-class-capacity-truth--0000025",
    "template": {
      "containers": [{"name":"app","image":"sociobotregistry.azurecr.io/sf-class-capacity-truth:00edf2a9a366","env":[{"name":"PORT","value":"8080"}]}],
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
  "containerapp show"*)
    if [[ " $* " == *" --query id "* ]]; then
      jq -r '.id' "$state"
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
    jq --slurpfile patch "$body" '.properties.template = $patch[0].properties.template' "$state" >"$state.next"
    mv "$state.next" "$state"
    ;;
  *) echo "unexpected az command: $*" >&2; exit 64 ;;
esac
SH
chmod +x "$fixture_dir/bin/az"

# First reproduce the failing revision without changing it.
jq -e '
  .properties.template.scale.maxReplicas == 3 and
  .properties.template.containers[0].env == [{name:"PORT", value:"8080"}] and
  (.properties.template.containers[0].volumeMounts | not) and
  (.properties.template.volumes | not)
' "$fixture_dir/state.json" >/dev/null

PATH="$fixture_dir/bin:$PATH" \
AZ_FIXTURE_STATE="$fixture_dir/state.json" \
AZ_FIXTURE_LOG="$fixture_dir/az.log" \
IMAGE="sociobotregistry.azurecr.io/sf-class-capacity-truth:deployment-regression" \
"$repo_root/scripts/deploy-container.sh" >/dev/null

jq -e '
  .properties.template.containers[0].image == "sociobotregistry.azurecr.io/sf-class-capacity-truth:deployment-regression" and
  .properties.template.scale == {minReplicas: 1, maxReplicas: 1} and
  (.properties.template.volumes | any(.name == "cct-data" and .storageType == "AzureFile" and .storageName == "cct-data")) and
  (.properties.template.containers[0].volumeMounts | any(.volumeName == "cct-data" and .mountPath == "/mnt/cct")) and
  (.properties.template.containers[0].env | any(.name == "DATA_DIR" and .value == "/mnt/cct/keys")) and
  (.properties.template.containers[0].env | any(.name == "DURABLE_BACKUP_PATH" and .value == "/mnt/cct/snapshots/class-capacity-truth.db"))
' "$fixture_dir/state.json" >/dev/null
grep -q 'containerapp env storage set' "$fixture_dir/az.log"
printf 'deployment topology regression passed\n'
