#!/usr/bin/env bash
set -euo pipefail

base_url="${1:-http://127.0.0.1:8080}"
results_file="$(mktemp)"
headers_dir="$(mktemp -d)"
cleanup() { rm -f "$results_file"; rm -rf "$headers_dir"; }
trap cleanup EXIT

seq 1 100 | xargs -P 25 -I '{}' sh -c \
  'curl -sS -D "$2/{}.headers" -o /dev/null -w "%{http_code}\n" -H "X-Forwarded-For: 198.51.100.240" "$1/api/demo/session"' \
  _ "$base_url" "$headers_dir" > "$results_file"

grep -q '^200$' "$results_file"
grep -q '^429$' "$results_file"
test "$(wc -l < "$results_file")" -eq 100
test "$(grep -Ec '^(200|429)$' "$results_file")" -eq 100
grep -qi '^retry-after:' "$headers_dir"/*.headers
printf '100 requests completed: %s accepted, %s rate-limited\n' \
  "$(grep -c '^200$' "$results_file")" "$(grep -c '^429$' "$results_file")"
