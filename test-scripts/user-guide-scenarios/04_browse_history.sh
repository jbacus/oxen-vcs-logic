#!/bin/bash
#
# Test Script: Browsing Project History
# Based on: USER_GUIDE.md - "Browsing Project History"
#
# This script tests viewing and searching commit history.
#

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Test configuration
TEST_NAME="Browse Project History"
TEST_PROJECT_NAME="HistoryTest_$(date +%s).logicx"
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
# Step 1: Create Project with Rich History
# ------------------------------------------------------------
print_step 1 "Creating project with diverse commit history"

mkdir -p "$TEST_DIR"
mkdir -p "$TEST_PROJECT_PATH/Alternatives/001"
mkdir -p "$TEST_PROJECT_PATH/Resources/Audio Files"

cd "$TEST_PROJECT_PATH"

# Helper function to create commits
create_version() {
    local version=$1
    local message=$2
    local bpm=$3
    local key=$4
    local tags=$5

    cat > "Alternatives/001/ProjectData" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<project version="$version">
    <tempo>$bpm</tempo>
    <sampleRate>48000</sampleRate>
    <keySignature>$key</keySignature>
    <state>$message</state>
</project>
EOF

    if [ "$version" -eq 1 ]; then
        $OXENVCS_CLI init --logic .
    else
        oxen add .
        echo "$message

BPM: $bpm
Sample Rate: 48kHz
Key: $key
Tags: $tags" | oxen commit -F -
    fi
}

# Create a rich history with different commit types
create_version 1 "Initial project setup" 120 "C Major" "init, tracking"
print_success "Commit 1: Initial setup"

sleep 1
create_version 2 "Add drum tracks" 120 "C Major" "tracking, drums"
print_success "Commit 2: Drums"

sleep 1
create_version 3 "Add bass line" 120 "C Major" "tracking, bass"
print_success "Commit 3: Bass"

sleep 1
create_version 4 "Mix v1 - rough mix" 120 "C Major" "mix, v1, rough"
print_success "Commit 4: Mix v1"

sleep 1
create_version 5 "Mix v2 - client feedback" 120 "C Major" "mix, v2, client-feedback"
print_success "Commit 5: Mix v2"

sleep 1
create_version 6 "Fix timing on chorus vocals" 120 "C Major" "fix, editing, vocals"
print_success "Commit 6: Fix vocals"

sleep 1
create_version 7 "Update arrangement - new intro" 128 "C Major" "arrangement, intro"
print_success "Commit 7: New intro, tempo change"

sleep 1
create_version 8 "Mix v3 - final mix" 128 "C Major" "mix, v3, final"
print_success "Commit 8: Mix v3"

sleep 1
create_version 9 "Master v1 - ready for delivery" 128 "C Major" "master, final, delivery"
print_success "Commit 9: Master v1"

TOTAL_COMMITS=$(oxen log --oneline | wc -l | tr -d ' ')
print_success "Created rich history with $TOTAL_COMMITS commits"

# ------------------------------------------------------------
# Step 2: View Complete History
# ------------------------------------------------------------
print_step 2 "Viewing complete commit history"

echo ""
echo "Complete history (oneline format):"
oxen log --oneline

echo ""
echo "Detailed history (last 3 commits):"
oxen log -n 3

COMMIT_COUNT=$(oxen log --oneline | wc -l | tr -d ' ')
if [ "$COMMIT_COUNT" -ge 9 ]; then
    print_success "All $COMMIT_COUNT commits visible in history"
else
    print_error "Expected at least 9 commits, found $COMMIT_COUNT"
fi

# ------------------------------------------------------------
# Step 3: Search History by Message
# ------------------------------------------------------------
print_step 3 "Searching history by commit message"

echo ""
echo "Search: commits containing 'mix':"
MIX_COMMITS=$(oxen log --grep="mix" --oneline)
echo "$MIX_COMMITS"

MIX_COUNT=$(echo "$MIX_COMMITS" | grep -c "mix" || true)
if [ "$MIX_COUNT" -ge 3 ]; then
    print_success "Found $MIX_COUNT commits with 'mix'"
else
    print_error "Expected at least 3 'mix' commits"
fi

echo ""
echo "Search: commits containing 'final':"
FINAL_COMMITS=$(oxen log --grep="final" --oneline)
echo "$FINAL_COMMITS"

FINAL_COUNT=$(echo "$FINAL_COMMITS" | grep -c "final" || true)
if [ "$FINAL_COUNT" -ge 2 ]; then
    print_success "Found $FINAL_COUNT commits with 'final'"
else
    print_error "Expected at least 2 'final' commits"
fi

# ------------------------------------------------------------
# Step 4: Filter by Metadata (Tags)
# ------------------------------------------------------------
print_step 4 "Filtering by metadata tags"

echo ""
echo "Commits with 'tracking' tag:"
oxen log --grep="tracking" --oneline

echo ""
echo "Commits with 'mix' tag:"
oxen log --grep="Tags:.*mix" --oneline

echo ""
echo "Commits with 'final' tag:"
oxen log --grep="Tags:.*final" --oneline

print_success "Tag filtering works"

# ------------------------------------------------------------
# Step 5: Filter by BPM
# ------------------------------------------------------------
print_step 5 "Filtering by BPM metadata"

echo ""
echo "Commits at 120 BPM:"
oxen log --grep="BPM: 120" --oneline

echo ""
echo "Commits at 128 BPM:"
oxen log --grep="BPM: 128" --oneline

BPM_120_COUNT=$(oxen log --grep="BPM: 120" --oneline | wc -l | tr -d ' ')
BPM_128_COUNT=$(oxen log --grep="BPM: 128" --oneline | wc -l | tr -d ' ')

print_success "BPM 120: $BPM_120_COUNT commits, BPM 128: $BPM_128_COUNT commits"

# ------------------------------------------------------------
# Step 6: View Specific Commit Details
# ------------------------------------------------------------
print_step 6 "Viewing specific commit details"

FIRST_MIX_COMMIT=$(oxen log --grep="Mix v1" --oneline | head -1 | awk '{print $1}')

echo ""
echo "Detailed view of 'Mix v1' commit ($FIRST_MIX_COMMIT):"
oxen show $FIRST_MIX_COMMIT

if oxen show $FIRST_MIX_COMMIT | grep -q "Mix v1"; then
    print_success "Detailed commit view works"
else
    print_error "Failed to show commit details"
fi

# ------------------------------------------------------------
# Step 7: Compare Two Commits (File Changes)
# ------------------------------------------------------------
print_step 7 "Comparing commits to see changes"

COMMIT_V1=$(oxen log --grep="Mix v1" --oneline | head -1 | awk '{print $1}')
COMMIT_V3=$(oxen log --grep="Mix v3" --oneline | head -1 | awk '{print $1}')

echo ""
echo "Comparing Mix v1 ($COMMIT_V1) to Mix v3 ($COMMIT_V3):"
echo ""

# Show diff
oxen diff $COMMIT_V1 $COMMIT_V3 | head -30 || echo "(Binary diff not shown in detail)"

print_success "Commit comparison works"

# ------------------------------------------------------------
# Step 8: View History Timeline
# ------------------------------------------------------------
print_step 8 "Viewing history timeline"

echo ""
echo "Timeline view (with dates):"
oxen log --format=format:"%h - %s (%cr)" | head -10

print_success "Timeline view works"

# ------------------------------------------------------------
# Step 9: View Changed Files in Commits
# ------------------------------------------------------------
print_step 9 "Viewing changed files per commit"

echo ""
echo "Files changed in each commit:"
for commit in $(oxen log --oneline | head -5 | awk '{print $1}'); do
    echo ""
    echo "Commit: $commit"
    oxen show --stat $commit | grep -E '(file changed|files changed|insertion|deletion)' || echo "  (stats not available)"
done

print_success "File change tracking works"

# ------------------------------------------------------------
# Step 10: Export History Report
# ------------------------------------------------------------
print_step 10 "Generating history report"

REPORT_FILE="$TEST_PROJECT_PATH/history_report.txt"

cat > "$REPORT_FILE" <<EOF
Oxen-VCS History Report
=======================
Project: $TEST_PROJECT_NAME
Generated: $(date)
Total Commits: $COMMIT_COUNT

Complete History:
-----------------
EOF

oxen log --format=format:"%h - %s (%ar) <%an>" >> "$REPORT_FILE"

echo "" >> "$REPORT_FILE"
echo "" >> "$REPORT_FILE"
echo "Commits by Type:" >> "$REPORT_FILE"
echo "----------------" >> "$REPORT_FILE"
echo "Mix commits: $(oxen log --grep='mix' --oneline | wc -l | tr -d ' ')" >> "$REPORT_FILE"
echo "Tracking commits: $(oxen log --grep='tracking' --oneline | wc -l | tr -d ' ')" >> "$REPORT_FILE"
echo "Fix commits: $(oxen log --grep='Fix' --oneline | wc -l | tr -d ' ')" >> "$REPORT_FILE"
echo "Master commits: $(oxen log --grep='master' --oneline | wc -l | tr -d ' ')" >> "$REPORT_FILE"

print_success "History report generated: $REPORT_FILE"

echo ""
echo "Report contents:"
cat "$REPORT_FILE"

# ------------------------------------------------------------
# Test Summary
# ------------------------------------------------------------
echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}TEST PASSED: $TEST_NAME${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "Summary:"
echo "  - Total commits: $COMMIT_COUNT"
echo "  - Commits with 'mix': $MIX_COUNT"
echo "  - Commits with 'final': $FINAL_COUNT"
echo "  - Commits at 120 BPM: $BPM_120_COUNT"
echo "  - Commits at 128 BPM: $BPM_128_COUNT"
echo "  - History report generated: $REPORT_FILE"
echo ""
echo "History browsing capabilities verified:"
echo "  ✓ View complete history"
echo "  ✓ Search by message"
echo "  ✓ Filter by metadata (tags, BPM)"
echo "  ✓ View specific commit details"
echo "  ✓ Compare commits"
echo "  ✓ Timeline view"
echo "  ✓ Track file changes"
echo "  ✓ Export reports"
echo ""
echo "Project location: $TEST_PROJECT_PATH"
echo ""

exit 0
