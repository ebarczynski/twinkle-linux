#!/bin/bash
#
# Twinkle Linux - Udev Rules Installation Script
# This script installs udev rules for DDC/CI monitor control.
#
# Usage: sudo ./install-udev-rules.sh
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
UDEV_RULES_FILE="99-twinkle-i2c.rules"
UDEV_DEST="/etc/udev/rules.d/$UDEV_RULES_FILE"
I2C_GROUP="i2c"

# Parse command line arguments
ADD_USER=true
for arg in "$@"; do
    case $arg in
        --no-add-user)
            ADD_USER=false
            shift
            ;;
        --help|-h)
            echo "Usage: sudo $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --no-add-user  Don't add current user to i2c group"
            echo "  --help, -h     Show this help message"
            echo ""
            echo "This script installs udev rules for DDC/CI monitor control."
            echo "It requires root privileges."
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

# Function to get current user (not root)
get_current_user() {
    if [[ -n "$SUDO_USER" ]]; then
        echo "$SUDO_USER"
    elif [[ -n "$USER" && "$USER" != "root" ]]; then
        echo "$USER"
    else
        echo ""
    fi
}

# Function to check if udev rules file exists
check_udev_file() {
    if [[ -f "$PROJECT_ROOT/packaging/$UDEV_RULES_FILE" ]]; then
        echo "$PROJECT_ROOT/packaging/$UDEV_RULES_FILE"
    elif [[ -f "$PROJECT_ROOT/$UDEV_RULES_FILE" ]]; then
        echo "$PROJECT_ROOT/$UDEV_RULES_FILE"
    else
        echo ""
    fi
}

# Function to create udev rules file if not found
create_udev_rules() {
    print_warning "Udev rules file not found, creating default rules..."
    
    cat > "$PROJECT_ROOT/packaging/$UDEV_RULES_FILE" << 'EOF'
# Twinkle Linux - DDC/CI udev rules
# These rules allow users in the i2c group to access DDC/CI monitors

# Grant read/write access to I2C devices for users in the i2c group
SUBSYSTEM=="i2c-dev", MODE="0660", GROUP="i2c"

# Alternative rules for specific DDC/CI devices (uncomment if needed)
# KERNEL=="i2c-[0-9]*", MODE="0660", GROUP="i2c"
EOF
    
    print_success "Default udev rules created at: $PROJECT_ROOT/packaging/$UDEV_RULES_FILE"
}

# Function to install udev rules
install_udev_rules() {
    print_info "Installing udev rules..."
    
    UDEV_SOURCE=$(check_udev_file)
    
    if [[ -z "$UDEV_SOURCE" ]]; then
        create_udev_rules
        UDEV_SOURCE="$PROJECT_ROOT/packaging/$UDEV_RULES_FILE"
    fi
    
    # Backup existing rules if present
    if [[ -f "$UDEV_DEST" ]]; then
        print_warning "Existing udev rules found, creating backup..."
        cp "$UDEV_DEST" "${UDEV_DEST}.backup.$(date +%Y%m%d%H%M%S)"
    fi
    
    # Copy udev rules
    cp "$UDEV_SOURCE" "$UDEV_DEST"
    
    # Set proper permissions
    chmod 644 "$UDEV_DEST"
    
    print_success "Udev rules installed to: $UDEV_DEST"
}

# Function to reload udev rules
reload_udev() {
    print_info "Reloading udev rules..."
    
    if command -v udevadm &> /dev/null; then
        udevadm control --reload-rules
        udevadm trigger
        print_success "Udev rules reloaded"
    else
        print_warning "udevadm not found, you may need to reboot"
    fi
}

# Function to ensure i2c group exists
ensure_i2c_group() {
    print_info "Checking for i2c group..."
    
    if ! getent group "$I2C_GROUP" &> /dev/null; then
        print_warning "i2c group does not exist, creating..."
        groupadd "$I2C_GROUP"
        print_success "i2c group created"
    else
        print_success "i2c group exists"
    fi
}

# Function to add user to i2c group
add_user_to_i2c() {
    if [[ "$ADD_USER" != true ]]; then
        print_info "Skipping user addition (--no-add-user specified)"
        return
    fi
    
    CURRENT_USER=$(get_current_user)
    
    if [[ -z "$CURRENT_USER" ]]; then
        print_warning "Could not determine current user"
        print_info "You may need to manually add your user to the i2c group:"
        echo "  sudo usermod -aG i2c <username>"
        return
    fi
    
    print_info "Adding user '$CURRENT_USER' to i2c group..."
    
    if usermod -aG "$I2C_GROUP" "$CURRENT_USER" 2>/dev/null; then
        print_success "User '$CURRENT_USER' added to i2c group"
    else
        print_warning "Failed to add user to i2c group"
        print_info "You may need to manually add your user:"
        echo "  sudo usermod -aG i2c $CURRENT_USER"
    fi
}

# Function to display post-installation instructions
display_instructions() {
    echo ""
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}  Udev Rules Installed Successfully!    ${NC}"
    echo -e "${GREEN}========================================${NC}"
    echo ""
    print_info "What was done:"
    echo "  - Udev rules installed to: $UDEV_DEST"
    echo "  - Udev rules reloaded"
    echo "  - User added to i2c group (if applicable)"
    echo ""
    print_warning "IMPORTANT: You must log out and log back in for the group changes to take effect."
    echo ""
    print_info "To verify the installation:"
    echo "  - After logging back in, run: groups"
    echo "  - You should see 'i2c' in the list of groups"
    echo ""
    print_info "To test DDC/CI access:"
    echo "  - Run: ddcutil detect"
    echo "  - If monitors are detected, everything is working correctly"
    echo ""
    print_info "If you still have issues:"
    echo "  - Try running: sudo ddcutil detect"
    echo "  - Check ddcutil documentation: https://www.ddcutil.com/"
    echo ""
}

# Main installation process
main() {
    echo ""
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}  Twinkle Linux Udev Rules Installer    ${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo ""
    
    check_root
    ensure_i2c_group
    install_udev_rules
    reload_udev
    add_user_to_i2c
    display_instructions
}

# Run main function
main "$@"
