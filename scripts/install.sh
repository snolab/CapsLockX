#!/bin/bash
# CapsLockX installer for macOS and Linux
# Usage: curl -fsSL https://raw.githubusercontent.com/snolab/CapsLockX/beta/scripts/install.sh | bash
set -euo pipefail

REPO="snolab/CapsLockX"
INSTALL_DIR="/usr/local/bin"
BIN_NAME="clx"

# Detect platform
OS="$(uname -s)"
ARCH="$(uname -m)"

case "${OS}-${ARCH}" in
    Darwin-arm64)  ASSET="capslockx-macos-arm64" ;;
    Darwin-x86_64) ASSET="capslockx-macos-arm64" ;; # Rosetta 2
    Linux-x86_64)  ASSET="capslockx-linux-x86_64" ;;
    *) echo "Unsupported platform: ${OS}-${ARCH}"; exit 1 ;;
esac

# Find latest release (or beta)
TAG="${CLX_VERSION:-}"
if [ -z "$TAG" ]; then
    TAG="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases?per_page=10" \
        | grep -o '"tag_name":"[^"]*"' | head -1 | cut -d'"' -f4)"
fi
echo "Installing CapsLockX ${TAG} for ${OS} ${ARCH}..."

# Download
URL="https://github.com/${REPO}/releases/download/${TAG}/${ASSET}"
TMP="$(mktemp)"
curl -fsSL -o "$TMP" "$URL"
chmod +x "$TMP"

# Install
if [ -w "$INSTALL_DIR" ]; then
    mv "$TMP" "${INSTALL_DIR}/${BIN_NAME}"
else
    echo "Installing to ${INSTALL_DIR} (requires sudo)..."
    sudo mv "$TMP" "${INSTALL_DIR}/${BIN_NAME}"
fi

# macOS: prompt for Accessibility permission
if [ "$OS" = "Darwin" ]; then
    echo ""
    echo "Grant Accessibility permission when prompted."
    echo "  System Settings → Privacy & Security → Accessibility → enable 'clx'"
fi

echo "Installed: $(which $BIN_NAME) (${TAG})"
echo "Run 'clx' to start CapsLockX."
