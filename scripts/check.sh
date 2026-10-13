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

on_disk_files() {
  find . \( -name .git -o -name node_modules -o -name target -o -name dist -o -name .progress -o -name .astro \) -prune \
    -o \( -type f -o -type l \) -print | sed 's|^\./||'
}

# macOS resolves file names case-insensitively, so a path that git tracks with a
# different casing than the file on disk only breaks on the Linux runners.
check_casing() {
  local mismatched
  mismatched="$(LC_ALL=C comm -23 <(git ls-files | LC_ALL=C sort) <(on_disk_files | LC_ALL=C sort) |
    while IFS= read -r path; do [[ -e "$path" ]] && printf '%s\n' "$path"; done)"
  [[ -z "$mismatched" ]] && return
  echo "Git tracks these paths with a different casing than the files on disk:" >&2
  echo "$mismatched" >&2
  echo "Fix each one with: git mv -f <tracked path> <path on disk>" >&2
  exit 1
}

cd "$ROOT"

gate "file name casing"
check_casing

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
