#!/bin/bash
#
# OxVCS Installation Script
# Automated installation for all OxVCS components
#

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
INSTALL_DIR="/usr/local/bin"
LAUNCH_AGENTS_DIR="$HOME/Library/LaunchAgents"
PLIST_NAME="com.oxen.logic.daemon.plist"
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Function to print colored messages
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

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check prerequisites
check_prerequisites() {
    print_header "Checking Prerequisites"

    local all_ok=true

    # Check macOS version
    if [[ $(sw_vers -productVersion | cut -d. -f1) -lt 14 ]]; then
        print_error "macOS 14.0 or later required"
        all_ok=false
    else
        print_success "macOS version: $(sw_vers -productVersion)"
    fi

    # Check Rust
    if command_exists cargo; then
        local rust_version=$(rustc --version | awk '{print $2}')
        print_success "Rust installed: $rust_version"
    else
        print_error "Rust toolchain not found. Install from https://rustup.rs"
        all_ok=false
    fi

    # Check Swift
    if command_exists swift; then
        local swift_version=$(swift --version | head -n1 | awk '{print $4}')
        print_success "Swift installed: $swift_version"
    else
        print_error "Swift not found. Install Xcode Command Line Tools"
        all_ok=false
    fi

    # Check for Xcode Command Line Tools
    if xcode-select -p >/dev/null 2>&1; then
        print_success "Xcode Command Line Tools installed"
    else
        print_error "Xcode Command Line Tools not found. Run: xcode-select --install"
        all_ok=false
    fi

    # Check oxen-ai CLI (optional but recommended)
    if command_exists oxen; then
        print_success "Oxen.ai CLI installed"
    else
        print_warning "Oxen.ai CLI not found (optional). Install: pip install oxen-ai"
    fi

    if [ "$all_ok" = false ]; then
        print_error "Missing prerequisites. Please install required tools and try again."
        exit 1
    fi

    echo ""
}

# Function to build Rust CLI
build_cli() {
    print_header "Building Rust CLI Wrapper"

    cd "$PROJECT_ROOT/OxVCS-CLI-Wrapper"

    print_info "Building oxenvcs-cli in release mode..."
    cargo build --release

    if [ -f "target/release/oxenvcs-cli" ]; then
        print_success "CLI built successfully"
        ls -lh target/release/oxenvcs-cli
    else
        print_error "CLI build failed"
        exit 1
    fi

    cd "$PROJECT_ROOT"
    echo ""
}

# Function to build Swift daemon
build_daemon() {
    print_header "Building Swift LaunchAgent Daemon"

    cd "$PROJECT_ROOT/OxVCS-LaunchAgent"

    print_info "Building oxvcs-daemon in release mode..."
    swift build -c release

    if [ -f ".build/release/oxvcs-daemon" ]; then
        print_success "Daemon built successfully"
        ls -lh .build/release/oxvcs-daemon
    else
        print_error "Daemon build failed"
        exit 1
    fi

    cd "$PROJECT_ROOT"
    echo ""
}

# Function to build Swift app
build_app() {
    print_header "Building Swift App (Optional)"

    cd "$PROJECT_ROOT/OxVCS-App"

    print_info "Building OxVCS-App in release mode..."
    if swift build -c release 2>/dev/null; then
        print_success "App built successfully"
    else
        print_warning "App build skipped or failed (this is optional)"
    fi

    cd "$PROJECT_ROOT"
    echo ""
}

# Function to install binaries
install_binaries() {
    print_header "Installing Binaries"

    # Check if we need sudo
    if [ ! -w "$INSTALL_DIR" ]; then
        print_info "Administrator privileges required to install to $INSTALL_DIR"
        USE_SUDO="sudo"
    else
        USE_SUDO=""
    fi

    # Install CLI
    print_info "Installing oxenvcs-cli to $INSTALL_DIR..."
    $USE_SUDO cp "$PROJECT_ROOT/OxVCS-CLI-Wrapper/target/release/oxenvcs-cli" "$INSTALL_DIR/"
    $USE_SUDO chmod +x "$INSTALL_DIR/oxenvcs-cli"
    print_success "CLI installed: $INSTALL_DIR/oxenvcs-cli"

    # Install daemon
    print_info "Installing oxvcs-daemon to $INSTALL_DIR..."
    $USE_SUDO cp "$PROJECT_ROOT/OxVCS-LaunchAgent/.build/release/oxvcs-daemon" "$INSTALL_DIR/"
    $USE_SUDO chmod +x "$INSTALL_DIR/oxvcs-daemon"
    print_success "Daemon installed: $INSTALL_DIR/oxvcs-daemon"

    echo ""
}

# Function to configure and install plist
install_plist() {
    print_header "Installing LaunchAgent Configuration"

    # Create LaunchAgents directory if it doesn't exist
    mkdir -p "$LAUNCH_AGENTS_DIR"

    # Read the plist template
    local plist_source="$PROJECT_ROOT/OxVCS-LaunchAgent/Resources/$PLIST_NAME"
    local plist_target="$LAUNCH_AGENTS_DIR/$PLIST_NAME"

    if [ ! -f "$plist_source" ]; then
        print_error "Plist template not found at $plist_source"
        exit 1
    fi

    # Copy and configure plist
    print_info "Configuring plist for user: $USER"
    cp "$plist_source" "$plist_target"

    # Replace the UserName placeholder with actual username
    # Note: The plist has a placeholder comment, we'll just ensure the UserName key is set
    if grep -q "<!-- Will be dynamically set during installation -->" "$plist_target"; then
        # Replace the placeholder with actual username
        sed -i.bak "s|<string><!-- Will be dynamically set during installation --></string>|<string>$USER</string>|g" "$plist_target"
        rm -f "$plist_target.bak"
    fi

    # Set correct permissions
    chmod 644 "$plist_target"

    print_success "Plist installed: $plist_target"
    echo ""
}

# Function to register service
register_service() {
    print_header "Registering LaunchAgent Service"

    print_info "Registering service with launchctl..."

    # Unload if already loaded (ignore errors)
    launchctl unload "$LAUNCH_AGENTS_DIR/$PLIST_NAME" 2>/dev/null || true

    # Load the service
    if launchctl load "$LAUNCH_AGENTS_DIR/$PLIST_NAME" 2>/dev/null; then
        print_success "Service registered with launchctl"
    else
        print_warning "Could not register with launchctl (may require system approval)"
    fi

    # Also try the new method using the daemon itself
    print_info "Registering service using SMAppService..."
    if "$INSTALL_DIR/oxvcs-daemon" --install 2>&1 | grep -q "requires approval"; then
        print_warning "Service requires approval in System Settings"
        echo ""
        echo "  To complete installation:"
        echo "  1. Open System Settings"
        echo "  2. Go to General → Login Items & Extensions"
        echo "  3. Find and enable 'Oxen VCS Daemon'"
        echo ""
    else
        print_success "Service registered successfully"
    fi

    echo ""
}

# Function to verify installation
verify_installation() {
    print_header "Verifying Installation"

    local all_ok=true

    # Check CLI
    if [ -x "$INSTALL_DIR/oxenvcs-cli" ]; then
        print_success "CLI binary: $INSTALL_DIR/oxenvcs-cli"
        if "$INSTALL_DIR/oxenvcs-cli" --help >/dev/null 2>&1; then
            print_success "CLI is executable and working"
        else
            print_error "CLI binary exists but is not working properly"
            all_ok=false
        fi
    else
        print_error "CLI binary not found or not executable"
        all_ok=false
    fi

    # Check daemon
    if [ -x "$INSTALL_DIR/oxvcs-daemon" ]; then
        print_success "Daemon binary: $INSTALL_DIR/oxvcs-daemon"
    else
        print_error "Daemon binary not found or not executable"
        all_ok=false
    fi

    # Check plist
    if [ -f "$LAUNCH_AGENTS_DIR/$PLIST_NAME" ]; then
        print_success "Plist file: $LAUNCH_AGENTS_DIR/$PLIST_NAME"
    else
        print_error "Plist file not found"
        all_ok=false
    fi

    # Check service status
    print_info "Checking daemon status..."
    "$INSTALL_DIR/oxvcs-daemon" --status || true

    echo ""

    if [ "$all_ok" = true ]; then
        print_success "Installation verification completed"
    else
        print_error "Installation verification found issues"
        return 1
    fi
}

# Function to print next steps
print_next_steps() {
    print_header "Installation Complete!"

    echo "Next steps:"
    echo ""
    echo "1. Initialize your first Logic Pro project:"
    echo "   cd ~/Music/YourProject.logicx"
    echo "   oxenvcs-cli init --logic ."
    echo ""
    echo "2. Check daemon status:"
    echo "   oxvcs-daemon --status"
    echo ""
    echo "3. If the daemon requires approval:"
    echo "   - Open System Settings"
    echo "   - Go to General → Login Items & Extensions"
    echo "   - Enable 'Oxen VCS Daemon'"
    echo ""
    echo "4. View the Quick Start Guide:"
    echo "   cat $PROJECT_ROOT/docs/QUICKSTART.md"
    echo ""
    echo "For more information, see:"
    echo "  - Quick Start: $PROJECT_ROOT/docs/QUICKSTART.md"
    echo "  - Usage Guide: $PROJECT_ROOT/OxVCS-CLI-Wrapper/USAGE.md"
    echo ""
}

# Function to show usage
show_usage() {
    cat << EOF
OxVCS Installation Script

Usage: $0 [OPTIONS]

Options:
  --help              Show this help message
  --skip-checks       Skip prerequisite checks (not recommended)
  --skip-app          Skip building the UI app
  --uninstall         Uninstall OxVCS components

Examples:
  $0                  # Full installation
  $0 --skip-app       # Install CLI and daemon only
  $0 --uninstall      # Remove all components

EOF
}

# Function to uninstall
uninstall() {
    print_header "Uninstalling OxVCS"

    print_info "Stopping and unregistering service..."
    launchctl unload "$LAUNCH_AGENTS_DIR/$PLIST_NAME" 2>/dev/null || true
    "$INSTALL_DIR/oxvcs-daemon" --uninstall 2>/dev/null || true
    print_success "Service stopped"

    print_info "Removing binaries..."
    if [ -w "$INSTALL_DIR" ]; then
        rm -f "$INSTALL_DIR/oxenvcs-cli" "$INSTALL_DIR/oxvcs-daemon"
    else
        sudo rm -f "$INSTALL_DIR/oxenvcs-cli" "$INSTALL_DIR/oxvcs-daemon"
    fi
    print_success "Binaries removed"

    print_info "Removing plist..."
    rm -f "$LAUNCH_AGENTS_DIR/$PLIST_NAME"
    print_success "Plist removed"

    print_info "Removing logs (optional)..."
    rm -f /tmp/com.oxen.logic.daemon.stdout
    rm -f /tmp/com.oxen.logic.daemon.stderr
    print_success "Logs removed"

    echo ""
    print_success "Uninstallation complete"
    echo ""
    print_info "Note: Repository data (.oxen directories) were not removed"
}

# Main installation flow
main() {
    local skip_checks=false
    local skip_app=false

    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --help)
                show_usage
                exit 0
                ;;
            --skip-checks)
                skip_checks=true
                shift
                ;;
            --skip-app)
                skip_app=true
                shift
                ;;
            --uninstall)
                uninstall
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                show_usage
                exit 1
                ;;
        esac
    done

    # Print banner
    echo ""
    echo "╔════════════════════════════════════════════╗"
    echo "║    OxVCS for Logic Pro - Installer        ║"
    echo "║    Version Control for DAW Projects       ║"
    echo "╚════════════════════════════════════════════╝"
    echo ""

    # Run installation steps
    if [ "$skip_checks" = false ]; then
        check_prerequisites
    fi

    build_cli
    build_daemon

    if [ "$skip_app" = false ]; then
        build_app
    fi

    install_binaries
    install_plist
    register_service
    verify_installation
    print_next_steps
}

# Run main function
main "$@"
