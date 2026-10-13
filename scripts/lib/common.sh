# shellcheck shell=bash disable=SC2034,SC2153
# Helpers shared by the scripts in this directory. Source this file, do not run it.
# Scripts that source this define usage() themselves.

readonly WAIT_TENTHS=100
readonly WAIT_SECONDS=$((WAIT_TENTHS / 10))
CURRENT_STEP="starting"

step() {
  CURRENT_STEP="$1"
  printf '==> %s\n' "$1"
}

fail() {
  printf 'error: %s\n' "$1" >&2
  exit 1
}

warn() {
  printf 'warning: %s\n' "$1" >&2
}

reject_argument() {
  usage >&2
  fail "unknown argument: $1"
}

require_value() {
  [[ $2 -ge 2 ]] || fail "$1 needs a value"
}

# Runs the given command every tenth of a second until it succeeds or WAIT_TENTHS runs out.
poll() {
  local tenth
  for ((tenth = 0; tenth < WAIT_TENTHS; tenth++)); do
    if "$@" >/dev/null 2>&1; then
      return 0
    fi
    sleep 0.1
  done
  return 1
}

is_unloaded() {
  ! launchctl print "system/$1"
}

stop_daemon() {
  local label="$1"
  launchctl bootout "system/$label" >/dev/null 2>&1 || true
  poll is_unloaded "$label" || fail "$label is still loaded after $WAIT_SECONDS seconds"
}

require_macos_root() {
  [[ "$(uname -s)" == "Darwin" ]] || fail "focusd only runs on macOS"
  [[ $EUID -eq 0 ]] || fail "run this script as root with sudo"
}

# Prints the home directory of a user. The plist output keeps paths with spaces intact.
home_directory_of() {
  dscl -plist . -read "/Users/$1" NFSHomeDirectory 2>/dev/null |
    plutil -extract 'dsAttrTypeStandard:NFSHomeDirectory.0' raw - 2>/dev/null
}

# Sets OWNER_UID and OWNER_HOME from an explicit user name or uid, else from SUDO_UID.
resolve_owner() {
  local requested="${1:-${SUDO_UID:-}}"
  [[ -n "$requested" ]] || fail "run this with sudo from your own account, or pass --owner <user|uid>"
  if [[ "$requested" =~ ^[0-9]+$ ]]; then
    OWNER_UID="$requested"
  else
    OWNER_UID="$(id -u "$requested" 2>/dev/null)" || fail "no such user: $requested"
  fi
  [[ "$OWNER_UID" -ne 0 ]] || fail "refusing to use root as the owner, pass --owner <user|uid> for your normal account"
  local owner_name
  owner_name="$(id -un "$OWNER_UID" 2>/dev/null)" || fail "no user has uid $OWNER_UID"
  OWNER_HOME="$(home_directory_of "$owner_name")" && [[ -n "$OWNER_HOME" ]] || fail "cannot find the home directory of $owner_name"
}

INSTALL_LOCK_HELD=0

# One installer at a time. A second run would delete the first run's backups and stop the daemon
# it is waiting for, so it fails at once instead. The lock names its holder, and one whose holder
# is gone (a killed run) is taken over.
acquire_install_lock() {
  if ! mkdir "$INSTALL_LOCK" 2>/dev/null; then
    local holder
    holder="$(cat "$INSTALL_LOCK/pid" 2>/dev/null || true)"
    if [[ -z "$holder" ]] || kill -0 "$holder" 2>/dev/null; then
      fail "another install is already running, wait for it to finish"
    fi
    rm -rf "$INSTALL_LOCK"
    mkdir "$INSTALL_LOCK" 2>/dev/null || fail "another install is already running, wait for it to finish"
  fi
  INSTALL_LOCK_HELD=1
  printf '%s\n' "$$" >"$INSTALL_LOCK/pid"
  chmod 0644 "$INSTALL_LOCK/pid"
}

release_install_lock() {
  [[ $INSTALL_LOCK_HELD -eq 0 ]] || rm -rf "$INSTALL_LOCK"
  INSTALL_LOCK_HELD=0
}

# Prints the program a LaunchAgent property list starts, or nothing when it cannot be read.
plist_program() {
  plutil -extract ProgramArguments.0 raw -o - "$1" 2>/dev/null ||
    plutil -extract Program raw -o - "$1" 2>/dev/null || true
}
