#!/bin/bash
#
# Twinkle Linux - Uninstallation Script
# This script removes Twinkle Linux from the system.
#
# Usage: sudo ./uninstall.sh
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
APP_NAME="twinkle-linux"
INSTALL_DIR="/opt/${APP_NAME}"
BIN_DIR="/usr/local/bin"
DESKTOP_DIR="/usr/share/applications"
ICON_DIR="/usr/share/icons/hicolor/scalable/apps"
UDEV_RULES="/etc/udev/rules.d/99-twinkle-i2c.rules"
CONFIG_DIR="$HOME/.config/$APP_NAME"
CACHE_DIR="$HOME/.cache/$APP_NAME"

# Parse command line arguments
KEEP_CONFIG=false
KEEP_UDEV=false
for arg in "$@"; do
    case $arg in
        --keep-config)
            KEEP_CONFIG=true
            shift
            ;;
        --keep-udev)
            KEEP_UDEV=true
            shift
            ;;
        --help|-h)
            echo "Usage: sudo $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --keep-config  Keep configuration files"
            echo "  --keep-udev    Keep udev rules"
            echo "  --help, -h     Show this help message"
            echo ""
            echo "This script removes Twinkle Linux from the system."
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

# Function to get current user
get_current_user() {
    if [[ -n "$SUDO_USER" ]]; then
        echo "$SUDO_USER"
    elif [[ -n "$USER" && "$USER" != "root" ]]; then
        echo "$USER"
    else
        echo ""
    fi
}

# Function to remove desktop entry
remove_desktop_entry() {
    print_info "Removing desktop entry..."
    
    if [[ -f "$DESKTOP_DIR/twinkle-linux.desktop" ]]; then
        rm -f "$DESKTOP_DIR/twinkle-linux.desktop"
        print_success "Desktop entry removed"
    else
        print_info "Desktop entry not found (already removed or never installed)"
    fi
}

# Function to remove icon
remove_icon() {
    print_info "Removing application icon..."
    
    if [[ -f "$ICON_DIR/twinkle-linux.svg" ]]; then
        rm -f "$ICON_DIR/twinkle-linux.svg"
        print_success "Icon removed"
    else
        print_info "Icon not found (already removed or never installed)"
    fi
}

# Function to remove executable symlink
remove_symlink() {
    print_info "Removing executable symlink..."
    
    if [[ -L "$BIN_DIR/$APP_NAME" ]]; then
        rm -f "$BIN_DIR/$APP_NAME"
        print_success "Symlink removed"
    elif [[ -f "$BIN_DIR/$APP_NAME" ]]; then
        rm -f "$BIN_DIR/$APP_NAME"
        print_success "Executable removed"
    else
        print_info "Executable not found (already removed or never installed)"
    fi
}

# Function to remove application directory
remove_application() {
    print_info "Removing application directory..."
    
    if [[ -d "$INSTALL_DIR" ]]; then
        rm -rf "$INSTALL_DIR"
        print_success "Application directory removed"
    else
        print_info "Application directory not found (already removed or never installed)"
    fi
}

# Function to remove udev rules
remove_udev_rules() {
    if [[ "$KEEP_UDEV" == true ]]; then
        print_info "Keeping udev rules (--keep-udev specified)"
        return
    fi
    
    print_info "Checking udev rules..."
    
    if [[ -f "$UDEV_RULES" ]]; then
        echo ""
        print_warning "Udev rules are installed. Removing them will prevent"
        print_warning "any application from using DDC/CI without sudo."
        echo ""
        read -p "Remove udev rules? (y/N): " -n 1 -r
        echo ""
        
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            # Backup the rules file
            cp "$UDEV_RULES" "${UDEV_RULES}.backup.$(date +%Y%m%d%H%M%S)"
            rm -f "$UDEV_RULES"
            
            # Reload udev
            if command -v udevadm &> /dev/null; then
                udevadm control --reload-rules
                udevadm trigger
            fi
            
            print_success "Udev rules removed (backup created)"
        else
            print_info "Keeping udev rules"
        fi
    else
        print_info "Udev rules not found (already removed or never installed)"
    fi
}

# Function to remove user from i2c group
remove_user_from_i2c() {
    if [[ "$KEEP_UDEV" == true ]]; then
        return
    fi
    
    CURRENT_USER=$(get_current_user)
    
    if [[ -z "$CURRENT_USER" ]]; then
        return
    fi
    
    print_info "Checking i2c group membership..."
    
    if groups "$CURRENT_USER" 2>/dev/null | grep -q '\bi2c\b'; then
        echo ""
        print_warning "User '$CURRENT_USER' is a member of the i2c group."
        print_warning "Removing the user from this group will prevent"
        print_warning "DDC/CI access for this user."
        echo ""
        read -p "Remove user from i2c group? (y/N): " -n 1 -r
        echo ""
        
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            if command -v gpasswd &> /dev/null; then
                gpasswd -d "$CURRENT_USER" i2c 2>/dev/null || true
                print_success "User removed from i2c group"
            else
                print_warning "gpasswd not found, cannot remove user from group"
                print_info "You may need to manually remove the user from the i2c group"
            fi
        else
            print_info "Keeping user in i2c group"
        fi
    else
        print_info "User is not a member of the i2c group"
    fi
}

# Function to remove config files
remove_config_files() {
    if [[ "$KEEP_CONFIG" == true ]]; then
        print_info "Keeping configuration files (--keep-config specified)"
        return
    fi
    
    CURRENT_USER=$(get_current_user)
    
    if [[ -z "$CURRENT_USER" ]]; then
        print_info "Cannot determine current user, skipping config removal"
        return
    fi
    
    print_info "Checking configuration files..."
    
    # Get user home directory
    USER_HOME=$(getent passwd "$CURRENT_USER" | cut -d: -f6)
    if [[ -z "$USER_HOME" ]]; then
        USER_HOME="/home/$CURRENT_USER"
    fi
    
    USER_CONFIG_DIR="$USER_HOME/.config/$APP_NAME"
    USER_CACHE_DIR="$USER_HOME/.cache/$APP_NAME"
    
    HAS_CONFIG=false
    
    if [[ -d "$USER_CONFIG_DIR" ]]; then
        HAS_CONFIG=true
    fi
    
    if [[ -d "$USER_CACHE_DIR" ]]; then
        HAS_CONFIG=true
    fi
    
    if [[ "$HAS_CONFIG" == true ]]; then
        echo ""
        print_warning "Configuration and cache files found:"
        [[ -d "$USER_CONFIG_DIR" ]] && echo "  - $USER_CONFIG_DIR"
        [[ -d "$USER_CACHE_DIR" ]] && echo "  - $USER_CACHE_DIR"
        echo ""
        read -p "Remove configuration files? (y/N): " -n 1 -r
        echo ""
        
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            rm -rf "$USER_CONFIG_DIR"
            rm -rf "$USER_CACHE_DIR"
            print_success "Configuration files removed"
        else
            print_info "Keeping configuration files"
        fi
    else
        print_info "No configuration files found"
    fi
}

# Function to remove Python packages
remove_python_packages() {
    print_info "Checking Python packages..."
    
    if command -v pip3 &> /dev/null; then
        INSTALLED_PACKAGES=$(pip3 list 2>/dev/null | grep -E 'PyQt6|pydantic|pyxdg' || true)
        
        if [[ -n "$INSTALLED_PACKAGES" ]]; then
            echo ""
            print_warning "The following Python packages were installed for Twinkle Linux:"
            echo "$INSTALLED_PACKAGES"
            echo ""
            print_warning "These packages might be used by other applications."
            echo ""
            read -p "Remove these Python packages? (y/N): " -n 1 -r
            echo ""
            
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                pip3 uninstall -y PyQt6 pydantic pyxdg 2>/dev/null || true
                print_success "Python packages removed"
            else
                print_info "Keeping Python packages"
            fi
        else
            print_info "No Twinkle Linux Python packages found"
        fi
    else
        print_info "pip3 not found, skipping package removal"
    fi
}

# Function to display completion message
display_completion() {
    echo ""
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}  Twinkle Linux Uninstalled             ${NC}"
    echo -e "${GREEN}========================================${NC}"
    echo ""
    print_info "What was removed:"
    echo "  - Application directory: $INSTALL_DIR"
    echo "  - Executable: $BIN_DIR/$APP_NAME"
    echo "  - Desktop entry: $DESKTOP_DIR/twinkle-linux.desktop"
    echo "  - Icon: $ICON_DIR/twinkle-linux.svg"
    
    if [[ "$KEEP_UDEV" != true ]]; then
        echo "  - Udev rules (if confirmed)"
    fi
    
    if [[ "$KEEP_CONFIG" != true ]]; then
        echo "  - Configuration files (if confirmed)"
    fi
    
    echo ""
    print_info "Note: Python packages and system dependencies (ddcutil)"
    print_info "      were only removed if you confirmed their removal."
    echo ""
    print_info "Thank you for using Twinkle Linux!"
    echo ""
}

# Main uninstallation process
main() {
    echo ""
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}  Twinkle Linux Uninstallation Script    ${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo ""
    print_warning "This will remove Twinkle Linux from your system."
    echo ""
    
    check_root
    remove_desktop_entry
    remove_icon
    remove_symlink
    remove_application
    remove_udev_rules
    remove_user_from_i2c
    remove_config_files
    remove_python_packages
    display_completion
}

# Run main function
main "$@"
