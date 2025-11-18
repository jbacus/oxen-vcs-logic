#!/bin/bash
set -e

# Restart Auxin Server

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Colors
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_info "Restarting auxin-server..."

# Stop if running
"$SCRIPT_DIR/stop.sh" 2>/dev/null || true

# Wait a moment
sleep 2

# Start
"$SCRIPT_DIR/start.sh"
