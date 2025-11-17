#!/bin/bash
#
# Test Script: Initialize Your First Project
# Based on: USER_GUIDE.md - "Initializing Your First Project"
#
# This script tests the project initialization workflow from the User Guide.
#

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test configuration
TEST_NAME="Initialize Project"
TEST_PROJECT_NAME="TestProject_$(date +%s).logicx"
TEST_DIR="$HOME/Desktop/oxenvcs-test-projects"
TEST_PROJECT_PATH="$TEST_DIR/$TEST_PROJECT_NAME"

# CLI path
OXENVCS_CLI="./Auxin-CLI-Wrapper/target/release/auxin"

# Function to print test header
print_header() {
    echo ""
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}TEST: $TEST_NAME${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo ""
}

# Function to print step
print_step() {
    echo -e "${YELLOW}STEP $1:${NC} $2"
}

# Function to print success
print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

# Function to print error and exit
print_error() {
    echo -e "${RED}✗ ERROR: $1${NC}"
    exit 1
}

# Function to verify prerequisite
verify_prerequisite() {
    local cmd=$1
    local name=$2
    if ! command -v "$cmd" &> /dev/null; then
        print_error "$name is not installed. Please install it first."
    fi
    print_success "$name is available"
}

# Function to cleanup
cleanup() {
    if [ -d "$TEST_PROJECT_PATH" ]; then
        echo ""
        read -p "Delete test project? (y/n) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            rm -rf "$TEST_PROJECT_PATH"
            print_success "Test project deleted"
        else
            echo "Test project kept at: $TEST_PROJECT_PATH"
        fi
    fi
}

# Trap to cleanup on exit
trap cleanup EXIT

# ============================================================
# TEST EXECUTION
# ============================================================

print_header

# ------------------------------------------------------------
# Prerequisites Check
# ------------------------------------------------------------
print_step 1 "Checking prerequisites"

verify_prerequisite "oxen" "Oxen CLI"

if [ ! -f "$OXENVCS_CLI" ]; then
    print_error "auxin not found. Build it first: cd Auxin-CLI-Wrapper && cargo build --release"
fi
print_success "auxin is available"

# ------------------------------------------------------------
# Step 2: Create Test Logic Pro Project Structure
# ------------------------------------------------------------
print_step 2 "Creating test Logic Pro project"

mkdir -p "$TEST_DIR"
mkdir -p "$TEST_PROJECT_PATH"
mkdir -p "$TEST_PROJECT_PATH/Alternatives/001"
mkdir -p "$TEST_PROJECT_PATH/Resources/Audio Files"
mkdir -p "$TEST_PROJECT_PATH/Media"

# Create dummy projectData file (simulated binary)
echo "<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<project>
    <tempo>120</tempo>
    <sampleRate>48000</sampleRate>
    <keySignature>C Major</keySignature>
    <tracks>
        <track id=\"1\" name=\"Audio 1\" type=\"audio\"/>
        <track id=\"2\" name=\"Software Instrument 1\" type=\"midi\"/>
    </tracks>
</project>" > "$TEST_PROJECT_PATH/Alternatives/001/ProjectData"

# Create some dummy audio files
dd if=/dev/zero of="$TEST_PROJECT_PATH/Resources/Audio Files/Audio_1.wav" bs=1024 count=1024 2>/dev/null
dd if=/dev/zero of="$TEST_PROJECT_PATH/Resources/Audio Files/Audio_2.wav" bs=1024 count=512 2>/dev/null

print_success "Test project created at: $TEST_PROJECT_PATH"
echo "  - ProjectData: $(ls -lh "$TEST_PROJECT_PATH/Alternatives/001/ProjectData" | awk '{print $5}')"
echo "  - Audio files: 2 files, $(du -sh "$TEST_PROJECT_PATH/Resources/Audio Files" | awk '{print $1}')"

# ------------------------------------------------------------
# Step 3: Initialize with Oxen-VCS
# ------------------------------------------------------------
print_step 3 "Initializing project with auxin"

cd "$TEST_PROJECT_PATH"
$OXENVCS_CLI init --logic .

# Verify .oxen directory created
if [ ! -d ".oxen" ]; then
    print_error ".oxen directory not created"
fi
print_success ".oxen directory created"

# ------------------------------------------------------------
# Step 4: Verify .oxenignore File
# ------------------------------------------------------------
print_step 4 "Verifying .oxenignore file"

if [ ! -f ".oxenignore" ]; then
    print_error ".oxenignore file not created"
fi
print_success ".oxenignore file created"

# Check for expected patterns
expected_patterns=("Bounces/" "Freeze Files/" "Autosave/" ".DS_Store")
for pattern in "${expected_patterns[@]}"; do
    if ! grep -q "$pattern" .oxenignore; then
        print_error ".oxenignore missing pattern: $pattern"
    fi
done
print_success "All expected patterns in .oxenignore"

echo ""
echo ".oxenignore contents:"
cat .oxenignore | head -20

# ------------------------------------------------------------
# Step 5: Verify Initial Commit
# ------------------------------------------------------------
print_step 5 "Verifying initial commit"

# Check oxen status
OXEN_STATUS=$(oxen status 2>&1 || echo "ERROR")
if [[ "$OXEN_STATUS" == *"ERROR"* ]]; then
    print_error "oxen status failed"
fi
print_success "oxen status works"

# Check for clean working tree
if echo "$OXEN_STATUS" | grep -q "nothing to commit"; then
    print_success "Working tree is clean (initial commit exists)"
else
    echo "$OXEN_STATUS"
    print_error "Working tree not clean - initial commit may not have been created"
fi

# Verify oxen log shows commits
COMMIT_COUNT=$(oxen log --oneline 2>/dev/null | wc -l | tr -d ' ')
if [ "$COMMIT_COUNT" -gt 0 ]; then
    print_success "Initial commit created ($COMMIT_COUNT commit(s) found)"
    echo ""
    echo "Commit history:"
    oxen log --oneline | head -5
else
    print_error "No commits found in history"
fi

# ------------------------------------------------------------
# Step 6: Verify Tracked Files
# ------------------------------------------------------------
print_step 6 "Verifying tracked files"

# List tracked files
echo ""
echo "Tracked files:"
oxen ls-files | head -20

TRACKED_COUNT=$(oxen ls-files | wc -l | tr -d ' ')
print_success "$TRACKED_COUNT files tracked"

# Verify ProjectData is tracked
if oxen ls-files | grep -q "ProjectData"; then
    print_success "ProjectData is tracked"
else
    print_error "ProjectData not tracked"
fi

# Verify audio files are tracked
if oxen ls-files | grep -q "Audio Files"; then
    print_success "Audio files are tracked"
else
    print_error "Audio files not tracked"
fi

# ------------------------------------------------------------
# Step 7: Verify Bounces/Freeze Files are Ignored
# ------------------------------------------------------------
print_step 7 "Verifying generated files are ignored"

# Create Bounces and Freeze Files directories
mkdir -p Bounces
mkdir -p "Freeze Files"

# Add dummy files
echo "bounce" > Bounces/bounce1.wav
echo "freeze" > "Freeze Files/freeze1.wav"

# Check that they don't appear in status
UNTRACKED=$(oxen status 2>&1)
if echo "$UNTRACKED" | grep -q "Bounces"; then
    print_error "Bounces/ directory not being ignored"
fi
if echo "$UNTRACKED" | grep -q "Freeze Files"; then
    print_error "Freeze Files/ directory not being ignored"
fi
print_success "Bounces/ and Freeze Files/ are properly ignored"

# ------------------------------------------------------------
# Test Summary
# ------------------------------------------------------------
echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}TEST PASSED: $TEST_NAME${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "Summary:"
echo "  - Project initialized: $TEST_PROJECT_PATH"
echo "  - .oxen directory: created"
echo "  - .oxenignore: configured correctly"
echo "  - Initial commit: created"
echo "  - Tracked files: $TRACKED_COUNT"
echo "  - Ignored files: working correctly"
echo ""
echo "Project is ready for use!"
echo ""

# Success
exit 0
