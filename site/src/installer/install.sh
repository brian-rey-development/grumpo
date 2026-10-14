#!/bin/bash
# Installs the latest Grumpo release. Source at @REPO_URL@
#
#   GRUMPO_INSTALL_DIR  where Grumpo.app goes (default /Applications)
#   GRUMPO_NO_OPEN=1    do not open Grumpo after installing
#
# The checksum catches a broken download, not a tampered release.

set -euo pipefail

readonly APP_NAME='Grumpo'
readonly DMG_URL='@DMG_URL@'
readonly CHECKSUM_URL='@CHECKSUM_URL@'
readonly MIN_MACOS_MAJOR='@MIN_MACOS@'
readonly DEFAULT_INSTALL_DIR='/Applications'
readonly INSTALL_DIR="${GRUMPO_INSTALL_DIR:-$DEFAULT_INSTALL_DIR}"
readonly APP_PATH="$INSTALL_DIR/$APP_NAME.app"
readonly STAGING_PATH="$INSTALL_DIR/.$APP_NAME.app.installing"
readonly PREVIOUS_PATH="$INSTALL_DIR/.$APP_NAME.app.previous"
readonly QUIT_TIMEOUT_SECONDS=10

work_dir=''
mount_dir=''

say() {
  printf '%s\n' "$*"
}

fail() {
  printf '%s\n' "$*" >&2
  exit 1
}

cleanup() {
  if [ -n "$mount_dir" ]; then
    hdiutil detach -quiet "$mount_dir" 2>/dev/null || hdiutil detach -force -quiet "$mount_dir" 2>/dev/null || true
  fi
  rm -rf "$STAGING_PATH" "$PREVIOUS_PATH"
  if [ -n "$work_dir" ]; then
    rm -rf "$work_dir"
  fi
}

check_macos() {
  [ "$(uname -s)" = 'Darwin' ] || fail "$APP_NAME only runs on macOS."
  local version
  version="$(sw_vers -productVersion)"
  if [ "${version%%.*}" -lt "$MIN_MACOS_MAJOR" ]; then
    fail "$APP_NAME needs macOS $MIN_MACOS_MAJOR or later. This Mac runs macOS $version."
  fi
}

check_install_dir() {
  mkdir -p "$INSTALL_DIR" 2>/dev/null || fail "Could not create $INSTALL_DIR."
  [ -w "$INSTALL_DIR" ] || fail "You can't write to $INSTALL_DIR. Try again from an admin account."
}

fetch() {
  curl -fsSL --proto '=https' --proto-redir '=https' --tlsv1.2 --retry 3 -o "$2" "$1"
}

download() {
  say "Downloading $APP_NAME..."
  fetch "$DMG_URL" "$work_dir/$APP_NAME.dmg" ||
    fail "The download failed. Check your connection and try again."
  fetch "$CHECKSUM_URL" "$work_dir/$APP_NAME.dmg.sha256" ||
    fail "The checksum download failed. Check your connection and try again."
}

verify() {
  say "Checking the download..."
  grep -Eq "^[0-9a-f]{64}  $APP_NAME\.dmg$" "$work_dir/$APP_NAME.dmg.sha256" ||
    fail "The checksum file looks wrong. Try again later."
  (cd "$work_dir" && shasum -a 256 -c "$APP_NAME.dmg.sha256" >/dev/null 2>&1) ||
    fail "The download is damaged. Try again."
}

attach() {
  say "Opening the disk image..."
  mkdir "$work_dir/mount"
  hdiutil attach -nobrowse -readonly -noautoopen -quiet -mountpoint "$work_dir/mount" "$work_dir/$APP_NAME.dmg" ||
    fail "Could not open the disk image."
  mount_dir="$work_dir/mount"
  [ -d "$mount_dir/$APP_NAME.app" ] || fail "The disk image has no $APP_NAME.app."
}

is_running() {
  local processes
  processes="$(ps -axo comm=)"
  grep -Fq -- "$APP_PATH/Contents/MacOS/" <<<"$processes"
}

quit_running_app() {
  is_running || return 0
  say "Quitting $APP_NAME..."
  osascript -e "quit app \"$APP_NAME\"" >/dev/null 2>&1 || true
  local waited=0
  while is_running && [ "$waited" -lt "$QUIT_TIMEOUT_SECONDS" ]; do
    sleep 1
    waited=$((waited + 1))
  done
  ! is_running || fail "$APP_NAME is still running. Quit it and run the installer again."
}

install_app() {
  say "Installing $APP_NAME in $INSTALL_DIR..."
  rm -rf "$STAGING_PATH" "$PREVIOUS_PATH"
  ditto "$mount_dir/$APP_NAME.app" "$STAGING_PATH" || fail "Could not copy $APP_NAME to $INSTALL_DIR."
  if [ -e "$APP_PATH" ]; then
    mv "$APP_PATH" "$PREVIOUS_PATH" || fail "Could not replace the old $APP_NAME. Quit it and try again."
  fi
  if ! mv "$STAGING_PATH" "$APP_PATH"; then
    [ ! -e "$PREVIOUS_PATH" ] || mv "$PREVIOUS_PATH" "$APP_PATH"
    fail "Could not install $APP_NAME. The previous version was kept."
  fi
  rm -rf "$PREVIOUS_PATH"
}

detach() {
  hdiutil detach -quiet "$mount_dir" || true
  mount_dir=''
}

main() {
  check_macos
  check_install_dir
  trap cleanup EXIT
  work_dir="$(mktemp -d "${TMPDIR:-/tmp}/grumpo-install.XXXXXX")"
  download
  verify
  attach
  quit_running_app
  install_app
  detach
  say "$APP_NAME is installed in $INSTALL_DIR."
  if [ "${GRUMPO_NO_OPEN:-}" != '1' ]; then
    open "$APP_PATH"
  fi
}

main "$@"
