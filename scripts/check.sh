#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
GENERATED_DIR="$ROOT/app/src/ipc/generated"
FRESH_DIR="$(mktemp -d)"
trap 'rm -rf "$FRESH_DIR"' EXIT

gate() {
  printf '\n==> %s\n' "$1"
}

check_bindings() {
  local status=0
  diff -r -x .gitkeep "$FRESH_DIR" "$GENERATED_DIR" || status=$?
  case "$status" in
    0) return ;;
    1)
      echo "Generated bindings are stale. Regenerate them with:" >&2
      echo "  scripts/gen-bindings.sh" >&2
      exit 1
      ;;
    *)
      echo "diff failed with exit code $status" >&2
      exit "$status"
      ;;
  esac
}

cd "$ROOT"

gate "shellcheck"
if command -v shellcheck >/dev/null 2>&1; then
  shellcheck -x scripts/*.sh scripts/lib/*.sh
else
  echo "shellcheck is not installed, skipping (CI runs it)"
fi

gate "cargo fmt"
cargo fmt --all --check

gate "cargo clippy"
cargo clippy --workspace --all-targets --all-features -- -D warnings

gate "cargo clippy (shipped daemon configuration)"
cargo clippy -p focus-core --all-targets -- -D warnings

gate "cargo test"
TS_RS_EXPORT_DIR="$FRESH_DIR" cargo test --workspace --all-features

gate "generated bindings are current"
check_bindings

cd "$ROOT/app"

gate "pnpm install"
pnpm install --frozen-lockfile

for script in typecheck lint format:check test; do
  gate "pnpm $script"
  pnpm "$script"
done

gate "all checks passed"
