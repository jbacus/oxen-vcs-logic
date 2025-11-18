#!/bin/bash
#
# Auxin Development Deployment Script
# Sets up both CLI and Server for local development
#

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

log_info() {
    echo -e "${BLUE}▶${NC} $1"
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

print_banner() {
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  🎵 Auxin - Development Environment Setup"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."

    local all_ok=true

    # Check Rust
    if command -v cargo &> /dev/null; then
        log_success "Rust: $(rustc --version | cut -d' ' -f2)"
    else
        log_error "Rust not found. Install from https://rustup.rs"
        all_ok=false
    fi

    # Check Node.js (for server frontend)
    if command -v node &> /dev/null; then
        log_success "Node.js: $(node --version)"
    else
        log_warn "Node.js not found - server frontend will not be built"
    fi

    if [ "$all_ok" = false ]; then
        exit 1
    fi

    echo ""
}

# Build CLI
build_cli() {
    log_info "Building Auxin CLI..."

    cd "$PROJECT_ROOT/Auxin-CLI-Wrapper"
    cargo build --release 2>&1 | tail -5

    if [ -f "target/release/auxin" ]; then
        local size=$(du -h target/release/auxin | cut -f1)
        log_success "CLI built: target/release/auxin ($size)"
    else
        log_error "CLI build failed"
        exit 1
    fi

    cd "$PROJECT_ROOT"
}

# Build Server
build_server() {
    log_info "Building Auxin Server..."

    cd "$PROJECT_ROOT/auxin-server"
    cargo build --release 2>&1 | tail -5

    if [ -f "target/release/auxin-server" ]; then
        local size=$(du -h target/release/auxin-server | cut -f1)
        log_success "Server built: target/release/auxin-server ($size)"
    else
        log_error "Server build failed"
        exit 1
    fi

    cd "$PROJECT_ROOT"
}

# Build Server Frontend
build_server_frontend() {
    if ! command -v node &> /dev/null; then
        log_warn "Skipping frontend build (Node.js not installed)"
        return
    fi

    if [ ! -d "$PROJECT_ROOT/auxin-server/frontend" ]; then
        log_warn "Frontend directory not found"
        return
    fi

    log_info "Building Server Frontend..."

    cd "$PROJECT_ROOT/auxin-server/frontend"

    if [ ! -d "node_modules" ]; then
        npm install --silent
    fi

    npm run build --silent

    if [ -d "dist" ]; then
        log_success "Frontend built: frontend/dist"
    fi

    cd "$PROJECT_ROOT"
}

# Setup test project
setup_test_project() {
    local test_dir="/tmp/auxin-test-project"

    if [ ! -d "$test_dir/.auxin" ]; then
        log_info "Creating test project directory..."
        mkdir -p "$test_dir/.auxin"

        # Create config with server settings
        cat > "$test_dir/.auxin/config.toml" <<EOF
[server]
url = "http://127.0.0.1:3000"
timeout_secs = 30
use_server_locks = true
use_server_metadata = true
default_namespace = "test"
EOF

        log_success "Test project created: $test_dir"
    else
        log_success "Test project exists: $test_dir"
    fi
}

# Setup server environment
setup_server_env() {
    log_info "Setting up server environment..."

    cd "$PROJECT_ROOT/auxin-server"

    # Create data directory
    mkdir -p .local-data

    # Create .env if it doesn't exist
    if [ ! -f ".env" ]; then
        cat > ".env" <<EOF
# Auxin Server - Local Development
SYNC_DIR=$PROJECT_ROOT/auxin-server/.local-data
OXEN_SERVER_PORT=3000
OXEN_SERVER_HOST=127.0.0.1
RUST_LOG=info,auxin_server=debug
ENABLE_REDIS_LOCKS=false
EOF
        log_success "Created server .env"
    fi

    cd "$PROJECT_ROOT"
}

# Print usage instructions
print_usage() {
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  Development Environment Ready!"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "📦 Built Components:"
    echo "  • CLI:    Auxin-CLI-Wrapper/target/release/auxin"
    echo "  • Server: auxin-server/target/release/auxin-server"
    echo ""
    echo "🚀 Quick Start:"
    echo ""
    echo "  # Terminal 1: Start the server"
    echo "  cd auxin-server && ./run-local.sh"
    echo ""
    echo "  # Terminal 2: Use the CLI"
    echo "  cd /tmp/auxin-test-project"
    echo "  $PROJECT_ROOT/Auxin-CLI-Wrapper/target/release/auxin server status"
    echo "  $PROJECT_ROOT/Auxin-CLI-Wrapper/target/release/auxin lock status"
    echo ""
    echo "🔗 Useful Commands:"
    echo ""
    echo "  # Check server health"
    echo "  curl http://127.0.0.1:3000/health"
    echo ""
    echo "  # View server logs"
    echo "  tail -f auxin-server/.local-data/*.log"
    echo ""
    echo "  # Run CLI tests"
    echo "  cd Auxin-CLI-Wrapper && cargo test"
    echo ""
    echo "  # Run server tests"
    echo "  cd auxin-server && cargo test"
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
}

# Main
main() {
    print_banner
    check_prerequisites
    build_cli
    build_server
    build_server_frontend
    setup_server_env
    setup_test_project
    print_usage
}

main "$@"
