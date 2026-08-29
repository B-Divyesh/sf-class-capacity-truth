#!/usr/bin/env bash
set -euo pipefail

# Wait until a newly-created Container Apps revision is both healthy and the
# traffic-serving revision. `latestRevisionName` changes before ingress shifts;
# callers that use a temporary credential must not send a request in that gap.
resource_group="${RESOURCE_GROUP:-sociobot}"
app_name="${CONTAINER_APP_NAME:-sf-class-capacity-truth}"
base_url="${BASE_URL:-https://class-capacity-truth.sociobot.in}"
previous_revision="${1:?Pass the revision name that preceded the update}"
max_attempts="${MAX_ATTEMPTS:-120}"
sleep_seconds="${SLEEP_SECONDS:-2}"

for _ in $(seq 1 "$max_attempts"); do
  expected="$(az containerapp show --resource-group "$resource_group" --name "$app_name" \
    --query properties.latestRevisionName -o tsv)"
  ready="$(az containerapp show --resource-group "$resource_group" --name "$app_name" \
    --query properties.latestReadyRevisionName -o tsv)"
  if [[ -n "$expected" && "$expected" != "$previous_revision" && "$ready" == "$expected" ]]; then
    health="$(az containerapp revision show --resource-group "$resource_group" --name "$app_name" \
      --revision "$expected" --query properties.healthState -o tsv)"
    provisioning="$(az containerapp show --resource-group "$resource_group" --name "$app_name" \
      --query properties.provisioningState -o tsv)"
    if [[ "$health" == "Healthy" && "$provisioning" == "Succeeded" ]] \
      && curl --silent --fail "$base_url/health" \
      | jq -e '.status == "ok" and .database == "ready"' >/dev/null; then
      printf '%s\n' "$expected"
      exit 0
    fi
  fi
  sleep "$sleep_seconds"
done

echo "new revision did not become healthy and traffic-serving" >&2
exit 1
