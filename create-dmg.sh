#!/bin/bash
#
# Auxin DMG Creator
# Creates a distributable DMG disk image containing the Auxin installer
#
# Usage:
#   ./create-dmg.sh [--version VERSION] [--installer PATH]
#
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VERSION="${VERSION:-1.0.0}"
INSTALLER_PATH=""
OUTPUT_DIR="$PROJECT_ROOT"
TEMP_DMG_DIR="/tmp/auxin-dmg-$$"

# Print colored messages
print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_header() {
    echo ""
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo ""
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --version)
                VERSION="$2"
                shift 2
                ;;
            --installer)
                INSTALLER_PATH="$2"
                shift 2
                ;;
            --output)
                OUTPUT_DIR="$2"
                shift 2
                ;;
            --help)
                cat << EOF
Auxin DMG Creator

Usage: $0 [OPTIONS]

Options:
  --version VERSION     Set version (default: 1.0.0)
  --installer PATH      Path to .pkg installer
  --output DIR          Output directory (default: current directory)
  --help               Show this help message

Examples:
  $0                                      # Auto-detect installer
  $0 --version 1.2.3                     # Specify version
  $0 --installer /path/to/Auxin.pkg      # Specify installer path

EOF
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                echo "Run '$0 --help' for usage information"
                exit 1
                ;;
        esac
    done
}

# Find installer package
find_installer() {
    if [ -z "$INSTALLER_PATH" ]; then
        # Try to auto-detect
        local detected="$PROJECT_ROOT/installer-build/Auxin-$VERSION.pkg"
        if [ -f "$detected" ]; then
            INSTALLER_PATH="$detected"
            print_info "Auto-detected installer: $INSTALLER_PATH"
        else
            print_error "Installer not found. Please specify with --installer"
            exit 1
        fi
    fi

    if [ ! -f "$INSTALLER_PATH" ]; then
        print_error "Installer not found: $INSTALLER_PATH"
        exit 1
    fi

    print_success "Using installer: $INSTALLER_PATH"
}

# Create temporary DMG directory
create_temp_dir() {
    print_header "Creating DMG Contents"

    print_info "Creating temporary directory..."
    rm -rf "$TEMP_DMG_DIR"
    mkdir -p "$TEMP_DMG_DIR"

    # Copy installer
    print_info "Copying installer..."
    cp "$INSTALLER_PATH" "$TEMP_DMG_DIR/Install Auxin.pkg"

    # Create README
    print_info "Creating README..."
    cat > "$TEMP_DMG_DIR/README.txt" << 'EOF'
Auxin - Version Control for Creative Applications
==================================================

INSTALLATION
------------
1. Double-click "Install Auxin.pkg"
2. Follow the installer prompts
3. Choose components to install:
   - Auxin CLI (Required)
   - Auxin Daemon (Required)
   - Auxin Application (Recommended)
   - Auxin Server (Optional - for LAN collaboration)

PREREQUISITES
-------------
Auxin requires Oxen CLI to be installed:
   pip3 install oxen-ai

GETTING STARTED
---------------
After installation:

1. Initialize a project:
   cd /path/to/YourProject.logicx
   auxin init --logic .

2. Launch the Auxin app:
   Open Applications/Auxin.app

3. Enable the background daemon:
   System Settings → General → Login Items & Extensions
   Enable "Auxin Daemon"

SERVER SETUP (Optional)
-----------------------
If you installed the Auxin Server component:

1. Start the server:
   auxin-server

2. Configure:
   ~/.config/auxin-server/.env

3. Access web UI:
   http://localhost:3000

DOCUMENTATION
-------------
User Guide: https://github.com/jbacus/auxin
Issues: https://github.com/jbacus/auxin/issues

SYSTEM REQUIREMENTS
-------------------
- macOS 14.0 or later
- Xcode Command Line Tools
- Oxen CLI (pip3 install oxen-ai)

LICENSE
-------
MIT License - See LICENSE file

Copyright (c) 2025 Auxin
EOF

    # Copy license if available
    if [ -f "$PROJECT_ROOT/LICENSE" ]; then
        cp "$PROJECT_ROOT/LICENSE" "$TEMP_DMG_DIR/LICENSE.txt"
    fi

    # Create a symbolic link to Applications folder for easy drag-install
    # (though this installer uses .pkg, some users may want to see this)
    print_info "Creating Applications symlink..."
    ln -s /Applications "$TEMP_DMG_DIR/Applications"

    print_success "DMG contents prepared"
    echo ""
}

# Create DMG
create_dmg() {
    print_header "Creating DMG Image"

    local dmg_name="Auxin-$VERSION.dmg"
    local dmg_path="$OUTPUT_DIR/$dmg_name"

    # Remove existing DMG
    if [ -f "$dmg_path" ]; then
        print_info "Removing existing DMG..."
        rm -f "$dmg_path"
    fi

    print_info "Creating DMG (this may take a moment)..."

    # Create DMG using hdiutil
    hdiutil create \
        -volname "Auxin $VERSION" \
        -srcfolder "$TEMP_DMG_DIR" \
        -ov \
        -format UDZO \
        -imagekey zlib-level=9 \
        "$dmg_path"

    if [ ! -f "$dmg_path" ]; then
        print_error "Failed to create DMG"
        exit 1
    fi

    # Clean up temp directory
    print_info "Cleaning up..."
    rm -rf "$TEMP_DMG_DIR"

    print_success "DMG created successfully!"
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  DMG: $dmg_path"
    echo "  Size: $(du -h "$dmg_path" | cut -f1)"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "To test:"
    echo "  open $dmg_path"
    echo ""
    echo "To distribute:"
    echo "  1. Upload to GitHub Releases"
    echo "  2. Share download link with users"
    echo ""
}

# Verify DMG
verify_dmg() {
    print_header "Verifying DMG"

    local dmg_path="$OUTPUT_DIR/Auxin-$VERSION.dmg"

    print_info "Verifying DMG integrity..."
    if hdiutil verify "$dmg_path"; then
        print_success "DMG verification passed"
    else
        print_error "DMG verification failed"
        exit 1
    fi

    echo ""
}

# Main
main() {
    print_header "Auxin DMG Creator"
    echo "Version: $VERSION"
    echo ""

    parse_args "$@"
    find_installer
    create_temp_dir
    create_dmg
    verify_dmg

    print_success "DMG creation complete!"
}

# Run main function
main "$@"
