#!/usr/bin/env bash
# shellcheck source-path=SCRIPTDIR
# Creates the release signing certificate and the updater key once per project, and backs both up
# in ~/.grumpo-signing. Losing either one cuts installed copies off from updates.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly SCRIPT_DIR
# shellcheck source=lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"
# shellcheck source=lib/sign.sh
source "$SCRIPT_DIR/lib/sign.sh"

ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
readonly ROOT
readonly BACKUP_DIR="$HOME/.grumpo-signing"
readonly VALID_DAYS=3650
readonly KEY_BITS=3072
readonly LOGIN_KEYCHAIN="$HOME/Library/Keychains/login.keychain-db"

UPLOAD=0
TRUST_LOCALLY=0
WORK=""

usage() {
  cat <<USAGE
usage: scripts/create-release-keys.sh [--upload] [--trust-locally]

Creates the release signing certificate and the updater key in $BACKUP_DIR.

  --upload          Store both as GitHub Actions secrets of this repository with gh.
  --trust-locally   Add the certificate to your login keychain, so local builds can use
                    FOCUS_SIGNING_IDENTITY="$RELEASE_IDENTITY". macOS asks for your password.
USAGE
}

parse_arguments() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --upload) UPLOAD=1 ;;
      --trust-locally) TRUST_LOCALLY=1 ;;
      -h | --help)
        usage
        exit 0
        ;;
      *) reject_argument "$1" ;;
    esac
    shift
  done
}

cleanup() {
  [[ -z "$WORK" ]] || rm -rf "$WORK"
}

write_certificate_config() {
  cat >"$WORK/certificate.cnf" <<CONFIG
[req]
distinguished_name = dn
x509_extensions = ext
prompt = no
[dn]
CN = $RELEASE_IDENTITY
[ext]
basicConstraints = critical,CA:false
keyUsage = critical,digitalSignature
extendedKeyUsage = critical,codeSigning
CONFIG
}

create_certificate() {
  local output
  if [[ -e "$BACKUP_DIR/certificate.p12" ]]; then
    printf 'Keeping the existing certificate in %s\n' "$BACKUP_DIR"
    return
  fi
  step "Creating the certificate \"$RELEASE_IDENTITY\""
  write_certificate_config
  output="$("$OPENSSL" req -x509 -newkey "rsa:$KEY_BITS" -nodes -days "$VALID_DAYS" -config "$WORK/certificate.cnf" \
    -keyout "$WORK/key.pem" -out "$BACKUP_DIR/certificate.pem" 2>&1)" ||
    fail "openssl could not create the certificate. $output"
  "$OPENSSL" rand -hex 24 >"$BACKUP_DIR/certificate-password.txt"
  P12_PASSWORD="$(<"$BACKUP_DIR/certificate-password.txt")" "$OPENSSL" pkcs12 -export -name "$RELEASE_IDENTITY" \
    -inkey "$WORK/key.pem" -in "$BACKUP_DIR/certificate.pem" -out "$BACKUP_DIR/certificate.p12" \
    -passout env:P12_PASSWORD
}

create_updater_key() {
  if [[ -e "$BACKUP_DIR/updater.key" ]]; then
    printf 'Keeping the existing updater key in %s\n' "$BACKUP_DIR"
    return
  fi
  step "Creating the updater key"
  "$OPENSSL" rand -hex 24 >"$BACKUP_DIR/updater-password.txt"
  (cd app && pnpm -s tauri signer generate --ci -w "$BACKUP_DIR/updater.key" \
    -p "$(<"$BACKUP_DIR/updater-password.txt")" >/dev/null)
}

trust_locally() {
  step "Adding the certificate to your login keychain, macOS asks for your password"
  if ! security find-identity -p codesigning "$LOGIN_KEYCHAIN" | grep -qF "\"$RELEASE_IDENTITY\""; then
    security import "$BACKUP_DIR/certificate.p12" -k "$LOGIN_KEYCHAIN" \
      -P "$(<"$BACKUP_DIR/certificate-password.txt")" -T /usr/bin/codesign >/dev/null
  fi
  security add-trusted-cert -r trustRoot -p codeSign -k "$LOGIN_KEYCHAIN" "$BACKUP_DIR/certificate.pem"
  # Without the partition list codesign needs a dialog for the key, which a script cannot answer.
  security set-key-partition-list -S apple-tool:,apple: -s -l "$RELEASE_IDENTITY" "$LOGIN_KEYCHAIN" >/dev/null
}

upload_secrets() {
  step "Storing the keys as GitHub Actions secrets"
  base64 -i "$BACKUP_DIR/certificate.p12" | gh secret set SIGNING_CERTIFICATE_P12
  gh secret set SIGNING_CERTIFICATE_PASSWORD <"$BACKUP_DIR/certificate-password.txt"
  gh secret set TAURI_SIGNING_PRIVATE_KEY <"$BACKUP_DIR/updater.key"
  gh secret set TAURI_SIGNING_PRIVATE_KEY_PASSWORD <"$BACKUP_DIR/updater-password.txt"
}

require_tools() {
  command -v pnpm >/dev/null 2>&1 || fail "pnpm is needed to create the updater key"
  [[ $UPLOAD -eq 0 ]] || command -v gh >/dev/null 2>&1 || fail "gh is needed for --upload"
}

print_next_steps() {
  cat <<DONE

The keys are in $BACKUP_DIR. Back that folder up somewhere safe, such as a password manager,
because a new certificate makes everyone grant permissions again and a new updater key cuts
every installed copy off from updates.

Set plugins.updater.pubkey in app/src-tauri/tauri.conf.json to this updater public key.
$(<"$BACKUP_DIR/updater.key.pub")
DONE
}

main() {
  parse_arguments "$@"
  require_tools
  trap cleanup EXIT
  cd "$ROOT"
  umask 077
  mkdir -p "$BACKUP_DIR"
  WORK="$(mktemp -d)"
  create_certificate
  create_updater_key
  [[ $TRUST_LOCALLY -eq 0 ]] || trust_locally
  [[ $UPLOAD -eq 0 ]] || upload_secrets
  print_next_steps
}

main "$@"
