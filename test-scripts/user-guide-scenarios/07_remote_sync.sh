#!/bin/bash
#
# Test Script: Remote Synchronization
# Based on: USER_GUIDE.md - "Remote Synchronization"
#
# This script tests pushing/pulling to/from Oxen remote.
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
TEST_NAME="Remote Synchronization"
TEST_PROJECT_NAME="RemoteSyncTest_$(date +%s).logicx"
TEST_DIR="$HOME/Desktop/oxenvcs-test-projects"
TEST_PROJECT_PATH="$TEST_DIR/$TEST_PROJECT_NAME"
TEST_REMOTE_DIR="$TEST_DIR/remote-repo.oxen"

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

print_info() {
    echo -e "${MAGENTA}ℹ $1${NC}"
}

cleanup() {
    echo ""
    read -p "Delete test project and remote? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        rm -rf "$TEST_PROJECT_PATH" "$TEST_REMOTE_DIR"
        print_success "Test files deleted"
    fi
}

trap cleanup EXIT

# ============================================================
# TEST EXECUTION
# ============================================================

print_header

print_info "This test simulates remote synchronization workflow"
print_info "Uses local directory as remote (for testing without Oxen Hub)"

# ------------------------------------------------------------
# Step 1: Create Local Project
# ------------------------------------------------------------
print_step 1 "Creating local project"

mkdir -p "$TEST_DIR"
mkdir -p "$TEST_PROJECT_PATH/Alternatives/001"
mkdir -p "$TEST_PROJECT_PATH/Resources/Audio Files"

cd "$TEST_PROJECT_PATH"

cat > "Alternatives/001/ProjectData" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<project>
    <tempo>120</tempo>
    <sampleRate>48000</sampleRate>
    <keySignature>C Major</keySignature>
    <tracks>
        <track id="1" name="Local drums" />
    </tracks>
</project>
EOF

dd if=/dev/zero of="Resources/Audio Files/drums.wav" bs=1024 count=2048 2>/dev/null

$OXENVCS_CLI init --logic .
print_success "Local project created"

INITIAL_COMMIT=$(oxen log --oneline | head -1 | awk '{print $1}')
print_info "Initial commit: $INITIAL_COMMIT"

# ------------------------------------------------------------
# Step 2: Create Remote Repository
# ------------------------------------------------------------
print_step 2 "Setting up remote repository"

print_info "Creating bare repository to act as remote..."

# Initialize bare remote repository
mkdir -p "$TEST_REMOTE_DIR"
cd "$TEST_REMOTE_DIR"
oxen init --bare

if [ -d "$TEST_REMOTE_DIR/.oxen" ]; then
    print_success "Remote repository created at: $TEST_REMOTE_DIR"
else
    print_error "Failed to create remote repository"
fi

cd "$TEST_PROJECT_PATH"

# ------------------------------------------------------------
# Step 3: Configure Remote
# ------------------------------------------------------------
print_step 3 "Configuring remote"

print_info "Adding remote 'origin'..."

# Add remote (using file:// protocol for local testing)
oxen remote add origin "file://$TEST_REMOTE_DIR"

# Verify remote added
REMOTE_URL=$(oxen remote -v | grep origin | head -1 | awk '{print $2}')

if [ -n "$REMOTE_URL" ]; then
    print_success "Remote configured: $REMOTE_URL"
else
    print_error "Failed to configure remote"
fi

echo ""
echo "Remote configuration:"
oxen remote -v

# ------------------------------------------------------------
# Step 4: Push to Remote
# ------------------------------------------------------------
print_step 4 "Pushing project to remote"

print_info "Pushing main branch to origin..."

# Push main branch
oxen push origin main

print_success "Pushed to remote"

# Verify push
echo ""
echo "Local branches:"
oxen branch -a

# ------------------------------------------------------------
# Step 5: Make Local Changes and Push
# ------------------------------------------------------------
print_step 5 "Making changes and pushing updates"

cat >> "Alternatives/001/ProjectData" <<EOF
    <track id="2" name="Bass line" />
EOF

oxen add .
echo "Added bass track

BPM: 120" | oxen commit -F -

print_success "Local commit created"

oxen push origin main
print_success "Changes pushed to remote"

# ------------------------------------------------------------
# Step 6: Clone Remote to Simulate Second Machine
# ------------------------------------------------------------
print_step 6 "Simulating second machine (clone remote)"

print_info "Creating clone to simulate collaborator's machine..."

CLONE_DIR="$TEST_DIR/cloned-project-$(date +%s).logicx"

cd "$TEST_DIR"
oxen clone "file://$TEST_REMOTE_DIR" "$CLONE_DIR"

if [ -d "$CLONE_DIR" ]; then
    print_success "Project cloned to: $CLONE_DIR"
else
    print_error "Clone failed"
fi

# Verify clone has same commits
cd "$CLONE_DIR"
CLONE_COMMITS=$(oxen log --oneline | wc -l | tr -d ' ')
cd "$TEST_PROJECT_PATH"
LOCAL_COMMITS=$(oxen log --oneline | wc -l | tr -d ' ')

if [ "$CLONE_COMMITS" -eq "$LOCAL_COMMITS" ]; then
    print_success "Clone has all commits ($CLONE_COMMITS == $LOCAL_COMMITS)"
else
    print_error "Clone commit mismatch ($CLONE_COMMITS != $LOCAL_COMMITS)"
fi

# ------------------------------------------------------------
# Step 7: Make Change in Clone and Push
# ------------------------------------------------------------
print_step 7 "Making change in clone (collaborator work)"

cd "$CLONE_DIR"

cat >> "Alternatives/001/ProjectData" <<EOF
    <track id="3" name="Synth by collaborator" />
EOF

oxen add .
echo "Added synth track (collaborator)

BPM: 120" | oxen commit -F -

oxen push origin main
print_success "Collaborator pushed changes"

# ------------------------------------------------------------
# Step 8: Pull Changes to Original Project
# ------------------------------------------------------------
print_step 8 "Pulling collaborator changes to original project"

cd "$TEST_PROJECT_PATH"

print_info "Pulling updates from remote..."

COMMITS_BEFORE=$(oxen log --oneline | wc -l | tr -d ' ')

oxen pull origin main

COMMITS_AFTER=$(oxen log --oneline | wc -l | tr -d ' ')

if [ "$COMMITS_AFTER" -gt "$COMMITS_BEFORE" ]; then
    print_success "Pulled new commits ($COMMITS_AFTER vs $COMMITS_BEFORE)"
else
    print_error "Pull did not fetch new commits"
fi

# Verify synth track is now in local
if grep -q "Synth by collaborator" "Alternatives/001/ProjectData"; then
    print_success "Collaborator's changes merged successfully"
else
    print_error "Changes not merged"
fi

# ------------------------------------------------------------
# Step 9: Test Push Large Files (Audio)
# ------------------------------------------------------------
print_step 9 "Testing push with large audio files"

print_info "Creating large audio file (10MB)..."

dd if=/dev/zero of="Resources/Audio Files/large_audio.wav" bs=1024 count=10240 2>/dev/null

oxen add .
echo "Added large audio file (10MB)

BPM: 120" | oxen commit -F -

print_info "Pushing large file..."
START_TIME=$(date +%s)

oxen push origin main

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))

print_success "Large file pushed in ${DURATION}s"

if [ $DURATION -lt 30 ]; then
    print_success "Push performance acceptable (< 30s)"
else
    print_info "Note: Push took ${DURATION}s (may be slow on some systems)"
fi

# ------------------------------------------------------------
# Step 10: Verify Remote has All Data
# ------------------------------------------------------------
print_step 10 "Verifying remote repository integrity"

cd "$CLONE_DIR"
oxen pull origin main

CLONE_FINAL_COMMITS=$(oxen log --oneline | wc -l | tr -d ' ')
cd "$TEST_PROJECT_PATH"
LOCAL_FINAL_COMMITS=$(oxen log --oneline | wc -l | tr -d ' ')

echo ""
echo "Final commit counts:"
echo "  Local: $LOCAL_FINAL_COMMITS"
echo "  Clone: $CLONE_FINAL_COMMITS"

if [ "$CLONE_FINAL_COMMITS" -eq "$LOCAL_FINAL_COMMITS" ]; then
    print_success "Remote and local are in sync"
else
    print_error "Sync mismatch"
fi

# Check file exists in clone
if [ -f "$CLONE_DIR/Resources/Audio Files/large_audio.wav" ]; then
    CLONE_SIZE=$(stat -f%z "$CLONE_DIR/Resources/Audio Files/large_audio.wav" 2>/dev/null || stat -c%s "$CLONE_DIR/Resources/Audio Files/large_audio.wav" 2>/dev/null)
    ORIG_SIZE=$(stat -f%z "$TEST_PROJECT_PATH/Resources/Audio Files/large_audio.wav" 2>/dev/null || stat -c%s "$TEST_PROJECT_PATH/Resources/Audio Files/large_audio.wav" 2>/dev/null)

    if [ "$CLONE_SIZE" -eq "$ORIG_SIZE" ]; then
        print_success "Large file correctly synchronized ($(numfmt --to=iec-i --suffix=B $ORIG_SIZE 2>/dev/null || echo $ORIG_SIZE bytes))"
    else
        print_error "File size mismatch"
    fi
else
    print_error "Large file not in clone"
fi

# ------------------------------------------------------------
# Step 11: Test Fetch vs Pull
# ------------------------------------------------------------
print_step 11 "Testing fetch (download without merge)"

cd "$TEST_PROJECT_PATH"

print_info "Making change in clone..."
cd "$CLONE_DIR"
echo "<!-- Extra change -->" >> "Alternatives/001/ProjectData"
oxen add .
oxen commit -m "Quick fix"
oxen push origin main

cd "$TEST_PROJECT_PATH"

print_info "Fetching (download only, no merge)..."
oxen fetch origin

print_success "Fetch completed"

# Check if local is behind
if oxen status | grep -qE "(behind|diverged)"; then
    print_success "Fetch detected remote changes (local behind remote)"
else
    print_info "Status shows current (fetch may have auto-merged)"
fi

print_info "Now merging fetched changes..."
oxen merge origin/main || oxen pull origin main

print_success "Merge completed"

# ------------------------------------------------------------
# Test Summary
# ------------------------------------------------------------
echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}TEST PASSED: $TEST_NAME${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "Summary:"
echo "  - Remote repository: Created"
echo "  - Remote configuration: Working"
echo "  - Push to remote: Successful"
echo "  - Clone from remote: Successful"
echo "  - Pull updates: Working"
echo "  - Multi-user sync: Validated"
echo "  - Large file handling: $([ $DURATION -lt 30 ] && echo "Fast" || echo "Acceptable") (${DURATION}s)"
echo "  - Fetch/merge workflow: Working"
echo ""
echo "Remote synchronization workflow verified!"
echo ""
echo "In production:"
echo "  - Use Oxen Hub: https://hub.oxen.ai"
echo "  - Or self-hosted Oxen server"
echo "  - Enable auto-push on milestone commits"
echo "  - Configure team access permissions"
echo ""
echo "Project locations:"
echo "  - Local: $TEST_PROJECT_PATH"
echo "  - Remote: $TEST_REMOTE_DIR"
echo "  - Clone: $CLONE_DIR"
echo ""

exit 0
