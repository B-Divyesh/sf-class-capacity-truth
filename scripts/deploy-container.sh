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
# A source SHA is supplied by the release build when available. For the
# factory's immutable SHA-tagged images, the tag itself is still enough to
# prove that the traffic-serving process came from the requested source.
expected_build_sha="${EXPECTED_BUILD_SHA:-}"
image_tag="${image##*:}"
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
previous_revision="$(az containerapp show --resource-group "$resource_group" --name "$app_name" \
  --query properties.latestRevisionName -o tsv)"
patch_file="$(mktemp)"
trap 'rm -f "$patch_file"' EXIT
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

# SQLite holds an exclusive lock for the single mounted /data owner. Container
# Apps normally overlaps revisions while probing a replacement, which would
# make the replacement wait forever on Azure Files. The durable release is an
# explicit sequential restart: the prior revision is stopped only after the
# new immutable template and mount have been read back.
if [[ -n "$previous_revision" && "$previous_revision" != "$(jq -r '.properties.latestRevisionName' <<<"$actual")" ]]; then
  az containerapp revision deactivate --resource-group "$resource_group" --name "$app_name" \
    --revision "$previous_revision" --only-show-errors >/dev/null
fi

# A template readback alone is not a release result: Container Apps can expose
# a new template before its revision becomes healthy and receives ingress
# traffic. Wait for that handoff, then prove the running process identifies as
# the source that was just deployed.
BASE_URL="$base_url" \
RESOURCE_GROUP="$resource_group" CONTAINER_APP_NAME="$app_name" \
  "$(dirname "$0")/wait-for-containerapp-revision.sh" "$previous_revision" >/dev/null

health="$(curl --silent --show-error --fail "$base_url/health")"
if [[ -n "$expected_build_sha" ]]; then
  jq -e --arg build "$expected_build_sha" '
    .status == "ok" and .database == "ready" and .build == $build
  ' <<<"$health" >/dev/null
else
  jq -e --arg tag "$image_tag" '
    .status == "ok" and .database == "ready" and (.build | startswith($tag))
  ' <<<"$health" >/dev/null
fi

jq '{
  revision: .properties.latestRevisionName,
  image: .properties.template.containers[0].image,
  min: .properties.template.scale.minReplicas,
  max: .properties.template.scale.maxReplicas,
  mounts: .properties.template.containers[0].volumeMounts,
  volumes: .properties.template.volumes,
  env: .properties.template.containers[0].env
}' <<<"$actual"
