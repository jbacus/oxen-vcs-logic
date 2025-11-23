#!/bin/bash
#
# Auxin Installation Script
# Automated installation for all Auxin components
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
PLIST_NAME="com.auxin.daemon.plist"
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

    cd "$PROJECT_ROOT/Auxin-CLI-Wrapper"

    print_info "Building auxin in release mode..."
    cargo build --release

    if [ -f "target/release/auxin" ]; then
        print_success "CLI built successfully"
        ls -lh target/release/auxin
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

    cd "$PROJECT_ROOT/Auxin-LaunchAgent"

    print_info "Building auxin-daemon in release mode..."
    swift build -c release

    if [ -f ".build/release/auxin-daemon" ]; then
        print_success "Daemon built successfully"
        ls -lh .build/release/auxin-daemon
    else
        print_error "Daemon build failed"
        exit 1
    fi

    cd "$PROJECT_ROOT"
    echo ""
}

# Function to build Swift app
build_app() {
    print_header "Building Auxin Application Bundle"
    # Note: Auxin-App uses SwiftUI (migrated from AppKit 2025-10-29)

    cd "$PROJECT_ROOT/Auxin-App"

    print_info "Building Auxin-App (SwiftUI) in release mode..."
    if swift build -c release; then
        print_success "App built successfully"
    else
        print_error "App build failed"
        cd "$PROJECT_ROOT"
        return 1
    fi

    print_info "Creating Auxin.app bundle..."
    if ./create-app-bundle.sh; then
        print_success "App bundle created: Auxin.app"
    else
        print_error "App bundle creation failed"
        cd "$PROJECT_ROOT"
        return 1
    fi

    cd "$PROJECT_ROOT"
    echo ""
}

# Function to clean previous installation state
clean_previous_install() {
    print_header "Cleaning Previous Installation State"

    # Stop and unload daemon if running
    if launchctl list | grep -q "com.auxin.daemon"; then
        print_info "Stopping existing daemon..."
        launchctl unload "$LAUNCH_AGENTS_DIR/$PLIST_NAME" 2>/dev/null || true
        # Wait a moment for daemon to stop
        sleep 1
        print_success "Existing daemon stopped"
    fi

    # Kill any running daemon processes
    if pgrep -x "auxin-daemon" >/dev/null; then
        print_info "Terminating running daemon processes..."
        pkill -TERM "auxin-daemon" 2>/dev/null || true
        sleep 1
        # Force kill if still running
        pkill -KILL "auxin-daemon" 2>/dev/null || true
        print_success "Daemon processes terminated"
    fi

    # Clear daemon logs
    if [ -f "/tmp/com.auxin.daemon.stdout" ] || [ -f "/tmp/com.auxin.daemon.stderr" ]; then
        print_info "Clearing daemon logs..."
        rm -f /tmp/com.auxin.daemon.stdout
        rm -f /tmp/com.auxin.daemon.stderr
        print_success "Daemon logs cleared"
    fi

    # Clear any XPC cache (macOS 10.15+)
    local xpc_cache="$HOME/Library/Caches/com.auxin.daemon"
    if [ -d "$xpc_cache" ]; then
        print_info "Clearing XPC cache..."
        rm -rf "$xpc_cache"
        print_success "XPC cache cleared"
    fi

    echo ""
}

# Function for full clean (--clean flag)
full_clean() {
    print_header "Full Clean - Removing All Auxin Components"

    # Stop and unload daemon if running
    if launchctl list | grep -q "com.auxin.daemon"; then
        print_info "Stopping daemon..."
        launchctl unload "$LAUNCH_AGENTS_DIR/$PLIST_NAME" 2>/dev/null || true
        sleep 1
    fi

    # Kill any running processes
    print_info "Terminating all Auxin processes..."
    pkill -TERM "auxin-daemon" 2>/dev/null || true
    pkill -TERM "Auxin" 2>/dev/null || true
    sleep 1
    pkill -KILL "auxin-daemon" 2>/dev/null || true
    pkill -KILL "Auxin" 2>/dev/null || true
    print_success "All processes terminated"

    # Remove installed binaries
    print_info "Removing installed binaries..."
    local removed_binaries=false
    if [ -f "$INSTALL_DIR/auxin" ] || [ -f "$INSTALL_DIR/auxin-daemon" ]; then
        if [ -w "$INSTALL_DIR" ]; then
            rm -f "$INSTALL_DIR/auxin" "$INSTALL_DIR/auxin-daemon"
        else
            sudo rm -f "$INSTALL_DIR/auxin" "$INSTALL_DIR/auxin-daemon"
        fi
        removed_binaries=true
    fi
    if [ "$removed_binaries" = true ]; then
        print_success "Binaries removed from $INSTALL_DIR"
    else
        print_info "No binaries found in $INSTALL_DIR"
    fi

    # Remove plist
    if [ -f "$LAUNCH_AGENTS_DIR/$PLIST_NAME" ]; then
        print_info "Removing LaunchAgent plist..."
        rm -f "$LAUNCH_AGENTS_DIR/$PLIST_NAME"
        print_success "Plist removed"
    fi

    # Remove app bundle
    if [ -d "/Applications/Auxin.app" ]; then
        print_info "Removing Auxin.app..."
        if [ -w "/Applications/Auxin.app" ]; then
            rm -rf "/Applications/Auxin.app"
        else
            sudo rm -rf "/Applications/Auxin.app"
        fi
        print_success "Auxin.app removed"
    fi

    # Remove logs
    print_info "Clearing logs and caches..."
    rm -f /tmp/com.auxin.daemon.stdout
    rm -f /tmp/com.auxin.daemon.stderr
    rm -f /tmp/auxin-app-debug.log
    rm -f /tmp/auxin-restart-button.log
    rm -rf "$HOME/Library/Caches/com.auxin.daemon"
    rm -rf "$HOME/Library/Caches/com.auxin.app"
    print_success "Logs and caches cleared"

    # Remove config (optional - ask user)
    if [ -d "$HOME/.config/auxin" ]; then
        print_warning "Config directory found: $HOME/.config/auxin"
        echo "  This contains your CLI configuration settings."
        read -p "  Remove config directory? (y/N): " remove_config
        if [[ "$remove_config" =~ ^[Yy]$ ]]; then
            rm -rf "$HOME/.config/auxin"
            print_success "Config directory removed"
        else
            print_info "Config directory kept"
        fi
    fi

    # Remove shell completions
    print_info "Removing shell completions..."
    rm -f /usr/local/etc/bash_completion.d/auxin 2>/dev/null || true
    rm -f /opt/homebrew/etc/bash_completion.d/auxin 2>/dev/null || true
    rm -f "$HOME/.local/share/bash-completion/completions/auxin" 2>/dev/null || true
    rm -f "$HOME/.zsh/completions/_auxin" 2>/dev/null || true
    rm -f "$HOME/.config/fish/completions/auxin.fish" 2>/dev/null || true
    print_success "Shell completions removed"

    echo ""
    print_success "Full clean complete - system is ready for fresh installation"
    echo ""
    print_info "Note: Repository data (.oxen directories in projects) was not removed"
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
    print_info "Installing auxin to $INSTALL_DIR..."
    $USE_SUDO cp "$PROJECT_ROOT/Auxin-CLI-Wrapper/target/release/auxin" "$INSTALL_DIR/"
    $USE_SUDO chmod +x "$INSTALL_DIR/auxin"
    print_success "CLI installed: $INSTALL_DIR/auxin"

    # Install daemon
    print_info "Installing auxin-daemon to $INSTALL_DIR..."
    $USE_SUDO cp "$PROJECT_ROOT/Auxin-LaunchAgent/.build/release/auxin-daemon" "$INSTALL_DIR/"
    $USE_SUDO chmod +x "$INSTALL_DIR/auxin-daemon"
    print_success "Daemon installed: $INSTALL_DIR/auxin-daemon"

    echo ""
}

# Function to install app bundle
install_app() {
    print_header "Installing Auxin Application"

    local APP_SOURCE="$PROJECT_ROOT/Auxin-App/Auxin.app"
    local APP_DEST="/Applications/Auxin.app"

    if [ ! -d "$APP_SOURCE" ]; then
        print_warning "App bundle not found at $APP_SOURCE (this is optional)"
        return 0
    fi

    print_info "Installing Auxin.app to /Applications..."

    # Check if we need sudo
    local USE_SUDO=""
    if [ -d "$APP_DEST" ] && [ ! -w "$APP_DEST" ]; then
        USE_SUDO="sudo"
    fi

    # Remove existing installation if present
    if [ -d "$APP_DEST" ]; then
        print_info "Removing existing installation..."
        $USE_SUDO rm -rf "$APP_DEST"
    fi

    # Copy app bundle to Applications (may need sudo if /Applications is not writable)
    if [ ! -w "/Applications" ]; then
        USE_SUDO="sudo"
    fi
    $USE_SUDO cp -R "$APP_SOURCE" "$APP_DEST"

    if [ -d "$APP_DEST" ]; then
        print_success "App installed: $APP_DEST"
        echo ""
        print_info "You can now:"
        echo "  • Double-click Auxin in Applications folder"
        echo "  • Or run: open /Applications/Auxin.app"
    else
        print_error "Failed to install app bundle"
        return 1
    fi

    echo ""
}

# Function to generate and install shell completions
generate_completions() {
    print_header "Generating Shell Completions"
    
    local COMPLETIONS_DIR="$PROJECT_ROOT/Auxin-CLI-Wrapper/completions"
    mkdir -p "$COMPLETIONS_DIR"

    print_info "Generating completions for bash, zsh, fish..."
    "$INSTALL_DIR/auxin" completions bash > "$COMPLETIONS_DIR/auxin.bash"
    "$INSTALL_DIR/auxin" completions zsh > "$COMPLETIONS_DIR/_auxin"
    "$INSTALL_DIR/auxin" completions fish > "$COMPLETIONS_DIR/auxin.fish"
    
    print_success "Shell completions generated in $COMPLETIONS_DIR"
    echo ""
}

install_completions() {
    print_header "Installing Shell Completions"
    
    local USER_SHELL=$(basename "$SHELL")
    print_info "Detected shell: $USER_SHELL"

    case "$USER_SHELL" in
        bash)
            local COMPLETION_DIR
            if [ -d "/usr/local/etc/bash_completion.d" ]; then
                COMPLETION_DIR="/usr/local/etc/bash_completion.d"
            elif [ -d "/opt/homebrew/etc/bash_completion.d" ]; then
                COMPLETION_DIR="/opt/homebrew/etc/bash_completion.d"
            else
                COMPLETION_DIR="$HOME/.local/share/bash-completion/completions"
                mkdir -p "$COMPLETION_DIR"
            fi
            cp "$PROJECT_ROOT/Auxin-CLI-Wrapper/completions/auxin.bash" "$COMPLETION_DIR/auxin"
            print_success "Bash completion installed to $COMPLETION_DIR"
            print_info "Restart your shell or run: source $COMPLETION_DIR/auxin"
            ;;
        zsh)
            local COMPLETION_DIR="$HOME/.zsh/completions"
            mkdir -p "$COMPLETION_DIR"
            cp "$PROJECT_ROOT/Auxin-CLI-Wrapper/completions/_auxin" "$COMPLETION_DIR/"
            print_success "Zsh completion installed to $COMPLETION_DIR"
            if ! grep -q "fpath=($COMPLETION_DIR" "$HOME/.zshrc" 2>/dev/null; then
                print_warning "To enable completions, add the following to your ~/.zshrc file:"
                echo '  fpath=(~/.zsh/completions $fpath)'
                echo '  autoload -Uz compinit && compinit'
            fi
            ;;
        fish)
            local COMPLETION_DIR="$HOME/.config/fish/completions"
            mkdir -p "$COMPLETION_DIR"
            cp "$PROJECT_ROOT/Auxin-CLI-Wrapper/completions/auxin.fish" "$COMPLETION_DIR/"
            print_success "Fish completion installed to $COMPLETION_DIR"
            ;;
        *)
            print_warning "Shell $USER_SHELL not recognized. Skipping completion installation."
            print_info "Manual installation instructions are in: Auxin-CLI-Wrapper/completions/README.md"
            ;;
    esac
    echo ""
}

# Function to create default config
create_default_config() {
    print_header "Creating Default CLI Configuration"
    
    local CONFIG_DIR="$HOME/.config/auxin"
    local CONFIG_FILE="$CONFIG_DIR/config.toml"
    local EXAMPLE_CONFIG="$PROJECT_ROOT/Auxin-CLI-Wrapper/config.toml.example"

    if [ ! -f "$EXAMPLE_CONFIG" ]; then
        print_warning "Example config not found at $EXAMPLE_CONFIG, skipping."
        return
    fi

    mkdir -p "$CONFIG_DIR"

    if [ -f "$CONFIG_FILE" ]; then
        print_warning "Config file already exists at $CONFIG_FILE. Skipping creation."
    else
        cp "$EXAMPLE_CONFIG" "$CONFIG_FILE"
        print_success "Default config file created at $CONFIG_FILE"
    fi
    echo ""
}

# Function to configure and install plist
install_plist() {
    print_header "Installing LaunchAgent Configuration"

    # Create LaunchAgents directory if it doesn't exist
    mkdir -p "$LAUNCH_AGENTS_DIR"

    # Read the plist template
    local plist_source="$PROJECT_ROOT/Auxin-LaunchAgent/Resources/$PLIST_NAME"
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
    if "$INSTALL_DIR/auxin-daemon" --install 2>&1 | grep -q "requires approval"; then
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
    if [ -x "$INSTALL_DIR/auxin" ]; then
        print_success "CLI binary: $INSTALL_DIR/auxin"
        if "$INSTALL_DIR/auxin" --help >/dev/null 2>&1; then
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
    if [ -x "$INSTALL_DIR/auxin-daemon" ]; then
        print_success "Daemon binary: $INSTALL_DIR/auxin-daemon"
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
    "$INSTALL_DIR/auxin-daemon" --status || true

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

    # Check if app was installed
    if [ -d "/Applications/Auxin.app" ]; then
        echo "1. Launch the Auxin application:"
        echo "   - Open Finder → Applications → Auxin"
        echo "   - Or run: open /Applications/Auxin.app"
        echo "   - Use the GUI to initialize and manage projects"
        echo ""
        echo "2. Or use the command-line interface:"
        echo "   cd ~/Music/YourProject.logicx"
        echo "   auxin init --logic ."
    else
        echo "1. Initialize your first Logic Pro project:"
        echo "   cd ~/Music/YourProject.logicx"
        echo "   auxin init --logic ."
    fi

    echo ""
    echo "3. Check daemon status:"
    echo "   auxin-daemon --status"
    echo ""
    echo "4. If the daemon requires approval:"
    echo "   - Open System Settings"
    echo "   - Go to General → Login Items & Extensions"
    echo "   - Enable 'Oxen VCS Daemon'"
    echo ""
    echo "5. View the Quick Start Guide:"
    echo "   cat $PROJECT_ROOT/docs/QUICKSTART.md"
    echo ""
    echo "For more information, see:"
    echo "  - Quick Start: $PROJECT_ROOT/docs/QUICKSTART.md"
    echo "  - Usage Guide: $PROJECT_ROOT/Auxin-CLI-Wrapper/USAGE.md"
    echo ""
}

# Function to show usage
show_usage() {
    cat << EOF
Auxin Installation Script

Usage: $0 [OPTIONS]

Options:
  --help              Show this help message
  --skip-checks       Skip prerequisite checks (not recommended)
  --skip-app          Skip building the UI app
  --clean             Full cleanup before install (kills processes, removes binaries)
  --uninstall         Uninstall Auxin components

Examples:
  $0                  # Full installation
  $0 --clean          # Clean install (removes everything first)
  $0 --skip-app       # Install CLI and daemon only
  $0 --uninstall      # Remove all components

EOF
}

# Function to uninstall
uninstall() {
    print_header "Uninstalling Auxin"

    print_info "Stopping and unregistering service..."
    launchctl unload "$LAUNCH_AGENTS_DIR/$PLIST_NAME" 2>/dev/null || true
    "$INSTALL_DIR/auxin-daemon" --uninstall 2>/dev/null || true
    print_success "Service stopped"

    print_info "Removing binaries..."
    if [ -w "$INSTALL_DIR" ]; then
        rm -f "$INSTALL_DIR/auxin" "$INSTALL_DIR/auxin-daemon"
    else
        sudo rm -f "$INSTALL_DIR/auxin" "$INSTALL_DIR/auxin-daemon"
    fi
    print_success "Binaries removed"

    print_info "Removing plist..."
    rm -f "$LAUNCH_AGENTS_DIR/$PLIST_NAME"
    print_success "Plist removed"

    print_info "Removing logs (optional)..."
    rm -f /tmp/com.auxin.daemon.stdout
    rm -f /tmp/com.auxin.daemon.stderr
    print_success "Logs removed"

    print_info "Removing app bundle (if installed)..."
    if [ -d "/Applications/Auxin.app" ]; then
        rm -rf "/Applications/Auxin.app"
        print_success "App bundle removed"
    else
        print_info "App bundle not found (skipping)"
    fi

    echo ""
    print_success "Uninstallation complete"
    echo ""
    print_info "Note: Repository data (.oxen directories) were not removed"
}

# Main installation flow
main() {
    local skip_checks=false
    local skip_app=false
    local do_clean=false

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
            --clean)
                do_clean=true
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
    echo "║    Auxin for Logic Pro - Installer        ║"
    echo "║    Version Control for DAW Projects       ║"
    echo "╚════════════════════════════════════════════╝"
    echo ""

    # Run installation steps
    if [ "$skip_checks" = false ]; then
        check_prerequisites
    fi

    # Perform full clean if requested
    if [ "$do_clean" = true ]; then
        full_clean
    else
        # Otherwise just clean state (logs, caches, running processes)
        clean_previous_install
    fi

    build_cli
    build_daemon

    if [ "$skip_app" = false ]; then
        build_app
    fi

    install_binaries

    if [ "$skip_app" = false ]; then
        install_app
    fi

    generate_completions
    install_completions
    create_default_config

    install_plist
    register_service
    verify_installation
    print_next_steps
}

# Run main function
main "$@"
