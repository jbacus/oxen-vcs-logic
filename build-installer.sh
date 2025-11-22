#!/bin/bash
#
# Auxin macOS Installer Builder
# Creates a complete .pkg installer for Auxin CLI, App, Daemon, and optionally Server
#
# Usage:
#   ./build-installer.sh [--version VERSION] [--output DIR]
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
OUTPUT_DIR="${OUTPUT_DIR:-$PROJECT_ROOT/installer-build}"
BUILD_DIR="$OUTPUT_DIR/build"
PACKAGES_DIR="$OUTPUT_DIR/packages"
RESOURCES_DIR="$OUTPUT_DIR/resources"
SCRIPTS_DIR="$OUTPUT_DIR/scripts"

# Component paths
CLI_BUILD="$PROJECT_ROOT/Auxin-CLI-Wrapper/target/release/auxin"
DAEMON_BUILD="$PROJECT_ROOT/Auxin-LaunchAgent/.build/release/auxin-daemon"
APP_BUILD="$PROJECT_ROOT/Auxin-App/Auxin.app"
SERVER_BUILD="$PROJECT_ROOT/auxin-server/target/release/auxin-server"

# Install locations
INSTALL_BIN="/usr/local/bin"
INSTALL_APP="/Applications"
INSTALL_LAUNCHAGENT="$HOME/Library/LaunchAgents"

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
            --output)
                OUTPUT_DIR="$2"
                BUILD_DIR="$OUTPUT_DIR/build"
                PACKAGES_DIR="$OUTPUT_DIR/packages"
                RESOURCES_DIR="$OUTPUT_DIR/resources"
                SCRIPTS_DIR="$OUTPUT_DIR/scripts"
                shift 2
                ;;
            --help)
                cat << EOF
Auxin macOS Installer Builder

Usage: $0 [OPTIONS]

Options:
  --version VERSION    Set installer version (default: 1.0.0)
  --output DIR         Set output directory (default: ./installer-build)
  --help              Show this help message

Examples:
  $0                           # Build with defaults
  $0 --version 1.2.3          # Build version 1.2.3
  $0 --output /tmp/auxin      # Custom output directory

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
        print_error "This installer builder requires macOS"
        all_ok=false
    else
        print_success "Running on macOS"
    fi

    # Check pkgbuild
    if ! command -v pkgbuild &> /dev/null; then
        print_error "pkgbuild not found (requires Xcode Command Line Tools)"
        all_ok=false
    else
        print_success "pkgbuild found"
    fi

    # Check productbuild
    if ! command -v productbuild &> /dev/null; then
        print_error "productbuild not found (requires Xcode Command Line Tools)"
        all_ok=false
    else
        print_success "productbuild found"
    fi

    # Check Rust
    if ! command -v cargo &> /dev/null; then
        print_error "Rust toolchain not found"
        all_ok=false
    else
        print_success "Rust installed: $(rustc --version | awk '{print $2}')"
    fi

    # Check Swift
    if ! command -v swift &> /dev/null; then
        print_error "Swift not found"
        all_ok=false
    else
        print_success "Swift installed: $(swift --version | head -n1 | awk '{print $4}')"
    fi

    if [ "$all_ok" = false ]; then
        print_error "Missing prerequisites. Please install required tools."
        exit 1
    fi

    echo ""
}

# Clean and create build directories
setup_build_dirs() {
    print_header "Setting Up Build Directories"

    print_info "Cleaning old build artifacts..."
    rm -rf "$OUTPUT_DIR"

    print_info "Creating directory structure..."
    mkdir -p "$BUILD_DIR"/{cli,daemon,app,server}
    mkdir -p "$PACKAGES_DIR"
    mkdir -p "$RESOURCES_DIR"
    mkdir -p "$SCRIPTS_DIR"/{cli,daemon,app,server}

    print_success "Build directories created at: $OUTPUT_DIR"
    echo ""
}

# Build Rust CLI
build_cli() {
    print_header "Building Auxin CLI"

    cd "$PROJECT_ROOT/Auxin-CLI-Wrapper"

    print_info "Building auxin CLI in release mode..."
    cargo build --release

    if [ ! -f "$CLI_BUILD" ]; then
        print_error "CLI build failed"
        exit 1
    fi

    print_success "CLI built successfully: $CLI_BUILD"
    ls -lh "$CLI_BUILD"

    cd "$PROJECT_ROOT"
    echo ""
}

# Build Swift daemon
build_daemon() {
    print_header "Building Auxin LaunchAgent Daemon"

    cd "$PROJECT_ROOT/Auxin-LaunchAgent"

    print_info "Building auxin-daemon in release mode..."
    swift build -c release

    if [ ! -f "$DAEMON_BUILD" ]; then
        print_error "Daemon build failed"
        exit 1
    fi

    print_success "Daemon built successfully: $DAEMON_BUILD"
    ls -lh "$DAEMON_BUILD"

    cd "$PROJECT_ROOT"
    echo ""
}

# Build Swift app
build_app() {
    print_header "Building Auxin Application"

    cd "$PROJECT_ROOT/Auxin-App"

    print_info "Building Auxin.app in release mode..."
    swift build -c release

    print_info "Creating app bundle..."
    ./create-app-bundle.sh

    if [ ! -d "$APP_BUILD" ]; then
        print_error "App build failed"
        exit 1
    fi

    print_success "App built successfully: $APP_BUILD"

    cd "$PROJECT_ROOT"
    echo ""
}

# Build Auxin Server
build_server() {
    print_header "Building Auxin Server (Optional)"

    cd "$PROJECT_ROOT/auxin-server"

    print_info "Building auxin-server in release mode..."
    cargo build --release

    if [ ! -f "$SERVER_BUILD" ]; then
        print_error "Server build failed"
        exit 1
    fi

    print_success "Server built successfully: $SERVER_BUILD"
    ls -lh "$SERVER_BUILD"

    cd "$PROJECT_ROOT"
    echo ""
}

# Prepare CLI package payload
prepare_cli_payload() {
    print_info "Preparing CLI payload..."

    mkdir -p "$BUILD_DIR/cli/usr/local/bin"
    cp "$CLI_BUILD" "$BUILD_DIR/cli/usr/local/bin/"
    chmod +x "$BUILD_DIR/cli/usr/local/bin/auxin"

    print_success "CLI payload prepared"
}

# Prepare daemon package payload
prepare_daemon_payload() {
    print_info "Preparing daemon payload..."

    mkdir -p "$BUILD_DIR/daemon/usr/local/bin"
    mkdir -p "$BUILD_DIR/daemon/Library/LaunchAgents"

    # Copy daemon binary
    cp "$DAEMON_BUILD" "$BUILD_DIR/daemon/usr/local/bin/"
    chmod +x "$BUILD_DIR/daemon/usr/local/bin/auxin-daemon"

    # Copy plist (will be customized in postinstall)
    cp "$PROJECT_ROOT/Auxin-LaunchAgent/Resources/com.auxin.daemon.plist" \
       "$BUILD_DIR/daemon/Library/LaunchAgents/"

    print_success "Daemon payload prepared"
}

# Prepare app package payload
prepare_app_payload() {
    print_info "Preparing app payload..."

    mkdir -p "$BUILD_DIR/app/Applications"
    cp -R "$APP_BUILD" "$BUILD_DIR/app/Applications/"

    print_success "App payload prepared"
}

# Prepare server package payload
prepare_server_payload() {
    print_info "Preparing server payload..."

    mkdir -p "$BUILD_DIR/server/usr/local/bin"
    mkdir -p "$BUILD_DIR/server/var/oxen"

    # Copy server binary
    cp "$SERVER_BUILD" "$BUILD_DIR/server/usr/local/bin/"
    chmod +x "$BUILD_DIR/server/usr/local/bin/auxin-server"

    print_success "Server payload prepared"
}

# Create postinstall scripts
create_postinstall_scripts() {
    print_header "Creating Postinstall Scripts"

    # CLI postinstall (minimal)
    cat > "$SCRIPTS_DIR/cli/postinstall" << 'EOF'
#!/bin/bash
# Auxin CLI Postinstall Script

# Verify installation
if [ -x /usr/local/bin/auxin ]; then
    echo "Auxin CLI installed successfully"
    /usr/local/bin/auxin --version
else
    echo "Warning: Auxin CLI not found" >&2
    exit 1
fi

exit 0
EOF

    # Daemon postinstall (register with launchd)
    cat > "$SCRIPTS_DIR/daemon/postinstall" << 'EOF'
#!/bin/bash
# Auxin Daemon Postinstall Script

PLIST_SOURCE="/Library/LaunchAgents/com.auxin.daemon.plist"
USER_HOME=$(eval echo ~$USER)
USER_LAUNCHAGENTS="$USER_HOME/Library/LaunchAgents"
USER_PLIST="$USER_LAUNCHAGENTS/com.auxin.daemon.plist"

# Create user's LaunchAgents directory if needed
mkdir -p "$USER_LAUNCHAGENTS"

# Copy plist to user's LaunchAgents directory
if [ -f "$PLIST_SOURCE" ]; then
    cp "$PLIST_SOURCE" "$USER_PLIST"

    # Set proper ownership
    chown $USER "$USER_PLIST"
    chmod 644 "$USER_PLIST"

    echo "LaunchAgent plist installed to $USER_PLIST"

    # Note: We don't auto-load the agent during installation
    # User will need to approve it in System Settings
    echo "To enable the daemon:"
    echo "1. Open System Settings"
    echo "2. Go to General → Login Items & Extensions"
    echo "3. Enable 'Auxin Daemon'"
else
    echo "Warning: Plist not found at $PLIST_SOURCE" >&2
fi

exit 0
EOF

    # App postinstall (minimal)
    cat > "$SCRIPTS_DIR/app/postinstall" << 'EOF'
#!/bin/bash
# Auxin App Postinstall Script

if [ -d "/Applications/Auxin.app" ]; then
    echo "Auxin.app installed successfully"
else
    echo "Warning: Auxin.app not found" >&2
    exit 1
fi

exit 0
EOF

    # Server postinstall (setup directories and config)
    cat > "$SCRIPTS_DIR/server/postinstall" << 'EOF'
#!/bin/bash
# Auxin Server Postinstall Script

DATA_DIR="/var/oxen/data"
USER_HOME=$(eval echo ~$USER)
CONFIG_DIR="$USER_HOME/.config/auxin-server"
ENV_FILE="$CONFIG_DIR/.env"

# Create data directory
mkdir -p "$DATA_DIR"
chown $USER "$DATA_DIR"

# Create config directory
mkdir -p "$CONFIG_DIR"
chown $USER "$CONFIG_DIR"

# Generate default config if it doesn't exist
if [ ! -f "$ENV_FILE" ]; then
    # Generate random auth token secret
    AUTH_SECRET=$(openssl rand -hex 32)

    cat > "$ENV_FILE" << ENVEOF
# Auxin Server Configuration
# Generated: $(date)

# Server
SYNC_DIR=$DATA_DIR
OXEN_SERVER_PORT=3000
OXEN_SERVER_HOST=0.0.0.0

# Authentication
AUTH_TOKEN_SECRET=$AUTH_SECRET
AUTH_TOKEN_EXPIRY_HOURS=24

# Logging
RUST_LOG=info,auxin_server=debug

# Optional Features
ENABLE_REDIS_LOCKS=false
ENABLE_WEB_UI=false
ENVEOF

    chown $USER "$ENV_FILE"
    chmod 600 "$ENV_FILE"

    echo "Server configuration created at $ENV_FILE"
fi

echo "Auxin Server installed successfully"
echo ""
echo "To start the server, run:"
echo "  auxin-server"
echo ""
echo "Configuration: $ENV_FILE"
echo "Data directory: $DATA_DIR"

exit 0
EOF

    # Make scripts executable
    chmod +x "$SCRIPTS_DIR"/*/postinstall

    print_success "Postinstall scripts created"
    echo ""
}

# Create installer resources
create_resources() {
    print_header "Creating Installer Resources"

    # Welcome text
    cat > "$RESOURCES_DIR/welcome.html" << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif; }
        h1 { color: #007AFF; }
        .highlight { background-color: #F0F0F0; padding: 10px; border-radius: 5px; }
    </style>
</head>
<body>
    <h1>Welcome to Auxin</h1>
    <p>This installer will install <strong>Auxin</strong>, a macOS-native version control system for creative applications.</p>

    <h2>What will be installed:</h2>
    <ul>
        <li><strong>Auxin CLI</strong> - Command-line interface for version control</li>
        <li><strong>Auxin Daemon</strong> - Background service for automatic draft commits</li>
        <li><strong>Auxin App</strong> - Native macOS application for managing projects</li>
        <li><strong>Auxin Server</strong> (Optional) - Collaboration server for LAN teams</li>
    </ul>

    <h2>Supported Applications:</h2>
    <ul>
        <li>Logic Pro</li>
        <li>SketchUp</li>
        <li>Blender</li>
    </ul>

    <div class="highlight">
        <strong>Note:</strong> Auxin requires <strong>Oxen CLI</strong> to be installed separately.
        <br>Install with: <code>pip3 install oxen-ai</code>
    </div>

    <p>Click <strong>Continue</strong> to proceed with the installation.</p>
</body>
</html>
EOF

    # Read Me
    cat > "$RESOURCES_DIR/readme.html" << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif; }
        h1 { color: #007AFF; }
        code { background-color: #F0F0F0; padding: 2px 6px; border-radius: 3px; font-family: 'Courier New', monospace; }
        .section { margin: 20px 0; }
    </style>
</head>
<body>
    <h1>Auxin Installation Complete</h1>

    <div class="section">
        <h2>Getting Started</h2>
        <ol>
            <li>Install Oxen CLI (if not already installed):
                <br><code>pip3 install oxen-ai</code>
            </li>
            <li>Initialize your first project:
                <br><code>cd ~/Music/YourProject.logicx</code>
                <br><code>auxin init --logic .</code>
            </li>
            <li>Launch the Auxin app from Applications folder</li>
        </ol>
    </div>

    <div class="section">
        <h2>Enable the Background Daemon</h2>
        <p>To enable automatic draft commits:</p>
        <ol>
            <li>Open <strong>System Settings</strong></li>
            <li>Go to <strong>General → Login Items &amp; Extensions</strong></li>
            <li>Find and enable <strong>Auxin Daemon</strong></li>
        </ol>
    </div>

    <div class="section">
        <h2>Using Auxin Server (Optional)</h2>
        <p>If you installed the server component for LAN collaboration:</p>
        <ol>
            <li>Start the server: <code>auxin-server</code></li>
            <li>Access the web UI at: <code>http://localhost:3000</code></li>
            <li>Configure in: <code>~/.config/auxin-server/.env</code></li>
        </ol>
    </div>

    <div class="section">
        <h2>Documentation</h2>
        <p>For more information, visit:</p>
        <ul>
            <li><a href="https://github.com/jbacus/auxin">GitHub Repository</a></li>
            <li>User Guide: <code>/usr/local/share/auxin/docs/</code></li>
        </ul>
    </div>

    <div class="section">
        <h2>Support</h2>
        <p>For issues and questions:</p>
        <ul>
            <li>GitHub Issues: <a href="https://github.com/jbacus/auxin/issues">github.com/jbacus/auxin/issues</a></li>
        </ul>
    </div>
</body>
</html>
EOF

    # License
    if [ -f "$PROJECT_ROOT/LICENSE" ]; then
        cp "$PROJECT_ROOT/LICENSE" "$RESOURCES_DIR/license.txt"
    else
        cat > "$RESOURCES_DIR/license.txt" << 'EOF'
MIT License

Copyright (c) 2025 Auxin

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
EOF
    fi

    print_success "Installer resources created"
    echo ""
}

# Create Distribution.xml for customizable installer
create_distribution_xml() {
    print_header "Creating Distribution Configuration"

    cat > "$OUTPUT_DIR/distribution.xml" << EOF
<?xml version="1.0" encoding="utf-8"?>
<installer-gui-script minSpecVersion="1">
    <title>Auxin</title>
    <organization>com.auxin</organization>
    <domains enable_anywhere="false" enable_currentUserHome="false" enable_localSystem="true"/>

    <options customize="allow" require-scripts="false" hostArchitectures="arm64,x86_64"/>

    <!-- Welcome and License -->
    <welcome file="welcome.html" mime-type="text/html"/>
    <license file="license.txt" mime-type="text/plain"/>
    <readme file="readme.html" mime-type="text/html"/>

    <!-- Background image (optional) -->
    <background file="background.png" mime-type="image/png" alignment="left" scaling="proportional"/>

    <!-- Installation choices -->
    <choices-outline>
        <line choice="auxin.cli"/>
        <line choice="auxin.daemon"/>
        <line choice="auxin.app"/>
        <line choice="auxin.server"/>
    </choices-outline>

    <!-- CLI Package (Required) -->
    <choice id="auxin.cli"
            visible="true"
            title="Auxin CLI"
            description="Command-line interface for Auxin version control"
            enabled="false"
            selected="true">
        <pkg-ref id="com.auxin.cli"/>
    </choice>

    <!-- Daemon Package (Required) -->
    <choice id="auxin.daemon"
            visible="true"
            title="Auxin Daemon"
            description="Background service for automatic draft commits and file monitoring"
            enabled="false"
            selected="true">
        <pkg-ref id="com.auxin.daemon"/>
    </choice>

    <!-- App Package (Recommended) -->
    <choice id="auxin.app"
            visible="true"
            title="Auxin Application"
            description="Native macOS application for managing projects with a graphical interface"
            start_selected="true">
        <pkg-ref id="com.auxin.app"/>
    </choice>

    <!-- Server Package (Optional) -->
    <choice id="auxin.server"
            visible="true"
            title="Auxin Server (Optional)"
            description="Collaboration server for LAN teams. Install this on a shared machine for team collaboration."
            start_selected="false">
        <pkg-ref id="com.auxin.server"/>
    </choice>

    <!-- Package references -->
    <pkg-ref id="com.auxin.cli"
             version="$VERSION"
             onConclusion="none">
        auxin-cli.pkg
    </pkg-ref>

    <pkg-ref id="com.auxin.daemon"
             version="$VERSION"
             onConclusion="none">
        auxin-daemon.pkg
    </pkg-ref>

    <pkg-ref id="com.auxin.app"
             version="$VERSION"
             onConclusion="none">
        auxin-app.pkg
    </pkg-ref>

    <pkg-ref id="com.auxin.server"
             version="$VERSION"
             onConclusion="none">
        auxin-server.pkg
    </pkg-ref>
</installer-gui-script>
EOF

    print_success "Distribution configuration created"
    echo ""
}

# Build component packages
build_component_packages() {
    print_header "Building Component Packages"

    # Build CLI package
    print_info "Building CLI package..."
    pkgbuild --root "$BUILD_DIR/cli" \
             --identifier "com.auxin.cli" \
             --version "$VERSION" \
             --scripts "$SCRIPTS_DIR/cli" \
             --install-location "/" \
             "$PACKAGES_DIR/auxin-cli.pkg"
    print_success "CLI package created"

    # Build daemon package
    print_info "Building daemon package..."
    pkgbuild --root "$BUILD_DIR/daemon" \
             --identifier "com.auxin.daemon" \
             --version "$VERSION" \
             --scripts "$SCRIPTS_DIR/daemon" \
             --install-location "/" \
             "$PACKAGES_DIR/auxin-daemon.pkg"
    print_success "Daemon package created"

    # Build app package
    print_info "Building app package..."
    pkgbuild --root "$BUILD_DIR/app" \
             --identifier "com.auxin.app" \
             --version "$VERSION" \
             --scripts "$SCRIPTS_DIR/app" \
             --install-location "/" \
             "$PACKAGES_DIR/auxin-app.pkg"
    print_success "App package created"

    # Build server package
    print_info "Building server package..."
    pkgbuild --root "$BUILD_DIR/server" \
             --identifier "com.auxin.server" \
             --version "$VERSION" \
             --scripts "$SCRIPTS_DIR/server" \
             --install-location "/" \
             "$PACKAGES_DIR/auxin-server.pkg"
    print_success "Server package created"

    echo ""
}

# Build final product
build_product() {
    print_header "Building Final Installer Package"

    local output_pkg="$OUTPUT_DIR/Auxin-$VERSION.pkg"

    print_info "Creating product package..."
    productbuild --distribution "$OUTPUT_DIR/distribution.xml" \
                 --resources "$RESOURCES_DIR" \
                 --package-path "$PACKAGES_DIR" \
                 "$output_pkg"

    if [ ! -f "$output_pkg" ]; then
        print_error "Failed to create product package"
        exit 1
    fi

    print_success "Installer package created successfully!"
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  Installer: $output_pkg"
    echo "  Size: $(du -h "$output_pkg" | cut -f1)"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "To install:"
    echo "  open $output_pkg"
    echo ""
    echo "To distribute:"
    echo "  1. Sign the package (optional but recommended)"
    echo "  2. Notarize with Apple (for distribution outside App Store)"
    echo "  3. Create a DMG for easy distribution"
    echo ""
}

# Create background image (placeholder)
create_background() {
    # If no background exists, create a placeholder
    # In a real scenario, this would be a proper branded image
    if [ ! -f "$RESOURCES_DIR/background.png" ]; then
        # Create a simple 1x1 transparent PNG as placeholder
        echo "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==" | base64 -d > "$RESOURCES_DIR/background.png"
    fi
}

# Main build flow
main() {
    print_header "Auxin macOS Installer Builder"
    echo "Version: $VERSION"
    echo "Output: $OUTPUT_DIR"
    echo ""

    # Parse arguments
    parse_args "$@"

    # Build steps
    check_prerequisites
    setup_build_dirs

    # Build all components
    build_cli
    build_daemon
    build_app
    build_server

    # Prepare payloads
    prepare_cli_payload
    prepare_daemon_payload
    prepare_app_payload
    prepare_server_payload

    # Create installer components
    create_postinstall_scripts
    create_resources
    create_background
    create_distribution_xml

    # Build packages
    build_component_packages
    build_product

    print_success "Installer build complete!"
}

# Run main function
main "$@"
