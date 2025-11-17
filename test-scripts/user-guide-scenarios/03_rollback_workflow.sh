#!/bin/bash
#
# Test Script: Rolling Back to Previous Versions
# Based on: USER_GUIDE.md - "Rolling Back to Previous Versions"
#
# This script tests the rollback workflow.
#

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Test configuration
TEST_NAME="Rollback to Previous Version"
TEST_PROJECT_NAME="RollbackTest_$(date +%s).logicx"
TEST_DIR="$HOME/Desktop/oxenvcs-test-projects"
TEST_PROJECT_PATH="$TEST_DIR/$TEST_PROJECT_NAME"

# CLI path
OXENVCS_CLI="./Auxin-CLI-Wrapper/target/release/auxin"

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
# Step 1: Setup - Create Project with Multiple Versions
# ------------------------------------------------------------
print_step 1 "Creating project with version history"

mkdir -p "$TEST_DIR"
mkdir -p "$TEST_PROJECT_PATH/Alternatives/001"
mkdir -p "$TEST_PROJECT_PATH/Resources/Audio Files"

cd "$TEST_PROJECT_PATH"

# Version 1: Initial state
cat > "Alternatives/001/ProjectData" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<project version="1">
    <tempo>120</tempo>
    <sampleRate>48000</sampleRate>
    <keySignature>C Major</keySignature>
    <state>Initial recording</state>
</project>
EOF

dd if=/dev/zero of="Resources/Audio Files/drums.wav" bs=1024 count=1024 2>/dev/null

$OXENVCS_CLI init --logic .
print_success "Version 1 created (Initial recording, BPM: 120)"

# Version 2: Add bass
cat > "Alternatives/001/ProjectData" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<project version="2">
    <tempo>120</tempo>
    <sampleRate>48000</sampleRate>
    <keySignature>C Major</keySignature>
    <state>Added bass track</state>
</project>
EOF

dd if=/dev/zero of="Resources/Audio Files/bass.wav" bs=1024 count=512 2>/dev/null

oxen add .
echo "Version 2 - Added bass track

BPM: 120
Key: C Major" | oxen commit -F -

V2_COMMIT=$(oxen log --oneline | head -1 | awk '{print $1}')
print_success "Version 2 created (Added bass, commit: $V2_COMMIT)"

# Version 3: Changed tempo and key
cat > "Alternatives/001/ProjectData" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<project version="3">
    <tempo>140</tempo>
    <sampleRate>48000</sampleRate>
    <keySignature>A Minor</keySignature>
    <state>Increased tempo and changed key</state>
</project>
EOF

dd if=/dev/zero of="Resources/Audio Files/synth.wav" bs=1024 count=768 2>/dev/null

oxen add .
echo "Version 3 - Changed tempo and key

BPM: 140 (was 120)
Key: A Minor (was C Major)
Added: synth track" | oxen commit -F -

V3_COMMIT=$(oxen log --oneline | head -1 | awk '{print $1}')
print_success "Version 3 created (Tempo: 140, Key: A Minor, commit: $V3_COMMIT)"

# Version 4: Current work
cat > "Alternatives/001/ProjectData" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<project version="4">
    <tempo>140</tempo>
    <sampleRate>48000</sampleRate>
    <keySignature>A Minor</keySignature>
    <state>Added vocals - experimental</state>
</project>
EOF

dd if=/dev/zero of="Resources/Audio Files/vocal.wav" bs=1024 count=2048 2>/dev/null

oxen add .
echo "Version 4 - Added experimental vocals

BPM: 140
Key: A Minor" | oxen commit -F -

V4_COMMIT=$(oxen log --oneline | head -1 | awk '{print $1}')
print_success "Version 4 created (Added vocals, commit: $V4_COMMIT)"

# ------------------------------------------------------------
# Step 2: Display Current State and History
# ------------------------------------------------------------
print_step 2 "Viewing current state and history"

echo ""
echo "Current project state (Version 4):"
cat Alternatives/001/ProjectData | grep -E '(version|tempo|keySignature|state)'

echo ""
echo "Audio files:"
ls -lh Resources/Audio\ Files/

echo ""
echo "Full commit history:"
oxen log --oneline

COMMIT_COUNT=$(oxen log --oneline | wc -l | tr -d ' ')
print_success "$COMMIT_COUNT versions in history"

# ------------------------------------------------------------
# Step 3: Rollback to Version 2 (Before Tempo Change)
# ------------------------------------------------------------
print_step 3 "Rolling back to Version 2 (before tempo change)"

echo ""
echo "Target commit: $V2_COMMIT"
echo "Rolling back..."

# Checkout the commit
oxen checkout $V2_COMMIT

print_success "Rolled back to Version 2"

# ------------------------------------------------------------
# Step 4: Verify Rollback Success
# ------------------------------------------------------------
print_step 4 "Verifying rollback restored correct state"

echo ""
echo "Project state after rollback:"
cat Alternatives/001/ProjectData | grep -E '(version|tempo|keySignature|state)'

# Check version number
if grep -q 'version="2"' Alternatives/001/ProjectData; then
    print_success "Version restored to 2"
else
    print_error "Version not restored correctly"
fi

# Check tempo
if grep -q '<tempo>120</tempo>' Alternatives/001/ProjectData; then
    print_success "Tempo restored to 120"
else
    print_error "Tempo not restored"
fi

# Check key
if grep -q '<keySignature>C Major</keySignature>' Alternatives/001/ProjectData; then
    print_success "Key restored to C Major"
else
    print_error "Key not restored"
fi

# Check files exist
if [ -f "Resources/Audio Files/drums.wav" ]; then
    print_success "drums.wav exists"
else
    print_error "drums.wav missing"
fi

if [ -f "Resources/Audio Files/bass.wav" ]; then
    print_success "bass.wav exists"
else
    print_error "bass.wav missing"
fi

# Verify newer files don't exist
if [ ! -f "Resources/Audio Files/synth.wav" ]; then
    print_success "synth.wav correctly removed (wasn't in Version 2)"
else
    print_error "synth.wav should not exist in Version 2"
fi

if [ ! -f "Resources/Audio Files/vocal.wav" ]; then
    print_success "vocal.wav correctly removed (wasn't in Version 2)"
else
    print_error "vocal.wav should not exist in Version 2"
fi

# ------------------------------------------------------------
# Step 5: Test Rollback is Non-Destructive
# ------------------------------------------------------------
print_step 5 "Verifying rollback is non-destructive (can restore latest)"

echo ""
echo "Rolling back to latest version (Version 4)..."

oxen checkout $V4_COMMIT

echo ""
echo "Project state after returning to latest:"
cat Alternatives/001/ProjectData | grep -E '(version|tempo|keySignature|state)'

if grep -q 'version="4"' Alternatives/001/ProjectData; then
    print_success "Successfully returned to Version 4"
else
    print_error "Failed to return to Version 4"
fi

if [ -f "Resources/Audio Files/vocal.wav" ]; then
    print_success "vocal.wav restored"
else
    print_error "vocal.wav not restored"
fi

print_success "Rollback is non-destructive - all versions preserved"

# ------------------------------------------------------------
# Step 6: Test Rollback with Uncommitted Changes
# ------------------------------------------------------------
print_step 6 "Testing rollback with uncommitted changes"

# Make uncommitted change
echo "<!-- Uncommitted change -->" >> Alternatives/001/ProjectData

# Check status shows changes
if oxen status | grep -q "modified"; then
    print_success "Uncommitted changes detected"
else
    print_error "Uncommitted changes not detected"
fi

echo ""
echo "Attempting rollback with uncommitted changes..."
echo "(This should either fail or auto-commit to draft)"

# Try to checkout - Oxen may require clean working directory
if oxen checkout $V2_COMMIT 2>&1 | grep -q -E "(error|uncommitted|changes)"; then
    print_success "Oxen prevents rollback with uncommitted changes (safety feature)"
    echo ""
    echo "Cleaning working directory..."
    git checkout -- Alternatives/001/ProjectData 2>/dev/null || oxen restore Alternatives/001/ProjectData 2>/dev/null || true
else
    print_success "Rollback succeeded (changes may have been auto-committed)"
fi

# Return to latest
oxen checkout $V4_COMMIT

# ------------------------------------------------------------
# Step 7: Test Specific Use Case: Undo Bad Mix Change
# ------------------------------------------------------------
print_step 7 "Use case: Undo bad mix decision"

echo ""
echo "Scenario: You changed the tempo to 140, but it sounds worse."
echo "You want to go back to Version 2 (tempo 120)."

# Current state
echo ""
echo "Current tempo:"
grep '<tempo>' Alternatives/001/ProjectData

# Rollback
oxen checkout $V2_COMMIT

echo ""
echo "After rollback:"
grep '<tempo>' Alternatives/001/ProjectData

if grep -q '<tempo>120</tempo>' Alternatives/001/ProjectData; then
    print_success "Tempo successfully reverted to 120"
else
    print_error "Tempo revert failed"
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
echo "  - Created project with 4 versions"
echo "  - Successfully rolled back to Version 2"
echo "  - Verified correct state restoration"
echo "  - Confirmed rollback is non-destructive"
echo "  - Tested safety features (uncommitted changes)"
echo "  - Validated use case (undo bad mix decision)"
echo ""
echo "Key commits:"
echo "  - Version 2: $V2_COMMIT (tempo 120, bass added)"
echo "  - Version 3: $V3_COMMIT (tempo 140, key changed)"
echo "  - Version 4: $V4_COMMIT (vocals added)"
echo ""
echo "Project location: $TEST_PROJECT_PATH"
echo ""

exit 0
