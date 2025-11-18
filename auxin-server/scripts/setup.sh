#!/bin/bash
set -e

# Auxin Server Local Deployment Setup Script
# macOS-specific deployment for local development/testing

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
INSTALL_DIR="/usr/local/bin"
DATA_DIR="/var/oxen/data"
CONFIG_DIR="$HOME/.config/auxin-server"
LAUNCHD_DIR="$HOME/Library/LaunchAgents"
LAUNCHD_LABEL="com.auxin.server"
LAUNCHD_PLIST="$LAUNCHD_DIR/$LAUNCHD_LABEL.plist"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Print colored message
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if running on macOS
check_macos() {
    if [[ "$OSTYPE" != "darwin"* ]]; then
        log_error "This script is for macOS only. Detected: $OSTYPE"
        exit 1
    fi
    log_success "Running on macOS"
}

# Check if Rust is installed
check_rust() {
    log_info "Checking for Rust installation..."
    if ! command -v cargo &> /dev/null; then
        log_error "Rust is not installed"
        log_info "Install Rust with: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi
    local rust_version=$(rustc --version)
    log_success "Rust found: $rust_version"
}

# Create necessary directories
create_directories() {
    log_info "Creating directories..."

    # Data directory (may need sudo)
    if [ ! -d "$DATA_DIR" ]; then
        log_info "Creating data directory: $DATA_DIR"
        if sudo mkdir -p "$DATA_DIR" && sudo chown "$USER" "$DATA_DIR"; then
            log_success "Data directory created"
        else
            log_error "Failed to create data directory"
            exit 1
        fi
    else
        log_success "Data directory already exists: $DATA_DIR"
    fi

    # Config directory
    if [ ! -d "$CONFIG_DIR" ]; then
        mkdir -p "$CONFIG_DIR"
        log_success "Config directory created: $CONFIG_DIR"
    else
        log_success "Config directory already exists: $CONFIG_DIR"
    fi

    # LaunchAgents directory
    if [ ! -d "$LAUNCHD_DIR" ]; then
        mkdir -p "$LAUNCHD_DIR"
        log_success "LaunchAgents directory created"
    fi
}

# Setup environment configuration
setup_config() {
    log_info "Setting up configuration..."

    local env_file="$CONFIG_DIR/.env"

    if [ -f "$env_file" ]; then
        log_warn "Configuration already exists: $env_file"
        read -p "Overwrite? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_info "Keeping existing configuration"
            return
        fi
    fi

    # Generate random auth token secret
    local auth_secret=$(openssl rand -hex 32)

    cat > "$env_file" <<EOF
# Auxin Server Configuration
# Generated: $(date)

# Server
SYNC_DIR=$DATA_DIR
OXEN_SERVER_PORT=3000
OXEN_SERVER_HOST=127.0.0.1

# Authentication
AUTH_TOKEN_SECRET=$auth_secret
AUTH_TOKEN_EXPIRY_HOURS=24

# Logging
RUST_LOG=info,auxin_server=debug

# Optional Features (set to true to enable)
ENABLE_REDIS_LOCKS=false
ENABLE_WEB_UI=false

# Optional: Redis (only if ENABLE_REDIS_LOCKS=true)
# REDIS_URL=redis://localhost:6379

# Optional: PostgreSQL (only if ENABLE_WEB_UI=true)
# DATABASE_URL=postgres://auxin:password@localhost:5432/auxin
EOF

    chmod 600 "$env_file"
    log_success "Configuration created: $env_file"
}

# Build the server
build_server() {
    log_info "Building auxin-server..."

    cd "$PROJECT_DIR"

    # Clean previous builds
    log_info "Cleaning previous builds..."
    cargo clean

    # Build release version
    log_info "Building release binary (this may take a few minutes)..."
    if cargo build --release; then
        log_success "Build completed successfully"
    else
        log_error "Build failed"
        exit 1
    fi

    # Verify binary exists
    if [ ! -f "$PROJECT_DIR/target/release/auxin-server" ]; then
        log_error "Binary not found at: $PROJECT_DIR/target/release/auxin-server"
        exit 1
    fi

    local binary_size=$(du -h "$PROJECT_DIR/target/release/auxin-server" | cut -f1)
    log_success "Binary size: $binary_size"
}

# Install the binary
install_binary() {
    log_info "Installing binary to $INSTALL_DIR..."

    if sudo cp "$PROJECT_DIR/target/release/auxin-server" "$INSTALL_DIR/"; then
        sudo chmod 755 "$INSTALL_DIR/auxin-server"
        log_success "Binary installed: $INSTALL_DIR/auxin-server"
    else
        log_error "Failed to install binary"
        exit 1
    fi

    # Verify installation
    if command -v auxin-server &> /dev/null; then
        local version=$(auxin-server --version 2>&1 || echo "unknown")
        log_success "auxin-server installed: $version"
    fi
}

# Create LaunchDaemon plist
create_launchd_plist() {
    log_info "Creating LaunchAgent plist..."

    cat > "$LAUNCHD_PLIST" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>$LAUNCHD_LABEL</string>

    <key>ProgramArguments</key>
    <array>
        <string>$INSTALL_DIR/auxin-server</string>
    </array>

    <key>EnvironmentVariables</key>
    <dict>
        <key>ENV_FILE</key>
        <string>$CONFIG_DIR/.env</string>
    </dict>

    <key>WorkingDirectory</key>
    <string>$DATA_DIR</string>

    <key>StandardOutPath</key>
    <string>$HOME/Library/Logs/auxin-server.log</string>

    <key>StandardErrorPath</key>
    <string>$HOME/Library/Logs/auxin-server-error.log</string>

    <key>RunAtLoad</key>
    <true/>

    <key>KeepAlive</key>
    <dict>
        <key>SuccessfulExit</key>
        <false/>
    </dict>

    <key>ProcessType</key>
    <string>Interactive</string>

    <key>Nice</key>
    <integer>0</integer>
</dict>
</plist>
EOF

    chmod 644 "$LAUNCHD_PLIST"
    log_success "LaunchAgent plist created: $LAUNCHD_PLIST"
}

# Print next steps
print_next_steps() {
    echo ""
    log_success "===== Installation Complete ====="
    echo ""
    echo "Configuration file: $CONFIG_DIR/.env"
    echo "Data directory: $DATA_DIR"
    echo "Binary location: $INSTALL_DIR/auxin-server"
    echo "LaunchAgent: $LAUNCHD_PLIST"
    echo ""
    echo "To start the server:"
    echo "  $SCRIPT_DIR/start.sh"
    echo ""
    echo "To stop the server:"
    echo "  $SCRIPT_DIR/stop.sh"
    echo ""
    echo "To check status:"
    echo "  $SCRIPT_DIR/status.sh"
    echo ""
    echo "To view logs:"
    echo "  tail -f ~/Library/Logs/auxin-server.log"
    echo ""
    echo "To test the server:"
    echo "  curl http://localhost:3000/health"
    echo ""
    echo "To uninstall:"
    echo "  $SCRIPT_DIR/uninstall.sh"
    echo ""
}

# Main installation flow
main() {
    echo ""
    log_info "===== Auxin Server Local Deployment Setup ====="
    echo ""

    check_macos
    check_rust
    create_directories
    setup_config
    build_server
    install_binary
    create_launchd_plist

    print_next_steps
}

# Run main function
main "$@"
