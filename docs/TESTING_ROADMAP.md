# Testing Roadmap - Oxen-VCS for Logic Pro

**Version**: 1.0
**Date**: 2025-10-28
**Status**: In Progress

This document outlines the comprehensive testing strategy for bringing Oxen-VCS from "code complete" to "production ready."

---

## Current State (Baseline)

### Test Coverage Summary

| Component | Unit Tests | Integration Tests | E2E Tests | Total Coverage |
|-----------|------------|-------------------|-----------|----------------|
| **Rust CLI** | ✅ 85% | ❌ 0% | ❌ 0% | 🟡 85% |
| **LaunchAgent** | 🟡 30% | ❌ 0% | ❌ 0% | 🔴 30% |
| **UI App** | 🔴 <5% | ❌ 0% | ❌ 0% | 🔴 <5% |
| **Overall** | 🟡 40% | ❌ 0% | ❌ 0% | 🔴 40% |

### What's Tested Today

✅ **Rust CLI Wrapper (121 tests)**
- logic_project.rs: 18 tests
- commit_metadata.rs: 39 tests
- ignore_template.rs: 18 tests
- draft_manager.rs: 18 tests
- logger.rs: 22 tests
- oxen_subprocess.rs: 6 tests (basic)

✅ **Swift LaunchAgent (1 test suite)**
- LockManager: ~20 tests

✅ **Swift UI App (1 test file)**
- MockXPCClient: Basic mocking setup

### What's NOT Tested

❌ **Rust Integration**
- Actual oxen CLI subprocess execution
- File system operations with real repos
- End-to-end commit workflows

❌ **Swift Unit Tests**
- FSEventsMonitor
- PowerManagement
- CommitOrchestrator
- XPCService
- All ViewModels
- All Views

❌ **Integration Tests**
- XPC communication
- Daemon-to-CLI interaction
- UI-to-Daemon communication

❌ **End-to-End Tests**
- Complete workflows with Logic Pro
- Long-running scenarios
- Multi-user collaboration

---

## Testing Phases

## Phase 1: Rust Foundation (Week 1-2)

**Goal**: Achieve 90% coverage for Rust components with integration tests

### Week 1: Subprocess Integration Tests

**Prerequisites:**
- macOS environment with oxen CLI installed
- Test Logic Pro project fixtures

#### 1.1 Basic Oxen Operations (2 days)

```bash
# Test: Initialize repository
# Test: Add files to staging
# Test: Create commits
# Test: List commits
# Test: Get status
```

**Tests to Write:**
- `test_init_creates_oxen_directory()`
- `test_add_single_file_appears_in_status()`
- `test_add_all_stages_multiple_files()`
- `test_commit_creates_new_commit_in_log()`
- `test_commit_requires_message()`
- `test_status_shows_untracked_files()`
- `test_status_shows_modified_files()`

**Files:**
- `OxVCS-CLI-Wrapper/tests/integration_oxen_subprocess.rs`

#### 1.2 Branch Operations (1 day)

```bash
# Test: Create branches
# Test: Switch branches
# Test: List branches
# Test: Current branch detection
```

**Tests to Write:**
- `test_create_branch_shows_in_list()`
- `test_checkout_switches_branch()`
- `test_current_branch_correct_after_checkout()`
- `test_draft_branch_workflow()`

#### 1.3 Error Handling (1 day)

```bash
# Test: Invalid paths
# Test: Non-existent repos
# Test: Merge conflicts
# Test: Network failures (push/pull)
```

**Tests to Write:**
- `test_init_fails_on_invalid_path()`
- `test_add_fails_on_nonexistent_file()`
- `test_commit_fails_without_staged_files()`
- `test_checkout_fails_on_nonexistent_branch()`

#### 1.4 Logic Pro Integration (2 days)

```bash
# Test: Detect .logicx projects
# Test: Initialize in .logicx directory
# Test: Add ProjectData files
# Test: Respect .oxenignore patterns
```

**Tests to Write:**
- `test_logic_project_init_workflow()`
- `test_projectdata_detected_and_added()`
- `test_bounces_ignored_per_oxenignore()`
- `test_freeze_files_ignored()`
- `test_alternatives_directory_tracked()`

**Test Fixtures Needed:**
- Minimal .logicx project (10MB)
- Project with audio files (100MB)
- Project with bounces/freeze files

### Week 2: Performance & Reliability

#### 1.5 Performance Benchmarks (2 days)

**Metrics to Establish:**
- Subprocess startup time (<50ms)
- Add operation on 100 files (<500ms)
- Commit operation (<1s)
- Log retrieval for 1000 commits (<200ms)
- Status check on large projects (<300ms)

**Tools:**
- Criterion.rs for benchmarking
- Flamegraph for profiling

**Files:**
- `OxVCS-CLI-Wrapper/benches/oxen_subprocess_bench.rs`

#### 1.6 Edge Cases & Stress Tests (2 days)

**Scenarios:**
- Very large files (>1GB)
- Many small files (10,000+)
- Deep directory structures (20+ levels)
- Unicode filenames
- Symbolic links
- Permission errors

**Tests to Write:**
- `test_add_large_file_succeeds()`
- `test_commit_with_many_files()`
- `test_unicode_filenames_handled()`
- `test_symlink_handling()`

#### 1.7 Concurrent Operations (1 day)

**Scenarios:**
- Multiple add operations
- Simultaneous commits
- Concurrent status checks

**Tests to Write:**
- `test_concurrent_status_checks()`
- `test_multiple_add_operations_sequentially()`

**Deliverable**: 90% Rust coverage including integration tests

---

## Phase 2: Swift Unit Tests (Week 3-5)

**Goal**: Achieve 70% coverage for Swift components

### Week 3: LaunchAgent Core (FSEvents, Power, Orchestrator)

#### 2.1 FSEventsMonitor Tests (2 days)

**Test Cases:**
- Monitor starts and stops correctly
- Detects file modifications
- Debounce logic works (30-60s)
- Handles rapid file changes
- Multiple paths monitored simultaneously
- Event stream error recovery

**Tests to Write:**
```swift
func testMonitorStartsSuccessfully()
func testMonitorDetectsFileChange()
func testDebounceDelaysCallback()
func testRapidChangesOnlyTriggerOnce()
func testMultiplePathsMonitored()
func testEventStreamErrorHandled()
```

**Mock Strategy:**
- Mock file system events
- Use test directories with controlled modifications

#### 2.2 PowerManagement Tests (1 day)

**Test Cases:**
- Sleep notification received
- Shutdown notification received
- Emergency commit triggered
- Battery level checked (<5% threshold)
- System load detection
- Notification observers registered

**Tests to Write:**
```swift
func testSleepNotificationTriggersCommit()
func testShutdownNotificationTriggersCommit()
func testLowBatterySkipsCommit()
func testHighSystemLoadSkipsCommit()
func testNotificationObserversRegistered()
```

**Mock Strategy:**
- Mock NSWorkspace notifications
- Mock IOKit power sources

#### 2.3 CommitOrchestrator Tests (2 days)

**Test Cases:**
- Auto-commit workflow completes
- Lock check before commit
- Concurrent commit prevention
- Change detection works
- Branch verification
- Error handling and retry

**Tests to Write:**
```swift
func testAutoCommitCompleteWorkflow()
func testCommitFailsIfLockedByOthers()
func testConcurrentCommitPrevented()
func testNoCommitIfNoChanges()
func testCommitOnCorrectBranch()
func testCommitRetryOnTransientError()
```

**Mock Strategy:**
- Mock XPC calls to CLI
- Mock lock manager
- Control file system state

### Week 4: XPC & IPC

#### 2.4 XPCService Tests (2 days)

**Test Cases:**
- Connection establishment
- Request/response cycle
- Error propagation
- Timeout handling
- Reconnection logic
- Security (entitlements)

**Tests to Write:**
```swift
func testXPCConnectionEstablished()
func testXPCRequestResponseSuccess()
func testXPCErrorPropagated()
func testXPCTimeoutHandled()
func testXPCReconnectionAfterFailure()
```

**Mock Strategy:**
- Mock XPC connection
- Simulate connection failures
- Test with in-process XPC

#### 2.5 ServiceManager Tests (1 day)

**Test Cases:**
- LaunchAgent registration
- Startup configuration
- Status checking
- Deregistration

**Tests to Write:**
```swift
func testServiceRegistered()
func testServiceStartsOnLogin()
func testServiceStatusQueried()
func testServiceDeregistered()
```

### Week 5: UI Components

#### 2.6 ViewModel Tests (3 days)

**ViewModels to Test:**
- ProjectListViewModel
- ProjectDetailViewModel

**For Each ViewModel:**
- Initial state correct
- Data loading triggers update
- User actions handled
- Error states shown
- XPC calls made correctly

**Tests to Write:**
```swift
// ProjectListViewModel
func testInitialStateEmpty()
func testProjectsLoadedFromXPC()
func testAddProjectTriggersInitialization()
func testRefreshUpdatesProjects()
func testErrorStateShown()

// ProjectDetailViewModel
func testCommitHistoryLoaded()
func testMilestoneCommitTriggered()
func testRollbackConfirmationShown()
func testLockAcquisitionRequested()
```

**Mock Strategy:**
- Mock OxenDaemonXPCClient
- Control async responses
- Verify XPC calls made

#### 2.7 View Tests (Basic Smoke Tests) (1 day)

**Not full UI testing**, just verify views:
- Initialize without crashing
- Render basic content
- Handle button clicks

**Tests to Write:**
```swift
func testProjectListViewRenders()
func testMilestoneCommitWindowOpens()
func testRollbackWindowOpens()
func testLockManagementViewRenders()
```

**Deliverable**: 70% Swift coverage with unit tests

---

## Phase 3: Integration Testing (Week 6-7)

**Goal**: Verify components work together

### Week 6: Component Integration

#### 3.1 UI → Daemon Communication (2 days)

**Test Scenarios:**
- UI requests commit via XPC
- Daemon executes commit via CLI
- Result returned to UI
- Error handling end-to-end

**Tests to Write:**
- `test_ui_commit_request_executed()`
- `test_daemon_receives_xpc_request()`
- `test_cli_executes_oxen_command()`
- `test_result_returned_to_ui()`
- `test_error_propagated_through_stack()`

**Infrastructure:**
- Launch actual daemon process
- Connect from test UI code
- Verify with real temp repos

#### 3.2 FSEvents → Auto-Commit Pipeline (2 days)

**Test Scenarios:**
- File change detected
- Debounce delay observed
- Commit orchestrator triggered
- CLI executes commit
- Lock checked before commit

**Tests to Write:**
- `test_file_change_triggers_auto_commit()`
- `test_debounce_prevents_rapid_commits()`
- `test_lock_prevents_auto_commit()`
- `test_auto_commit_creates_draft_commit()`

**Infrastructure:**
- Monitor test .logicx project
- Modify files programmatically
- Wait for auto-commit
- Verify commit in oxen log

#### 3.3 Power Management → Emergency Commit (1 day)

**Test Scenarios:**
- Simulate sleep notification
- Verify immediate commit triggered
- Verify commit completes before sleep
- Handle already-committed state

**Tests to Write:**
- `test_sleep_triggers_immediate_commit()`
- `test_emergency_commit_completes_fast()`
- `test_no_commit_if_no_changes()`

### Week 7: End-to-End Workflows

#### 3.4 Complete User Workflows (3 days)

**Workflow 1: Initialize New Project**
1. User selects .logicx project in UI
2. UI calls daemon init via XPC
3. Daemon calls CLI to oxen init
4. .oxenignore created
5. Initial commit made
6. Monitoring starts
7. UI shows project in list

**Workflow 2: Make Changes & Auto-Commit**
1. Daemon monitors project
2. User modifies file in Logic Pro
3. FSEvents detects change after save
4. Debounce waits 30s
5. Auto-commit triggered
6. Commit appears in oxen log

**Workflow 3: Milestone Commit**
1. User opens milestone dialog
2. Enters metadata (BPM, key, tags)
3. UI sends XPC request
4. Daemon calls CLI with metadata
5. Commit created on main branch
6. UI refreshes commit history

**Workflow 4: Rollback**
1. User selects commit in history
2. Clicks rollback button
3. Confirmation dialog shown
4. UI sends rollback XPC request
5. Daemon calls CLI checkout
6. Files restored to commit state
7. UI shows updated status

**Workflow 5: Locking**
1. User clicks "Acquire Lock"
2. UI sends XPC request
3. Daemon creates lock manifest
4. Lock shown in UI
5. Second user attempts lock
6. Lock denied with owner info

**Test Implementation:**
- Automated UI tests (XCUITest)
- Or scripted workflows with AppleScript
- Verify with filesystem inspection
- Verify with oxen log inspection

#### 3.5 Multi-Project Monitoring (1 day)

**Test Scenarios:**
- Monitor 3 projects simultaneously
- Changes in each detected
- Auto-commits work independently
- No interference between projects

**Tests to Write:**
- `test_three_projects_monitored()`
- `test_concurrent_auto_commits()`
- `test_correct_project_committed()`

**Deliverable**: All integration tests passing

---

## Phase 4: System & Load Testing (Week 8-9)

**Goal**: Verify stability and performance under load

### Week 8: Long-Running Stability

#### 4.1 8-Hour Continuous Monitoring (2 days)

**Test Setup:**
- Launch daemon
- Monitor test project
- Simulate file changes every 5 minutes
- Run for 8 hours
- Measure:
  - Memory usage over time
  - CPU usage
  - Number of commits created
  - Any crashes or errors

**Acceptance Criteria:**
- Memory growth <10MB over 8 hours
- CPU usage <5% average
- All commits successful
- No crashes or hangs

#### 4.2 Resource Leak Detection (1 day)

**Tools:**
- Instruments (Leaks, Allocations)
- Memory profiler
- File descriptor monitoring

**Tests:**
- Run daemon for 1 hour
- Create 100 commits
- Verify no leaked memory
- Verify file descriptors closed
- Verify threads cleaned up

#### 4.3 Large Project Handling (2 days)

**Test Projects:**
- 1GB project (typical)
- 10GB project (large)
- 50GB project (extreme)

**Metrics:**
- Init time
- First commit time
- Subsequent commit time
- Status check time
- Memory usage during operations

**Acceptance Criteria:**
- 10GB project: <30s init, <10s commit
- 50GB project: <2min init, <30s commit
- Memory usage proportional to file count, not size

### Week 9: Stress & Edge Cases

#### 4.4 Rapid File Changes (1 day)

**Scenario:**
- Modify 100 files rapidly
- Verify debounce works correctly
- Only 1 commit created after debounce
- All changes captured

#### 4.5 Concurrent User Simulation (2 days)

**Scenario:**
- 3 users (3 machines or VMs)
- All attempt to acquire lock
- Only 1 succeeds
- Others cannot commit
- Lock released and reacquired by another

**Tests:**
- Lock contention
- Lock timeout expiration
- Force-break mechanics
- Lock status synchronization

#### 4.6 Error Recovery (2 days)

**Scenarios to Test:**
- Daemon crashes mid-commit
- CLI process hangs
- Network failure during push
- Disk full during commit
- Oxen CLI not found
- Repository corruption

**Expected Behavior:**
- Daemon restarts automatically
- Operations time out gracefully
- User notified of errors
- No data loss or corruption
- Recovery suggestions provided

**Deliverable**: Production-ready stability

---

## Phase 5: User Acceptance Testing (Week 10-12)

**Goal**: Validate with real users and projects

### Week 10: Beta User Recruitment

#### 5.1 Prepare Beta Release

- Create installer .app bundle
- Write user documentation
- Create quick start guide
- Set up feedback mechanism (GitHub issues)
- Prepare sample projects

#### 5.2 Recruit Beta Testers

**Target Users:**
- 5-10 Logic Pro users
- Mix of solo and collaborative users
- Range of project sizes
- Willingness to report bugs

**Provide:**
- Beta installer
- User guide
- Feedback form
- Support contact

### Week 11-12: Beta Testing & Iteration

#### 5.3 Monitored Usage

**Track:**
- Installation success rate
- Usage patterns
- Feature adoption
- Bug reports
- Performance issues
- User satisfaction

#### 5.4 Bug Fixes & Iteration

**Process:**
- Triage reported issues
- Fix critical bugs
- Release updates to beta users
- Verify fixes
- Collect feedback on fixes

#### 5.5 Final Validation

**Checklist:**
- [ ] All critical bugs fixed
- [ ] No data loss incidents
- [ ] Performance acceptable
- [ ] Users can complete workflows
- [ ] Documentation sufficient
- [ ] Ready for v1.0 release

---

## Testing Infrastructure

### Required Tools

**macOS:**
- Xcode 15+ with XCTest
- Swift Package Manager
- Cargo + Rust toolchain
- Oxen CLI (`pip install oxen-ai`)

**Testing Frameworks:**
- XCTest (Swift unit/integration)
- XCUITest (UI automation)
- Criterion.rs (Rust benchmarking)
- Instruments (profiling)

**CI/CD:**
- GitHub Actions (macOS runners)
- Automated test runs on PR
- Code coverage reporting

### Test Data & Fixtures

**Rust Tests:**
- `tests/fixtures/minimal_project.logicx` (10MB)
- `tests/fixtures/medium_project.logicx` (100MB)
- `tests/fixtures/large_project.logicx` (1GB)

**Swift Tests:**
- Mock .logicx projects in temp directories
- Mock XPC responses
- Controlled FSEvents triggers

### Continuous Integration

**GitHub Actions Workflow:**
```yaml
name: Test Suite
on: [push, pull_request]

jobs:
  rust-tests:
    runs-on: macos-14
    steps:
      - uses: actions/checkout@v3
      - name: Install oxen
        run: pip install oxen-ai
      - name: Run Rust tests
        run: cd OxVCS-CLI-Wrapper && cargo test
      - name: Run benchmarks
        run: cargo bench

  swift-tests:
    runs-on: macos-14
    steps:
      - uses: actions/checkout@v3
      - name: Build LaunchAgent
        run: cd OxVCS-LaunchAgent && swift build
      - name: Run LaunchAgent tests
        run: swift test
      - name: Build App
        run: cd OxVCS-App && swift build
      - name: Run App tests
        run: swift test

  integration-tests:
    runs-on: macos-14
    needs: [rust-tests, swift-tests]
    steps:
      - uses: actions/checkout@v3
      - name: Install oxen
        run: pip install oxen-ai
      - name: Run integration tests
        run: ./run_integration_tests.sh
```

---

## Success Metrics

### Coverage Goals

| Component | Week 2 | Week 5 | Week 7 | Week 9 |
|-----------|--------|--------|--------|--------|
| Rust CLI | 90% | 90% | 90% | 90% |
| LaunchAgent | 30% | 70% | 75% | 75% |
| UI App | 5% | 60% | 70% | 70% |
| Integration | 0% | 0% | 80% | 90% |
| **Overall** | 40% | 60% | 75% | 80% |

### Quality Gates

**Phase 1 Complete When:**
- ✅ 90% Rust unit test coverage
- ✅ All integration tests passing
- ✅ Performance benchmarks established

**Phase 2 Complete When:**
- ✅ 70% Swift unit test coverage
- ✅ All ViewModels tested
- ✅ Core daemon components tested

**Phase 3 Complete When:**
- ✅ All 5 E2E workflows pass
- ✅ XPC communication verified
- ✅ Multi-project handling works

**Phase 4 Complete When:**
- ✅ 8-hour stability test passes
- ✅ No memory leaks detected
- ✅ Large projects handled
- ✅ Error recovery validated

**Phase 5 Complete When:**
- ✅ 10 beta users successfully using
- ✅ No critical bugs reported
- ✅ User satisfaction >80%
- ✅ Documentation complete

---

## Risk & Mitigation

### High-Risk Areas

**1. Daemon Stability**
- Risk: Crashes during long-running operation
- Mitigation: Extensive stability testing, automatic restart
- Fallback: Manual restart documented

**2. Lock Synchronization**
- Risk: Race conditions with concurrent access
- Mitigation: Thorough concurrent testing
- Fallback: Force-break with confirmation

**3. Data Loss**
- Risk: Commit fails, changes lost
- Mitigation: Robust error handling, rollback capability
- Fallback: Manual recovery documentation

**4. Performance**
- Risk: Slow on large projects
- Mitigation: Performance testing, optimization
- Fallback: Document limitations, suggest workflows

### Testing Blockers

**Blocker 1: macOS Access**
- Current: Linux environment
- Required: macOS 14.0+ for testing
- Timeline: Before Week 1 of testing

**Blocker 2: Logic Pro License**
- Required: For E2E testing
- Timeline: Before Week 7 (integration tests)

**Blocker 3: Multi-Machine Setup**
- Required: For concurrent user testing
- Timeline: Before Week 9 (stress testing)

---

## Timeline Summary

| Phase | Duration | Dependencies | Deliverable |
|-------|----------|--------------|-------------|
| Phase 1: Rust Foundation | 2 weeks | macOS, oxen CLI | 90% Rust coverage |
| Phase 2: Swift Unit Tests | 3 weeks | Phase 1 | 70% Swift coverage |
| Phase 3: Integration | 2 weeks | Phase 1-2 | E2E workflows |
| Phase 4: System/Load | 2 weeks | Phase 3 | Stability validated |
| Phase 5: UAT | 3 weeks | Phase 4 | Beta feedback |
| **Total** | **12 weeks** | | **Production ready** |

---

**Next Steps:**
1. Obtain macOS development environment
2. Install oxen CLI (`pip install oxen-ai`)
3. Begin Phase 1 Rust integration tests
4. Create test fixtures for Logic Pro projects

**Status Tracking:** Update this document weekly with progress and blockers.

---

*Last Updated: 2025-10-28*
