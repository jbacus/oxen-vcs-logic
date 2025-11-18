#!/bin/bash
set -e

# Stop Auxin Server

LAUNCHD_LABEL="com.auxin.server"
LAUNCHD_PLIST="$HOME/Library/LaunchAgents/$LAUNCHD_LABEL.plist"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

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

# Check if plist exists
if [ ! -f "$LAUNCHD_PLIST" ]; then
    log_error "LaunchAgent plist not found: $LAUNCHD_PLIST"
    exit 1
fi

# Check if running
if ! launchctl list | grep -q "$LAUNCHD_LABEL"; then
    log_warn "Server is not running"
    exit 0
fi

# Unload the LaunchAgent
log_info "Stopping auxin-server..."
if launchctl unload "$LAUNCHD_PLIST" 2>/dev/null; then
    sleep 1

    # Verify it stopped
    if ! launchctl list | grep -q "$LAUNCHD_LABEL"; then
        log_success "auxin-server stopped successfully"
    else
        log_error "Failed to stop server"
        exit 1
    fi
else
    log_error "Failed to unload LaunchAgent"
    exit 1
fi
