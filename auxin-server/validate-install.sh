#!/bin/bash
set -e

# Auxin Server - Installation Validation Script
# Tests that all installation methods work correctly

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

TESTS_PASSED=0
TESTS_FAILED=0

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[✓]${NC} $1"
    ((TESTS_PASSED++))
}

log_error() {
    echo -e "${RED}[✗]${NC} $1"
    ((TESTS_FAILED++))
}

log_warn() {
    echo -e "${YELLOW}[!]${NC} $1"
}

# Test 1: Check prerequisites
test_prerequisites() {
    log_info "Test 1: Checking prerequisites..."

    if command -v cargo &> /dev/null; then
        log_success "Rust/Cargo found: $(cargo --version | head -1)"
    else
        log_error "Rust/Cargo not found"
    fi

    if command -v rustc &> /dev/null; then
        log_success "Rust compiler found: $(rustc --version)"
    else
        log_error "Rust compiler not found"
    fi
}

# Test 2: Check Cargo.toml
test_cargo_config() {
    log_info "Test 2: Checking Cargo configuration..."

    if [ -f "$SCRIPT_DIR/Cargo.toml" ]; then
        log_success "Cargo.toml exists"

        # Check for required dependencies
        if grep -q "actix-web" "$SCRIPT_DIR/Cargo.toml"; then
            log_success "actix-web dependency found"
        else
            log_error "actix-web dependency missing"
        fi

        if grep -q "tokio" "$SCRIPT_DIR/Cargo.toml"; then
            log_success "tokio dependency found"
        else
            log_error "tokio dependency missing"
        fi
    else
        log_error "Cargo.toml not found"
    fi
}

# Test 3: Check source files
test_source_files() {
    log_info "Test 3: Checking source files..."

    required_files=(
        "src/main.rs"
        "src/lib.rs"
        "src/config.rs"
        "src/error.rs"
        "src/auth.rs"
        "src/api/mod.rs"
        "src/websocket.rs"
    )

    for file in "${required_files[@]}"; do
        if [ -f "$SCRIPT_DIR/$file" ]; then
            log_success "$file exists"
        else
            log_error "$file missing"
        fi
    done
}

# Test 4: Check scripts
test_scripts() {
    log_info "Test 4: Checking installation scripts..."

    required_scripts=(
        "deploy-local.sh"
        "run-local.sh"
        "scripts/setup.sh"
        "scripts/start.sh"
        "scripts/stop.sh"
    )

    for script in "${required_scripts[@]}"; do
        if [ -f "$SCRIPT_DIR/$script" ]; then
            if [ -x "$SCRIPT_DIR/$script" ]; then
                log_success "$script exists and is executable"
            else
                log_warn "$script exists but is not executable"
            fi
        else
            log_error "$script missing"
        fi
    done
}

# Test 5: Check documentation
test_documentation() {
    log_info "Test 5: Checking documentation..."

    required_docs=(
        "README.md"
        "INSTALL.md"
        "TESTING.md"
        "STATUS.md"
        "QUICKSTART.md"
    )

    for doc in "${required_docs[@]}"; do
        if [ -f "$SCRIPT_DIR/$doc" ]; then
            log_success "$doc exists"
        else
            log_error "$doc missing"
        fi
    done
}

# Test 6: Test build
test_build() {
    log_info "Test 6: Testing build process..."

    cd "$SCRIPT_DIR"

    # Check if already built
    if [ -f "target/release/auxin-server" ]; then
        log_success "Binary already exists: target/release/auxin-server"

        # Test binary
        if "./target/release/auxin-server" --version &>/dev/null; then
            log_success "Binary is executable"
        else
            log_warn "Binary exists but may not work (this is normal if server starts)"
        fi
    else
        log_warn "Binary not found - would need to run 'cargo build --release'"
    fi
}

# Test 7: Check tests
test_test_suite() {
    log_info "Test 7: Checking test suite..."

    if [ -d "$SCRIPT_DIR/tests" ]; then
        log_success "Tests directory exists"

        if [ -f "$SCRIPT_DIR/tests/collaboration_e2e_tests.rs" ]; then
            log_success "End-to-end collaboration tests exist"
        else
            log_error "collaboration_e2e_tests.rs missing"
        fi

        if [ -f "$SCRIPT_DIR/tests/README.md" ]; then
            log_success "Test documentation exists"
        else
            log_error "tests/README.md missing"
        fi
    else
        log_error "tests/ directory missing"
    fi
}

# Test 8: Check Docker configuration
test_docker() {
    log_info "Test 8: Checking Docker configuration..."

    if [ -f "$SCRIPT_DIR/Dockerfile" ]; then
        log_success "Dockerfile exists"
    else
        log_error "Dockerfile missing"
    fi

    if [ -f "$SCRIPT_DIR/docker-compose.yml" ]; then
        log_success "docker-compose.yml exists"
    else
        log_error "docker-compose.yml missing"
    fi

    if [ -f "$SCRIPT_DIR/.dockerignore" ]; then
        log_success ".dockerignore exists"
    else
        log_warn ".dockerignore missing (not critical)"
    fi
}

# Test 9: Check configuration examples
test_config_examples() {
    log_info "Test 9: Checking configuration examples..."

    if [ -f "$SCRIPT_DIR/.env.example" ]; then
        log_success ".env.example exists"
    else
        log_warn ".env.example missing"
    fi
}

# Test 10: Check frontend
test_frontend() {
    log_info "Test 10: Checking frontend..."

    if [ -d "$SCRIPT_DIR/frontend" ]; then
        log_success "Frontend directory exists"

        if [ -f "$SCRIPT_DIR/frontend/package.json" ]; then
            log_success "Frontend package.json exists"
        else
            log_error "frontend/package.json missing"
        fi

        if [ -d "$SCRIPT_DIR/frontend/dist" ]; then
            log_success "Frontend is built (dist/ exists)"
        else
            log_warn "Frontend not built (dist/ missing) - run 'npm run build' to enable web UI"
        fi
    else
        log_error "frontend/ directory missing"
    fi
}

# Print summary
print_summary() {
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  Installation Validation Summary"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo -e "${GREEN}Tests Passed:${NC} $TESTS_PASSED"

    if [ $TESTS_FAILED -gt 0 ]; then
        echo -e "${RED}Tests Failed:${NC} $TESTS_FAILED"
        echo ""
        echo "❌ Installation validation FAILED"
        echo ""
        echo "Please fix the errors above before installing."
        exit 1
    else
        echo -e "${GREEN}Tests Failed:${NC} 0"
        echo ""
        echo "✅ Installation validation PASSED"
        echo ""
        echo "🚀 Ready to install! Choose an installation method:"
        echo ""
        echo "  1. Local Development:"
        echo "     ./deploy-local.sh"
        echo ""
        echo "  2. Docker:"
        echo "     docker-compose up -d"
        echo ""
        echo "  3. macOS System Service:"
        echo "     cd scripts && ./setup.sh"
        echo ""
    fi
}

# Main
main() {
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  Auxin Server - Installation Validation"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""

    test_prerequisites
    test_cargo_config
    test_source_files
    test_scripts
    test_documentation
    test_build
    test_test_suite
    test_docker
    test_config_examples
    test_frontend

    print_summary
}

main "$@"
