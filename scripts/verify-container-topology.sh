#!/usr/bin/env bash
set -euo pipefail

resource_group="${RESOURCE_GROUP:-sociobot}"
app_name="${CONTAINER_APP_NAME:-sf-class-capacity-truth}"
actual="$(az containerapp show -g "$resource_group" -n "$app_name" -o json)"
jq -e '
  .properties.template.scale.minReplicas == 1 and
  .properties.template.scale.maxReplicas == 1 and
  (.properties.template.volumes | any(.name == "cct-data" and .storageType == "AzureFile" and .storageName == "cct-data")) and
  (.properties.template.containers[0].volumeMounts | any(.volumeName == "cct-data" and .mountPath == "/mnt/cct")) and
  (.properties.template.containers[0].env | any(.name == "DATA_DIR" and .value == "/mnt/cct/keys")) and
  (.properties.template.containers[0].env | any(.name == "DURABLE_BACKUP_PATH" and .value == "/mnt/cct/snapshots/class-capacity-truth.db"))
' <<<"$actual" >/dev/null
echo "one replica, Azure Files mount, persisted keys, and durable snapshot path verified"
