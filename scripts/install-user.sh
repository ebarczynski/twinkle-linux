#!/bin/bash
#
# Twinkle Linux - User-level Installation Script
# Builds from source and installs to ~/.local/
#
# Usage: ./install-user.sh
#

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
APP_NAME="twinkle-linux"
BIN_DIR="$HOME/.local/bin"
DESKTOP_DIR="$HOME/.local/share/applications"

print_info()  { echo -e "${BLUE}[INFO]${NC} $1"; }
print_ok()    { echo -e "${GREEN}[OK]${NC} $1"; }
print_warn()  { echo -e "${YELLOW}[WARN]${NC} $1"; }
print_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Check Rust
if ! command -v cargo &>/dev/null; then
    print_error "Rust not found. Install from https://rustup.rs"
    exit 1
fi
print_ok "Rust found: $(rustc --version)"

# Check GTK4 dev libs
if ! pkg-config --exists gtk4 2>/dev/null; then
    print_error "GTK4 development libraries not found."
    echo "  Ubuntu/Debian: sudo apt install libgtk-4-dev pkg-config"
    echo "  Fedora: sudo dnf install gtk4-devel pkgconfig"
    echo "  Arch: sudo pacman -S gtk4 pkgconf"
    exit 1
fi
print_ok "GTK4 dev libraries found"

# Build
print_info "Building release binary..."
cd "$PROJECT_ROOT/twinkle-rust"
cargo build --release
BINARY="target/release/$APP_NAME"

if [ ! -f "$BINARY" ]; then
    print_error "Build failed - binary not found at $BINARY"
    exit 1
fi
print_ok "Build successful"

# Install binary
mkdir -p "$BIN_DIR"
cp "$BINARY" "$BIN_DIR/$APP_NAME"
chmod +x "$BIN_DIR/$APP_NAME"
print_ok "Installed to $BIN_DIR/$APP_NAME"

# Install .desktop file
mkdir -p "$DESKTOP_DIR"
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

echo ""
print_ok "Installation complete! Run: $APP_NAME"
