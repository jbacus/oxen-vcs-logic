#!/bin/bash

# Test script to validate deployment setup
# This performs a dry-run check of requirements without actually installing

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

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
log_info "===== Auxin Server Installation Test ====="
echo ""

# Check OS
log_info "Checking operating system..."
if [[ "$OSTYPE" == "darwin"* ]]; then
    log_success "Running on macOS"
    sw_vers | sed 's/^/  /'
else
    log_error "Not macOS (detected: $OSTYPE)"
    exit 1
fi

# Check Rust
echo ""
log_info "Checking Rust installation..."
if command -v cargo &> /dev/null; then
    log_success "Rust toolchain found"
    rustc --version | sed 's/^/  /'
    cargo --version | sed 's/^/  /'
else
    log_error "Rust not found"
    echo "  Install with: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Check for required build tools
echo ""
log_info "Checking build tools..."

if command -v git &> /dev/null; then
    log_success "git: $(git --version)"
else
    log_error "git not found"
fi

if command -v openssl &> /dev/null; then
    log_success "openssl: $(openssl version)"
else
    log_warn "openssl not found (needed for random secret generation)"
fi

# Check permissions
echo ""
log_info "Checking permissions..."

# Check if we can write to /usr/local/bin
if [ -w "/usr/local/bin" ] || sudo -n true 2>/dev/null; then
    log_success "Can write to /usr/local/bin (or sudo available)"
else
    log_warn "May need sudo password for /usr/local/bin"
fi

# Check if we can create /var/oxen
if [ -d "/var/oxen" ]; then
    log_success "/var/oxen already exists"
    if [ -w "/var/oxen" ]; then
        log_success "Can write to /var/oxen"
    else
        log_warn "Cannot write to /var/oxen (may need sudo)"
    fi
elif sudo -n mkdir -p /var/oxen 2>/dev/null && sudo -n rmdir /var/oxen 2>/dev/null; then
    log_success "Can create /var/oxen"
else
    log_warn "May need sudo password for /var/oxen"
fi

# Check cargo dependencies
echo ""
log_info "Checking if project can compile..."
cd "$PROJECT_DIR"

if cargo check --quiet 2>&1 | grep -q "error"; then
    log_error "Project has compilation errors"
    cargo check 2>&1 | tail -20
    exit 1
else
    log_success "Project compiles successfully"
fi

# Check disk space
echo ""
log_info "Checking disk space..."
df_output=$(df -h /var 2>/dev/null || df -h / 2>/dev/null)
available=$(echo "$df_output" | tail -1 | awk '{print $4}')
log_success "Available disk space: $available"

# Summary
echo ""
log_success "===== Pre-Installation Check Passed ====="
echo ""
echo "System is ready for auxin-server installation."
echo ""
echo "To install:"
echo "  cd $SCRIPT_DIR"
echo "  ./setup.sh"
echo ""
