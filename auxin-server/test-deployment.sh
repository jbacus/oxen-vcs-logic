#!/bin/bash

# Test deployment script - validates the local deployment

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
API_BASE="http://localhost:3000"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

PASSED=0
FAILED=0

test_api() {
    local name="$1"
    local method="$2"
    local endpoint="$3"
    local expected_status="${4:-200}"
    local data="$5"

    echo -n "Testing: $name... "

    local cmd="curl -s -w '\n%{http_code}' -X $method"
    if [ -n "$data" ]; then
        cmd="$cmd -H 'Content-Type: application/json' -d '$data'"
    fi
    cmd="$cmd $API_BASE$endpoint"

    local response=$(eval $cmd)
    local status=$(echo "$response" | tail -n1)
    local body=$(echo "$response" | head -n-1)

    if [ "$status" = "$expected_status" ]; then
        echo -e "${GREEN}✓ PASS${NC} (HTTP $status)"
        PASSED=$((PASSED + 1))
        return 0
    else
        echo -e "${RED}✗ FAIL${NC} (Expected HTTP $expected_status, got $status)"
        FAILED=$((FAILED + 1))
        return 1
    fi
}

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  🧪 Auxin Server - Deployment Test Suite"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check if server is running
echo -e "${BLUE}▶${NC} Checking if server is running..."
if ! curl -s --connect-timeout 2 "$API_BASE/health" > /dev/null 2>&1; then
    echo -e "${RED}✗${NC} Server is not running at $API_BASE"
    echo -e "${BLUE}▶${NC} Start it with: ./run-local.sh"
    exit 1
fi
echo -e "${GREEN}✓${NC} Server is running"
echo ""

# Run API tests
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  API Tests"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Core endpoints
test_api "Health check" "GET" "/health"
test_api "List repositories" "GET" "/api/repos"

# Create test repository
REPO_DATA='{"description":"Test repository for validation"}'
test_api "Create repository" "POST" "/api/repos/test/validation-repo" "201" "$REPO_DATA"

# Get created repository
test_api "Get repository info" "GET" "/api/repos/test/validation-repo"

# Repository operations
test_api "List commits" "GET" "/api/repos/test/validation-repo/commits"
test_api "List branches" "GET" "/api/repos/test/validation-repo/branches"

# Lock management
test_api "Get lock status" "GET" "/api/repos/test/validation-repo/locks/status"
LOCK_DATA='{"timeout_hours":1}'
test_api "Acquire lock" "POST" "/api/repos/test/validation-repo/locks/acquire" "200" "$LOCK_DATA"
test_api "Lock heartbeat" "POST" "/api/repos/test/validation-repo/locks/heartbeat"
test_api "Release lock" "POST" "/api/repos/test/validation-repo/locks/release"

# Frontend check
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Frontend Tests"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

if [ -d "$SCRIPT_DIR/frontend/dist" ]; then
    echo -n "Testing: Web UI availability... "
    response=$(curl -s -w '\n%{http_code}' "$API_BASE/")
    status=$(echo "$response" | tail -n1)

    if [ "$status" = "200" ]; then
        echo -e "${GREEN}✓ PASS${NC} (HTTP $status)"
        PASSED=$((PASSED + 1))
    else
        echo -e "${RED}✗ FAIL${NC} (HTTP $status)"
        FAILED=$((FAILED + 1))
    fi
else
    echo -e "${YELLOW}⚠${NC} Frontend not built - skipping Web UI tests"
fi

# Summary
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  Test Summary"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo -e "  ${GREEN}✓ Passed:${NC} $PASSED"
echo -e "  ${RED}✗ Failed:${NC} $FAILED"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    echo ""
    exit 0
else
    echo -e "${RED}✗ Some tests failed${NC}"
    echo ""
    exit 1
fi
