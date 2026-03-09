#!/bin/bash
#
# Twinkle Linux - User-level Uninstallation Script
# This script removes Twinkle Linux from the user's home directory.
#
# Usage: ./uninstall-user.sh
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
INSTALL_DIR="$HOME/.local/share/$APP_NAME"
BIN_DIR="$HOME/.local/bin"
DESKTOP_DIR="$HOME/.local/share/applications"
ICON_DIR="$HOME/.local/share/icons/hicolor/scalable/apps"
CONFIG_DIR="$HOME/.config/$APP_NAME"
CACHE_DIR="$HOME/.cache/$APP_NAME"

# Parse command line arguments
KEEP_CONFIG=false
for arg in "$@"; do
    case $arg in
        --keep-config)
            KEEP_CONFIG=true
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --keep-config  Keep configuration files"
            echo "  --help, -h     Show this help message"
            echo ""
            echo "This script removes Twinkle Linux from your user directory."
            echo "No root privileges required."
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

# Function to remove executable script
remove_executable() {
    print_info "Removing executable script..."
    
    if [[ -f "$BIN_DIR/$APP_NAME" ]]; then
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

# Function to remove config files
remove_config_files() {
    if [[ "$KEEP_CONFIG" == true ]]; then
        print_info "Keeping configuration files (--keep-config specified)"
        return
    fi
    
    print_info "Checking configuration files..."
    
    HAS_CONFIG=false
    
    if [[ -d "$CONFIG_DIR" ]]; then
        HAS_CONFIG=true
    fi
    
    if [[ -d "$CACHE_DIR" ]]; then
        HAS_CONFIG=true
    fi
    
    if [[ "$HAS_CONFIG" == true ]]; then
        echo ""
        print_warning "Configuration and cache files found:"
        [[ -d "$CONFIG_DIR" ]] && echo "  - $CONFIG_DIR"
        [[ -d "$CACHE_DIR" ]] && echo "  - $CACHE_DIR"
        echo ""
        read -p "Remove configuration files? (y/N): " -n 1 -r
        echo ""
        
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            rm -rf "$CONFIG_DIR"
            rm -rf "$CACHE_DIR"
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
        INSTALLED_PACKAGES=$(pip3 list --user 2>/dev/null | grep -E 'PyQt6|pydantic|pyxdg' || true)
        
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
                pip3 uninstall --user -y PyQt6 pydantic pyxdg 2>/dev/null || true
                print_success "Python packages removed"
            else
                print_info "Keeping Python packages"
            fi
        else
            print_info "No Twinkle Linux Python packages found in user site-packages"
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
    
    if [[ "$KEEP_CONFIG" != true ]]; then
        echo "  - Configuration files (if confirmed)"
    fi
    
    echo ""
    print_info "Note: Python packages were only removed if you confirmed their removal."
    print_info "      Udev rules and system dependencies were not affected."
    echo ""
    print_info "To remove udev rules, run: sudo scripts/uninstall-udev.sh"
    echo ""
    print_info "Thank you for using Twinkle Linux!"
    echo ""
}

# Main uninstallation process
main() {
    echo ""
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}  Twinkle Linux User Uninstallation     ${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo ""
    print_warning "This will remove Twinkle Linux from your user directory."
    echo ""
    
    remove_desktop_entry
    remove_icon
    remove_executable
    remove_application
    remove_config_files
    remove_python_packages
    display_completion
}

# Run main function
main "$@"
