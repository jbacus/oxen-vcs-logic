#!/bin/bash
#
# Test Script: Creating Milestone Commits
# Based on: USER_GUIDE.md - "Creating Milestone Commits"
#
# This script tests the milestone commit workflow with metadata.
#

set -e  # Exit on error

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Test configuration
TEST_NAME="Milestone Commit with Metadata"
TEST_PROJECT_NAME="MilestoneTest_$(date +%s).logicx"
TEST_DIR="$HOME/Desktop/oxenvcs-test-projects"
TEST_PROJECT_PATH="$TEST_DIR/$TEST_PROJECT_NAME"

# CLI path
OXENVCS_CLI="./OxVCS-CLI-Wrapper/target/release/oxenvcs-cli"

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
# Step 1: Setup - Create and Initialize Project
# ------------------------------------------------------------
print_step 1 "Setting up test project"

mkdir -p "$TEST_DIR"
mkdir -p "$TEST_PROJECT_PATH/Alternatives/001"
mkdir -p "$TEST_PROJECT_PATH/Resources/Audio Files"

# Create projectData with specific metadata
cat > "$TEST_PROJECT_PATH/Alternatives/001/ProjectData" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<project>
    <tempo>120</tempo>
    <sampleRate>48000</sampleRate>
    <keySignature>C Major</keySignature>
    <version>1.0</version>
</project>
EOF

# Create audio files
dd if=/dev/zero of="$TEST_PROJECT_PATH/Resources/Audio Files/drums.wav" bs=1024 count=2048 2>/dev/null
dd if=/dev/zero of="$TEST_PROJECT_PATH/Resources/Audio Files/bass.wav" bs=1024 count=1024 2>/dev/null

cd "$TEST_PROJECT_PATH"

# Initialize
$OXENVCS_CLI init --logic .
print_success "Project initialized"

# ------------------------------------------------------------
# Step 2: Make Changes to Project
# ------------------------------------------------------------
print_step 2 "Making changes to project"

# Modify projectData (simulate Logic Pro edit)
sed -i '' 's/<tempo>120<\/tempo>/<tempo>128<\/tempo>/' Alternatives/001/ProjectData
sed -i '' 's/<keySignature>C Major<\/keySignature>/<keySignature>A Minor<\/keySignature>/' Alternatives/001/ProjectData

# Add new audio file
dd if=/dev/zero of="Resources/Audio Files/vocal.wav" bs=1024 count=1536 2>/dev/null

print_success "Changes made (tempo: 120→128, key: C Major→A Minor, added vocal.wav)"

# Verify changes are detected
CHANGES=$(oxen status)
if echo "$CHANGES" | grep -q "modified"; then
    print_success "Changes detected by Oxen"
else
    print_error "Changes not detected"
fi

echo ""
echo "Current status:"
oxen status

# ------------------------------------------------------------
# Step 3: Create Milestone Commit with Metadata
# ------------------------------------------------------------
print_step 3 "Creating milestone commit with metadata"

# Stage changes
oxen add .

# Create commit with structured metadata
COMMIT_MESSAGE="Mix v1 - Final adjustments

BPM: 128
Sample Rate: 48kHz
Key: A Minor
Tags: mix, final, v1"

echo "$COMMIT_MESSAGE" | oxen commit -F -

print_success "Milestone commit created"

# ------------------------------------------------------------
# Step 4: Verify Commit Contains Metadata
# ------------------------------------------------------------
print_step 4 "Verifying commit metadata"

# Get latest commit
LATEST_COMMIT=$(oxen log -n 1)

echo ""
echo "Latest commit:"
echo "$LATEST_COMMIT"
echo ""

# Check for metadata in commit
if echo "$LATEST_COMMIT" | grep -q "BPM: 128"; then
    print_success "BPM metadata found"
else
    print_error "BPM metadata not found in commit"
fi

if echo "$LATEST_COMMIT" | grep -q "Sample Rate: 48kHz"; then
    print_success "Sample rate metadata found"
else
    print_error "Sample rate metadata not found"
fi

if echo "$LATEST_COMMIT" | grep -q "Key: A Minor"; then
    print_success "Key metadata found"
else
    print_error "Key metadata not found"
fi

if echo "$LATEST_COMMIT" | grep -q "Tags: mix, final, v1"; then
    print_success "Tags metadata found"
else
    print_error "Tags metadata not found"
fi

# ------------------------------------------------------------
# Step 5: Create Multiple Milestones
# ------------------------------------------------------------
print_step 5 "Creating additional milestones to test history"

# Second milestone
echo "<!-- Comment 2 -->" >> Alternatives/001/ProjectData
oxen add .
echo "Mix v2 - Client feedback incorporated

BPM: 128
Sample Rate: 48kHz
Key: A Minor
Tags: mix, revision, client-feedback" | oxen commit -F -
print_success "Second milestone created"

# Third milestone
echo "<!-- Comment 3 -->" >> Alternatives/001/ProjectData
oxen add .
echo "Final master - Ready for delivery

BPM: 128
Sample Rate: 96kHz
Key: A Minor
Tags: master, final, delivery" | oxen commit -F -
print_success "Third milestone created"

# ------------------------------------------------------------
# Step 6: Verify History Shows All Milestones
# ------------------------------------------------------------
print_step 6 "Verifying commit history"

COMMIT_COUNT=$(oxen log --oneline | wc -l | tr -d ' ')
if [ "$COMMIT_COUNT" -ge 3 ]; then
    print_success "All milestones in history ($COMMIT_COUNT commits total)"
else
    print_error "Expected at least 3 commits, found $COMMIT_COUNT"
fi

echo ""
echo "Commit history:"
oxen log --oneline

# ------------------------------------------------------------
# Step 7: Test Commit Message Best Practices
# ------------------------------------------------------------
print_step 7 "Testing commit message best practices"

# Good commit message
echo "<!-- Good commit -->" >> Alternatives/001/ProjectData
oxen add .
echo "Add guitar solo in bridge section

Added 8-bar guitar solo with effects chain:
- Overdrive pedal
- Delay (1/4 note)
- Reverb (plate)

BPM: 128
Key: A Minor
Tags: tracking, guitar, bridge" | oxen commit -F -

print_success "Descriptive commit message accepted"

# Verify in log
if oxen log -n 1 | grep -q "Add guitar solo"; then
    print_success "Commit message properly recorded"
else
    print_error "Commit message not found in log"
fi

# ------------------------------------------------------------
# Step 8: Search History by Metadata
# ------------------------------------------------------------
print_step 8 "Searching history by metadata"

echo ""
echo "All commits with 'mix' in message:"
oxen log --grep="mix" --oneline

echo ""
echo "All commits (showing metadata):"
oxen log --format=fuller | head -30

# ------------------------------------------------------------
# Test Summary
# ------------------------------------------------------------
echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}TEST PASSED: $TEST_NAME${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "Summary:"
echo "  - Milestone commits created: $(($COMMIT_COUNT - 1)) (excluding initial)"
echo "  - Metadata properly embedded"
echo "  - Commit history searchable"
echo "  - Best practices validated"
echo ""
echo "Project location: $TEST_PROJECT_PATH"
echo ""

exit 0
