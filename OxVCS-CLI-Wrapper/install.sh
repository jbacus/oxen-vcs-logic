#!/usr/bin/env bash
#
# OxVCS CLI Installation Script
# Installs oxenvcs-cli binary, shell completions, and config template

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Utility functions
log_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

log_success() {
    echo -e "${GREEN}✓${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

log_error() {
    echo -e "${RED}✗${NC} $1"
}

# Detect platform
detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Darwin*)
            PLATFORM="macos"
            ;;
        Linux*)
            PLATFORM="linux"
            ;;
        *)
            log_error "Unsupported operating system: $OS"
            exit 1
            ;;
    esac

    case "$ARCH" in
        x86_64|amd64)
            ARCH="x86_64"
            ;;
        arm64|aarch64)
            ARCH="arm64"
            ;;
        *)
            log_error "Unsupported architecture: $ARCH"
            exit 1
            ;;
    esac

    log_info "Detected platform: $PLATFORM-$ARCH"
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."

    # Check for Rust (required for building from source)
    if ! command -v cargo &> /dev/null; then
        log_warn "Cargo not found. Install Rust from https://rustup.rs/"
        log_info "Will attempt to download pre-built binary..."
        BUILD_FROM_SOURCE=false
    else
        log_success "Rust toolchain found"
        BUILD_FROM_SOURCE=true
    fi

    # Check for Oxen CLI (required for operation)
    if ! command -v oxen &> /dev/null; then
        log_warn "Oxen CLI not found"
        log_info "Install with: pip3 install oxen-ai  OR  cargo install oxen"
        log_info "OxVCS requires Oxen CLI to function"
        OXEN_MISSING=true
    else
        OXEN_VERSION=$(oxen --version 2>&1 | head -n1 || echo "unknown")
        log_success "Oxen CLI found: $OXEN_VERSION"
        OXEN_MISSING=false
    fi
}

# Build from source
build_from_source() {
    log_info "Building oxenvcs-cli from source..."

    if [ ! -f "Cargo.toml" ]; then
        log_error "Cargo.toml not found. Run this script from the OxVCS-CLI-Wrapper directory."
        exit 1
    fi

    # Build release binary
    cargo build --release

    if [ ! -f "target/release/oxenvcs-cli" ]; then
        log_error "Build failed: binary not found at target/release/oxenvcs-cli"
        exit 1
    fi

    BINARY_PATH="target/release/oxenvcs-cli"
    log_success "Build complete: $BINARY_PATH"
}

# Install binary
install_binary() {
    log_info "Installing binary to /usr/local/bin..."

    INSTALL_DIR="/usr/local/bin"

    # Check if we need sudo
    if [ -w "$INSTALL_DIR" ]; then
        cp "$BINARY_PATH" "$INSTALL_DIR/oxenvcs-cli"
        chmod +x "$INSTALL_DIR/oxenvcs-cli"
    else
        log_info "Requesting sudo access to install to $INSTALL_DIR..."
        sudo cp "$BINARY_PATH" "$INSTALL_DIR/oxenvcs-cli"
        sudo chmod +x "$INSTALL_DIR/oxenvcs-cli"
    fi

    log_success "Installed to $INSTALL_DIR/oxenvcs-cli"
}

# Generate shell completions
generate_completions() {
    log_info "Generating shell completions..."

    mkdir -p completions

    # Generate for all shells
    "$BINARY_PATH" completions bash > completions/oxenvcs-cli.bash
    "$BINARY_PATH" completions zsh > completions/_oxenvcs-cli
    "$BINARY_PATH" completions fish > completions/oxenvcs-cli.fish
    "$BINARY_PATH" completions powershell > completions/oxenvcs-cli.ps1

    log_success "Shell completions generated in completions/"
}

# Install shell completions
install_completions() {
    log_info "Installing shell completions..."

    # Detect user's shell
    USER_SHELL=$(basename "$SHELL")

    case "$USER_SHELL" in
        bash)
            install_bash_completion
            ;;
        zsh)
            install_zsh_completion
            ;;
        fish)
            install_fish_completion
            ;;
        *)
            log_warn "Shell $USER_SHELL not recognized. Skipping completion installation."
            log_info "Manual installation: see completions/README.md"
            return
            ;;
    esac
}

install_bash_completion() {
    if [ "$PLATFORM" = "macos" ]; then
        # macOS with Homebrew
        if [ -d "/usr/local/etc/bash_completion.d" ]; then
            COMPLETION_DIR="/usr/local/etc/bash_completion.d"
        elif [ -d "/opt/homebrew/etc/bash_completion.d" ]; then
            COMPLETION_DIR="/opt/homebrew/etc/bash_completion.d"
        else
            # User-specific fallback
            COMPLETION_DIR="$HOME/.local/share/bash-completion/completions"
            mkdir -p "$COMPLETION_DIR"
        fi
    else
        # Linux
        if [ -d "/etc/bash_completion.d" ] && [ -w "/etc/bash_completion.d" ]; then
            COMPLETION_DIR="/etc/bash_completion.d"
        else
            COMPLETION_DIR="$HOME/.local/share/bash-completion/completions"
            mkdir -p "$COMPLETION_DIR"
        fi
    fi

    cp completions/oxenvcs-cli.bash "$COMPLETION_DIR/oxenvcs-cli"
    log_success "Bash completion installed to $COMPLETION_DIR"
    log_info "Restart your shell or run: source $COMPLETION_DIR/oxenvcs-cli"
}

install_zsh_completion() {
    # User-specific installation (most portable)
    COMPLETION_DIR="$HOME/.zsh/completions"
    mkdir -p "$COMPLETION_DIR"

    cp completions/_oxenvcs-cli "$COMPLETION_DIR/"
    log_success "Zsh completion installed to $COMPLETION_DIR"

    # Check if fpath includes our completion dir
    if ! grep -q "fpath=($COMPLETION_DIR" "$HOME/.zshrc" 2>/dev/null; then
        log_info "Add to ~/.zshrc:"
        echo ""
        echo "  fpath=($COMPLETION_DIR \$fpath)"
        echo "  autoload -Uz compinit"
        echo "  compinit"
        echo ""
    fi

    log_info "Restart your shell or run: rm -f ~/.zcompdump* && compinit"
}

install_fish_completion() {
    COMPLETION_DIR="$HOME/.config/fish/completions"
    mkdir -p "$COMPLETION_DIR"

    cp completions/oxenvcs-cli.fish "$COMPLETION_DIR/"
    log_success "Fish completion installed to $COMPLETION_DIR"
    log_info "Fish will automatically load completions on next shell start"
}

# Create default config
create_default_config() {
    log_info "Creating default config..."

    CONFIG_DIR="$HOME/.oxenvcs"
    CONFIG_FILE="$CONFIG_DIR/config.toml"

    mkdir -p "$CONFIG_DIR"

    if [ -f "$CONFIG_FILE" ]; then
        log_warn "Config already exists at $CONFIG_FILE"
        log_info "Backup created at $CONFIG_FILE.backup"
        cp "$CONFIG_FILE" "$CONFIG_FILE.backup"
    fi

    # Copy example config if it exists
    if [ -f "config.toml.example" ]; then
        cp config.toml.example "$CONFIG_FILE"
        log_success "Config created at $CONFIG_FILE"
    else
        log_warn "config.toml.example not found, skipping config creation"
    fi
}

# Verify installation
verify_installation() {
    log_info "Verifying installation..."

    if ! command -v oxenvcs-cli &> /dev/null; then
        log_error "oxenvcs-cli not found in PATH"
        log_info "You may need to add /usr/local/bin to your PATH"
        return 1
    fi

    VERSION=$(oxenvcs-cli --version 2>&1 || echo "unknown")
    log_success "oxenvcs-cli installed successfully: $VERSION"

    if [ "$OXEN_MISSING" = true ]; then
        log_warn "Oxen CLI is not installed (required for operation)"
        log_info "Install with:"
        echo ""
        echo "  pip3 install oxen-ai"
        echo "  OR"
        echo "  cargo install oxen"
        echo ""
    fi

    return 0
}

# Print summary
print_summary() {
    echo ""
    log_success "Installation complete!"
    echo ""
    echo "Next steps:"
    echo "  1. Initialize a Logic Pro project:"
    echo "     $ cd /path/to/your-project.logicx"
    echo "     $ oxenvcs-cli init"
    echo ""
    echo "  2. View available commands:"
    echo "     $ oxenvcs-cli --help"
    echo ""
    echo "  3. Configure settings (optional):"
    echo "     $ nano ~/.oxenvcs/config.toml"
    echo ""

    if [ "$USER_SHELL" = "zsh" ]; then
        echo "  4. Enable completions (add to ~/.zshrc):"
        echo "     fpath=(~/.zsh/completions \$fpath)"
        echo "     autoload -Uz compinit && compinit"
        echo ""
    fi

    if [ "$OXEN_MISSING" = true ]; then
        log_warn "Don't forget to install Oxen CLI!"
        echo "  pip3 install oxen-ai"
        echo ""
    fi

    echo "Documentation: https://github.com/jbacus/oxen-vcs-logic"
}

# Main installation flow
main() {
    echo ""
    log_info "OxVCS CLI Installation Script"
    echo ""

    detect_platform
    check_prerequisites

    # Build or download
    if [ "$BUILD_FROM_SOURCE" = true ]; then
        build_from_source
    else
        log_error "Pre-built binaries not yet available. Please install Rust and try again."
        log_info "Install Rust: https://rustup.rs/"
        exit 1
    fi

    # Install components
    install_binary
    generate_completions
    install_completions
    create_default_config

    # Verify and summarize
    if verify_installation; then
        print_summary
        exit 0
    else
        log_error "Installation verification failed"
        exit 1
    fi
}

# Run main function
main
