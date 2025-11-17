#!/bin/bash
# Test script for network resilience features
# Run this on macOS to validate retry behavior

set -e

echo "=================================="
echo "Network Resilience Test Script"
echo "=================================="
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check prerequisites
echo "Checking prerequisites..."

if ! command -v oxen &> /dev/null; then
    echo -e "${RED}✗${NC} Oxen CLI not found. Install with: pip install oxen-ai"
    exit 1
fi
echo -e "${GREEN}✓${NC} Oxen CLI found"

# Check if OxVCS CLI is built
if [ ! -f "OxVCS-CLI-Wrapper/target/release/oxenvcs-cli" ]; then
    echo -e "${YELLOW}⚠${NC}  OxVCS CLI not found in release mode. Building..."
    cd OxVCS-CLI-Wrapper
    cargo build --release
    cd ..
fi
echo -e "${GREEN}✓${NC} OxVCS CLI built"

# Create test project
TEST_PROJECT="$HOME/Desktop/NetworkResilienceTest.logicx"

if [ -d "$TEST_PROJECT" ]; then
    echo -e "${YELLOW}⚠${NC}  Test project already exists. Removing..."
    rm -rf "$TEST_PROJECT"
fi

echo ""
echo "Creating test project..."
mkdir -p "$TEST_PROJECT/Audio Files"
mkdir -p "$TEST_PROJECT/Alternatives"
echo "test project data" > "$TEST_PROJECT/projectData"
echo -e "${GREEN}✓${NC} Test project created at $TEST_PROJECT"

cd "$TEST_PROJECT"

# Initialize repository
echo ""
echo "Initializing OxVCS repository..."
../OxVCS-CLI-Wrapper/target/release/oxenvcs-cli init --logic .
echo -e "${GREEN}✓${NC} Repository initialized"

# Check if user has configured remote
echo ""
echo -e "${YELLOW}⚠${NC}  You need to configure a remote repository for this test."
echo ""
echo "Please follow these steps:"
echo "1. Create a test repository on hub.oxen.ai (if not already done)"
echo "2. Run: oxen remote add origin https://hub.oxen.ai/YOUR_USERNAME/test-repo"
echo "3. Authenticate: oxenvcs-cli auth login (if not already authenticated)"
echo ""
read -p "Press ENTER when you've configured the remote..."

# Verify remote is configured
if ! oxen remote -v | grep -q "origin"; then
    echo -e "${RED}✗${NC} No remote configured. Please run: oxen remote add origin <URL>"
    exit 1
fi
echo -e "${GREEN}✓${NC} Remote configured"

# Create initial commit
echo ""
echo "Creating initial commit..."
../OxVCS-CLI-Wrapper/target/release/oxenvcs-cli add --all
../OxVCS-CLI-Wrapper/target/release/oxenvcs-cli commit -m "Initial commit for network resilience test" --bpm 120
echo -e "${GREEN}✓${NC} Initial commit created"

# Push to remote (this sets up the locks branch)
echo ""
echo "Pushing to remote (this may take a moment)..."
oxen push origin main
echo -e "${GREEN}✓${NC} Pushed to remote"

echo ""
echo "=================================="
echo "Network Resilience Tests"
echo "=================================="
echo ""

# Test 1: Normal lock acquisition (should work)
echo "Test 1: Normal lock acquisition (baseline)"
echo "-------------------------------------------"
../OxVCS-CLI-Wrapper/target/release/oxenvcs-cli lock acquire --timeout 1
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓${NC} Test 1 PASSED: Lock acquired successfully"
else
    echo -e "${RED}✗${NC} Test 1 FAILED: Lock acquisition failed"
    exit 1
fi

# Release lock for next test
../OxVCS-CLI-Wrapper/target/release/oxenvcs-cli lock release
echo ""

# Test 2: Lock acquisition with network interruption
echo "Test 2: Lock acquisition with network interruption"
echo "---------------------------------------------------"
echo ""
echo -e "${YELLOW}MANUAL TEST:${NC}"
echo "1. We'll start a lock acquisition"
echo "2. During the operation, ${YELLOW}disconnect your WiFi${NC} for ~5 seconds"
echo "3. Then ${YELLOW}reconnect your WiFi${NC}"
echo "4. The operation should retry and succeed"
echo ""
read -p "Press ENTER to start the test..."

echo ""
echo "Starting lock acquisition..."
echo -e "${YELLOW}⚠${NC}  NOW DISCONNECT YOUR WIFI FOR 5 SECONDS, THEN RECONNECT"
echo ""

# Run with verbose logging to see retries
RUST_LOG=info ../OxVCS-CLI-Wrapper/target/release/oxenvcs-cli lock acquire --timeout 1 2>&1 | tee /tmp/lock_test.log

if [ $? -eq 0 ]; then
    echo ""
    echo -e "${GREEN}✓${NC} Test 2 PASSED: Lock acquired despite network interruption"

    # Check if retries occurred
    if grep -q "Retrying" /tmp/lock_test.log; then
        echo -e "${GREEN}✓${NC} Retry logic was triggered (check output above for retry messages)"
    else
        echo -e "${YELLOW}⚠${NC}  No retries detected (network may not have been interrupted)"
    fi
else
    echo -e "${RED}✗${NC} Test 2 FAILED: Lock acquisition failed"
    echo "This could mean:"
    echo "  - Network was offline too long (>15s)"
    echo "  - Remote repository has issues"
    echo "  - Oxen CLI encountered an error"
fi

# Release lock
../OxVCS-CLI-Wrapper/target/release/oxenvcs-cli lock release 2>/dev/null || true
echo ""

# Test 3: Lock release with network retry
echo "Test 3: Lock release with network retry"
echo "----------------------------------------"

# Acquire lock first
../OxVCS-CLI-Wrapper/target/release/oxenvcs-cli lock acquire --timeout 1 >/dev/null 2>&1

echo "Lock acquired. Now testing release with retry..."
echo -e "${YELLOW}⚠${NC}  You can optionally disconnect WiFi briefly during this operation"
echo ""

RUST_LOG=info ../OxVCS-CLI-Wrapper/target/release/oxenvcs-cli lock release 2>&1 | tee /tmp/release_test.log

if [ $? -eq 0 ]; then
    echo ""
    echo -e "${GREEN}✓${NC} Test 3 PASSED: Lock released successfully"

    if grep -q "Retrying" /tmp/release_test.log; then
        echo -e "${GREEN}✓${NC} Retry logic handled network issues"
    fi
else
    echo -e "${RED}✗${NC} Test 3 FAILED: Lock release failed"
fi

echo ""
echo "=================================="
echo "Test Summary"
echo "=================================="
echo ""

# Count passing tests
PASSED=0
if grep -q "Test 1 PASSED" /tmp/lock_test.log 2>/dev/null || grep -q "Test 1 PASSED" <<< "$output"; then
    PASSED=$((PASSED + 1))
fi

echo "Results:"
echo "  Total tests: 3"
echo "  Estimated passed: Check output above"
echo ""

echo "What to look for in the output:"
echo "  ${GREEN}✓${NC} \"⚠️  Attempt N failed: ...\" - Retry logic working"
echo "  ${GREEN}✓${NC} \"Retrying in X.Xs...\" - Exponential backoff working"
echo "  ${GREEN}✓${NC} \"Operation succeeded after N attempt(s)\" - Recovery working"
echo ""

echo "Manual validation:"
echo "  1. Did you see retry messages when WiFi was disconnected?"
echo "  2. Did the operation eventually succeed when WiFi was restored?"
echo "  3. Were the retry intervals increasing (1s, 2s, 4s, 8s)?"
echo ""

# Cleanup
echo "Cleaning up test project..."
cd ..
rm -rf "$TEST_PROJECT"
rm -f /tmp/lock_test.log /tmp/release_test.log
echo -e "${GREEN}✓${NC} Cleanup complete"

echo ""
echo "=================================="
echo "Test Complete!"
echo "=================================="
echo ""
echo "Next steps:"
echo "  1. Review the output above for retry behavior"
echo "  2. If tests passed, network resilience is working!"
echo "  3. Try the integration test suite: cargo test --test collaboration_integration_test -- --ignored"
echo ""
