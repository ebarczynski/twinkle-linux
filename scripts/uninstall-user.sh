#!/bin/bash
#
# Twinkle Linux - User-level Uninstall Script
#

set -e

APP_NAME="twinkle-linux"
BIN_DIR="$HOME/.local/bin"
DESKTOP_DIR="$HOME/.local/share/applications"

echo "Uninstalling Twinkle Linux..."

rm -f "$BIN_DIR/$APP_NAME" 2>/dev/null && echo "  Removed $BIN_DIR/$APP_NAME" || true
rm -f "$DESKTOP_DIR/$APP_NAME.desktop" 2>/dev/null && echo "  Removed desktop entry" || true

echo "Done."
