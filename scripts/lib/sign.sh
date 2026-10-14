# shellcheck shell=bash disable=SC2034
# Code signing for Grumpo.app. Source this file, do not run it.
# FOCUS_SIGNING_IDENTITY names a keychain identity, and without it everything is signed ad hoc.

# LibreSSL writes a PKCS#12 file that security import reads, which the OpenSSL 3 defaults do not.
readonly OPENSSL="/usr/bin/openssl"
readonly AD_HOC_IDENTITY="-"
readonly RELEASE_IDENTITY="Grumpo Release Signing"
readonly ENTITLEMENT_APPLE_EVENTS="com.apple.security.automation.apple-events"
readonly DAEMON_IDENTIFIER="io.github.brian-rey-development.grumpod"
readonly SHA1_HEX_LENGTH=40

signing_identity() {
  printf '%s' "${FOCUS_SIGNING_IDENTITY:-$AD_HOC_IDENTITY}"
}

require_signing_identity() {
  local identity
  identity="$(signing_identity)"
  if [[ "$identity" == *[\"\\]* ]]; then
    fail "FOCUS_SIGNING_IDENTITY must not contain quotes or backslashes"
  fi
  if [[ "$identity" == "$AD_HOC_IDENTITY" ]]; then
    warn "FOCUS_SIGNING_IDENTITY is not set, signing ad hoc. macOS will ask for permissions again after every rebuild"
    return 0
  fi
  security find-identity -p codesigning | grep -qF -- "$identity" ||
    fail "no code signing identity matches \"$identity\" in the keychain, see security find-identity -p codesigning"
}

# Tauri signs the .app before it builds the .dmg, so the identity has to reach it through its config.
tauri_signing_config() {
  printf '{"bundle":{"macOS":{"signingIdentity":"%s"}}}' "$(signing_identity)"
}

expected_bundle_identifier() {
  plutil -extract identifier raw -o - "$1/src-tauri/tauri.conf.json"
}

# The name that codesign prints as the Authority of a signature made with the configured identity.
signing_authority() {
  local identity="$1" line
  if [[ ${#identity} -ne $SHA1_HEX_LENGTH || ! "$identity" =~ ^[0-9A-Fa-f]+$ ]]; then
    printf '%s' "$identity"
    return
  fi
  line="$(security find-identity -p codesigning | grep -iF -- "$identity")" || return 1
  sed -E 's/^[^"]*"([^"]*)".*$/\1/' <<<"$line" | head -n 1
}

assert_signer() {
  local details="$1" identity authority
  identity="$(signing_identity)"
  if [[ "$identity" == "$AD_HOC_IDENTITY" ]]; then
    grep -qx "Signature=adhoc" <<<"$details" ||
      fail "the signature is not ad hoc although FOCUS_SIGNING_IDENTITY is unset, something overrode the signing identity"
    return 0
  fi
  authority="$(signing_authority "$identity")" || fail "no code signing identity matches \"$identity\" in the keychain"
  grep -qxF -- "Authority=$authority" <<<"$details" ||
    fail "the signature was not made by \"$authority\", something overrode FOCUS_SIGNING_IDENTITY"
}

verify_signature() {
  local app="$1" app_dir="$2" expected details
  step "Verifying the code signature of $(basename "$app")"
  codesign --verify --deep --strict --verbose=2 "$app" || fail "codesign verification failed for $app"
  details="$(codesign -dvv --entitlements - "$app" 2>&1)"
  printf '%s\n' "$details"
  expected="$(expected_bundle_identifier "$app_dir")"
  grep -qx "Identifier=$expected" <<<"$details" ||
    fail "the code signature identifier is not $expected, the macOS permissions would not stick to the app"
  assert_signer "$details"
  grep -q "$ENTITLEMENT_APPLE_EVENTS" <<<"$details" ||
    fail "the app is missing the $ENTITLEMENT_APPLE_EVENTS entitlement, browser automation would be denied"
}

# macOS keeps permission grants across updates only when the designated requirement names the
# certificate, and an ad hoc signature gives a cdhash requirement that changes with every build.
assert_stable_requirement() {
  local app="$1" requirement
  requirement="$(codesign -d -r - "$app" 2>&1)"
  if grep -q 'cdhash' <<<"$requirement" || ! grep -q 'certificate' <<<"$requirement"; then
    printf '%s\n' "$requirement" >&2
    fail "$app has no certificate based requirement, permission grants would not survive updates"
  fi
}

# The hardened runtime is on, and the daemon needs no entitlement under it.
sign_daemon() {
  local binary="$1"
  step "Signing $(basename "$binary") as $DAEMON_IDENTIFIER"
  codesign --force --sign "$(signing_identity)" --identifier "$DAEMON_IDENTIFIER" --options runtime "$binary" ||
    fail "codesign could not sign $binary"
}

verify_daemon_signature() {
  local binary="$1" details
  step "Verifying the code signature of $(basename "$binary")"
  codesign --verify --strict --verbose=2 "$binary" || fail "codesign verification failed for $binary"
  details="$(codesign -dvv "$binary" 2>&1)"
  printf '%s\n' "$details"
  grep -qx "Identifier=$DAEMON_IDENTIFIER" <<<"$details" ||
    fail "the code signature identifier is not $DAEMON_IDENTIFIER"
  grep -Eq '^CodeDirectory .*flags=0x[0-9a-f]+\([^)]*runtime' <<<"$details" ||
    fail "the hardened runtime is not enabled on $binary"
  assert_signer "$details"
}
