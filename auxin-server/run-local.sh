#!/bin/bash

# Quick local server runner for development/testing

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BINARY="$SCRIPT_DIR/target/release/auxin-server"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Check if server binary exists
if [ ! -f "$BINARY" ]; then
    echo -e "${RED}✗${NC} Server binary not found!"
    echo -e "${BLUE}▶${NC} Run './deploy-local.sh' first to build the server"
    exit 1
fi

# Check if frontend is built
if [ -d "$SCRIPT_DIR/frontend/dist" ]; then
    echo -e "${GREEN}✓${NC} Frontend detected - Web UI will be available"
else
    echo -e "${YELLOW}⚠${NC} Frontend not built - Web UI will not be available"
    echo -e "${BLUE}▶${NC} Run 'cd frontend && npm install && npm run build' to enable it"
fi

# Load .env if exists
if [ -f "$SCRIPT_DIR/.env" ]; then
    export $(grep -v '^#' "$SCRIPT_DIR/.env" | xargs)
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  🎵 Starting Auxin Server"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "  🌐 Server:      http://${OXEN_SERVER_HOST:-127.0.0.1}:${OXEN_SERVER_PORT:-3000}"
echo "  📁 Data:        ${SYNC_DIR:-$SCRIPT_DIR/.local-data}"
echo "  📝 Logs:        RUST_LOG=${RUST_LOG:-info}"
echo ""
echo "  Press Ctrl+C to stop"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Run the server
exec "$BINARY"
