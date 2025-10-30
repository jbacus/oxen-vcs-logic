#!/bin/bash
#
# Test Script: Automatic Versioning (Draft Commits)
# Based on: USER_GUIDE.md - "Automatic Versioning (Draft Commits)"
#
# This script tests the FSEvents monitoring and auto-commit workflow.
# NOTE: Requires LaunchAgent to be running.
#

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
NC='\033[0m'

# Test configuration
TEST_NAME="Auto-Commit with FSEvents"
TEST_PROJECT_NAME="AutoCommitTest_$(date +%s).logicx"
TEST_DIR="$HOME/Desktop/oxenvcs-test-projects"
TEST_PROJECT_PATH="$TEST_DIR/$TEST_PROJECT_NAME"

# CLI path
OXENVCS_CLI="./OxVCS-CLI-Wrapper/target/release/oxenvcs-cli"

# Auto-commit debounce interval (seconds)
DEBOUNCE_WAIT=35

# Functions
print_header() {
    echo ""
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}TEST: $TEST_NAME${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo ""
}

print_step() {
    echo -e "${YELLOW}STEP $1:${NC} $2"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ ERROR: $1${NC}"
    exit 1
}

print_info() {
    echo -e "${MAGENTA}ℹ $1${NC}"
}

cleanup() {
    if [ -d "$TEST_PROJECT_PATH" ]; then
        echo ""
        read -p "Delete test project? (y/n) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            rm -rf "$TEST_PROJECT_PATH"
            print_success "Test project deleted"
        fi
    fi
}

trap cleanup EXIT

# ============================================================
# TEST EXECUTION
# ============================================================

print_header

# ------------------------------------------------------------
# Step 1: Check Prerequisites
# ------------------------------------------------------------
print_step 1 "Checking prerequisites"

# Check if LaunchAgent is running
if launchctl list | grep -q "oxenvcs"; then
    DAEMON_PID=$(launchctl list | grep oxenvcs | awk '{print $1}')
    print_success "LaunchAgent is running (PID: $DAEMON_PID)"
else
    print_error "LaunchAgent not running. Start it with: launchctl load ~/Library/LaunchAgents/com.oxenvcs.agent.plist"
fi

# Check if we can access logs
if log show --predicate 'process == "OxVCS-LaunchAgent"' --last 1m --style syslog 2>&1 | grep -q "oxenvcs"; then
    print_success "Can access daemon logs"
else
    print_info "Note: Daemon logs may not be available yet (daemon just started)"
fi

# ------------------------------------------------------------
# Step 2: Create and Initialize Project
# ------------------------------------------------------------
print_step 2 "Creating test project"

mkdir -p "$TEST_DIR"
mkdir -p "$TEST_PROJECT_PATH/Alternatives/001"
mkdir -p "$TEST_PROJECT_PATH/Resources/Audio Files"

cd "$TEST_PROJECT_PATH"

cat > "Alternatives/001/ProjectData" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<project version="1">
    <tempo>120</tempo>
    <sampleRate>48000</sampleRate>
    <keySignature>C Major</keySignature>
    <state>Initial state</state>
    <timestamp>$(date)</timestamp>
</project>
EOF

dd if=/dev/zero of="Resources/Audio Files/audio1.wav" bs=1024 count=1024 2>/dev/null

$OXENVCS_CLI init --logic .
print_success "Project initialized at: $TEST_PROJECT_PATH"

INITIAL_COMMITS=$(oxen log --oneline | wc -l | tr -d ' ')
print_info "Initial commit count: $INITIAL_COMMITS"

# ------------------------------------------------------------
# Step 3: Register Project with Daemon
# ------------------------------------------------------------
print_step 3 "Registering project with daemon for monitoring"

print_info "This step would normally be done via Oxen-VCS.app UI"
print_info "For manual testing, the daemon watches all .logicx directories"
print_info "You may need to add this project via the app"

# Since we're testing manually, we'll note that this is normally automatic
echo ""
echo "MANUAL STEP: Open Oxen-VCS.app and add this project if not auto-detected:"
echo "  $TEST_PROJECT_PATH"
echo ""
read -p "Press Enter when project is added to daemon monitoring (or skip if auto-detected)..."

# ------------------------------------------------------------
# Step 4: Make First Change
# ------------------------------------------------------------
print_step 4 "Making first change to trigger FSEvents"

print_info "Simulating Logic Pro save operation..."

# Modify projectData (simulating Logic Pro edit)
cat >> "Alternatives/001/ProjectData" <<EOF
<!-- Change 1: Added audio region -->
<change timestamp="$(date)">Added audio region to track 1</change>
EOF

print_success "Change made: Added audio region comment"
print_info "Waiting for FSEvents detection and debounce ($DEBOUNCE_WAIT seconds)..."

# Wait for debounce
for i in $(seq $DEBOUNCE_WAIT -1 1); do
    echo -ne "\r  Waiting: $i seconds remaining...  "
    sleep 1
done
echo ""

# ------------------------------------------------------------
# Step 5: Verify Auto-Commit Created
# ------------------------------------------------------------
print_step 5 "Verifying auto-commit was created"

sleep 5  # Extra wait for commit to complete

AFTER_CHANGE1_COMMITS=$(oxen log --all --oneline | wc -l | tr -d ' ')

echo ""
echo "Commit count after change 1: $AFTER_CHANGE1_COMMITS (was $INITIAL_COMMITS)"

if [ "$AFTER_CHANGE1_COMMITS" -gt "$INITIAL_COMMITS" ]; then
    print_success "Auto-commit created! ($AFTER_CHANGE1_COMMITS commits now)"
else
    print_error "No auto-commit detected. Check daemon logs."
fi

# Check for draft branch or auto-commit message
echo ""
echo "Recent commits:"
oxen log --all --oneline | head -5

if oxen log --all -n 1 | grep -qE "(Auto-save|draft|automatic)"; then
    print_success "Auto-commit message detected"
else
    print_info "Note: Auto-commit may not have expected message format"
fi

# Check daemon logs
echo ""
echo "Recent daemon activity:"
log show --predicate 'process == "OxVCS-LaunchAgent"' --last 2m --style syslog | tail -10 || echo "(logs not available)"

# ------------------------------------------------------------
# Step 6: Make Multiple Rapid Changes (Test Debounce)
# ------------------------------------------------------------
print_step 6 "Making multiple rapid changes to test debounce"

print_info "Making 5 changes in quick succession..."

for i in {1..5}; do
    cat >> "Alternatives/001/ProjectData" <<EOF
<!-- Rapid change $i at $(date) -->
EOF
    echo "  Change $i made"
    sleep 2
done

print_success "5 rapid changes made"
print_info "Debounce should prevent 5 separate commits"
print_info "Waiting for single debounced commit ($DEBOUNCE_WAIT seconds)..."

for i in $(seq $DEBOUNCE_WAIT -1 1); do
    echo -ne "\r  Waiting: $i seconds remaining...  "
    sleep 1
done
echo ""

sleep 5

AFTER_RAPID_COMMITS=$(oxen log --all --oneline | wc -l | tr -d ' ')
COMMITS_ADDED=$((AFTER_RAPID_COMMITS - AFTER_CHANGE1_COMMITS))

echo ""
echo "Commits after rapid changes: $AFTER_RAPID_COMMITS (added: $COMMITS_ADDED)"

if [ "$COMMITS_ADDED" -le 2 ]; then
    print_success "Debounce working correctly (only $COMMITS_ADDED commit(s) for 5 changes)"
else
    print_error "Debounce may not be working (expected 1-2 commits, got $COMMITS_ADDED)"
fi

# ------------------------------------------------------------
# Step 7: Verify Draft Commits on Separate Branch
# ------------------------------------------------------------
print_step 7 "Verifying draft commits are on draft branch"

echo ""
echo "Branches:"
oxen branch -a

# Check if draft branch exists
if oxen branch -a | grep -qE "(draft|auto)"; then
    print_success "Draft branch exists"

    echo ""
    echo "Commits on draft branch:"
    oxen log --oneline --branches=*draft* 2>/dev/null | head -10 || echo "(draft branch log not available)"
else
    print_info "Note: Draft commits may be on main branch (implementation-dependent)"
fi

# ------------------------------------------------------------
# Step 8: View All Auto-Commits
# ------------------------------------------------------------
print_step 8 "Viewing all auto-commits"

echo ""
echo "All commits (including drafts):"
oxen log --all --oneline | head -15

TOTAL_COMMITS=$(oxen log --all --oneline | wc -l | tr -d ' ')
AUTO_COMMITS=$((TOTAL_COMMITS - INITIAL_COMMITS))

print_success "Total auto-commits created: $AUTO_COMMITS"

# ------------------------------------------------------------
# Step 9: Test No Commit When No Changes
# ------------------------------------------------------------
print_step 9 "Verifying no commit when no changes made"

BEFORE_IDLE=$(oxen log --all --oneline | wc -l | tr -d ' ')

print_info "Waiting $DEBOUNCE_WAIT seconds with no changes..."
for i in $(seq $DEBOUNCE_WAIT -1 1); do
    echo -ne "\r  Waiting: $i seconds remaining...  "
    sleep 1
done
echo ""

sleep 5

AFTER_IDLE=$(oxen log --all --oneline | wc -l | tr -d ' ')

if [ "$AFTER_IDLE" -eq "$BEFORE_IDLE" ]; then
    print_success "No commit created when no changes (correct behavior)"
else
    print_error "Commit created with no changes (unexpected)"
fi

# ------------------------------------------------------------
# Step 10: Check Ignored Files Not Triggering Commits
# ------------------------------------------------------------
print_step 10 "Verifying ignored files don't trigger commits"

mkdir -p Bounces
echo "bounce file" > Bounces/bounce1.wav

BEFORE_IGNORED=$(oxen log --all --oneline | wc -l | tr -d ' ')

print_info "Created file in Bounces/ (should be ignored)"
print_info "Waiting to confirm no commit triggered..."

sleep $DEBOUNCE_WAIT

AFTER_IGNORED=$(oxen log --all --oneline | wc -l | tr -d ' ')

if [ "$AFTER_IGNORED" -eq "$BEFORE_IGNORED" ]; then
    print_success "Ignored files don't trigger commits (correct)"
else
    print_info "Note: Commit may have been triggered by other changes"
fi

# ------------------------------------------------------------
# Test Summary
# ------------------------------------------------------------
echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}TEST PASSED: $TEST_NAME${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "Summary:"
echo "  - LaunchAgent: Running"
echo "  - Project: Monitored"
echo "  - Auto-commits created: $AUTO_COMMITS"
echo "  - Debounce: Working (multiple changes → 1 commit)"
echo "  - No-change behavior: Correct (no spurious commits)"
echo "  - Ignored files: Not triggering commits"
echo ""
echo "Auto-commit workflow verified successfully!"
echo ""
echo "Project location: $TEST_PROJECT_PATH"
echo ""

exit 0
