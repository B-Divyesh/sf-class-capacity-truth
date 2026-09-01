#!/usr/bin/env bash
set -euo pipefail

# Deploy the current image to the existing factory Container App with the
# durable one-replica topology. The work order registers this product's Azure
# Files storage and mounts it at deploy.data_dir=/data. This guard only updates
# this product's Container App; it never reads storage credentials or modifies
# DNS, billing, or shared infrastructure.
resource_group="${RESOURCE_GROUP:-sociobot}"
app_name="${CONTAINER_APP_NAME:-sf-class-capacity-truth}"
base_url="${BASE_URL:-https://class-capacity-truth.sociobot.in}"
storage_name="${DATA_STORAGE_NAME:-sf-class-capacity-truth-data}"
image="${IMAGE:?Set IMAGE to the immutable ACR image tag to deploy}"
# A release is not identifiable from an image tag alone: a stale revision can
# keep serving an older binary while a tag-shaped image is present in the
# template. Require the full source commit that was passed to Docker's
# BUILD_SHA argument, then require /health to return that exact value after
# ingress switches. This prevents the Verification 18 stale-build release.
expected_build_sha="${EXPECTED_BUILD_SHA:?Set EXPECTED_BUILD_SHA to the exact 40-character source commit}"
if [[ ! "$expected_build_sha" =~ ^[0-9a-f]{40}$ ]]; then
  echo "EXPECTED_BUILD_SHA must be a lowercase 40-character source commit" >&2
  exit 64
fi
image_tag="${image##*:}"
if [[ "$image_tag" != "${expected_build_sha:0:12}" ]]; then
  echo "IMAGE tag must equal the first 12 characters of EXPECTED_BUILD_SHA" >&2
  exit 64
fi
# A revision suffix must be unique across a Container App. Do not copy the
# currently-ready suffix from the readback template: ARM will accept that
# request asynchronously but Container Apps cannot create the next revision.
# Keep this below the Container App's suffix-length limit.
revision_suffix="${REVISION_SUFFIX:-d-$(date +%s)-$RANDOM}"

wait_for_effective_template() {
  local actual
  for _ in $(seq 1 120); do
    actual="$(az containerapp show --resource-group "$resource_group" --name "$app_name" -o json)"
    if jq -e --arg image "$image" --arg storage_name "$storage_name" '
      .properties.template.containers[0].image == $image and
      .properties.template.scale.minReplicas == 1 and
      .properties.template.scale.maxReplicas == 1 and
      (.properties.template.volumes | any(.name == "data" and .storageType == "AzureFile" and .storageName == $storage_name)) and
      (.properties.template.containers[0].volumeMounts | any(.volumeName == "data" and .mountPath == "/data")) and
      (.properties.template.containers[0].env | any(.name == "PORT" and .value == "8080")) and
      ((.properties.template.containers[0].env | any(.name == "DATA_DIR" or .name == "DURABLE_BACKUP_PATH")) | not)
    ' <<<"$actual" >/dev/null; then
      return 0
    fi
    sleep 2
  done
  echo "Azure did not apply the requested durable template and image in time" >&2
  return 1
}

apply_template_patch() {
  local response
  for _ in $(seq 1 120); do
    if response="$(az rest --method PATCH --uri "${app_id}?api-version=2024-03-01" \
      --body "@$patch_file" --only-show-errors 2>&1)"; then
      return 0
    fi
    if grep -q 'ContainerAppOperationInProgress' <<<"$response"; then
      sleep 2
      continue
    fi
    printf '%s\n' "$response" >&2
    return 1
  done
  echo "Azure did not accept the durable template patch in time" >&2
  return 1
}

app_id="$(az containerapp show --resource-group "$resource_group" --name "$app_name" --query id -o tsv)"
# `latestRevisionName` can point at a failed replacement while Container Apps
# still serves its last ready fallback. Stop the process that can actually own
# /data, not merely the newest template.
previous_revision="$(az containerapp show --resource-group "$resource_group" --name "$app_name" \
  --query properties.latestReadyRevisionName -o tsv)"
previous_active=false
if [[ -n "$previous_revision" ]]; then
  previous_active="$(az containerapp revision show --resource-group "$resource_group" --name "$app_name" \
    --revision "$previous_revision" --query properties.active -o tsv 2>/dev/null || printf 'false')"
fi
previous_deactivated=false
deployment_complete=false
patch_file="$(mktemp)"
cleanup() {
  local status=$?
  rm -f "$patch_file"
  if [[ $status -ne 0 && "$previous_deactivated" == true && -n "$previous_revision" ]]; then
    # Leave a failed candidate unable to open the lockless SQLite file before
    # restoring the known-ready fallback. Recovery stays scoped to this app.
    latest_revision="$(az containerapp show --resource-group "$resource_group" --name "$app_name" \
      --query properties.latestRevisionName -o tsv 2>/dev/null || true)"
    if [[ -n "$latest_revision" && "$latest_revision" != "$previous_revision" ]]; then
      az containerapp revision deactivate --resource-group "$resource_group" --name "$app_name" \
        --revision "$latest_revision" --only-show-errors >/dev/null 2>&1 || true
    fi
    az containerapp revision activate --resource-group "$resource_group" --name "$app_name" \
      --revision "$previous_revision" --only-show-errors >/dev/null 2>&1 || true
  fi
  exit "$status"
}
trap cleanup EXIT
jq --arg image "$image" --arg revision_suffix "$revision_suffix" --arg storage_name "$storage_name" '
  { properties: { template: (
    .properties.template |
    .revisionSuffix = $revision_suffix |
    .containers[0].image = $image |
    .containers[0].env = [
      {name:"PORT", value:"8080"}
    ] |
    .containers[0].volumeMounts = [{volumeName:"data", mountPath:"/data"}] |
    .volumes = [{name:"data", storageType:"AzureFile", storageName:$storage_name}] |
    .scale = {minReplicas: 1, maxReplicas: 1}
  )} }
' < <(az containerapp show --resource-group "$resource_group" --name "$app_name" -o json) > "$patch_file"

# The /data database uses SQLite's lockless VFS because Azure Files does not
# provide compatible POSIX byte-range locks. There must never be two processes
# with this mount open. A short, explicit restart gap is safer than a rolling
# overlap that could corrupt the school ledger.
if [[ -n "$previous_revision" && "$previous_active" == "true" ]]; then
  az containerapp revision deactivate --resource-group "$resource_group" --name "$app_name" \
    --revision "$previous_revision" --only-show-errors >/dev/null
  previous_deactivated=true
fi
apply_template_patch

# The PATCH is asynchronous in Container Apps. Never read a still-old template
# as though it were a deployment result; wait for the exact image and durable
# topology before attempting a follow-up environment cleanup.
wait_for_effective_template

# A short-lived exact test credential used for a persistence drill must never
# survive the subsequent production deploy, even though it is not part of the
# contract.
actual="$(az containerapp show --resource-group "$resource_group" --name "$app_name" -o json)"
if jq -e '.properties.template.containers[0].env | any(.name == "TEST_AUTH_TOKEN")' <<<"$actual" >/dev/null; then
  az containerapp update --resource-group "$resource_group" --name "$app_name" \
    --remove-env-vars TEST_AUTH_TOKEN --only-show-errors >/dev/null
  wait_for_effective_template
fi

# Do not treat a successful PATCH as a successful deployment. This is the
# regression guard for the revision that kept only PORT, omitted the /data
# mount, and scaled SQLite to three replicas: read the effective Container App
# template back from Azure.
RESOURCE_GROUP="$resource_group" CONTAINER_APP_NAME="$app_name" \
  "$(dirname "$0")/verify-container-topology.sh"

wait_for_effective_template
actual="$(az containerapp show --resource-group "$resource_group" --name "$app_name" -o json)"
jq -e --arg image "$image" '
  .properties.template.containers[0].image == $image
' <<<"$actual" >/dev/null

# A template readback alone is not a release result: Container Apps can expose
# a new template before its revision becomes healthy and receives ingress
# traffic. Wait for that handoff, then prove the running process identifies as
# the source that was just deployed.
BASE_URL="$base_url" \
RESOURCE_GROUP="$resource_group" CONTAINER_APP_NAME="$app_name" \
  "$(dirname "$0")/wait-for-containerapp-revision.sh" "$previous_revision" >/dev/null

health="$(curl --silent --show-error --fail "$base_url/health")"
jq -e --arg build "$expected_build_sha" '
  .status == "ok" and .database == "ready" and .build == $build
' <<<"$health" >/dev/null

deployment_complete=true

jq '{
  revision: .properties.latestRevisionName,
  image: .properties.template.containers[0].image,
  min: .properties.template.scale.minReplicas,
  max: .properties.template.scale.maxReplicas,
  mounts: .properties.template.containers[0].volumeMounts,
  volumes: .properties.template.volumes,
  env: .properties.template.containers[0].env
}' <<<"$actual"
