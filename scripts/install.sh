#!/bin/bash
#
# Twinkle Linux - System-wide Installation Script
# Builds from source and installs to /usr/local
#
# Usage: sudo ./install.sh
#

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
APP_NAME="twinkle-linux"
BIN_DIR="/usr/local/bin"
DESKTOP_DIR="/usr/share/applications"

print_info()  { echo -e "${BLUE}[INFO]${NC} $1"; }
print_ok()    { echo -e "${GREEN}[OK]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

if [ "$EUID" -ne 0 ]; then
    print_error "Run with sudo for system-wide installation"
    exit 1
fi

# Check Rust
if ! command -v cargo &>/dev/null; then
    print_error "Rust not found. Install from https://rustup.rs"
    exit 1
fi

# Check GTK4
if ! pkg-config --exists gtk4 2>/dev/null; then
    print_error "GTK4 development libraries not found."
    echo "  Ubuntu/Debian: sudo apt install libgtk-4-dev pkg-config"
    echo "  Fedora: sudo dnf install gtk4-devel pkgconfig"
    exit 1
fi

# Build
print_info "Building release binary..."
cd "$PROJECT_ROOT/twinkle-rust"
cargo build --release
BINARY="target/release/$APP_NAME"

if [ ! -f "$BINARY" ]; then
    print_error "Build failed"
    exit 1
fi

# Install
cp "$BINARY" "$BIN_DIR/$APP_NAME"
chmod +x "$BIN_DIR/$APP_NAME"
print_ok "Installed to $BIN_DIR/$APP_NAME"

# Desktop entry
cat > "$DESKTOP_DIR/$APP_NAME.desktop" << EOF
[Desktop Entry]
Type=Application
Name=Twinkle Linux
Comment=Monitor brightness control
Exec=$BIN_DIR/$APP_NAME
Icon=display-brightness
Terminal=false
Categories=Utility;
EOF
print_ok "Desktop entry installed"

# I2C udev rules
print_info "Installing I2C udev rules for DDC/CI..."
cp "$PROJECT_ROOT/packaging/99-twinkle-i2c.rules" /etc/udev/rules.d/
udevadm control --reload-rules 2>/dev/null || true
print_ok "Udev rules installed"

echo ""
print_ok "Installation complete! Run: $APP_NAME"
