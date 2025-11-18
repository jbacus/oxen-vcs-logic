#!/bin/bash
set -e

# Start Auxin Server

LAUNCHD_LABEL="com.auxin.server"
LAUNCHD_PLIST="$HOME/Library/LaunchAgents/$LAUNCHD_LABEL.plist"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if plist exists
if [ ! -f "$LAUNCHD_PLIST" ]; then
    log_error "LaunchAgent plist not found: $LAUNCHD_PLIST"
    log_info "Run setup.sh first to install the server"
    exit 1
fi

# Check if already running
if launchctl list | grep -q "$LAUNCHD_LABEL"; then
    log_info "Server is already running"
    log_info "Use stop.sh to stop it first, or restart.sh to restart"
    exit 0
fi

# Load the LaunchAgent
log_info "Starting auxin-server..."
if launchctl load "$LAUNCHD_PLIST" 2>/dev/null; then
    sleep 2

    # Verify it started
    if launchctl list | grep -q "$LAUNCHD_LABEL"; then
        log_success "auxin-server started successfully"
        log_info "Check status with: ./status.sh"
        log_info "View logs with: tail -f ~/Library/Logs/auxin-server.log"

        # Test health endpoint
        sleep 1
        log_info "Testing health endpoint..."
        if curl -s http://localhost:3000/health > /dev/null 2>&1; then
            log_success "Server is responding at http://localhost:3000"
        else
            log_error "Server started but not responding yet. Check logs."
        fi
    else
        log_error "Failed to start server"
        exit 1
    fi
else
    log_error "Failed to load LaunchAgent"
    exit 1
fi
