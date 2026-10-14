#!/usr/bin/env bash
# shellcheck source-path=SCRIPTDIR
# Packs dist/release/Grumpo.app into the archive the in-app updater downloads, signs it with the
# updater key and writes latest.json. Installed copies read that manifest from the latest
# published release, so a draft reaches nobody until it is published.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly SCRIPT_DIR
# shellcheck source=lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"

readonly APP_NAME="Grumpo"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
readonly ROOT
readonly OUT_DIR="$ROOT/dist/release"
readonly APP_PATH="$OUT_DIR/$APP_NAME.app"
readonly ARCHIVE="$APP_PATH.tar.gz"
readonly MANIFEST="$OUT_DIR/latest.json"
readonly REPOSITORY="${GITHUB_REPOSITORY:-brian-rey-development/grumpo}"

usage() {
  cat <<USAGE
usage: scripts/package-update.sh

Packs $APP_PATH, built by scripts/build-release.sh, into $APP_NAME.app.tar.gz, signs it with the
updater key and writes latest.json.

Environment:
  TAURI_SIGNING_PRIVATE_KEY, TAURI_SIGNING_PRIVATE_KEY_PATH   The updater key, as text or as a file.
  TAURI_SIGNING_PRIVATE_KEY_PASSWORD                          Its password.
  GITHUB_REPOSITORY                                           The repository that hosts the release.
USAGE
}

# Both platforms point to the one universal archive.
platform() {
  printf '"%s": { "signature": "%s", "url": "%s" }' "$1" "$2" "$3"
}

app_version() {
  node -p "require(process.argv[1]).version" "$ROOT/app/package.json"
}

pack_archive() {
  rm -f "$ARCHIVE" "$ARCHIVE.sig"
  COPYFILE_DISABLE=1 tar -czf "$ARCHIVE" -C "$OUT_DIR" "$APP_NAME.app"
}

sign_archive() {
  (cd "$ROOT/app" && pnpm -s tauri signer sign "$ARCHIVE" >/dev/null)
  [[ -s "$ARCHIVE.sig" ]] || fail "the signer wrote no signature"
}

write_manifest() {
  local version="$1" signature url
  signature="$(<"$ARCHIVE.sig")"
  url="https://github.com/$REPOSITORY/releases/download/v$version/$APP_NAME.app.tar.gz"
  cat >"$MANIFEST" <<JSON
{
  "version": "$version",
  "pub_date": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "platforms": {
    $(platform darwin-aarch64 "$signature" "$url"),
    $(platform darwin-x86_64 "$signature" "$url")
  }
}
JSON
  node -e "JSON.parse(require('fs').readFileSync(process.argv[1], 'utf8'))" "$MANIFEST" ||
    fail "$MANIFEST is not valid JSON"
}

main() {
  [[ $# -eq 0 ]] || reject_argument "$1"
  [[ -d "$APP_PATH" ]] || fail "$APP_PATH is missing, run scripts/build-release.sh first"
  [[ -n "${TAURI_SIGNING_PRIVATE_KEY:-}${TAURI_SIGNING_PRIVATE_KEY_PATH:-}" ]] ||
    fail "set TAURI_SIGNING_PRIVATE_KEY or TAURI_SIGNING_PRIVATE_KEY_PATH, see scripts/create-release-keys.sh"
  codesign --verify --deep --strict "$APP_PATH" || fail "$APP_PATH does not pass codesign --verify"
  step "Packing $APP_NAME.app.tar.gz"
  pack_archive
  step "Signing the archive"
  sign_archive
  step "Writing latest.json"
  write_manifest "$(app_version)"
  step "Wrote $ARCHIVE, its signature and $MANIFEST"
}

main "$@"
