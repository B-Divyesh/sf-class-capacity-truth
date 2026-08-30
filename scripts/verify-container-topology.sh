#!/usr/bin/env bash
set -euo pipefail

resource_group="${RESOURCE_GROUP:-sociobot}"
app_name="${CONTAINER_APP_NAME:-sf-class-capacity-truth}"
storage_name="${DATA_STORAGE_NAME:-sf-class-capacity-truth-data}"
actual="$(az containerapp show -g "$resource_group" -n "$app_name" -o json)"
jq -e --arg storage_name "$storage_name" '
  .properties.template.scale.minReplicas == 1 and
  .properties.template.scale.maxReplicas == 1 and
  (.properties.template.volumes | any(.name == "data" and .storageType == "AzureFile" and .storageName == $storage_name)) and
  (.properties.template.containers[0].volumeMounts | any(.volumeName == "data" and .mountPath == "/data")) and
  (.properties.template.containers[0].env | any(.name == "PORT" and .value == "8080")) and
  ((.properties.template.containers[0].env | any(.name == "DATA_DIR" or .name == "DURABLE_BACKUP_PATH")) | not)
' <<<"$actual" >/dev/null
echo "one replica and the Azure Files /data mount for SQLite and generated keys verified"
