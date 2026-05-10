#!/bin/bash
#
# Twinkle Linux - Udev Rules Installation for DDC/CI
#
# Usage: sudo ./install-udev-rules.sh
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

if [ "$EUID" -ne 0 ]; then
    echo "Run with sudo."
    exit 1
fi

echo "Installing I2C udev rules..."
cp "$PROJECT_ROOT/packaging/99-twinkle-i2c.rules" /etc/udev/rules.d/
udevadm control --reload-rules
echo "Done. I2C permissions are now available after relogin or: udevadm trigger"
