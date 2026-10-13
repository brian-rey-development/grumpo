#!/usr/bin/env bash
# shellcheck source-path=SCRIPTDIR
# Packs the signed dist/release/Grumpo.app into dist/release/Grumpo.dmg with an Applications link,
# plus a SHA-256 checksum. The name has no version so the download page can link to the latest
# release with a URL that never changes.
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
readonly DMG_PATH="$OUT_DIR/$APP_NAME.dmg"

usage() {
  cat <<USAGE
usage: scripts/package-dmg.sh

Packs $APP_PATH, built by scripts/build-release.sh, into $APP_NAME.dmg and writes its checksum.
USAGE
}

stage_volume() {
  local stage="$1"
  ditto "$APP_PATH" "$stage/$APP_NAME.app"
  ln -s /Applications "$stage/Applications"
}

STAGE=""

cleanup() {
  [[ -z "$STAGE" ]] || rm -rf "$STAGE"
}

create_dmg() {
  STAGE="$(mktemp -d)"
  stage_volume "$STAGE"
  rm -f "$DMG_PATH"
  hdiutil create -volname "$APP_NAME" -srcfolder "$STAGE" -fs HFS+ -format UDZO -ov "$DMG_PATH" >/dev/null
  hdiutil verify "$DMG_PATH" >/dev/null || fail "hdiutil could not verify $DMG_PATH"
}

write_checksum() {
  (cd "$OUT_DIR" && shasum -a 256 "$APP_NAME.dmg" >"$APP_NAME.dmg.sha256")
}

main() {
  [[ $# -eq 0 ]] || reject_argument "$1"
  trap cleanup EXIT
  [[ -d "$APP_PATH" ]] || fail "$APP_PATH is missing, run scripts/build-release.sh first"
  codesign --verify --deep --strict "$APP_PATH" || fail "$APP_PATH does not pass codesign --verify"
  step "Packing $APP_NAME.dmg"
  create_dmg
  write_checksum
  step "Wrote $DMG_PATH"
  cat "$OUT_DIR/$APP_NAME.dmg.sha256"
}

main "$@"
