#!/bin/bash

# Check Auxin Server Status

LAUNCHD_LABEL="com.auxin.server"
LAUNCHD_PLIST="$HOME/Library/LaunchAgents/$LAUNCHD_LABEL.plist"
CONFIG_DIR="$HOME/.config/auxin-server"
LOG_FILE="$HOME/Library/Logs/auxin-server.log"
ERROR_LOG="$HOME/Library/Logs/auxin-server-error.log"

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
    echo -e "${GREEN}[✓]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[!]${NC} $1"
}

log_error() {
    echo -e "${RED}[✗]${NC} $1"
}

echo ""
echo "===== Auxin Server Status ====="
echo ""

# Check if installed
if command -v auxin-server &> /dev/null; then
    log_success "Binary installed: $(which auxin-server)"
else
    log_error "Binary not found (run setup.sh to install)"
    exit 1
fi

# Check if plist exists
if [ -f "$LAUNCHD_PLIST" ]; then
    log_success "LaunchAgent configured: $LAUNCHD_PLIST"
else
    log_error "LaunchAgent not configured (run setup.sh)"
    exit 1
fi

# Check if running
if launchctl list | grep -q "$LAUNCHD_LABEL"; then
    log_success "Service is running"

    # Get PID
    local pid=$(launchctl list | grep "$LAUNCHD_LABEL" | awk '{print $1}')
    if [ "$pid" != "-" ]; then
        echo "  PID: $pid"

        # Get memory usage
        local mem=$(ps -o rss= -p "$pid" 2>/dev/null | awk '{printf "%.1f MB", $1/1024}')
        if [ -n "$mem" ]; then
            echo "  Memory: $mem"
        fi
    fi
else
    log_error "Service is not running"
    echo "  Start with: ./start.sh"
fi

# Check HTTP endpoint
echo ""
log_info "Testing HTTP endpoint..."
if curl -s -f http://localhost:3000/health > /dev/null 2>&1; then
    log_success "Server responding at http://localhost:3000"

    # Try to get repo list
    local repo_count=$(curl -s http://localhost:3000/api/repos 2>/dev/null | grep -o '\[.*\]' | grep -o ',' | wc -l)
    if [ -n "$repo_count" ]; then
        echo "  Repositories: $((repo_count + 1))"
    fi
else
    log_error "Server not responding"
fi

# Configuration status
echo ""
log_info "Configuration:"
if [ -f "$CONFIG_DIR/.env" ]; then
    log_success "Config file: $CONFIG_DIR/.env"

    # Show key settings (without secrets)
    if [ -f "$CONFIG_DIR/.env" ]; then
        echo "  Port: $(grep OXEN_SERVER_PORT "$CONFIG_DIR/.env" | cut -d= -f2)"
        echo "  Data Dir: $(grep SYNC_DIR "$CONFIG_DIR/.env" | cut -d= -f2)"
    fi
else
    log_warn "Config file not found"
fi

# Log files
echo ""
log_info "Logs:"
if [ -f "$LOG_FILE" ]; then
    local log_size=$(du -h "$LOG_FILE" | cut -f1)
    local log_lines=$(wc -l < "$LOG_FILE" | tr -d ' ')
    log_success "Output log: $LOG_FILE ($log_size, $log_lines lines)"
else
    log_warn "Output log not found: $LOG_FILE"
fi

if [ -f "$ERROR_LOG" ]; then
    local error_size=$(du -h "$ERROR_LOG" | cut -f1)
    local error_lines=$(wc -l < "$ERROR_LOG" | tr -d ' ')
    if [ "$error_lines" -gt 0 ]; then
        log_warn "Error log: $ERROR_LOG ($error_size, $error_lines lines)"
        echo ""
        echo "Recent errors:"
        tail -5 "$ERROR_LOG" | sed 's/^/  /'
    else
        log_success "Error log: $ERROR_LOG (no errors)"
    fi
fi

echo ""
echo "Commands:"
echo "  View logs: tail -f ~/Library/Logs/auxin-server.log"
echo "  Start: ./start.sh"
echo "  Stop: ./stop.sh"
echo "  Restart: ./restart.sh"
echo ""
