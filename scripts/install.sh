#!/bin/bash
#
# Twinkle Linux - System-wide Installation Script
# This script installs Twinkle Linux to the system-wide directories.
#
# Usage: sudo ./install.sh [--skip-udev]
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
APP_NAME="twinkle-linux"
APP_VERSION="0.1.0"
INSTALL_DIR="/opt/${APP_NAME}"
BIN_DIR="/usr/local/bin"
DESKTOP_DIR="/usr/share/applications"
ICON_DIR="/usr/share/icons/hicolor/scalable/apps"
PYTHON_MIN_VERSION="3.8"

# Parse command line arguments
SKIP_UDEV=false
for arg in "$@"; do
    case $arg in
        --skip-udev)
            SKIP_UDEV=true
            shift
            ;;
        --help|-h)
            echo "Usage: sudo $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --skip-udev    Skip udev rules installation"
            echo "  --help, -h     Show this help message"
            echo ""
            echo "This script installs Twinkle Linux system-wide."
            echo "It requires root privileges for installation."
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $arg${NC}"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Function to print colored messages
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if running as root
check_root() {
    if [[ $EUID -ne 0 ]]; then
        print_error "This script must be run as root (use sudo)"
        exit 1
    fi
}

# Function to check Python version
check_python() {
    print_info "Checking Python version..."
    
    if ! command -v python3 &> /dev/null; then
        print_error "Python 3 is not installed"
        print_info "Please install Python 3.8 or higher"
        exit 1
    fi
    
    PYTHON_VERSION=$(python3 -c 'import sys; print(".".join(map(str, sys.version_info[:2])))')
    PYTHON_MAJOR=$(echo $PYTHON_VERSION | cut -d. -f1)
    PYTHON_MINOR=$(echo $PYTHON_VERSION | cut -d. -f2)
    
    if [[ $PYTHON_MAJOR -lt 3 ]] || [[ $PYTHON_MAJOR -eq 3 && $PYTHON_MINOR -lt 8 ]]; then
        print_error "Python $PYTHON_VERSION is not supported. Minimum required: $PYTHON_MIN_VERSION"
        exit 1
    fi
    
    print_success "Python $PYTHON_VERSION found"
}

# Function to check pip
check_pip() {
    print_info "Checking pip..."
    
    if ! command -v pip3 &> /dev/null; then
        print_warning "pip3 not found, attempting to install..."
        if command -v apt-get &> /dev/null; then
            apt-get update && apt-get install -y python3-pip
        elif command -v dnf &> /dev/null; then
            dnf install -y python3-pip
        elif command -v yum &> /dev/null; then
            yum install -y python3-pip
        elif command -v pacman &> /dev/null; then
            pacman -S --noconfirm python-pip
        else
            print_error "Cannot install pip automatically. Please install pip3 manually"
            exit 1
        fi
    fi
    
    print_success "pip3 is available"
}

# Function to install ddcutil
install_ddcutil() {
    print_info "Checking ddcutil..."
    
    if command -v ddcutil &> /dev/null; then
        print_success "ddcutil is already installed"
        return
    fi
    
    print_warning "ddcutil not found, installing..."
    
    if command -v apt-get &> /dev/null; then
        apt-get update && apt-get install -y ddcutil
    elif command -v dnf &> /dev/null; then
        dnf install -y ddcutil
    elif command -v yum &> /dev/null; then
        yum install -y ddcutil
    elif command -v pacman &> /dev/null; then
        pacman -S --noconfirm ddcutil
    elif command -v zypper &> /dev/null; then
        zypper install -y ddcutil
    else
        print_error "Cannot install ddcutil automatically. Please install it manually"
        print_info "Visit: https://www.ddcutil.com/"
        exit 1
    fi
    
    print_success "ddcutil installed successfully"
}

# Function to install Python dependencies
install_python_deps() {
    print_info "Installing Python dependencies..."
    
    if [[ -f "$PROJECT_ROOT/requirements.txt" ]]; then
        pip3 install --upgrade -r "$PROJECT_ROOT/requirements.txt"
    else
        print_warning "requirements.txt not found, installing from pyproject.toml..."
        pip3 install --upgrade PyQt6 pydantic pyxdg
    fi
    
    print_success "Python dependencies installed"
}

# Function to create installation directories
create_directories() {
    print_info "Creating installation directories..."
    
    mkdir -p "$INSTALL_DIR"
    mkdir -p "$BIN_DIR"
    mkdir -p "$DESKTOP_DIR"
    mkdir -p "$ICON_DIR"
    
    print_success "Directories created"
}

# Function to copy application files
copy_application_files() {
    print_info "Copying application files..."
    
    # Copy source code
    cp -r "$PROJECT_ROOT/src" "$INSTALL_DIR/"
    
    # Copy configuration and metadata files
    cp "$PROJECT_ROOT/pyproject.toml" "$INSTALL_DIR/" 2>/dev/null || true
    cp "$PROJECT_ROOT/requirements.txt" "$INSTALL_DIR/" 2>/dev/null || true
    
    print_success "Application files copied"
}

# Function to install desktop entry
install_desktop_entry() {
    print_info "Installing desktop entry..."
    
    if [[ -f "$PROJECT_ROOT/packaging/twinkle-linux.desktop" ]]; then
        cp "$PROJECT_ROOT/packaging/twinkle-linux.desktop" "$DESKTOP_DIR/"
        print_success "Desktop entry installed"
    else
        print_warning "Desktop entry file not found"
    fi
}

# Function to install icon
install_icon() {
    print_info "Installing application icon..."
    
    if [[ -f "$PROJECT_ROOT/src/ui/resources/icons/twinkle.svg" ]]; then
        cp "$PROJECT_ROOT/src/ui/resources/icons/twinkle.svg" "$ICON_DIR/twinkle-linux.svg"
        print_success "Icon installed"
    else
        print_warning "Icon file not found"
    fi
}

# Function to create executable symlink
create_symlink() {
    print_info "Creating executable symlink..."
    
    # Remove existing symlink if present
    rm -f "$BIN_DIR/$APP_NAME"
    
    # Create new symlink
    cat > "$BIN_DIR/$APP_NAME" << 'EOF'
#!/bin/bash
# Twinkle Linux launcher script

SCRIPT_DIR="/opt/twinkle-linux"
export PYTHONPATH="$SCRIPT_DIR:$PYTHONPATH"

python3 "$SCRIPT_DIR/src/main.py" "$@"
EOF
    
    chmod +x "$BIN_DIR/$APP_NAME"
    print_success "Executable symlink created"
}

# Function to prompt for udev rules installation
prompt_udev_rules() {
    if [[ "$SKIP_UDEV" == true ]]; then
        print_info "Skipping udev rules installation (--skip-udev specified)"
        return
    fi
    
    echo ""
    print_warning "Udev rules are required for DDC/CI monitor control."
    print_warning "Without udev rules, you may need to run Twinkle Linux with sudo."
    echo ""
    read -p "Install udev rules now? (y/N): " -n 1 -r
    echo ""
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        if [[ -x "$SCRIPT_DIR/install-udev-rules.sh" ]]; then
            "$SCRIPT_DIR/install-udev-rules.sh"
        else
            print_warning "Udev rules installation script not found or not executable"
            print_info "You can install udev rules later by running: sudo scripts/install-udev-rules.sh"
        fi
    else
        print_info "Skipping udev rules installation"
        print_info "You can install them later by running: sudo scripts/install-udev-rules.sh"
    fi
}

# Function to display success message
display_success() {
    echo ""
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}  Twinkle Linux installed successfully!  ${NC}"
    echo -e "${GREEN}========================================${NC}"
    echo ""
    print_info "Installation Summary:"
    echo "  - Application: $INSTALL_DIR"
    echo "  - Executable:  $BIN_DIR/$APP_NAME"
    echo "  - Desktop:     $DESKTOP_DIR/twinkle-linux.desktop"
    echo "  - Icon:        $ICON_DIR/twinkle-linux.svg"
    echo ""
    print_info "Usage:"
    echo "  - Launch from menu: Look for 'Twinkle Linux' in your application menu"
    echo "  - Launch from terminal: $APP_NAME"
    echo "  - Launch with debug: $APP_NAME --debug"
    echo ""
    print_info "If you installed udev rules:"
    echo "  - Log out and log back in for group changes to take effect"
    echo ""
    print_info "For more information, visit: https://github.com/twinkle-twinkle/twinkle-linux"
    echo ""
}

# Main installation process
main() {
    echo ""
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}  Twinkle Linux Installation Script    ${NC}"
    echo -e "${BLUE}  Version: $APP_VERSION                 ${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo ""
    
    check_root
    check_python
    check_pip
    install_ddcutil
    create_directories
    copy_application_files
    install_python_deps
    install_desktop_entry
    install_icon
    create_symlink
    prompt_udev_rules
    display_success
}

# Run main function
main "$@"
