#!/bin/bash
#
# Auxin Development Server Starter
# Starts the auxin-server for local development
#

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SERVER_DIR="$PROJECT_ROOT/auxin-server"
PID_FILE="/tmp/auxin-server.pid"
LOG_FILE="/tmp/auxin-server.log"

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

# Check if server is already running
check_running() {
    if [ -f "$PID_FILE" ]; then
        local pid=$(cat "$PID_FILE")
        if kill -0 "$pid" 2>/dev/null; then
            return 0  # Running
        else
            rm -f "$PID_FILE"
        fi
    fi
    return 1  # Not running
}

# Stop the server
stop_server() {
    if [ -f "$PID_FILE" ]; then
        local pid=$(cat "$PID_FILE")
        if kill -0 "$pid" 2>/dev/null; then
            log_info "Stopping server (PID: $pid)..."
            kill "$pid"
            sleep 1
            rm -f "$PID_FILE"
            log_success "Server stopped"
        else
            rm -f "$PID_FILE"
            log_warn "Server was not running"
        fi
    else
        log_warn "No PID file found"
    fi
}

# Start the server
start_server() {
    if check_running; then
        local pid=$(cat "$PID_FILE")
        log_warn "Server already running (PID: $pid)"
        log_info "Use '$0 stop' to stop it, or '$0 restart' to restart"
        return
    fi

    # Check if binary exists
    local binary="$SERVER_DIR/target/release/auxin-server"
    if [ ! -f "$binary" ]; then
        log_error "Server binary not found. Run ./deploy-dev.sh first"
        exit 1
    fi

    # Start the server
    log_info "Starting auxin-server..."

    cd "$SERVER_DIR"

    # Load environment
    if [ -f ".env" ]; then
        export $(grep -v '^#' .env | xargs)
    fi

    # Start in background
    nohup "$binary" > "$LOG_FILE" 2>&1 &
    local pid=$!
    echo "$pid" > "$PID_FILE"

    # Wait for startup
    sleep 2

    # Check if it started successfully
    if kill -0 "$pid" 2>/dev/null; then
        log_success "Server started (PID: $pid)"
        log_info "Log file: $LOG_FILE"

        # Test health endpoint
        if curl -s http://127.0.0.1:3000/health | grep -q "ok\|healthy"; then
            log_success "Server is healthy: http://127.0.0.1:3000"
        else
            log_warn "Server started but health check failed"
        fi
    else
        log_error "Server failed to start"
        log_error "Check logs: $LOG_FILE"
        rm -f "$PID_FILE"
        exit 1
    fi

    cd "$PROJECT_ROOT"
}

# Show server status
show_status() {
    if check_running; then
        local pid=$(cat "$PID_FILE")
        log_success "Server is running (PID: $pid)"

        # Check health
        if curl -s --max-time 2 http://127.0.0.1:3000/health | grep -q "ok\|healthy"; then
            log_success "Health check: OK"
        else
            log_warn "Health check: Failed"
        fi
    else
        log_info "Server is not running"
    fi
}

# Show logs
show_logs() {
    if [ -f "$LOG_FILE" ]; then
        tail -f "$LOG_FILE"
    else
        log_error "Log file not found: $LOG_FILE"
    fi
}

# Print usage
print_usage() {
    echo "Auxin Development Server"
    echo ""
    echo "Usage: $0 <command>"
    echo ""
    echo "Commands:"
    echo "  start    Start the server"
    echo "  stop     Stop the server"
    echo "  restart  Restart the server"
    echo "  status   Show server status"
    echo "  logs     Follow server logs"
    echo ""
}

# Main
case "${1:-start}" in
    start)
        start_server
        ;;
    stop)
        stop_server
        ;;
    restart)
        stop_server
        sleep 1
        start_server
        ;;
    status)
        show_status
        ;;
    logs)
        show_logs
        ;;
    *)
        print_usage
        exit 1
        ;;
esac
