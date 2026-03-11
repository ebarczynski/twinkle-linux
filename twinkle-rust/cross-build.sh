#!/bin/bash
# Cross-compilation script for Twinkle Linux
# This script helps build Linux binaries from macOS using cross

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Print usage
usage() {
    echo "Usage: $0 [TARGET] [COMMAND]"
    echo ""
    echo "Targets:"
    echo "  x86_64-unknown-linux-gnu     Standard Linux (x86_64, glibc)"
    echo "  x86_64-unknown-linux-musl    Standard Linux (x86_64, musl, static)"
    echo "  aarch64-unknown-linux-gnu     Linux ARM64 (e.g., Raspberry Pi 4)"
    echo "  armv7-unknown-linux-gnueabihf Linux ARMv7 (e.g., Raspberry Pi 3/Zero)"
    echo ""
    echo "Commands:"
    echo "  build              Build debug binary (default)"
    echo "  release           Build release binary"
    echo "  clean             Clean build artifacts"
    echo "  package           Package of binary"
    echo ""
    echo "Examples:"
    echo "  $0 x86_64-unknown-linux-gnu release"
    echo "  $0 aarch64-unknown-linux-gnu build"
}

# Check if cross is installed
check_cross() {
    if ! command -v cross &> /dev/null; then
        echo -e "${RED}Error: 'cross' is not installed${NC}"
        echo "Install it with: cargo install cross"
        exit 1
    fi
}

# Check if Docker is running
check_docker() {
    if ! podman info &> /dev/null; then
        echo -e "${RED}Error: Docker is not running${NC}"
        exit 1
    fi
}

# Main function
main() {
    # Default values
    TARGET="${1:-x86_64-unknown-linux-gnu}"
    COMMAND="${2:-release}"

    # Parse arguments
    case "$1" in
        -h|--help|help)
            usage
            exit 0
            ;;
    esac

    # Check prerequisites
    check_cross
    check_docker

    # Validate target
    case "$TARGET" in
        x86_64-unknown-linux-gnu|x86_64-unknown-linux-musl|aarch64-unknown-linux-gnu|armv7-unknown-linux-gnueabihf)
            ;;
        *)
            echo -e "${RED}Error: Invalid target '$TARGET'${NC}"
            echo ""
            usage
            exit 1
            ;;
    esac

    echo -e "${GREEN}Cross-compiling for: $TARGET${NC}"
    echo -e "${GREEN}Command: $COMMAND${NC}"
    echo ""

    # Execute command
    case "$COMMAND" in
        build)
            echo -e "${YELLOW}Building debug binary...${NC}"
            cross build --target "$TARGET"
            echo -e "${GREEN}Debug binary: target/$TARGET/debug/twinkle-linux${NC}"
            ;;
        release)
            echo -e "${YELLOW}Building release binary...${NC}"
            cross build --release --target "$TARGET"
            echo -e "${GREEN}Release binary: target/$TARGET/release/twinkle-linux${NC}"
            ;;
        clean)
            echo -e "${YELLOW}Cleaning build artifacts...${NC}"
            cross clean --target "$TARGET"
            echo -e "${GREEN}Cleaned successfully${NC}"
            ;;
        package)
            echo -e "${YELLOW}Building and packaging release binary...${NC}"
            cross build --release --target "$TARGET"

            # Create package directory
            PKG_DIR="dist/$TARGET"
            mkdir -p "$PKG_DIR"

            # Copy binary
            cp "target/$TARGET/release/twinkle-linux" "$PKG_DIR/"

            # Create archive
            cd dist
            tar -czf "twinkle-linux-$TARGET.tar.gz" "$TARGET"
            cd ..

            echo -e "${GREEN}Package created: dist/twinkle-linux-$TARGET.tar.gz${NC}"
            ;;
        *)
            echo -e "${RED}Error: Invalid command '$COMMAND'${NC}"
            echo ""
            usage
            exit 1
            ;;
    esac
}

# Run main function
main "$@"
