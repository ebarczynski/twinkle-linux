#!/bin/bash
#
# Twinkle Linux - System-wide Uninstall Script
#

set -e

APP_NAME="twinkle-linux"

echo "Uninstalling Twinkle Linux..."

rm -f "/usr/local/bin/$APP_NAME" 2>/dev/null && echo "  Removed /usr/local/bin/$APP_NAME" || true
rm -f "/usr/share/applications/$APP_NAME.desktop" 2>/dev/null && echo "  Removed desktop entry" || true
rm -f "/etc/udev/rules.d/99-twinkle-i2c.rules" 2>/dev/null && echo "  Removed udev rules" || true

echo "Done."
