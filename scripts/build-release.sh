#!/usr/bin/env bash
# shellcheck source-path=SCRIPTDIR
# Builds the release artifacts (grumpod, Grumpo.app and the .dmg) into dist/release/. Grumpo.app carries
# grumpod and the installer, so the app can install the service itself.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly SCRIPT_DIR
# shellcheck source=lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"
# shellcheck source=lib/sign.sh
source "$SCRIPT_DIR/lib/sign.sh"

readonly APP_NAME="Grumpo"
readonly UNIVERSAL_TARGET="universal-apple-darwin"
readonly ARM_TARGET="aarch64-apple-darwin"
readonly INTEL_TARGET="x86_64-apple-darwin"
readonly ARM_ARCH="arm64"
readonly INTEL_ARCH="x86_64"

ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
readonly ROOT
readonly APP_DIR="$ROOT/app"
readonly OUT_DIR="$ROOT/dist/release"
readonly BUNDLED_DAEMON_PATH="Contents/MacOS/grumpod"
readonly MAIN_EXECUTABLE_PATH="Contents/MacOS/$APP_NAME"
readonly BUNDLED_INSTALLER_PATH="Contents/Resources/installer/scripts/install.sh"
readonly BUNDLED_PACKAGING_PATH="Contents/Resources/installer/packaging/macos"
readonly BUNDLED_INSTALLER_FILES=(
  "Contents/Resources/installer/scripts/lib/layout.sh"
  "Contents/Resources/installer/scripts/lib/common.sh"
  "Contents/Resources/installer/scripts/lib/legacy.sh"
  "Contents/Resources/installer/scripts/lib/rollback.sh"
  "Contents/Resources/installer/packaging/macos/io.github.brian-rey-development.grumpod.plist"
  "Contents/Resources/installer/packaging/macos/grumpod.newsyslog.conf"
)
TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"
readonly TARGET_DIR
readonly RUSTUP_BIN="${CARGO_HOME:-$HOME/.cargo}/bin"
readonly STAGING_DIR="$TARGET_DIR/release-staging"

UNIVERSAL=0
BUNDLE_ROOT=""
DAEMON_BINARY=""
DMG_BUILT=1

usage() {
  cat <<USAGE
usage: scripts/build-release.sh [--universal]

Builds grumpod, $APP_NAME.app and the .dmg, and collects them in dist/release/.

  (default)     Build for this Mac's architecture only.
  --universal   Build for $ARM_TARGET and $INTEL_TARGET and merge them.

Environment:
  FOCUS_SIGNING_IDENTITY   Keychain identity that signs $APP_NAME.app and grumpod. Without it both
                           are signed ad hoc, and macOS asks for permissions after each rebuild.

$APP_NAME.app carries the signed grumpod and the installer, and installs the service from its
Settings. The standalone grumpod in dist/release/ is for scripts/install.sh.
USAGE
}

parse_args() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --universal) UNIVERSAL=1 ;;
      -h | --help)
        usage
        exit 0
        ;;
      *) reject_argument "$1" ;;
    esac
    shift
  done
}

prefer_rustup_toolchain() {
  if [[ -x "$RUSTUP_BIN/cargo" ]]; then
    export PATH="$RUSTUP_BIN:$PATH"
  else
    warn "no rustup proxies in $RUSTUP_BIN, using $(command -v cargo || echo 'no cargo')"
  fi
}

require_tools() {
  local tool
  for tool in cargo pnpm ditto lipo codesign; do
    command -v "$tool" >/dev/null 2>&1 || fail "$tool is not on PATH, install the Xcode command line tools if it is an Apple tool"
  done
}

require_targets() {
  local installed target
  command -v rustup >/dev/null 2>&1 || fail "rustup is needed to check the Rust targets for --universal"
  installed="$(cd "$ROOT" && rustup target list --installed)"
  for target in "$ARM_TARGET" "$INTEL_TARGET"; do
    grep -qx "$target" <<<"$installed" ||
      fail "the Rust target $target is missing, add it with: rustup target add $target"
  done
}

require_universal() {
  local archs
  archs="$(lipo -archs "$1")"
  [[ " $archs " == *" $ARM_ARCH "* && " $archs " == *" $INTEL_ARCH "* ]] ||
    fail "$1 is not universal, lipo reports: $archs"
}

build_daemon() {
  step "Building grumpod (release, host architecture)"
  (cd "$ROOT" && cargo build --release -p grumpod)
  DAEMON_BINARY="$TARGET_DIR/release/grumpod"
}

build_daemon_universal() {
  local target
  for target in "$ARM_TARGET" "$INTEL_TARGET"; do
    step "Building grumpod (release, $target)"
    (cd "$ROOT" && cargo build --release -p grumpod --target "$target")
  done
  step "Merging the grumpod binaries with lipo"
  DAEMON_BINARY="$TARGET_DIR/$UNIVERSAL_TARGET/release/grumpod"
  mkdir -p "$(dirname "$DAEMON_BINARY")"
  lipo -create -output "$DAEMON_BINARY" \
    "$TARGET_DIR/$ARM_TARGET/release/grumpod" \
    "$TARGET_DIR/$INTEL_TARGET/release/grumpod"
  require_universal "$DAEMON_BINARY"
}

# Signed before it goes into the app, because the app seal covers it and signing it afterwards
# would break the seal.
stage_daemon() {
  step "Signing the grumpod that goes into $APP_NAME.app"
  local path
  for path in "$ROOT" "$STAGING_DIR"; do
    [[ "$path" != *[\"\\]* ]] || fail "the path $path must not contain quotes or backslashes"
  done
  mkdir -p "$STAGING_DIR"
  cp -p "$DAEMON_BINARY" "$STAGING_DIR/grumpod"
  DAEMON_BINARY="$STAGING_DIR/grumpod"
  sign_daemon "$DAEMON_BINARY"
}

# Passed only from here, so tauri dev, cargo test and scripts/check.sh work without these files.
# The paths go into JSON as they are, so stage_daemon has already refused the ones that would break it.
tauri_bundle_config() {
  printf '{"bundle":{"macOS":{"files":{"MacOS/grumpod":"%s",' "$DAEMON_BINARY"
  printf '"Resources/installer/scripts/install.sh":"%s/scripts/install.sh",' "$ROOT"
  printf '"Resources/installer/scripts/lib":"%s/scripts/lib",' "$ROOT"
  printf '"Resources/installer/packaging/macos":"%s/packaging/macos"}}}}' "$ROOT"
}

tauri_build() {
  (cd "$APP_DIR" && env -u APPLE_SIGNING_IDENTITY -u APPLE_CERTIFICATE -u APPLE_CERTIFICATE_PASSWORD \
    pnpm tauri build --config "$(tauri_signing_config)" --config "$(tauri_bundle_config)" "$@")
}

build_app() {
  local -a target_args=()
  if [[ $UNIVERSAL -eq 1 ]]; then
    target_args=(--target "$UNIVERSAL_TARGET")
    BUNDLE_ROOT="$TARGET_DIR/$UNIVERSAL_TARGET/release/bundle"
  else
    BUNDLE_ROOT="$TARGET_DIR/release/bundle"
  fi

  step "Installing JavaScript dependencies"
  (cd "$APP_DIR" && pnpm install --frozen-lockfile)

  step "Building $APP_NAME.app and the .dmg with Tauri"
  if tauri_build ${target_args[@]+"${target_args[@]}"}; then
    return
  fi
  warn "the full bundle failed, retrying with the .app only (creating a .dmg can need a GUI session)"
  DMG_BUILT=0
  tauri_build ${target_args[@]+"${target_args[@]}"} --bundles app ||
    fail "the Tauri build failed, see the output above"
}

first_dmg() {
  local dmg
  for dmg in "$BUNDLE_ROOT"/dmg/*.dmg; do
    if [[ -f "$dmg" ]]; then
      printf '%s' "$dmg"
      return
    fi
  done
}

collect() {
  local dmg
  step "Collecting artifacts in dist/release/"
  [[ -d "$BUNDLE_ROOT/macos/$APP_NAME.app" ]] || fail "$APP_NAME.app was not produced in $BUNDLE_ROOT/macos"
  rm -rf "$OUT_DIR"
  mkdir -p "$OUT_DIR"
  cp -p "$DAEMON_BINARY" "$OUT_DIR/grumpod"
  ditto "$BUNDLE_ROOT/macos/$APP_NAME.app" "$OUT_DIR/$APP_NAME.app"
  if [[ $DMG_BUILT -eq 1 ]]; then
    dmg="$(first_dmg)"
    [[ -n "$dmg" ]] || fail "no .dmg was produced in $BUNDLE_ROOT/dmg"
    cp -p "$dmg" "$OUT_DIR/"
  fi
}

verify_main_executable() {
  local app="$1"
  [[ -x "$app/$MAIN_EXECUTABLE_PATH" ]] ||
    fail "$app has no $MAIN_EXECUTABLE_PATH, System Settings would list the app under its Cargo binary name"
}

verify_bundled_installer() {
  local app="$1" path
  step "Checking that $(basename "$app") carries grumpod and the installer"
  for path in "$BUNDLED_DAEMON_PATH" "$BUNDLED_INSTALLER_PATH" "$BUNDLED_PACKAGING_PATH" \
    "${BUNDLED_INSTALLER_FILES[@]}"; do
    [[ -e "$app/$path" ]] || fail "$(basename "$app") is missing $path"
  done
  verify_daemon_signature "$app/$BUNDLED_DAEMON_PATH"
}

summarize() {
  step "Release artifacts"
  (cd "$OUT_DIR" && du -sh -- *)
  lipo -info "$OUT_DIR/grumpod"
  if [[ $DMG_BUILT -eq 0 ]]; then
    warn "no .dmg was built, only $APP_NAME.app is available"
  fi
  cat <<NEXT

Next steps (the app installs the service itself, macOS asks for your password once):
  rm -rf /Applications/Focus.app /Applications/$APP_NAME.app && ditto dist/release/$APP_NAME.app /Applications/$APP_NAME.app && open /Applications/$APP_NAME.app
  then click Install service when $APP_NAME asks for it.

If you used this app under its earlier name Focus, clear its stale permission entries once:
  tccutil reset All io.github.brian-rey-development.focus

Developers can install the service from Terminal instead:
  sudo scripts/install.sh --binary dist/release/grumpod
NEXT
}

main() {
  parse_args "$@"
  prefer_rustup_toolchain
  require_tools
  require_signing_identity
  if [[ $UNIVERSAL -eq 1 ]]; then
    require_targets
    build_daemon_universal
  else
    build_daemon
  fi
  stage_daemon
  build_app
  verify_signature "$BUNDLE_ROOT/macos/$APP_NAME.app" "$APP_DIR"
  collect
  verify_signature "$OUT_DIR/$APP_NAME.app" "$APP_DIR"
  verify_main_executable "$OUT_DIR/$APP_NAME.app"
  verify_bundled_installer "$OUT_DIR/$APP_NAME.app"
  verify_daemon_signature "$OUT_DIR/grumpod"
  summarize
}

main "$@"
