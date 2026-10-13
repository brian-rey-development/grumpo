#!/usr/bin/env bash
# shellcheck source-path=SCRIPTDIR
# Installs or upgrades the focusd LaunchDaemon. Run with sudo from the user who owns the Grumpo app,
# or let Grumpo.app run it through the macOS administrator prompt, which sets no SUDO_UID, a minimal
# PATH and no terminal, and so passes --binary and --owner.
set -euo pipefail

# Fixed because this runs as root, and the prompt of Grumpo.app gives it little more than this.
export PATH="/usr/bin:/bin:/usr/sbin:/sbin"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly SCRIPT_DIR
# shellcheck source=lib/layout.sh
source "$SCRIPT_DIR/lib/layout.sh"
# shellcheck source=lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"
# shellcheck source=lib/legacy.sh
source "$SCRIPT_DIR/lib/legacy.sh"
# shellcheck source=lib/rollback.sh
source "$SCRIPT_DIR/lib/rollback.sh"

ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
readonly ROOT
readonly PLIST_TEMPLATE="$ROOT/packaging/macos/${LABEL}.plist"
readonly NEWSYSLOG_SOURCE="$ROOT/packaging/macos/focusd.newsyslog.conf"

BINARY_SOURCE="$ROOT/target/release/focusd"
OWNER_ARG=""
OWNER_UID=""

# Progress markers that on_exit reads to decide what a failed install has to undo.
CHANGES_STARTED=0
PREVIOUS_RAN=0
PREVIOUS_BINARY_SAVED=0
PREVIOUS_PLIST_SAVED=0
PREVIOUS_STATE_SAVED=0
PREVIOUS_STATE_CHECKED=0
NEW_DAEMON_STARTED=0

usage() {
  cat <<USAGE
usage: sudo scripts/install.sh [--binary <path>] [--owner <user|uid>] [--skip-legacy-import]

Installs focusd as a LaunchDaemon that accepts connections only from its owner.
The owner is the user who ran sudo, or the one given with --owner.
Without --binary it installs target/release/focusd. Grumpo.app passes the focusd it carries.
With --skip-legacy-import the rules of an old focus-hosts install are not carried over,
and its files are still removed.
USAGE
}

trap on_exit EXIT
trap 'exit 130' INT
trap 'exit 143' TERM HUP

parse_args() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --binary)
        require_value "$1" $#
        BINARY_SOURCE="$2"
        shift 2
        ;;
      --owner)
        require_value "$1" $#
        OWNER_ARG="$2"
        shift 2
        ;;
      --skip-legacy-import)
        SKIP_LEGACY_IMPORT=1
        shift
        ;;
      -h | --help)
        usage
        exit 0
        ;;
      *) reject_argument "$1" ;;
    esac
  done
}

preflight() {
  step "Checking prerequisites"
  require_macos_root
  acquire_install_lock
  resolve_owner "$OWNER_ARG"
  [[ -f "$BINARY_SOURCE" && -x "$BINARY_SOURCE" ]] || fail "no executable focusd at $BINARY_SOURCE, build it with: scripts/build-release.sh"
  [[ -f "$PLIST_TEMPLATE" ]] || fail "missing $PLIST_TEMPLATE"
  [[ -f "$NEWSYSLOG_SOURCE" ]] || fail "missing $NEWSYSLOG_SOURCE"
}

save_if_present() {
  local file="$1" marker="$2"
  [[ -f "$file" ]] || return 0
  cp -p "$file" "$file.old"
  printf -v "$marker" 1
}

save_previous_install() {
  step "Keeping the current binary and plist so a failure can be undone"
  discard_previous_install
  if launchctl print "system/$LABEL" >/dev/null 2>&1; then
    PREVIOUS_RAN=1
  fi
  save_if_present "$BINARY_DEST" PREVIOUS_BINARY_SAVED
  save_if_present "$PLIST_DEST" PREVIOUS_PLIST_SAVED
}

stop_previous_focusd() {
  step "Stopping any running focusd"
  CHANGES_STARTED=1
  stop_daemon "$LABEL"
  rm -f "$SOCKET"
}

# Taken only once the old daemon is gone, so a save it makes while stopping is not lost.
# PREVIOUS_STATE_CHECKED tells rollback that an absent backup means there was no state file before.
save_previous_state() {
  step "Keeping the current state.json so a failure can be undone"
  save_if_present "$STATE_FILE" PREVIOUS_STATE_SAVED
  PREVIOUS_STATE_CHECKED=1
}

install_binary() {
  step "Installing the binary to $BINARY_DEST"
  install -d -m 0755 -o root -g wheel "$(dirname "$BINARY_DEST")"
  install -m 0755 -o root -g wheel "$BINARY_SOURCE" "$BINARY_DEST.new"
  mv -f "$BINARY_DEST.new" "$BINARY_DEST"
}

create_directories() {
  step "Creating the state and log directories"
  install -d -m 0700 -o root -g wheel "$STATE_DIR"
  install -d -m 0755 -o root -g wheel "$LOG_DIR"
}

# Rendered where only root can write, never in a temp directory the user owns, because launchd
# runs whatever this file names as root.
render_plist() {
  local staged="$PLIST_DEST.new"
  rm -f "$staged"
  (umask 077 && sed "s/__OWNER_UID__/$OWNER_UID/g" "$PLIST_TEMPLATE" >"$staged")
  chown root:wheel "$staged"
  chmod 0644 "$staged"
  plutil -lint "$staged" >/dev/null
  mv -f "$staged" "$PLIST_DEST"
}

install_launch_files() {
  step "Installing the LaunchDaemon for uid $OWNER_UID"
  render_plist
  install -d -m 0755 -o root -g wheel "$(dirname "$NEWSYSLOG_DEST")"
  install -m 0644 -o root -g wheel "$NEWSYSLOG_SOURCE" "$NEWSYSLOG_DEST"
}

start_daemon() {
  step "Starting focusd"
  NEW_DAEMON_STARTED=1
  launchctl enable "system/$LABEL"
  launchctl bootstrap system "$PLIST_DEST"
  poll "$BINARY_DEST" ping ||
    fail "focusd did not answer on $SOCKET within $WAIT_SECONDS seconds, check $LOG_DIR/focusd.log"
}

main() {
  parse_args "$@"
  preflight
  save_previous_install
  stop_previous_focusd
  save_previous_state
  install_binary
  create_directories
  migrate_legacy
  install_launch_files
  start_daemon
  remove_legacy
  discard_previous_install
  CURRENT_STEP="finished"
  printf '\nfocusd is running as %s for uid %s.\n  socket  %s\n  state   %s\n  log     %s/focusd.log\n' \
    "$LABEL" "$OWNER_UID" "$SOCKET" "$STATE_DIR" "$LOG_DIR"
}

main "$@"
