# shellcheck shell=bash disable=SC2034,SC2153
# Paths and names shared by install.sh and uninstall.sh. Source this file, do not run it.

readonly LABEL="io.github.brian-rey-development.grumpod"
readonly BINARY_DEST="/Library/PrivilegedHelperTools/grumpod"
readonly PLIST_DEST="/Library/LaunchDaemons/${LABEL}.plist"
readonly STATE_DIR="/Library/Application Support/Grumpo"
readonly STATE_FILE="$STATE_DIR/state.json"
readonly LOG_DIR="/Library/Logs/Grumpo"
readonly NEWSYSLOG_DEST="/etc/newsyslog.d/grumpod.conf"
readonly SOCKET="/var/run/grumpod.sock"
readonly INSTALL_LOCK="/var/run/grumpod-install.lock"

readonly APP_LABEL="io.github.brian-rey-development.grumpo"
readonly LEGACY_APP_LABELS=("io.github.brian-rey-development.focus" "Focus")
