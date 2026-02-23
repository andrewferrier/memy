#!/usr/bin/env sh

set -o errexit
set -o noglob
set -o nounset

REPO="andrewferrier/memy"
BIN_NAME="memy"

echo "Fetching latest release version..." >&2
VERSION=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" |
  grep '"tag_name":' |
  head -n 1 |
  sed -E 's/.*"([^"]+)".*/\1/') || { echo "Error: Failed to fetch latest release version. Exiting." >&2; exit 1; }

if [ -z "$VERSION" ]; then
  echo "Error: Could not determine latest release version. Exiting." >&2
  exit 1
fi

OS=$(uname | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$ARCH" in
x86_64) ARCH="x86_64" ;;
aarch64 | arm64) ARCH="aarch64" ;;
*)
  echo "Error: Unsupported architecture: $ARCH. Exiting." >&2
  exit 1
  ;;
esac

URL="https://github.com/$REPO/releases/download/$VERSION/${BIN_NAME}-${OS}-${ARCH}"

DOWNLOAD_TMP=$(mktemp -d) || { echo "Error: Failed to create temporary directory. Exiting." >&2; exit 1; }
echo "Downloading $BIN_NAME..." >&2
curl -L "$URL" -o "${DOWNLOAD_TMP}/${BIN_NAME}" || { echo "Error: Failed to download $BIN_NAME from $URL. Exiting." >&2; exit 1; }

DEST="${HOME}/.local/bin"
TARGET_BIN_PATH="$DEST/$BIN_NAME"

mkdir -p "$DEST"

if [ -f "$TARGET_BIN_PATH" ]; then
  printf "The binary %s already exists at %s. Overwrite? (y/N) " "$BIN_NAME" "$TARGET_BIN_PATH"
  read -r REPLY
  if [ "${REPLY}" != "y" ] && [ "${REPLY}" != "Y" ]; then
    echo "Installation aborted."
    rm -r "$DOWNLOAD_TMP"
    exit 1
  fi
fi

mv "$DOWNLOAD_TMP/$BIN_NAME" "$TARGET_BIN_PATH"
chmod +x "$TARGET_BIN_PATH"

echo "Installed $BIN_NAME to $TARGET_BIN_PATH"
echo "Make sure $DEST is in your PATH"
