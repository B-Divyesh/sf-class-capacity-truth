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

cleanup() {
  # Never leave the temporary production auth bypass behind, even if a request
  # or revision fails. Failures here are intentionally quiet so the original
  # failing command keeps its exit status; operators can inspect Azure history.
  if [[ -n "$workspace_key" && "$token_attached" == true ]]; then
    curl --silent --output /dev/null --max-time 20 -X DELETE "$base_url/api/workspaces/data" \
      -H "Authorization: Bearer $token" -H "X-Workspace-Key: $workspace_key" || true
  fi
  if [[ "$token_attached" == true ]]; then
    az containerapp update --resource-group "$resource_group" --name "$app_name" \
      --remove-env-vars TEST_AUTH_TOKEN --revision-suffix "d-c-$drill_id" \
      --only-show-errors >/dev/null 2>&1 || true
  fi
  if [[ "$token_created" == true ]]; then
    az containerapp secret remove --resource-group "$resource_group" --name "$app_name" \
      --secret-names "$secret_name" --only-show-errors >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

wait_for_revision() {
  local previous="$1"
  local expected=""
  for _ in $(seq 1 120); do
    expected="$(az containerapp show --resource-group "$resource_group" --name "$app_name" \
      --query properties.latestRevisionName -o tsv)"
    if [[ -n "$expected" && "$expected" != "$previous" ]]; then
      local state
      state="$(az containerapp revision show --resource-group "$resource_group" --name "$app_name" \
        --revision "$expected" --query properties.healthState -o tsv)"
      if [[ "$state" == "Healthy" ]] && curl --silent --fail "$base_url/health" \
        | jq -e '.status == "ok" and .database == "ready"' >/dev/null; then
        printf '%s\n' "$expected"
        return 0
      fi
    fi
    sleep 2
  done
  echo "new revision did not become healthy" >&2
  return 1
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
az containerapp update --resource-group "$resource_group" --name "$app_name" \
  --set-env-vars "TEST_AUTH_TOKEN=secretref:$secret_name" \
  --revision-suffix "d-a-$drill_id" --only-show-errors >/dev/null
token_attached=true
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
az containerapp update --resource-group "$resource_group" --name "$app_name" \
  --remove-env-vars TEST_AUTH_TOKEN --revision-suffix "d-c-$drill_id" \
  --only-show-errors >/dev/null
token_attached=false
final_revision="$(wait_for_revision "$before")"
az containerapp secret remove --resource-group "$resource_group" --name "$app_name" \
  --secret-names "$secret_name" --only-show-errors >/dev/null
token_created=false
RESOURCE_GROUP="$resource_group" CONTAINER_APP_NAME="$app_name" \
  "$(dirname "$0")/verify-container-topology.sh"
az containerapp show --resource-group "$resource_group" --name "$app_name" -o json \
  | jq -e '(.properties.template.containers[0].env | any(.name == "TEST_AUTH_TOKEN")) | not' >/dev/null

printf 'durability drill passed: auth=%s restart=%s cleanup=%s; synthetic workspace removed and temporary credential removed\n' \
  "$auth_revision" "$restart_revision" "$final_revision"
