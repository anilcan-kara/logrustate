#!/bin/sh
set -e

# logrustate Installer
# Usage: curl -fsSL https://raw.githubusercontent.com/anilcan-kara/logrustate/master/install.sh | sh

REPO="anilcan-kara/logrustate"
VERSION="0.1.2"

echo "Detecting system architecture..."
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
  linux)
    case "$ARCH" in
      x86_64) BIN_NAME="logrustate-linux-x64" ;;
      aarch64|arm64) BIN_NAME="logrustate-linux-arm64" ;;
      *) echo "Error: Unsupported architecture: $ARCH"; exit 1 ;;
    esac
    ;;
  darwin)
    case "$ARCH" in
      x86_64) BIN_NAME="logrustate-darwin-x64" ;;
      aarch64|arm64) BIN_NAME="logrustate-darwin-arm64" ;;
      *) echo "Error: Unsupported architecture: $ARCH"; exit 1 ;;
    esac
    ;;
  *)
    echo "Error: Unsupported operating system: $OS"
    exit 1
    ;;
esac

URL="https://github.com/$REPO/releases/download/v$VERSION/$BIN_NAME"
echo "Downloading logrustate binary for $OS ($ARCH)..."
TEMP_DIR=$(mktemp -d)
curl -sL "$URL" -o "$TEMP_DIR/logrustate"

INSTALL_DIR="/usr/local/bin"
if [ ! -w "$INSTALL_DIR" ]; then
  echo "No write permission to $INSTALL_DIR. Installing to $HOME/.local/bin..."
  INSTALL_DIR="$HOME/.local/bin"
  mkdir -p "$INSTALL_DIR"
fi

mv "$TEMP_DIR/logrustate" "$INSTALL_DIR/logrustate"
chmod +x "$INSTALL_DIR/logrustate"

rm -rf "$TEMP_DIR"

echo "Installation complete! logrustate installed to $INSTALL_DIR/logrustate"
if [ "$INSTALL_DIR" = "$HOME/.local/bin" ]; then
  echo "Ensure $HOME/.local/bin is in your PATH."
fi
