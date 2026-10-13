#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
GENERATED_DIR="$ROOT/app/src/ipc/generated"

cd "$ROOT"

find "$GENERATED_DIR" -mindepth 1 ! -name .gitkeep -delete
cargo test --workspace --all-features
