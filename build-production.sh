#!/bin/bash
#
# Auxin Production Build Script
# Builds, signs, and notarizes the installer for distribution
#
# Usage:
#   ./build-production.sh [VERSION] [--skip-notarization]
#
# Prerequisites:
#   - Apple Developer Program membership
#   - Developer ID certificates installed
#   - Notarization credentials stored with: xcrun notarytool store-credentials
#
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration - CUSTOMIZE THESE
VERSION="${1:-1.0.0}"
APP_CERT="${APP_SIGNING_CERT:-Developer ID Application}"
INSTALLER_CERT="${INSTALLER_SIGNING_CERT:-Developer ID Installer}"
NOTARY_PROFILE="${NOTARY_PROFILE:-auxin-notary}"
SKIP_NOTARIZATION=false

# Directories
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUTPUT_DIR="$PROJECT_ROOT/release-$VERSION"

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
    echo -e "${BLUE}═══════════════════════════════════════${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}═══════════════════════════════════════${NC}"
    echo ""
}

print_step() {
    echo ""
    echo -e "${GREEN}▶${NC} ${BLUE}$1${NC}"
    echo ""
}

# Parse arguments
parse_args() {
    shift # Skip version argument
    while [[ $# -gt 0 ]]; do
        case $1 in
            --skip-notarization)
                SKIP_NOTARIZATION=true
                shift
                ;;
            --app-cert)
                APP_CERT="$2"
                shift 2
                ;;
            --installer-cert)
                INSTALLER_CERT="$2"
                shift 2
                ;;
            --notary-profile)
                NOTARY_PROFILE="$2"
                shift 2
                ;;
            --help)
                cat << EOF
Auxin Production Build Script

Usage: $0 [VERSION] [OPTIONS]

Arguments:
  VERSION               Version number (default: 1.0.0)

Options:
  --skip-notarization   Skip notarization (for testing)
  --app-cert NAME       Application signing certificate name
  --installer-cert NAME Installer signing certificate name
  --notary-profile NAME Notarization keychain profile name
  --help               Show this help message

Environment Variables:
  APP_SIGNING_CERT      Override app signing certificate
  INSTALLER_SIGNING_CERT Override installer signing certificate
  NOTARY_PROFILE        Override notarization profile

Prerequisites:
  1. Apple Developer Program membership
  2. Developer ID certificates in Keychain
  3. Notarization credentials stored:
     xcrun notarytool store-credentials "$NOTARY_PROFILE" \\
       --apple-id "your-email@example.com" \\
       --team-id "YOUR-TEAM-ID" \\
       --password "xxxx-xxxx-xxxx-xxxx"

Examples:
  $0 1.2.3                    # Build version 1.2.3
  $0 1.2.3 --skip-notarization # Build without notarization
  $0 1.2.3 --app-cert "Developer ID Application: John Doe (ABC123)"

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

# Check prerequisites
check_prerequisites() {
    print_header "Checking Prerequisites"

    local all_ok=true

    # Check macOS
    if [[ "$OSTYPE" != "darwin"* ]]; then
        print_error "This script requires macOS"
        all_ok=false
    else
        print_success "Running on macOS $(sw_vers -productVersion)"
    fi

    # Check for code signing tools
    if ! command -v codesign &> /dev/null; then
        print_error "codesign not found"
        all_ok=false
    else
        print_success "codesign found"
    fi

    if ! command -v productsign &> /dev/null; then
        print_error "productsign not found"
        all_ok=false
    else
        print_success "productsign found"
    fi

    # Check for certificates
    print_info "Checking for signing certificates..."
    if security find-identity -v -p codesigning | grep -q "Developer ID Application"; then
        print_success "Developer ID Application certificate found"
    else
        print_error "Developer ID Application certificate not found"
        print_info "Install from: https://developer.apple.com/account/resources/certificates/list"
        all_ok=false
    fi

    if security find-identity -v -p codesigning | grep -q "Developer ID Installer"; then
        print_success "Developer ID Installer certificate found"
    else
        print_error "Developer ID Installer certificate not found"
        print_info "Install from: https://developer.apple.com/account/resources/certificates/list"
        all_ok=false
    fi

    # Check notarization credentials (if not skipping)
    if [ "$SKIP_NOTARIZATION" = false ]; then
        print_info "Checking notarization credentials..."
        if xcrun notarytool history --keychain-profile "$NOTARY_PROFILE" &> /dev/null; then
            print_success "Notarization profile '$NOTARY_PROFILE' configured"
        else
            print_error "Notarization profile '$NOTARY_PROFILE' not found"
            print_info "Set up with: xcrun notarytool store-credentials"
            all_ok=false
        fi
    else
        print_warning "Skipping notarization checks"
    fi

    # Check build tools
    if ! command -v cargo &> /dev/null; then
        print_error "Rust/Cargo not found"
        all_ok=false
    else
        print_success "Rust $(rustc --version | awk '{print $2}')"
    fi

    if ! command -v swift &> /dev/null; then
        print_error "Swift not found"
        all_ok=false
    else
        print_success "Swift $(swift --version | head -n1 | awk '{print $4}')"
    fi

    if [ "$all_ok" = false ]; then
        print_error "Prerequisites not met. Please fix issues and try again."
        exit 1
    fi

    echo ""
}

# Create output directory
setup_output_dir() {
    print_step "Setting up output directory"

    rm -rf "$OUTPUT_DIR"
    mkdir -p "$OUTPUT_DIR"

    print_success "Output directory: $OUTPUT_DIR"
}

# Build Rust components
build_rust_components() {
    print_step "Building Rust components"

    # Build CLI
    print_info "Building Auxin CLI..."
    cd "$PROJECT_ROOT/Auxin-CLI-Wrapper"
    cargo build --release
    print_success "CLI built"

    # Build Server
    print_info "Building Auxin Server..."
    cd "$PROJECT_ROOT/auxin-server"
    cargo build --release
    print_success "Server built"

    cd "$PROJECT_ROOT"
}

# Build Swift components
build_swift_components() {
    print_step "Building Swift components"

    # Build Daemon
    print_info "Building Auxin Daemon..."
    cd "$PROJECT_ROOT/Auxin-LaunchAgent"
    swift build -c release
    print_success "Daemon built"

    # Build and bundle App
    print_info "Building Auxin App..."
    cd "$PROJECT_ROOT/Auxin-App"
    swift build -c release
    ./create-app-bundle.sh
    print_success "App built and bundled"

    cd "$PROJECT_ROOT"
}

# Sign binaries
sign_binaries() {
    print_step "Code signing binaries"

    local binaries=(
        "Auxin-CLI-Wrapper/target/release/auxin"
        "Auxin-LaunchAgent/.build/release/auxin-daemon"
        "auxin-server/target/release/auxin-server"
    )

    for binary in "${binaries[@]}"; do
        print_info "Signing $(basename "$binary")..."

        codesign --sign "$APP_CERT" \
                 --options runtime \
                 --timestamp \
                 --force \
                 "$PROJECT_ROOT/$binary"

        # Verify
        if codesign --verify --verbose "$PROJECT_ROOT/$binary" 2>&1; then
            print_success "$(basename "$binary") signed successfully"
        else
            print_error "Failed to sign $(basename "$binary")"
            exit 1
        fi
    done
}

# Sign app bundle
sign_app_bundle() {
    print_step "Code signing application bundle"

    local app_path="$PROJECT_ROOT/Auxin-App/Auxin.app"

    print_info "Signing Auxin.app..."

    codesign --sign "$APP_CERT" \
             --options runtime \
             --timestamp \
             --deep \
             --force \
             "$app_path"

    # Verify
    if codesign --verify --verbose "$app_path" 2>&1; then
        print_success "Auxin.app signed successfully"

        # Check Gatekeeper assessment
        if spctl --assess --verbose "$app_path" 2>&1 | grep -q "accepted"; then
            print_success "Auxin.app passes Gatekeeper assessment"
        else
            print_warning "Gatekeeper assessment warning (may need notarization)"
        fi
    else
        print_error "Failed to sign Auxin.app"
        exit 1
    fi
}

# Build installer package
build_installer_package() {
    print_step "Building installer package"

    cd "$PROJECT_ROOT"

    print_info "Running build-installer.sh..."
    OUTPUT_DIR="$OUTPUT_DIR/installer-build" \
        ./build-installer.sh --version "$VERSION"

    if [ -f "$OUTPUT_DIR/installer-build/Auxin-$VERSION.pkg" ]; then
        print_success "Installer package built"
    else
        print_error "Failed to build installer package"
        exit 1
    fi
}

# Sign installer package
sign_installer_package() {
    print_step "Signing installer package"

    local unsigned_pkg="$OUTPUT_DIR/installer-build/Auxin-$VERSION.pkg"
    local signed_pkg="$OUTPUT_DIR/Auxin-$VERSION-signed.pkg"

    print_info "Signing with productsign..."

    productsign --sign "$INSTALLER_CERT" \
                "$unsigned_pkg" \
                "$signed_pkg"

    # Verify signature
    print_info "Verifying package signature..."
    if pkgutil --check-signature "$signed_pkg"; then
        print_success "Installer package signed successfully"

        # Show signature details
        echo ""
        pkgutil --check-signature "$signed_pkg" | head -n 5
        echo ""
    else
        print_error "Package signature verification failed"
        exit 1
    fi
}

# Notarize installer
notarize_installer() {
    if [ "$SKIP_NOTARIZATION" = true ]; then
        print_step "Skipping notarization (--skip-notarization flag set)"
        return 0
    fi

    print_step "Submitting for notarization"

    local signed_pkg="$OUTPUT_DIR/Auxin-$VERSION-signed.pkg"

    print_info "Uploading to Apple for notarization..."
    print_warning "This typically takes 5-15 minutes"
    echo ""

    # Submit for notarization
    if xcrun notarytool submit "$signed_pkg" \
            --keychain-profile "$NOTARY_PROFILE" \
            --wait; then
        print_success "Notarization accepted!"
    else
        print_error "Notarization failed or was rejected"
        print_info "Check logs with: xcrun notarytool log <SUBMISSION-ID> --keychain-profile $NOTARY_PROFILE"
        exit 1
    fi
}

# Staple notarization
staple_notarization() {
    if [ "$SKIP_NOTARIZATION" = true ]; then
        return 0
    fi

    print_step "Stapling notarization ticket"

    local signed_pkg="$OUTPUT_DIR/Auxin-$VERSION-signed.pkg"

    print_info "Stapling ticket to package..."

    if xcrun stapler staple "$signed_pkg"; then
        print_success "Notarization ticket stapled"

        # Validate
        if xcrun stapler validate "$signed_pkg"; then
            print_success "Staple validation passed"
        else
            print_error "Staple validation failed"
            exit 1
        fi
    else
        print_error "Failed to staple notarization"
        exit 1
    fi
}

# Create DMG
create_dmg() {
    print_step "Creating distribution DMG"

    cd "$PROJECT_ROOT"

    local signed_pkg="$OUTPUT_DIR/Auxin-$VERSION-signed.pkg"

    print_info "Building DMG..."

    ./create-dmg.sh --version "$VERSION" \
                    --installer "$signed_pkg" \
                    --output "$OUTPUT_DIR"

    if [ -f "$OUTPUT_DIR/Auxin-$VERSION.dmg" ]; then
        print_success "DMG created"
    else
        print_error "Failed to create DMG"
        exit 1
    fi
}

# Sign and notarize DMG
sign_notarize_dmg() {
    print_step "Signing and notarizing DMG"

    local dmg_path="$OUTPUT_DIR/Auxin-$VERSION.dmg"

    # Sign DMG
    print_info "Signing DMG..."
    codesign --sign "$APP_CERT" \
             --timestamp \
             --force \
             "$dmg_path"

    if codesign --verify --verbose "$dmg_path" 2>&1; then
        print_success "DMG signed"
    else
        print_error "DMG signing failed"
        exit 1
    fi

    # Notarize DMG (if not skipping)
    if [ "$SKIP_NOTARIZATION" = false ]; then
        print_info "Submitting DMG for notarization..."

        if xcrun notarytool submit "$dmg_path" \
                --keychain-profile "$NOTARY_PROFILE" \
                --wait; then
            print_success "DMG notarization accepted"

            # Staple
            print_info "Stapling DMG..."
            if xcrun stapler staple "$dmg_path"; then
                print_success "DMG notarization stapled"
            else
                print_warning "DMG stapling failed (not critical)"
            fi
        else
            print_error "DMG notarization failed"
            exit 1
        fi
    fi
}

# Verify final products
verify_products() {
    print_step "Verifying final products"

    local signed_pkg="$OUTPUT_DIR/Auxin-$VERSION-signed.pkg"
    local dmg_path="$OUTPUT_DIR/Auxin-$VERSION.dmg"

    # Verify PKG
    print_info "Verifying installer package..."
    if spctl --assess --type install --verbose "$signed_pkg" 2>&1 | grep -q "accepted"; then
        print_success "Installer package passes Gatekeeper"
    else
        print_warning "Installer may show Gatekeeper warnings"
    fi

    # Verify DMG
    if [ -f "$dmg_path" ]; then
        print_info "Verifying DMG..."
        if codesign --verify "$dmg_path" 2>&1; then
            print_success "DMG signature valid"
        else
            print_warning "DMG signature check failed"
        fi
    fi
}

# Create release notes
create_release_notes() {
    print_step "Creating release notes"

    local notes_file="$OUTPUT_DIR/RELEASE_NOTES.txt"

    cat > "$notes_file" << EOF
Auxin $VERSION - Production Release
=====================================

Build Date: $(date)
Build Host: $(hostname)

Files:
------
- Auxin-$VERSION-signed.pkg    Signed and notarized installer package
- Auxin-$VERSION.dmg            Signed and notarized DMG for distribution

Installation:
-------------
1. Download Auxin-$VERSION.dmg
2. Open the DMG
3. Double-click "Install Auxin.pkg"
4. Follow installer prompts

Components:
-----------
- Auxin CLI (required)
- Auxin Daemon (required)
- Auxin Application (recommended)
- Auxin Server (optional - for LAN collaboration)

Prerequisites:
--------------
- macOS 14.0 or later
- Oxen CLI: pip3 install oxen-ai

Documentation:
--------------
- User Guide: https://github.com/jbacus/auxin
- Issues: https://github.com/jbacus/auxin/issues

Signature Verification:
-----------------------
PKG: $(pkgutil --check-signature "$OUTPUT_DIR/Auxin-$VERSION-signed.pkg" 2>&1 | grep "Status" || echo "N/A")
DMG: $(codesign --display --verbose "$OUTPUT_DIR/Auxin-$VERSION.dmg" 2>&1 | grep "Authority" || echo "N/A")

Notarization:
-------------
Status: $([ "$SKIP_NOTARIZATION" = true ] && echo "Skipped (testing build)" || echo "Completed")

EOF

    print_success "Release notes created: $notes_file"
}

# Print summary
print_summary() {
    print_header "Production Build Complete!"

    echo ""
    echo "Version: $VERSION"
    echo "Output directory: $OUTPUT_DIR"
    echo ""
    echo "Files ready for distribution:"
    echo "  📦 $(basename "$OUTPUT_DIR/Auxin-$VERSION-signed.pkg")"
    echo "     Size: $(du -h "$OUTPUT_DIR/Auxin-$VERSION-signed.pkg" | cut -f1)"
    echo "     Signed: ✓"
    echo "     Notarized: $([ "$SKIP_NOTARIZATION" = true ] && echo "⊗ (skipped)" || echo "✓")"
    echo ""
    echo "  💿 $(basename "$OUTPUT_DIR/Auxin-$VERSION.dmg")"
    echo "     Size: $(du -h "$OUTPUT_DIR/Auxin-$VERSION.dmg" | cut -f1)"
    echo "     Signed: ✓"
    echo "     Notarized: $([ "$SKIP_NOTARIZATION" = true ] && echo "⊗ (skipped)" || echo "✓")"
    echo ""
    echo "Next steps:"
    echo "  1. Test installation on clean macOS system"
    echo "  2. Create GitHub release: git tag v$VERSION && git push origin v$VERSION"
    echo "  3. Upload files to GitHub Release"
    echo "  4. Update documentation with download links"
    echo ""
    print_success "Ready for distribution!"
    echo ""
}

# Main build flow
main() {
    parse_args "$@"

    print_header "Auxin Production Build"
    echo "Version: $VERSION"
    echo "Skip Notarization: $SKIP_NOTARIZATION"
    echo ""

    check_prerequisites
    setup_output_dir
    build_rust_components
    build_swift_components
    sign_binaries
    sign_app_bundle
    build_installer_package
    sign_installer_package
    notarize_installer
    staple_notarization
    create_dmg
    sign_notarize_dmg
    verify_products
    create_release_notes
    print_summary
}

# Run main function
main "$@"
