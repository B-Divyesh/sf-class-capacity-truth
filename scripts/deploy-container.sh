#!/usr/bin/env bash
set -euo pipefail

# Deploy the current image to the existing factory Container App with the
# durable one-replica topology. This intentionally does not change DNS or
# billing. Azure Files holds generated keys and atomic SQLite snapshots.
resource_group="${RESOURCE_GROUP:-sociobot}"
app_name="${CONTAINER_APP_NAME:-sf-class-capacity-truth}"
environment_name="${CONTAINER_APP_ENVIRONMENT:-factory-env}"
storage_account="${AZURE_FILES_ACCOUNT:-sociobotblob}"
share_name="${AZURE_FILES_SHARE:-class-capacity-truth}"
storage_name="cct-data"
image="${IMAGE:?Set IMAGE to the immutable ACR image tag to deploy}"
# A revision suffix must be unique across a Container App. Do not copy the
# currently-ready suffix from the readback template: ARM will accept that
# request asynchronously but Container Apps cannot create the next revision.
# Keep this below the Container App's suffix-length limit.
revision_suffix="${REVISION_SUFFIX:-d-$(date +%s)-$RANDOM}"

wait_for_effective_template() {
  local actual
  for _ in $(seq 1 120); do
    actual="$(az containerapp show --resource-group "$resource_group" --name "$app_name" -o json)"
    if jq -e --arg image "$image" '
      .properties.template.containers[0].image == $image and
      .properties.template.scale.minReplicas == 1 and
      .properties.template.scale.maxReplicas == 1 and
      (.properties.template.volumes | any(.name == "cct-data" and .storageType == "AzureFile" and .storageName == "cct-data")) and
      (.properties.template.containers[0].volumeMounts | any(.volumeName == "cct-data" and .mountPath == "/mnt/cct")) and
      (.properties.template.containers[0].env | any(.name == "DATA_DIR" and .value == "/mnt/cct/keys")) and
      (.properties.template.containers[0].env | any(.name == "DURABLE_BACKUP_PATH" and .value == "/mnt/cct/snapshots/class-capacity-truth.db"))
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

account_key="$(az storage account keys list --resource-group "$resource_group" --account-name "$storage_account" --query '[0].value' -o tsv)"
az storage share create --account-name "$storage_account" --account-key "$account_key" --name "$share_name" --quota 5 --only-show-errors >/dev/null
az containerapp env storage set --resource-group "$resource_group" --name "$environment_name" \
  --storage-name "$storage_name" --azure-file-account-name "$storage_account" \
  --azure-file-account-key "$account_key" --azure-file-share-name "$share_name" \
  --access-mode ReadWrite --only-show-errors >/dev/null

app_id="$(az containerapp show --resource-group "$resource_group" --name "$app_name" --query id -o tsv)"
patch_file="$(mktemp)"
trap 'rm -f "$patch_file"' EXIT
jq --arg image "$image" --arg revision_suffix "$revision_suffix" '
  { properties: { template: (
    .properties.template |
    .revisionSuffix = $revision_suffix |
    .containers[0].image = $image |
    .containers[0].env = [
      {name:"PORT", value:"8080"},
      {name:"DATA_DIR", value:"/mnt/cct/keys"},
      {name:"DURABLE_BACKUP_PATH", value:"/mnt/cct/snapshots/class-capacity-truth.db"}
    ] |
    .containers[0].volumeMounts = [{volumeName:"cct-data", mountPath:"/mnt/cct"}] |
    .volumes = [{name:"cct-data", storageType:"AzureFile", storageName:"cct-data"}] |
    .scale = {minReplicas: 1, maxReplicas: 1}
  )} }
' < <(az containerapp show --resource-group "$resource_group" --name "$app_name" -o json) > "$patch_file"
apply_template_patch

# The PATCH is asynchronous in Container Apps. Never read a still-old template
# as though it were a deployment result; wait for the exact image and durable
# topology before attempting a follow-up environment cleanup.
wait_for_effective_template

# Azure's template PATCH merges environment entries by name. A short-lived
# exact test credential used for a persistence drill must never survive the
# subsequent production deploy, even though it is not part of the contract.
actual="$(az containerapp show --resource-group "$resource_group" --name "$app_name" -o json)"
if jq -e '.properties.template.containers[0].env | any(.name == "TEST_AUTH_TOKEN")' <<<"$actual" >/dev/null; then
  az containerapp update --resource-group "$resource_group" --name "$app_name" \
    --remove-env-vars TEST_AUTH_TOKEN --only-show-errors >/dev/null
  wait_for_effective_template
fi

# Do not treat a successful PATCH as a successful deployment. This is the
# regression guard for the revision that kept only PORT and scaled SQLite to
# three replicas: read the effective Container App template back from Azure.
RESOURCE_GROUP="$resource_group" CONTAINER_APP_NAME="$app_name" \
  "$(dirname "$0")/verify-container-topology.sh"

wait_for_effective_template
actual="$(az containerapp show --resource-group "$resource_group" --name "$app_name" -o json)"
jq -e --arg image "$image" '
  .properties.template.containers[0].image == $image
' <<<"$actual" >/dev/null

jq '{
  revision: .properties.latestRevisionName,
  image: .properties.template.containers[0].image,
  min: .properties.template.scale.minReplicas,
  max: .properties.template.scale.maxReplicas,
  mounts: .properties.template.containers[0].volumeMounts,
  volumes: .properties.template.volumes,
  env: .properties.template.containers[0].env
}' <<<"$actual"
