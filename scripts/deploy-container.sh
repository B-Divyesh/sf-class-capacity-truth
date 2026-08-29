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

account_key="$(az storage account keys list --resource-group "$resource_group" --account-name "$storage_account" --query '[0].value' -o tsv)"
az storage share create --account-name "$storage_account" --account-key "$account_key" --name "$share_name" --quota 5 --only-show-errors >/dev/null
az containerapp env storage set --resource-group "$resource_group" --name "$environment_name" \
  --storage-name "$storage_name" --azure-file-account-name "$storage_account" \
  --azure-file-account-key "$account_key" --azure-file-share-name "$share_name" \
  --access-mode ReadWrite --only-show-errors >/dev/null

app_id="$(az containerapp show --resource-group "$resource_group" --name "$app_name" --query id -o tsv)"
patch_file="$(mktemp)"
trap 'rm -f "$patch_file"' EXIT
jq --arg image "$image" '
  { properties: { template: (
    .properties.template |
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
az rest --method PATCH --uri "${app_id}?api-version=2024-03-01" --body "@$patch_file" --only-show-errors >/dev/null

# Do not treat a successful PATCH as a successful deployment. This is the
# regression guard for the revision that kept only PORT and scaled SQLite to
# three replicas: read the effective Container App template back from Azure.
RESOURCE_GROUP="$resource_group" CONTAINER_APP_NAME="$app_name" \
  "$(dirname "$0")/verify-container-topology.sh"

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
