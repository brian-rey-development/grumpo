#!/usr/bin/env bash
# shellcheck source-path=SCRIPTDIR
# Imports the release certificate into a temporary keychain on a GitHub macOS runner, trusts it
# for code signing and exports FOCUS_SIGNING_IDENTITY, so every release carries the same signature.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly SCRIPT_DIR
# shellcheck source=../lib/common.sh
source "$SCRIPT_DIR/../lib/common.sh"
# shellcheck source=../lib/sign.sh
source "$SCRIPT_DIR/../lib/sign.sh"

readonly KEYCHAIN="$RUNNER_TEMP/signing.keychain-db"
readonly KEYCHAIN_TIMEOUT_SECONDS=21600
readonly SYSTEM_KEYCHAIN="/Library/Keychains/System.keychain"
readonly P12_PATH="$RUNNER_TEMP/certificate.p12"
readonly PEM_PATH="$RUNNER_TEMP/certificate.pem"

cleanup() {
  rm -f "$P12_PATH"
}

require_secrets() {
  [[ -n "${SIGNING_CERTIFICATE_P12:-}" ]] ||
    fail "the SIGNING_CERTIFICATE_P12 secret is missing, create it with scripts/create-release-keys.sh --upload"
  [[ -n "${SIGNING_CERTIFICATE_PASSWORD:-}" ]] || fail "the SIGNING_CERTIFICATE_PASSWORD secret is missing"
}

create_keychain() {
  local keychain_password="$1"
  security create-keychain -p "$keychain_password" "$KEYCHAIN"
  security set-keychain-settings -lut "$KEYCHAIN_TIMEOUT_SECONDS" "$KEYCHAIN"
  security unlock-keychain -p "$keychain_password" "$KEYCHAIN"
  # shellcheck disable=SC2046 # the existing keychains are separate, quoted words
  security list-keychains -d user -s "$KEYCHAIN" $(security list-keychains -d user | tr -d '"')
}

import_certificate() {
  local keychain_password="$1"
  base64 --decode <<<"$SIGNING_CERTIFICATE_P12" >"$P12_PATH"
  security import "$P12_PATH" -k "$KEYCHAIN" -P "$SIGNING_CERTIFICATE_PASSWORD" -T /usr/bin/codesign >/dev/null
  security set-key-partition-list -S apple-tool:,apple: -s -k "$keychain_password" "$KEYCHAIN" >/dev/null
}

# Nobody can answer the trust prompt on a runner, so allow admin trust changes without one first.
trust_certificate() {
  P12_PASSWORD="$SIGNING_CERTIFICATE_PASSWORD" "$OPENSSL" pkcs12 -in "$P12_PATH" \
    -clcerts -nokeys -passin env:P12_PASSWORD | "$OPENSSL" x509 -out "$PEM_PATH"
  sudo security authorizationdb write com.apple.trust-settings.admin allow
  sudo security add-trusted-cert -d -r trustRoot -p codeSign -k "$SYSTEM_KEYCHAIN" "$PEM_PATH"
}

main() {
  local keychain_password
  trap cleanup EXIT
  require_secrets
  keychain_password="$("$OPENSSL" rand -hex 16)"
  step "Importing \"$RELEASE_IDENTITY\""
  create_keychain "$keychain_password"
  import_certificate "$keychain_password"
  trust_certificate
  security find-identity -v -p codesigning "$KEYCHAIN" | grep -qF "\"$RELEASE_IDENTITY\"" ||
    fail "\"$RELEASE_IDENTITY\" is not a valid code signing identity after the import"
  echo "FOCUS_SIGNING_IDENTITY=$RELEASE_IDENTITY" >>"$GITHUB_ENV"
}

main
