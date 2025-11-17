#!/bin/bash
#
# Master Test Runner
# Executes all User Guide scenario tests in sequence
#

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m'

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOG_DIR="$SCRIPT_DIR/test-logs"
LOG_FILE="$LOG_DIR/run_all_tests_$(date +%Y%m%d_%H%M%S).log"

# Test scripts to run
TEST_SCRIPTS=(
    "01_initialize_project.sh"
    "02_milestone_commit.sh"
    "03_rollback_workflow.sh"
    "04_browse_history.sh"
    "05_auto_commit_fsevents.sh"
    "06_lock_workflow.sh"
    "07_remote_sync.sh"
)

# Test metadata
TEST_NAMES=(
    "Initialize Project"
    "Milestone Commits"
    "Rollback Workflow"
    "Browse History"
    "Auto-Commit (FSEvents)"
    "Lock Management"
    "Remote Sync"
)

# Results tracking
PASSED_TESTS=()
FAILED_TESTS=()
SKIPPED_TESTS=()

# Functions
print_banner() {
    echo ""
    echo -e "${CYAN}╔════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║                                                        ║${NC}"
    echo -e "${CYAN}║        ${BLUE}OXEN-VCS USER GUIDE TEST SUITE${CYAN}             ║${NC}"
    echo -e "${CYAN}║                                                        ║${NC}"
    echo -e "${CYAN}╚════════════════════════════════════════════════════════╝${NC}"
    echo ""
}

print_test_header() {
    local num=$1
    local total=$2
    local name=$3
    echo ""
    echo -e "${BLUE}┌────────────────────────────────────────────────────────┐${NC}"
    echo -e "${BLUE}│ Test $num/$total: $name${NC}"
    echo -e "${BLUE}└────────────────────────────────────────────────────────┘${NC}"
}

print_result() {
    local status=$1
    local name=$2
    case $status in
        "PASSED")
            echo -e "${GREEN}✓ PASSED${NC}: $name"
            PASSED_TESTS+=("$name")
            ;;
        "FAILED")
            echo -e "${RED}✗ FAILED${NC}: $name"
            FAILED_TESTS+=("$name")
            ;;
        "SKIPPED")
            echo -e "${YELLOW}○ SKIPPED${NC}: $name"
            SKIPPED_TESTS+=("$name")
            ;;
    esac
}

print_summary() {
    local total=${#TEST_SCRIPTS[@]}
    local passed=${#PASSED_TESTS[@]}
    local failed=${#FAILED_TESTS[@]}
    local skipped=${#SKIPPED_TESTS[@]}

    echo ""
    echo -e "${CYAN}════════════════════════════════════════════════════════${NC}"
    echo -e "${CYAN}                    TEST SUMMARY                        ${NC}"
    echo -e "${CYAN}════════════════════════════════════════════════════════${NC}"
    echo ""
    echo -e "Total Tests:    $total"
    echo -e "${GREEN}Passed:         $passed${NC}"
    echo -e "${RED}Failed:         $failed${NC}"
    echo -e "${YELLOW}Skipped:        $skipped${NC}"
    echo ""

    if [ $passed -gt 0 ]; then
        echo -e "${GREEN}Passed Tests:${NC}"
        for test in "${PASSED_TESTS[@]}"; do
            echo "  ✓ $test"
        done
        echo ""
    fi

    if [ $failed -gt 0 ]; then
        echo -e "${RED}Failed Tests:${NC}"
        for test in "${FAILED_TESTS[@]}"; do
            echo "  ✗ $test"
        done
        echo ""
    fi

    if [ $skipped -gt 0 ]; then
        echo -e "${YELLOW}Skipped Tests:${NC}"
        for test in "${SKIPPED_TESTS[@]}"; do
            echo "  ○ $test"
        done
        echo ""
    fi

    echo -e "Log file: $LOG_FILE"
    echo ""

    if [ $failed -eq 0 ]; then
        echo -e "${GREEN}════════════════════════════════════════════════════════${NC}"
        echo -e "${GREEN}           ALL TESTS PASSED! ✓✓✓                       ${NC}"
        echo -e "${GREEN}════════════════════════════════════════════════════════${NC}"
        return 0
    else
        echo -e "${RED}════════════════════════════════════════════════════════${NC}"
        echo -e "${RED}           SOME TESTS FAILED                            ${NC}"
        echo -e "${RED}════════════════════════════════════════════════════════${NC}"
        return 1
    fi
}

check_prerequisites() {
    echo -e "${YELLOW}Checking prerequisites...${NC}"
    echo ""

    local missing=false

    # Check Oxen CLI
    if command -v oxen &> /dev/null; then
        OXEN_VERSION=$(oxen --version 2>&1 | head -1)
        echo -e "${GREEN}✓${NC} Oxen CLI: $OXEN_VERSION"
    else
        echo -e "${RED}✗${NC} Oxen CLI not found (install: pip3 install oxen-ai)"
        missing=true
    fi

    # Check auxin
    if [ -f "./Auxin-CLI-Wrapper/target/release/auxin" ]; then
        echo -e "${GREEN}✓${NC} auxin built"
    else
        echo -e "${RED}✗${NC} auxin not built (run: cd Auxin-CLI-Wrapper && cargo build --release)"
        missing=true
    fi

    # Check LaunchAgent (optional)
    if launchctl list | grep -q "oxenvcs"; then
        DAEMON_PID=$(launchctl list | grep oxenvcs | awk '{print $1}')
        echo -e "${GREEN}✓${NC} LaunchAgent running (PID: $DAEMON_PID)"
    else
        echo -e "${YELLOW}○${NC} LaunchAgent not running (optional, needed for test 5)"
    fi

    echo ""

    if [ "$missing" = true ]; then
        echo -e "${RED}Missing required prerequisites. Please install them first.${NC}"
        exit 1
    fi

    echo -e "${GREEN}Prerequisites check passed!${NC}"
    echo ""
}

run_test() {
    local script=$1
    local name=$2
    local num=$3
    local total=$4

    print_test_header $num $total "$name"

    # Check if script exists
    if [ ! -f "$SCRIPT_DIR/$script" ]; then
        echo -e "${RED}Script not found: $script${NC}"
        print_result "FAILED" "$name"
        return 1
    fi

    # Make executable
    chmod +x "$SCRIPT_DIR/$script"

    # Run test
    echo "Running: $script"
    echo ""

    # Capture output to log
    if "$SCRIPT_DIR/$script" 2>&1 | tee -a "$LOG_FILE"; then
        print_result "PASSED" "$name"
        return 0
    else
        print_result "FAILED" "$name"
        return 1
    fi
}

# ============================================================
# MAIN EXECUTION
# ============================================================

print_banner

# Setup logging
mkdir -p "$LOG_DIR"
echo "Test run started at $(date)" > "$LOG_FILE"
echo "=====================================" >> "$LOG_FILE"

# Check prerequisites
check_prerequisites

# Confirmation prompt
echo -e "${YELLOW}This will run ${#TEST_SCRIPTS[@]} test scripts.${NC}"
echo -e "${YELLOW}Each test creates temporary projects on your Desktop.${NC}"
echo ""
echo "Test duration: ~10-15 minutes total"
echo ""
read -p "Continue? (y/n) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Test run cancelled."
    exit 0
fi

# Run all tests
TOTAL_TESTS=${#TEST_SCRIPTS[@]}
for i in "${!TEST_SCRIPTS[@]}"; do
    TEST_NUM=$((i + 1))
    TEST_SCRIPT="${TEST_SCRIPTS[$i]}"
    TEST_NAME="${TEST_NAMES[$i]}"

    # Run test (continue even if it fails)
    run_test "$TEST_SCRIPT" "$TEST_NAME" $TEST_NUM $TOTAL_TESTS || true

    # Pause between tests (optional)
    if [ $TEST_NUM -lt $TOTAL_TESTS ]; then
        echo ""
        echo -e "${YELLOW}Press Enter to continue to next test (or Ctrl+C to stop)...${NC}"
        read
    fi
done

# Print summary
print_summary

# Exit with appropriate code
if [ ${#FAILED_TESTS[@]} -eq 0 ]; then
    exit 0
else
    exit 1
fi
