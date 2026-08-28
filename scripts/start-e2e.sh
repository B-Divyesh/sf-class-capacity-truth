#!/usr/bin/env bash
set -euo pipefail

repo_dir="$(cd "$(dirname "$0")/.." && pwd)"
runtime_dir="$(mktemp -d)"
cleanup() { rm -rf "$runtime_dir"; }
trap cleanup EXIT

cd "$repo_dir"
npm run build:web
PORT=4173 \
DATA_DIR="$runtime_dir" \
FRONTEND_DIST="$repo_dir/dist" \
BUILD_SHA=e2e \
cargo run --manifest-path services/api/Cargo.toml
