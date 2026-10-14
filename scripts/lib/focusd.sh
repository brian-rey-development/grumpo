# shellcheck shell=bash disable=SC2034,SC2153
# Migration from focusd, the name of the daemon before grumpod. Source this file, do not run it.

readonly FOCUSD_LABEL="io.github.brian-rey-development.focusd"
readonly FOCUSD_PLIST="/Library/LaunchDaemons/${FOCUSD_LABEL}.plist"
readonly FOCUSD_BINARY="/Library/PrivilegedHelperTools/focusd"
readonly FOCUSD_STATE_FILE="/Library/Application Support/Focus/state.json"
readonly FOCUSD_DIRS=("/Library/Application Support/Focus" "/Library/Logs/Focus")
readonly FOCUSD_FILES=("$FOCUSD_PLIST" "$FOCUSD_BINARY" "/etc/newsyslog.d/focusd.conf" "/var/run/focusd.sock")

FOCUSD_STOPPED=0

has_focusd() {
  [[ -f "$FOCUSD_PLIST" || -f "$FOCUSD_BINARY" || -f "$FOCUSD_STATE_FILE" ]]
}

stop_focusd() {
  step "Stopping focusd, the earlier name of this service"
  FOCUSD_STOPPED=1
  stop_daemon "$FOCUSD_LABEL"
}

# Copied, not moved, so a failed install leaves focusd the state it restarts with. A state file
# grumpod already has is newer and wins.
carry_over_focusd_state() {
  if [[ -f "$STATE_FILE" || ! -f "$FOCUSD_STATE_FILE" ]]; then
    return 0
  fi
  step "Carrying the rules over from focusd"
  cp -p "$FOCUSD_STATE_FILE" "$STATE_FILE"
}

migrate_focusd() {
  has_focusd || return 0
  stop_focusd
  carry_over_focusd_state
}

remove_focusd_files() {
  rm -f "${FOCUSD_FILES[@]}"
}

remove_focusd() {
  has_focusd || return 0
  step "Removing the files of focusd"
  remove_focusd_files
  rm -rf "${FOCUSD_DIRS[@]}"
  FOCUSD_STOPPED=0
}

restart_focusd() {
  [[ $FOCUSD_STOPPED -eq 1 && -f "$FOCUSD_PLIST" ]] || return 0
  restart_with_hint "$FOCUSD_LABEL" "$FOCUSD_PLIST" "focusd"
}
