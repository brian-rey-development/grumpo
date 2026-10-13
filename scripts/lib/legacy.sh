# shellcheck shell=bash disable=SC2034,SC2153
# Migration from the old focus-hosts setup. Source this file, do not run it.

readonly LEGACY_LABEL="com.focus.hosts"
readonly LEGACY_PLIST="/Library/LaunchDaemons/${LEGACY_LABEL}.plist"
readonly LEGACY_SCRIPT="/usr/local/bin/focus-hosts"
readonly LEGACY_STATE_DIR="/Library/Application Support/focus-hosts"

SKIP_LEGACY_IMPORT=0
LEGACY_STOPPED=0

has_legacy() {
  [[ -f "$LEGACY_PLIST" || -f "$LEGACY_SCRIPT" ]]
}

import_legacy_rules() {
  if [[ $SKIP_LEGACY_IMPORT -eq 1 ]]; then
    warn "skipping the import, the old focus-hosts rules will not be carried over"
    return 0
  fi
  if [[ ! -f "$LEGACY_SCRIPT" ]]; then
    warn "$LEGACY_SCRIPT is missing, so the old rules cannot be imported"
    return 0
  fi
  step "Importing rules from the old focus-hosts script"
  "$BINARY_DEST" import-legacy "$LEGACY_SCRIPT" ||
    fail "importing the old focus-hosts rules failed, see the message above. Fix the cause and re-run, or pass --skip-legacy-import to install without carrying them over"
}

migrate_legacy() {
  has_legacy || return 0
  import_legacy_rules
  step "Stopping the old focus-hosts daemon"
  LEGACY_STOPPED=1
  stop_daemon "$LEGACY_LABEL"
}

remove_legacy() {
  has_legacy || return 0
  step "Removing the old focus-hosts files"
  rm -f "$LEGACY_PLIST" "$LEGACY_SCRIPT"
  rm -rf "$LEGACY_STATE_DIR"
  LEGACY_STOPPED=0
}

restart_legacy() {
  [[ $LEGACY_STOPPED -eq 1 && -f "$LEGACY_PLIST" ]] || return 0
  restart_with_hint "$LEGACY_LABEL" "$LEGACY_PLIST" "the old focus-hosts daemon"
}
