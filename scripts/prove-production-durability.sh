#!/usr/bin/env bash
set -euo pipefail

# Explicit, controlled production recovery drill. It uses a CSPRNG one-time
# test token stored only as a Container App secret, creates a synthetic school
# record, rolls a new revision, verifies the public capacity and encrypted
# guardian contact after that revision, deletes the synthetic workspace, and
# rolls a final revision with the temporary credential removed. Do not add this
# to npm test: it intentionally changes the live Container App.
resource_group="${RESOURCE_GROUP:-sociobot}"
app_name="${CONTAINER_APP_NAME:-sf-class-capacity-truth}"
base_url="${BASE_URL:-https://class-capacity-truth.sociobot.in}"
secret_name="cct-persist-drill"
# Container App revision names include the app name and `--`, so keep this
# unique suffix below the combined 54-character Azure limit.
drill_id="$(date -u +%s)-$RANDOM"
token="$(openssl rand -hex 32)"
token_created=false
token_attached=false
workspace_key=""

patch_test_token() {
  local mode="$1"
  local suffix="$2"
  local app_id body response
  app_id="$(az containerapp show --resource-group "$resource_group" --name "$app_name" --query id -o tsv)"
  body="$(az containerapp show --resource-group "$resource_group" --name "$app_name" -o json \
    | jq --arg mode "$mode" --arg suffix "$suffix" --arg secret "$secret_name" '
        {properties: {template: (
          .properties.template
          # `az containerapp show` expands server-default scale fields that
          # this PATCH API rejects. Preserve the one-replica contract without
          # sending those read-only defaults back to Azure.
          | .scale = {minReplicas: 1, maxReplicas: 1}
          | .revisionSuffix = $suffix
          | .containers[0].env |= (
              map(select(.name != "TEST_AUTH_TOKEN"))
              + (if $mode == "attach" then [{name:"TEST_AUTH_TOKEN", secretRef:$secret}] else [] end)
            )
        )}}
      ')"
  # A secret update and a template update are separate Azure operations. The
  # control plane can accept the first before it permits the second, so retry
  # only that documented transient conflict rather than racing it.
  for _ in $(seq 1 60); do
    if response="$(az rest --method PATCH --uri "${app_id}?api-version=2024-03-01" \
      --body "$body" --only-show-errors 2>&1)"; then
      return 0
    fi
    if grep -q 'ContainerAppOperationInProgress' <<<"$response"; then
      sleep 5
      continue
    fi
    printf '%s\n' "$response" >&2
    return 1
  done
  echo "Container Apps did not accept the $mode template patch in time" >&2
  return 1
}

remove_test_secret() {
  local response
  for _ in $(seq 1 60); do
    if response="$(az containerapp secret remove --resource-group "$resource_group" --name "$app_name" \
      --secret-names "$secret_name" --only-show-errors 2>&1)"; then
      return 0
    fi
    if grep -q 'ContainerAppOperationInProgress' <<<"$response"; then
      sleep 5
      continue
    fi
    # A failed earlier cleanup may already have removed the one-time secret.
    if grep -q 'SecretRef.*not found' <<<"$response"; then
      return 0
    fi
    printf '%s\n' "$response" >&2
    return 1
  done
  echo "Container Apps did not remove the one-time drill secret in time" >&2
  return 1
}

wait_for_test_token_reference() {
  for _ in $(seq 1 60); do
    if az containerapp show --resource-group "$resource_group" --name "$app_name" -o json \
      | jq -e --arg secret "$secret_name" '
          .properties.template.containers[0].env
          | any(.name == "TEST_AUTH_TOKEN" and .secretRef == $secret)
        ' >/dev/null; then
      return 0
    fi
    sleep 2
  done
  echo "temporary test token reference did not appear in the effective template" >&2
  return 1
}

cleanup() {
  # Never leave the temporary production auth bypass behind, even if a request
  # or revision fails. Failures here are intentionally quiet so the original
  # failing command keeps its exit status; operators can inspect Azure history.
  if [[ -n "$workspace_key" && "$token_attached" == true ]]; then
    curl --silent --output /dev/null --max-time 20 -X DELETE "$base_url/api/workspaces/data" \
      -H "Authorization: Bearer $token" -H "X-Workspace-Key: $workspace_key" || true
  fi
  if [[ "$token_attached" == true ]]; then
    patch_test_token detach "d-c-$drill_id" >/dev/null 2>&1 || true
  fi
  if [[ "$token_created" == true ]]; then
    remove_test_secret >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

wait_for_revision() {
  RESOURCE_GROUP="$resource_group" CONTAINER_APP_NAME="$app_name" BASE_URL="$base_url" \
    "$(dirname "$0")/wait-for-containerapp-revision.sh" "$1"
}

stop_revision_for_sqlite_restart() {
  az containerapp revision deactivate --resource-group "$resource_group" --name "$app_name" \
    --revision "$1" --only-show-errors >/dev/null
}

current="$(az containerapp show --resource-group "$resource_group" --name "$app_name" -o json)"
if jq -e '.properties.template.containers[0].env | any(.name == "TEST_AUTH_TOKEN")' <<<"$current" >/dev/null; then
  echo "refusing to replace an existing TEST_AUTH_TOKEN" >&2
  exit 1
fi
if az containerapp secret list --resource-group "$resource_group" --name "$app_name" -o json \
  | jq -e --arg name "$secret_name" 'any(.[]; .name == $name)' >/dev/null; then
  echo "refusing to replace an existing $secret_name secret" >&2
  exit 1
fi

RESOURCE_GROUP="$resource_group" CONTAINER_APP_NAME="$app_name" \
  "$(dirname "$0")/verify-container-topology.sh"

az containerapp secret set --resource-group "$resource_group" --name "$app_name" \
  --secrets "$secret_name=$token" --only-show-errors >/dev/null
token_created=true
before="$(az containerapp show --resource-group "$resource_group" --name "$app_name" \
  --query properties.latestRevisionName -o tsv)"
patch_test_token attach "d-a-$drill_id"
token_attached=true
wait_for_test_token_reference
stop_revision_for_sqlite_restart "$before"
auth_revision="$(wait_for_revision "$before")"

workspace="$(curl --silent --show-error --fail-with-body -X POST "$base_url/api/workspaces" \
  -H "Authorization: Bearer $token" -H 'Content-Type: application/json' \
  --data '{"schoolName":"Synthetic persistence drill school"}')"
workspace_key="$(jq -r '.accessKey' <<<"$workspace")"
test -n "$workspace_key" && [[ "$workspace_key" != "null" ]]
now="$(date +%s)"
class="$(curl --silent --show-error --fail-with-body -X POST "$base_url/api/workspaces/classes" \
  -H "Authorization: Bearer $token" -H "X-Workspace-Key: $workspace_key" \
  -H 'Content-Type: application/json' \
  --data "{\"name\":\"Synthetic persistence drill class\",\"startsAt\":$((now + 172800)),\"bookingCutoff\":$((now + 86400)),\"timezone\":\"Etc/UTC\",\"capacity\":2}")"
class_id="$(jq -r '.id' <<<"$class")"
public_id="$(jq -r '.publicId' <<<"$class")"
curl --silent --show-error --fail-with-body -X POST \
  "$base_url/api/workspaces/classes/$class_id/publish" \
  -H "Authorization: Bearer $token" -H "X-Workspace-Key: $workspace_key" >/dev/null
curl --silent --show-error --fail-with-body -X POST "$base_url/api/classes/$public_id/book" \
  -H 'Content-Type: application/json' -H 'Idempotency-Key: cct-persistence-drill-booking' \
  --data '{"guardianName":"Synthetic Drill Guardian","guardianEmail":"persistence-drill@example.invalid"}' \
  | jq -e '.confirmed == 1 and .openSeats == 1' >/dev/null

# A suffix change creates a new revision without changing data paths, mounts,
# or the temporary exact credential used only to read the decrypted contact.
before="$auth_revision"
az containerapp update --resource-group "$resource_group" --name "$app_name" \
  --revision-suffix "d-r-$drill_id" --only-show-errors >/dev/null
stop_revision_for_sqlite_restart "$before"
restart_revision="$(wait_for_revision "$before")"
RESOURCE_GROUP="$resource_group" CONTAINER_APP_NAME="$app_name" \
  "$(dirname "$0")/verify-container-topology.sh"
curl --silent --show-error --fail-with-body "$base_url/api/classes/$public_id" \
  | jq -e '.confirmed == 1 and .openSeats == 1' >/dev/null
curl --silent --show-error --fail-with-body "$base_url/api/workspaces/classes/$class_id/bookings" \
  -H "Authorization: Bearer $token" -H "X-Workspace-Key: $workspace_key" \
  | jq -e 'length == 1 and .[0].guardianName == "Synthetic Drill Guardian" and .[0].guardianEmail == "persistence-drill@example.invalid"' >/dev/null

curl --silent --show-error --fail-with-body -X DELETE "$base_url/api/workspaces/data" \
  -H "Authorization: Bearer $token" -H "X-Workspace-Key: $workspace_key" >/dev/null
workspace_key=""
status="$(curl --silent --output /dev/null --write-out '%{http_code}' "$base_url/api/classes/$public_id")"
[[ "$status" == "404" ]]

before="$restart_revision"
patch_test_token detach "d-c-$drill_id"
token_attached=false
stop_revision_for_sqlite_restart "$before"
final_revision="$(wait_for_revision "$before")"
remove_test_secret
token_created=false
RESOURCE_GROUP="$resource_group" CONTAINER_APP_NAME="$app_name" \
  "$(dirname "$0")/verify-container-topology.sh"
az containerapp show --resource-group "$resource_group" --name "$app_name" -o json \
  | jq -e '(.properties.template.containers[0].env | any(.name == "TEST_AUTH_TOKEN")) | not' >/dev/null

printf 'durability drill passed: auth=%s restart=%s cleanup=%s; synthetic workspace removed and temporary credential removed\n' \
  "$auth_revision" "$restart_revision" "$final_revision"
