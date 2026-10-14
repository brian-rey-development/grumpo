#!/usr/bin/env bash
# shellcheck source-path=SCRIPTDIR
# Removes the grumpod LaunchDaemon, its managed hosts block and the Grumpo login item.
# State and logs are kept unless --purge.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly SCRIPT_DIR
# shellcheck source=lib/layout.sh
source "$SCRIPT_DIR/lib/layout.sh"
# shellcheck source=lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"
# shellcheck source=lib/focusd.sh
source "$SCRIPT_DIR/lib/focusd.sh"

PURGE=0
OWNER_ARG=""
OWNER_UID=""
OWNER_HOME=""

usage() {
  cat <<USAGE
usage: sudo scripts/uninstall.sh [--purge] [--owner <user|uid>]

Stops grumpod, removes the Grumpo block from /etc/hosts, deletes the daemon files and
removes the Grumpo login item of the owner, which is the user who ran sudo unless
--owner says otherwise. State and logs are kept unless --purge is given.
USAGE
}

on_exit() {
  local status=$?
  [[ $status -ne 0 ]] || return 0
  printf '\nUninstall failed while: %s\n' "$CURRENT_STEP" >&2
  printf 'Some files may remain. Fix the problem above and re-run this script, it is safe to repeat.\n' >&2
}
trap on_exit EXIT

parse_args() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --purge)
        PURGE=1
        shift
        ;;
      --owner)
        require_value "$1" $#
        OWNER_ARG="$2"
        shift 2
        ;;
      -h | --help)
        usage
        exit 0
        ;;
      *) reject_argument "$1" ;;
    esac
  done
}

strip_hosts() {
  step "Removing the Grumpo block from /etc/hosts"
  # The daemon is already stopped here, which strip-hosts requires because it shares the daemon's instance lock.
  if [[ -x "$BINARY_DEST" ]]; then
    "$BINARY_DEST" strip-hosts
  elif [[ -x "$FOCUSD_BINARY" ]]; then
    "$FOCUSD_BINARY" strip-hosts
  else
    warn "$BINARY_DEST is missing, remove the \"# >>> grumpo\" block from /etc/hosts by hand"
  fi
}

remove_files() {
  step "Removing the daemon files"
  rm -f "$PLIST_DEST" "$PLIST_DEST.old" "$SOCKET" "$NEWSYSLOG_DEST" "$BINARY_DEST" "$BINARY_DEST.old"
  remove_focusd_files
}

remove_login_item() {
  step "Removing the Grumpo login item"
  local agents="$OWNER_HOME/Library/LaunchAgents" label
  rm -f "$agents/$APP_LABEL.plist"
  for label in "${LEGACY_APP_LABELS[@]}"; do
    remove_legacy_login_item "$agents/$label.plist"
  done
}

# "Focus" is a common name, so a property list under a legacy label is only removed when it
# starts Focus.app or Grumpo.app.
remove_legacy_login_item() {
  local plist="$1" program
  [[ -f "$plist" ]] || return 0
  program="$(plist_program "$plist")"
  case "$program" in
    */Focus.app/* | */Grumpo.app/*) rm -f "$plist" ;;
    *) warn "left $plist alone, it does not start Focus.app or Grumpo.app" ;;
  esac
}

purge_data() {
  step "Deleting state and logs"
  rm -rf "$STATE_DIR" "$LOG_DIR" "${FOCUSD_DIRS[@]}"
}

main() {
  parse_args "$@"
  require_macos_root
  resolve_owner "$OWNER_ARG"
  step "Stopping grumpod"
  stop_daemon "$LABEL"
  if has_focusd; then
    stop_focusd
  fi
  strip_hosts
  remove_files
  remove_login_item
  [[ $PURGE -eq 0 ]] || purge_data
  CURRENT_STEP="finished"
  printf '\ngrumpod is removed.\n'
  if [[ $PURGE -eq 0 ]]; then
    printf 'State kept in %s and logs in %s, add --purge to delete them.\n' "$STATE_DIR" "$LOG_DIR"
  fi
}

main "$@"
