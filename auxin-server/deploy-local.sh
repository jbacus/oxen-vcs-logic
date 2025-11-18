#!/bin/bash
set -e

# Auxin Server - Local Development Deployment
# Quick setup for testing without system installation

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DATA_DIR="$SCRIPT_DIR/.local-data"
FRONTEND_DIST="$SCRIPT_DIR/frontend/dist"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

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

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."

    # Check Rust
    if ! command -v cargo &> /dev/null; then
        log_error "Rust is not installed"
        echo "  Install with: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi
    log_success "Rust found: $(rustc --version | cut -d' ' -f2)"

    # Check Node.js (optional for frontend)
    if command -v node &> /dev/null; then
        log_success "Node.js found: $(node --version)"
    else
        log_warn "Node.js not found - frontend will not be built"
        log_warn "Install Node.js from https://nodejs.org/ to enable web UI"
    fi
}

# Setup local environment
setup_environment() {
    log_info "Setting up local environment..."

    # Create local data directory
    if [ ! -d "$DATA_DIR" ]; then
        mkdir -p "$DATA_DIR"
        log_success "Created data directory: $DATA_DIR"
    fi

    # Create .env file
    if [ ! -f "$SCRIPT_DIR/.env" ]; then
        cat > "$SCRIPT_DIR/.env" <<EOF
# Auxin Server - Local Development Configuration
# Generated: $(date)

# Server
SYNC_DIR=$DATA_DIR
OXEN_SERVER_PORT=3000
OXEN_SERVER_HOST=127.0.0.1

# Logging
RUST_LOG=info,auxin_server=debug

# Features
ENABLE_REDIS_LOCKS=false
EOF
        log_success "Created .env configuration"
    else
        log_success ".env already exists"
    fi
}

# Build frontend
build_frontend() {
    if [ ! -d "$SCRIPT_DIR/frontend" ]; then
        log_warn "Frontend directory not found - skipping frontend build"
        return
    fi

    if ! command -v node &> /dev/null; then
        log_warn "Node.js not installed - skipping frontend build"
        return
    fi

    log_info "Building frontend..."
    cd "$SCRIPT_DIR/frontend"

    # Install dependencies if needed
    if [ ! -d "node_modules" ]; then
        log_info "Installing npm dependencies..."
        npm install
    fi

    # Build production bundle
    log_info "Building production bundle..."
    npm run build

    if [ -d "dist" ]; then
        log_success "Frontend built successfully"
    else
        log_error "Frontend build failed"
        exit 1
    fi

    cd "$SCRIPT_DIR"
}

# Build Rust backend
build_backend() {
    log_info "Building Rust backend..."
    cd "$SCRIPT_DIR"

    cargo build --release

    if [ -f "target/release/auxin-server" ]; then
        local size=$(du -h target/release/auxin-server | cut -f1)
        log_success "Backend built successfully (size: $size)"
    else
        log_error "Backend build failed"
        exit 1
    fi
}

# Create sample test data
create_sample_data() {
    log_info "Creating sample test data..."

    # Create a sample namespace/repo structure
    local sample_repo="$DATA_DIR/demo/my-logic-project"

    if [ ! -d "$sample_repo/.oxen" ]; then
        mkdir -p "$sample_repo/.oxen"

        # Create minimal .oxen structure
        mkdir -p "$sample_repo/.oxen/"{history,refs/heads,tree,versions,metadata,locks}

        # Create config
        cat > "$sample_repo/.oxen/config.toml" <<EOF
[repository]
name = "my-logic-project"
namespace = "demo"
created_at = "$(date -u +%Y-%m-%dT%H:%M:%SZ)"

[remote "origin"]
url = "http://localhost:3000/demo/my-logic-project"
EOF

        # Create HEAD
        echo "ref: refs/heads/main" > "$sample_repo/.oxen/HEAD"

        log_success "Created sample repository: demo/my-logic-project"
    else
        log_success "Sample repository already exists"
    fi
}

# Print usage information
print_usage() {
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  🎵 Auxin Server - Local Deployment Complete!"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "📁 Data directory:    $DATA_DIR"
    echo "🔧 Configuration:     $SCRIPT_DIR/.env"
    echo "🚀 Binary:            $SCRIPT_DIR/target/release/auxin-server"

    if [ -d "$FRONTEND_DIST" ]; then
        echo "🎨 Web UI:            ✓ Built (frontend/dist)"
    else
        echo "🎨 Web UI:            ✗ Not built"
    fi

    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "🚀 Quick Start:"
    echo ""
    echo "  # Start the server"
    echo "  ./run-local.sh"
    echo ""
    echo "  # Or run directly"
    echo "  ./target/release/auxin-server"
    echo ""
    echo "  # Test the API"
    echo "  curl http://localhost:3000/health"
    echo "  curl http://localhost:3000/api/repos"
    echo ""

    if [ -d "$FRONTEND_DIST" ]; then
        echo "  # Open Web UI"
        echo "  open http://localhost:3000"
        echo ""
    fi

    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "📚 Documentation:"
    echo "  • Main README:      README.md"
    echo "  • Frontend Setup:   FRONTEND_SETUP.md"
    echo "  • API Docs:         See README.md#api-endpoints"
    echo ""
    echo "🔗 Useful Commands:"
    echo "  • View logs:        tail -f ~/.local-data/auxin-server.log"
    echo "  • Check status:     ./scripts/status.sh"
    echo "  • Rebuild frontend: cd frontend && npm run build"
    echo "  • Rebuild backend:  cargo build --release"
    echo ""
}

# Main deployment
main() {
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  🎵 Auxin Server - Local Development Deployment"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""

    check_prerequisites
    setup_environment
    build_frontend
    build_backend
    create_sample_data
    print_usage
}

main "$@"
