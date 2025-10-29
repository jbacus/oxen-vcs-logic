# macOS Testing Tasks - Complete Validation Checklist

**Purpose**: Validate all 540+ tests on macOS and ensure production readiness
**Estimated Time**: 4-6 hours for complete validation
**Prerequisites**: macOS 14.0+, Xcode 15+, Rust toolchain

---

## 📋 Quick Checklist

- [ ] Environment setup (30 min)
- [ ] Rust unit tests (15 min)
- [ ] Rust integration tests (30 min)
- [ ] Swift compilation verification (15 min)
- [ ] Swift unit tests (30 min)
- [ ] Integration testing with real Logic Pro project (1-2 hours)
- [ ] Performance validation (30 min)
- [ ] Error scenario testing (30 min)
- [ ] Documentation review (15 min)

**Total**: 4-6 hours

---

## Phase 1: Environment Setup (30 minutes)

### 1.1 Install Prerequisites

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Verify Rust installation
rustc --version
cargo --version

# Install oxen CLI
pip3 install oxen-ai

# Verify oxen installation
oxen --version

# Install Xcode Command Line Tools (if not already)
xcode-select --install

# Verify Swift
swift --version
```

**Expected Output:**
- Rust: `rustc 1.70+`
- Cargo: `cargo 1.70+`
- Oxen: `oxen 0.x.x`
- Swift: `Swift version 5.9+`

### 1.2 Clone and Navigate

```bash
cd ~/path/to/oxen-vcs-logic
git checkout claude/session-011CUa3sb9HKKJzkJ1nyr5ax
git pull origin claude/session-011CUa3sb9HKKJzkJ1nyr5ax
```

**Checkpoint**: ✅ All tools installed and working

---

## Phase 2: Rust Unit Tests (15 minutes)

### 2.1 Build Rust CLI Wrapper

```bash
cd OxVCS-CLI-Wrapper
cargo build --release
```

**Expected**: Clean build with no errors

### 2.2 Run All Unit Tests

```bash
cargo test --lib
```

**Expected Output:**
```
running 253 tests
test logic_project::tests::test_detect_valid_project ... ok
test commit_metadata::tests::test_builder_pattern ... ok
test ignore_template::tests::test_generate_ignore ... ok
test draft_manager::tests::test_draft_stats ... ok
test logger::tests::test_verbose_flag ... ok
test oxen_ops::tests::test_new_with_absolute_path ... ok
test oxen_subprocess::tests::test_parse_commit_id ... ok
...

test result: ok. 253 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

**Troubleshooting:**
- If any test fails, note the failure and continue
- Check if it's macOS-specific (path separators, etc.)
- Document failures in a `TEST_RESULTS.md` file

### 2.3 Run Tests with Output

```bash
cargo test --lib -- --nocapture --test-threads=1
```

**Purpose**: See detailed output, run tests sequentially to avoid conflicts

**Checkpoint**: ✅ 253 unit tests pass (or document failures)

---

## Phase 3: Rust Integration Tests (30 minutes)

### 3.1 Verify Oxen CLI is Available

```bash
which oxen
oxen --version
```

**Expected**: Path to oxen binary and version number

### 3.2 Run Integration Tests

```bash
cargo test --test oxen_subprocess_integration_test -- --nocapture
```

**Expected Output:**
```
running 40+ tests
test tests::test_init_creates_oxen_directory ... ok
test tests::test_add_single_file ... ok
test tests::test_commit_after_add ... ok
test tests::test_log_after_commits ... ok
test tests::test_status_with_untracked_files ... ok
test tests::test_checkout_previous_commit ... ok
test tests::test_complete_workflow ... ok
...

test result: ok. 40+ passed; 0 failed; 0 ignored; 0 measured
```

### 3.3 Test Individual Workflows

```bash
# Test init workflow
cargo test --test oxen_subprocess_integration_test test_init -- --nocapture

# Test commit workflow
cargo test --test oxen_subprocess_integration_test test_commit -- --nocapture

# Test large file handling
cargo test --test oxen_subprocess_integration_test test_large_file -- --nocapture
```

**Checkpoint**: ✅ All integration tests pass with real oxen CLI

---

## Phase 4: Swift Compilation (15 minutes)

### 4.1 Build LaunchAgent

```bash
cd ../OxVCS-LaunchAgent

# Build in debug mode
swift build

# Build in release mode
swift build -c release
```

**Expected**: Clean build with no compilation errors

### 4.2 Check for Warnings

```bash
swift build 2>&1 | grep -i warning
```

**Expected**: Minimal or no warnings (document any warnings found)

**Checkpoint**: ✅ Swift code compiles successfully

---

## Phase 5: Swift Unit Tests (30 minutes)

### 5.1 Run All Swift Tests

```bash
swift test
```

**Expected Output:**
```
Test Suite 'All tests' started
Test Suite 'FSEventsMonitorTests' started
Test Case 'FSEventsMonitorTests.testInitWithDefaultDebounce' passed
Test Case 'FSEventsMonitorTests.testStartMonitoring' passed
...
Test Suite 'PowerManagementTests' started
Test Case 'PowerManagementTests.testStartMonitoring' passed
...
Test Suite 'CommitOrchestratorTests' started
Test Case 'CommitOrchestratorTests.testRegisterProject' passed
...
Test Suite 'LockManagerTests' started
Test Case 'LockManagerTests.testAcquireLock_Success' passed
...
Test Suite 'XPCServiceTests' started
Test Case 'XPCServiceTests.testInitializeProject' passed
...

Executed 220+ tests, with 0 failures
```

### 5.2 Run Individual Test Suites

```bash
# FSEventsMonitor tests (40+ tests)
swift test --filter FSEventsMonitorTests

# PowerManagement tests (40+ tests)
swift test --filter PowerManagementTests

# CommitOrchestrator tests (50+ tests)
swift test --filter CommitOrchestratorTests

# LockManager tests (50+ tests)
swift test --filter LockManagerTests

# XPC Service tests (40+ tests)
swift test --filter XPCServiceTests
```

### 5.3 Run Tests with Verbose Output

```bash
swift test --verbose
```

**Checkpoint**: ✅ All 220+ Swift tests pass (or document failures)

---

## Phase 6: Integration Testing with Real Logic Pro Project (1-2 hours)

### 6.1 Prepare Test Project

**Option A: Use Existing Logic Pro Project**
```bash
# Find a Logic Pro project
ls ~/Music/Logic/*.logicx
```

**Option B: Create Test Project**
1. Open Logic Pro
2. Create new project: "OxenVCS Test Project"
3. Add a few tracks (audio, MIDI, etc.)
4. Save as: `~/Music/Logic/OxenVCS_Test.logicx`

### 6.2 Test Scenario 1: Initialize Repository

```bash
cd ~/Music/Logic/OxenVCS_Test.logicx

# Initialize with Rust CLI
/path/to/oxen-vcs-logic/OxVCS-CLI-Wrapper/target/release/oxenvcs-cli init --logic .

# Verify .oxen directory created
ls -la .oxen/

# Verify .oxenignore created
cat .oxenignore
```

**Expected**:
- `.oxen/` directory exists
- `.oxenignore` contains Logic-specific patterns (Bounces/, Freeze Files/, etc.)

### 6.3 Test Scenario 2: Make Changes and Commit

```bash
# Make a change in Logic Pro
# 1. Open the project in Logic Pro
# 2. Add a new audio track
# 3. Save the project
# 4. Close Logic Pro

# Check status
oxenvcs-cli status

# Stage changes
oxenvcs-cli add --all

# Commit with metadata
oxenvcs-cli commit -m "Added new audio track" --bpm 120 --sample-rate 48000
```

**Expected**:
- Status shows modified files
- Add succeeds
- Commit creates new commit with metadata

### 6.4 Test Scenario 3: View History

```bash
oxenvcs-cli log --limit 5
```

**Expected**: Shows commits with metadata in formatted output

### 6.5 Test Scenario 4: Lock Management

```bash
# Test from Swift if possible, or document for future testing
# LockManager.shared.acquireLock(projectPath: "path")
```

**Note**: Full lock testing requires the daemon running

### 6.6 Test Scenario 5: Large File Handling

```bash
# Add a large audio file (> 100MB) to the project
# Commit and verify it's tracked properly

oxenvcs-cli add --all
oxenvcs-cli commit -m "Added large audio file"

# Check repository size
du -sh .oxen/
```

**Checkpoint**: ✅ Real Logic Pro project works with all operations

---

## Phase 7: Performance Validation (30 minutes)

### 7.1 Test Large Project

Create or use a Logic Pro project with:
- 10+ audio tracks
- 20+ audio files (totaling > 500MB)
- Multiple alternatives

```bash
time oxenvcs-cli add --all
time oxenvcs-cli commit -m "Performance test"
```

**Expected**:
- Add operation: < 5 seconds for 500MB
- Commit operation: < 10 seconds

### 7.2 Test Rapid Commits

```bash
#!/bin/bash
for i in {1..10}; do
    echo "Change $i" >> test_file.txt
    oxenvcs-cli add test_file.txt
    oxenvcs-cli commit -m "Commit $i"
done
```

**Expected**: Each commit < 2 seconds

### 7.3 Test History Retrieval

```bash
time oxenvcs-cli log --limit 100
```

**Expected**: < 1 second for 100 commits

**Checkpoint**: ✅ Performance meets targets

---

## Phase 8: Error Scenario Testing (30 minutes)

### 8.1 Test Invalid Inputs

```bash
# Test with non-existent path
oxenvcs-cli init --logic /nonexistent/path
# Expected: Error message

# Test commit without staging
oxenvcs-cli commit -m "Test"
# Expected: Error or "no changes"

# Test with invalid commit ID
oxenvcs-cli restore invalid_hash
# Expected: Error message
```

### 8.2 Test Lock Conflicts

If you can run two instances:
```bash
# Terminal 1
swift test --filter LockManagerTests.testAcquireLock

# Terminal 2 (while Terminal 1 lock is active)
swift test --filter LockManagerTests.testAcquireLock_AlreadyLocked
```

### 8.3 Test File System Edge Cases

```bash
# Project with spaces in name
oxenvcs-cli init --logic "/path/with spaces/Project.logicx"

# Unicode in path
oxenvcs-cli init --logic "/path/プロジェクト.logicx"

# Very long path
oxenvcs-cli init --logic "$(printf 'a%.0s' {1..200}).logicx"
```

**Checkpoint**: ✅ Error handling works correctly

---

## Phase 9: Documentation Review (15 minutes)

### 9.1 Verify Documentation Accuracy

Review these files for accuracy:
- [ ] `docs/MACOS_SETUP.md` - Instructions work as written
- [ ] `docs/INTEGRATION_GUIDE.md` - Integration steps are clear
- [ ] `docs/FIRST_TEST_GUIDE.md` - Scenarios work correctly
- [ ] `TEST_COVERAGE_REPORT.md` - Statistics match reality
- [ ] `CLAUDE.md` - Status assessment is accurate

### 9.2 Update Documentation

If you find issues:
```bash
# Edit the file
vim docs/MACOS_SETUP.md

# Commit the fix
git add docs/MACOS_SETUP.md
git commit -m "docs: Fix macOS setup instructions based on testing"
git push
```

**Checkpoint**: ✅ Documentation is accurate

---

## Final Results Template

Create a file `MACOS_TEST_RESULTS.md`:

```markdown
# macOS Testing Results

**Date**: [Date]
**macOS Version**: [e.g., macOS 14.1]
**Hardware**: [e.g., M1 MacBook Pro]
**Tester**: [Your name]

## Summary

- Total Tests Run: [number]
- Tests Passed: [number]
- Tests Failed: [number]
- Tests Skipped: [number]

## Rust Tests

### Unit Tests (253 expected)
- ✅ Passed: [number]
- ❌ Failed: [number]
- Details: [any failures]

### Integration Tests (40+ expected)
- ✅ Passed: [number]
- ❌ Failed: [number]
- Details: [any failures]

## Swift Tests

### FSEventsMonitor (40+ expected)
- ✅ Passed: [number]
- ❌ Failed: [number]
- Details: [any failures]

### PowerManagement (40+ expected)
- ✅ Passed: [number]
- ❌ Failed: [number]
- Details: [any failures]

### CommitOrchestrator (50+ expected)
- ✅ Passed: [number]
- ❌ Failed: [number]
- Details: [any failures]

### LockManager (50+ expected)
- ✅ Passed: [number]
- ❌ Failed: [number]
- Details: [any failures]

### XPCService (40+ expected)
- ✅ Passed: [number]
- ❌ Failed: [number]
- Details: [any failures]

## Real-World Testing

### Logic Pro Integration
- ✅ Initialize repository: [pass/fail]
- ✅ Commit changes: [pass/fail]
- ✅ View history: [pass/fail]
- ✅ Large files (>100MB): [pass/fail]

### Performance
- Add operation (500MB): [time]
- Commit operation: [time]
- Log retrieval (100 commits): [time]

## Issues Found

1. [Issue description]
   - Severity: [Critical/High/Medium/Low]
   - Steps to reproduce: [...]
   - Expected vs Actual: [...]

2. [Issue description]
   ...

## Recommendations

- [ ] Ready for merge: [Yes/No/With conditions]
- [ ] Additional testing needed: [describe]
- [ ] Documentation updates needed: [list]
- [ ] Code fixes needed: [list]

## Notes

[Any additional observations, suggestions, or context]
```

---

## Success Criteria

### Minimum Requirements (Must Pass)
- ✅ 95%+ of Rust unit tests pass (240+ out of 253)
- ✅ 90%+ of integration tests pass (36+ out of 40)
- ✅ 95%+ of Swift tests pass (210+ out of 220)
- ✅ Real Logic Pro project initialization works
- ✅ Commit workflow works end-to-end
- ✅ No critical bugs found

### Production Ready (Ideal)
- ✅ 100% of all tests pass
- ✅ Performance targets met
- ✅ Error handling works correctly
- ✅ Documentation is accurate
- ✅ No bugs found

---

## Troubleshooting Guide

### Rust Tests Fail on macOS

**Problem**: Path separator issues
```bash
# macOS uses / but test might expect \
```
**Solution**: Check test assertions, ensure they work with Unix paths

**Problem**: Permission errors
```bash
# Error: Permission denied
```
**Solution**: Run with proper permissions
```bash
sudo cargo test  # Only if necessary
```

### Swift Tests Fail

**Problem**: XCTest framework not found
```bash
# Error: XCTest not available
```
**Solution**: Install Xcode Command Line Tools
```bash
xcode-select --install
```

**Problem**: Tests timeout
```bash
# Error: Test timed out
```
**Solution**: Increase timeout or run tests individually
```bash
swift test --filter TestName
```

### Integration Tests Fail

**Problem**: Oxen CLI not found
```bash
# Error: oxen command not found
```
**Solution**: Verify oxen installation
```bash
pip3 install oxen-ai
export PATH="$HOME/.local/bin:$PATH"
```

**Problem**: Tests auto-skip
```bash
# All tests skipped
```
**Solution**: Tests are designed to skip if oxen not available - this is OK

### Performance Issues

**Problem**: Operations very slow
```bash
# Commit takes > 30 seconds
```
**Solution**:
- Check available disk space
- Check if running on battery power
- Try with smaller test project first

---

## Quick Reference Commands

```bash
# Run all Rust tests
cd OxVCS-CLI-Wrapper && cargo test

# Run all Swift tests
cd OxVCS-LaunchAgent && swift test

# Run specific test suite
cargo test --test oxen_subprocess_integration_test
swift test --filter FSEventsMonitorTests

# Build release binaries
cargo build --release
swift build -c release

# Clean and rebuild
cargo clean && cargo build
swift package clean && swift build

# View test output
cargo test -- --nocapture
swift test --verbose
```

---

## Estimated Timeline

| Phase | Time | Cumulative |
|-------|------|------------|
| Setup | 30 min | 30 min |
| Rust Unit Tests | 15 min | 45 min |
| Rust Integration Tests | 30 min | 1h 15min |
| Swift Compilation | 15 min | 1h 30min |
| Swift Unit Tests | 30 min | 2h |
| Real Logic Pro Testing | 1-2h | 3-4h |
| Performance Testing | 30 min | 3.5-4.5h |
| Error Scenarios | 30 min | 4-5h |
| Documentation | 15 min | 4.25-5.25h |

**Total: 4-6 hours** (depending on issues found)

---

## Next Steps After Testing

1. **Document Results**: Fill out `MACOS_TEST_RESULTS.md`
2. **Fix Issues**: Create issues for any failures found
3. **Update PR**: Add test results to PR description
4. **Merge**: If tests pass, merge the PR
5. **Deploy**: Follow deployment guide (to be created)

---

**Questions?** Check:
- `docs/MACOS_SETUP.md` for setup details
- `docs/INTEGRATION_GUIDE.md` for integration help
- `TEST_COVERAGE_REPORT.md` for test details
- GitHub Issues for known problems

Good luck with testing! 🚀
