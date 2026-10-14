# shellcheck shell=bash disable=SC2034,SC2153
# Undo logic for install.sh. Source this file, do not run it.
# It reads the progress markers that install.sh sets while it works.

note() {
  printf 'rollback: %s\n' "$1" >&2
}

restore_saved() {
  local saved="$1" original="$2" what="$3"
  mv -f "$saved" "$original" && note "restored the previous $what at $original"
}

restore_state_file() {
  if [[ $PREVIOUS_STATE_SAVED -eq 1 ]]; then
    restore_saved "$STATE_FILE.old" "$STATE_FILE" "state file"
  elif [[ $PREVIOUS_STATE_CHECKED -eq 1 ]]; then
    rm -f "$STATE_FILE" && note "removed the state file written during the install, because there was none before"
  fi
}

unload_new_grumpod() {
  launchctl bootout "system/$LABEL" >/dev/null 2>&1
  poll is_unloaded "$LABEL" || warn "$LABEL is still loaded after $WAIT_SECONDS seconds, the rollback continues anyway"
}

restore_previous_files() {
  [[ $PREVIOUS_BINARY_SAVED -eq 0 ]] || restore_saved "$BINARY_DEST.old" "$BINARY_DEST" "binary"
  [[ $PREVIOUS_PLIST_SAVED -eq 0 ]] || restore_saved "$PLIST_DEST.old" "$PLIST_DEST" "LaunchDaemon plist"
}

strip_new_hosts_block() {
  if "$BINARY_DEST" strip-hosts >/dev/null 2>&1; then
    note "removed the hosts block written by the new grumpod"
  else
    warn "could not remove the hosts block written by the new grumpod, run: sudo $(printf '%q' "$BINARY_DEST") strip-hosts"
  fi
}

remove_partial_install() {
  if [[ $NEW_DAEMON_STARTED -eq 1 && -x "$BINARY_DEST" ]]; then
    strip_new_hosts_block
  fi
  rm -f "$BINARY_DEST" "$PLIST_DEST"
  note "there was no previous grumpod, so the partly installed files were removed"
}

restart_with_hint() {
  local label="$1" plist="$2" what="$3"
  if launchctl enable "system/$label" && launchctl bootstrap system "$plist" 2>/dev/null; then
    note "started $what again as $label"
  else
    note "could not start $what, start it with: sudo launchctl bootstrap system $(printf '%q' "$plist")"
  fi
}

restart_previous_grumpod() {
  if [[ $PREVIOUS_RAN -eq 0 ]]; then
    note "the previous grumpod was not running before the install, so it was not started"
    return 0
  fi
  restart_with_hint "$LABEL" "$PLIST_DEST" "the previous grumpod"
}

rollback() {
  if [[ $CHANGES_STARTED -eq 0 ]]; then
    note "nothing had been changed yet"
    return 0
  fi
  unload_new_grumpod
  restore_state_file
  if [[ $PREVIOUS_BINARY_SAVED -eq 1 || $PREVIOUS_PLIST_SAVED -eq 1 ]]; then
    restore_previous_files
    restart_previous_grumpod
  else
    remove_partial_install
  fi
  restart_focusd
  restart_legacy
}

discard_previous_install() {
  rm -f "$BINARY_DEST.old" "$PLIST_DEST.old" "$STATE_FILE.old"
}

on_exit() {
  local status=$?
  set +e
  [[ $status -ne 0 ]] || {
    release_install_lock
    return 0
  }
  printf '\nInstall failed while: %s\n' "$CURRENT_STEP" >&2
  if [[ $INSTALL_LOCK_HELD -eq 1 ]]; then
    rm -f "$BINARY_DEST.new" "$PLIST_DEST.new"
    rollback
    discard_previous_install
    release_install_lock
  fi
  printf 'Fix the problem above and re-run this script, it is safe to repeat.\n' >&2
}
