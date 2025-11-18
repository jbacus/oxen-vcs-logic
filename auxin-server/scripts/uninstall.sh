#!/bin/bash
set -e

# Uninstall Auxin Server

INSTALL_DIR="/usr/local/bin"
CONFIG_DIR="$HOME/.config/auxin-server"
LAUNCHD_DIR="$HOME/Library/LaunchAgents"
LAUNCHD_LABEL="com.auxin.server"
LAUNCHD_PLIST="$LAUNCHD_DIR/$LAUNCHD_LABEL.plist"
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
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

echo ""
log_info "===== Auxin Server Uninstallation ====="
echo ""

# Confirm uninstallation
read -p "This will remove auxin-server and all configuration. Continue? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    log_info "Uninstallation cancelled"
    exit 0
fi

# Stop the service if running
if launchctl list | grep -q "$LAUNCHD_LABEL"; then
    log_info "Stopping service..."
    launchctl unload "$LAUNCHD_PLIST" 2>/dev/null || true
    log_success "Service stopped"
fi

# Remove LaunchAgent plist
if [ -f "$LAUNCHD_PLIST" ]; then
    log_info "Removing LaunchAgent plist..."
    rm "$LAUNCHD_PLIST"
    log_success "LaunchAgent removed"
fi

# Remove binary
if [ -f "$INSTALL_DIR/auxin-server" ]; then
    log_info "Removing binary..."
    sudo rm "$INSTALL_DIR/auxin-server"
    log_success "Binary removed"
fi

# Ask about configuration
echo ""
read -p "Remove configuration directory ($CONFIG_DIR)? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    if [ -d "$CONFIG_DIR" ]; then
        rm -rf "$CONFIG_DIR"
        log_success "Configuration removed"
    fi
else
    log_info "Configuration preserved at: $CONFIG_DIR"
fi

# Ask about data directory
echo ""
log_warn "Data directory: /var/oxen/data"
read -p "Remove data directory (WARNING: This deletes all repositories!)? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    read -p "Are you ABSOLUTELY SURE? This cannot be undone! (yes/NO) " -r
    echo
    if [[ $REPLY == "yes" ]]; then
        sudo rm -rf /var/oxen/data
        log_success "Data directory removed"
    else
        log_info "Data directory preserved"
    fi
else
    log_info "Data directory preserved at: /var/oxen/data"
fi

# Ask about logs
echo ""
read -p "Remove log files? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    [ -f "$LOG_FILE" ] && rm "$LOG_FILE" && log_success "Output log removed"
    [ -f "$ERROR_LOG" ] && rm "$ERROR_LOG" && log_success "Error log removed"
else
    log_info "Logs preserved"
fi

echo ""
log_success "===== Uninstallation Complete ====="
echo ""
