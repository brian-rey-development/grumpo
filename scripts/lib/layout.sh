# shellcheck shell=bash disable=SC2034,SC2153
# Paths and names shared by install.sh and uninstall.sh. Source this file, do not run it.

readonly LABEL="io.github.brian-rey-development.focusd"
readonly BINARY_DEST="/Library/PrivilegedHelperTools/focusd"
readonly PLIST_DEST="/Library/LaunchDaemons/${LABEL}.plist"
readonly STATE_DIR="/Library/Application Support/Focus"
readonly STATE_FILE="$STATE_DIR/state.json"
readonly LOG_DIR="/Library/Logs/Focus"
readonly NEWSYSLOG_DEST="/etc/newsyslog.d/focusd.conf"
readonly SOCKET="/var/run/focusd.sock"
readonly INSTALL_LOCK="/var/run/focusd-install.lock"

readonly APP_LABEL="io.github.brian-rey-development.grumpo"
readonly LEGACY_APP_LABELS=("io.github.brian-rey-development.focus" "Focus")
