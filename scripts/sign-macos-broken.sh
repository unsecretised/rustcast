#!/bin/bash
set -euo pipefail

RELEASE_DIR="target/release"
APP_DIR="$RELEASE_DIR/macos"
APP_NAME="Rustcast.app"
APP_PATH="$APP_DIR/$APP_NAME"

# --- Required env vars (using the names you provided) ---
environment=(
  "MACOS_CERTIFICATE"
  "MACOS_CERTIFICATE_PWD"
  "MACOS_CI_KEYCHAIN_PWD"
  "MACOS_CERTIFICATE_NAME"
  "MACOS_NOTARIZATION_PWD"
  "MACOS_NOTARY_TEAM_ID"
  "MACOS_NOTARY_KEY_ID"
  "MACOS_NOTARY_KEY"
)

for var in "${environment[@]}"; do
  if [[ -z "${!var:-}" ]]; then
    echo "Error: $var is not set"
    exit 1
  fi
done

# Optional: only needed if you still want to keep this around
: "${MACOS_NOTARISATION_APPLE_ID:=}"

echo "Decoding certificate"
echo "$MACOS_CERTIFICATE" | base64 --decode > certificate.p12

echo "Installing cert in a new keychain"
security create-keychain -p "$MACOS_CI_KEYCHAIN_PWD" build.keychain
security default-keychain -s build.keychain
security unlock-keychain -p "$MACOS_CI_KEYCHAIN_PWD" build.keychain
security import certificate.p12 -k build.keychain -P "$MACOS_CERTIFICATE_PWD" -T /usr/bin/codesign
security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "$MACOS_CI_KEYCHAIN_PWD" build.keychain

echo "Signing..."
/usr/bin/codesign --force -s "$MACOS_CERTIFICATE_NAME" --options runtime --timestamp "$APP_PATH" -v

echo "Creating temp notarization archive"
ditto -c -k --keepParent "$APP_PATH" "notarization.zip"

echo "Notarize app (API key auth)"
# MACOS_NOTARY_KEY can be either:
# - the *contents* of the .p8 key, or
# - base64 of the .p8 key (recommended for CI)
#
# If it's base64, decode it first.
NOTARY_KEY_FILE="AuthKey.p8"
if printf '%s' "$MACOS_NOTARY_KEY" | grep -q "BEGIN PRIVATE KEY"; then
  printf '%s' "$MACOS_NOTARY_KEY" > "$NOTARY_KEY_FILE"
else
  printf '%s' "$MACOS_NOTARY_KEY" | base64 --decode > "$NOTARY_KEY_FILE"
fi

# xcrun notarytool submit "notarization.zip" \
#   --team-id "$MACOS_NOTARY_TEAM_ID" \
#   --issuer "$MACOS_NOTARY_ISSUER_ID" \
#   --key-id "$MACOS_NOTARY_KEY_ID" \
#   --key "$NOTARY_KEY_FILE" \
#   --wait

echo "Attach staple"
xcrun stapler staple "$APP_PATH"
