#!/bin/bash
set -e

# Auxin Server - Complete Build & Launch Script
# One script to build everything and start the server

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DATA_DIR="$SCRIPT_DIR/.local-data"
BINARY="$SCRIPT_DIR/target/release/auxin-server"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
NC='\033[0m'

log_info() { echo -e "${BLUE}▶${NC} $1"; }
log_success() { echo -e "${GREEN}✓${NC} $1"; }
log_warn() { echo -e "${YELLOW}⚠${NC} $1"; }
log_error() { echo -e "${RED}✗${NC} $1"; }
log_step() { echo -e "${CYAN}━━━ $1 ━━━${NC}"; }

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  🎵 Auxin Server - Build & Launch"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Step 1: Check prerequisites
log_step "Checking Prerequisites"

# Check Rust
if ! command -v cargo &> /dev/null; then
    log_error "Rust is not installed"
    echo "  Install with: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi
log_success "Rust: $(rustc --version | cut -d' ' -f2)"

# Check Node.js
HAS_NODE=false
if command -v node &> /dev/null; then
    HAS_NODE=true
    log_success "Node.js: $(node --version)"
else
    log_warn "Node.js not found - frontend will be skipped"
    log_warn "Install from https://nodejs.org/ to enable Web UI"
fi
echo ""

# Step 2: Setup environment
log_step "Setting Up Environment"

# Create data directory
mkdir -p "$DATA_DIR"
log_success "Data directory: $DATA_DIR"

# Create .env if it doesn't exist
if [ ! -f "$SCRIPT_DIR/.env" ]; then
    cat > "$SCRIPT_DIR/.env" <<EOF
# Auxin Server Configuration
SYNC_DIR=$DATA_DIR
OXEN_SERVER_PORT=3000
OXEN_SERVER_HOST=127.0.0.1
RUST_LOG=info,auxin_server=debug
ENABLE_REDIS_LOCKS=false
EOF
    log_success "Created .env configuration"
else
    log_success "Using existing .env"
fi
echo ""

# Step 3: Build frontend (if Node.js available)
if [ "$HAS_NODE" = true ] && [ -d "$SCRIPT_DIR/frontend" ]; then
    log_step "Building Frontend"

    cd "$SCRIPT_DIR/frontend"

    # Install dependencies if needed
    if [ ! -d "node_modules" ]; then
        log_info "Installing npm dependencies..."
        npm install --silent
    fi

    # Build production bundle
    log_info "Building React application..."
    npm run build --silent

    if [ -d "dist" ]; then
        log_success "Frontend built: frontend/dist/"
    else
        log_error "Frontend build failed"
        exit 1
    fi

    cd "$SCRIPT_DIR"
    echo ""
fi

# Step 4: Build backend
log_step "Building Backend"

log_info "Compiling Rust (release mode)..."
cargo build --release --quiet

if [ -f "$BINARY" ]; then
    SIZE=$(du -h "$BINARY" | cut -f1)
    log_success "Backend built: $SIZE"
else
    log_error "Backend build failed"
    exit 1
fi
echo ""

# Step 5: Create sample data
log_step "Creating Sample Data"

SAMPLE_REPO="$DATA_DIR/demo/sample-project"
if [ ! -d "$SAMPLE_REPO/.oxen" ]; then
    mkdir -p "$SAMPLE_REPO/.oxen"/{history,refs/heads,tree,versions,metadata,locks}

    cat > "$SAMPLE_REPO/.oxen/config.toml" <<EOF
[repository]
name = "sample-project"
namespace = "demo"
created_at = "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
EOF

    echo "ref: refs/heads/main" > "$SAMPLE_REPO/.oxen/HEAD"
    log_success "Created sample repository: demo/sample-project"
else
    log_success "Sample repository exists"
fi
echo ""

# Step 6: Load environment and start server
log_step "Starting Server"

# Load .env
export $(grep -v '^#' "$SCRIPT_DIR/.env" | xargs)

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "  🌐 API:         http://${OXEN_SERVER_HOST}:${OXEN_SERVER_PORT}/api/repos"

if [ -d "$SCRIPT_DIR/frontend/dist" ]; then
    echo "  🎨 Web UI:      http://${OXEN_SERVER_HOST}:${OXEN_SERVER_PORT}/"
fi

echo "  📁 Data:        $DATA_DIR"
echo "  📋 Health:      curl http://${OXEN_SERVER_HOST}:${OXEN_SERVER_PORT}/health"
echo ""
echo "  Press Ctrl+C to stop the server"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Run the server
exec "$BINARY"
