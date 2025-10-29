# macOS Test Plan - Oxen-VCS for Logic Pro

**Version**: 1.0
**Date**: 2025-10-29
**Target Platform**: macOS 14.0+ with Xcode 15+
**Status**: Ready for execution (requires macOS environment)

---

## Executive Summary

This test plan addresses the critical testing gaps identified in the project's reality check. The current state shows:
- **Rust CLI**: 85% coverage (Linux) - needs macOS validation
- **Swift LaunchAgent**: ~30% coverage - needs comprehensive testing
- **Swift UI App**: <5% coverage - needs comprehensive testing
- **Integration**: 0% - needs full E2E validation
- **Real-world usage**: Untested with actual Logic Pro projects

**Estimated Timeline**: 2-3 weeks of dedicated testing on macOS
**Prerequisites**: macOS 14.0+, Xcode 15+, Logic Pro 11.x, test Logic Pro projects

---

## Test Environment Setup

### Required Hardware
- [ ] Mac with macOS 14.0 or later
- [ ] Minimum 16GB RAM (32GB recommended for large project tests)
- [ ] 100GB+ free disk space for test fixtures
- [ ] SSD storage (for realistic FSEvents timing)

### Required Software
```bash
# Install dependencies
brew install rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Oxen CLI
pip3 install oxen-ai
# OR
cargo install oxen

# Verify installation
oxen --version

# Install Xcode Command Line Tools
xcode-select --install

# Install Xcode 15+
# (Download from App Store or developer.apple.com)
```

### Test Project Setup
```bash
# Clone repository
git clone https://github.com/jbacus/oxen-vcs-logic.git
cd oxen-vcs-logic

# Checkout test branch
git checkout claude/macos-test-plan-011CUbfAYBzNAKHBKUP77PKE

# Build Rust components
cd OxVCS-CLI-Wrapper
cargo build --release
cargo test
cd ..

# Open in Xcode
open OxVCS-App/OxVCS.xcodeproj
```

### Test Fixtures Required
Create the following test Logic Pro projects:

1. **Tiny Project** (< 100MB)
   - 4 audio tracks, 2 software instrument tracks
   - 10 audio files
   - No plugins, basic effects only
   - Purpose: Fast iteration testing

2. **Medium Project** (1-5GB)
   - 32 audio tracks, 8 MIDI tracks
   - 100+ audio files
   - Third-party plugins
   - Purpose: Realistic use case

3. **Large Project** (10-50GB)
   - 64+ audio tracks
   - 500+ audio files
   - Extensive plugin chains
   - Purpose: Stress testing

4. **Corrupted Project**
   - Intentionally malformed .logicx structure
   - Missing required files
   - Purpose: Error handling validation

---

## Phase 1: Unit Tests (Priority: Critical)

### 1.1 Rust CLI Wrapper Tests (Validation)

**Status**: Code exists, needs macOS validation
**Estimated Time**: 2-3 hours

```bash
cd OxVCS-CLI-Wrapper

# Run full test suite on macOS
cargo test --verbose

# Run with output capture
cargo test -- --nocapture

# Run specific test modules
cargo test oxen_ops::tests
cargo test commit::tests
cargo test project_detection::tests

# Run benchmarks
cargo bench

# Check for platform-specific issues
cargo test --features macos-specific
```

**Expected Results**:
- [ ] All 121 unit tests pass on macOS
- [ ] No platform-specific failures
- [ ] Performance targets met:
  - Single file add: < 10ms
  - Commit operation: < 100ms
  - Memory usage: < 50MB

**Critical Tests to Verify**:
- [ ] `tests/integration_tests.rs::test_oxen_subprocess_wrapper()`
- [ ] `tests/logic_detection.rs::test_detect_folder_based_project()`
- [ ] `tests/commit_metadata.rs::test_parse_commit_with_metadata()`

**Failure Handling**:
- If tests fail, document platform-specific issues
- Check for path separator differences (/ vs \)
- Verify FFI boundary behavior on macOS

---

### 1.2 LaunchAgent Unit Tests (New Tests Required)

**Status**: Only LockManager tested (~30% coverage)
**Estimated Time**: 3-4 days

#### 1.2.1 FSEventsMonitor Tests

**File**: `OxVCS-LaunchAgent/Tests/FSEventsMonitorTests.swift`

```swift
import XCTest
@testable import OxVCS_LaunchAgent

class FSEventsMonitorTests: XCTestCase {
    var monitor: FSEventsMonitor!
    var tempDir: URL!

    override func setUp() async throws {
        tempDir = FileManager.default.temporaryDirectory
            .appendingPathComponent(UUID().uuidString)
        try FileManager.default.createDirectory(at: tempDir, withIntermediateDirectories: true)
        monitor = FSEventsMonitor()
    }

    override func tearDown() async throws {
        monitor.stopMonitoring()
        try? FileManager.default.removeItem(at: tempDir)
    }

    // Test: Basic event detection
    func testDetectsFileModification() async throws {
        let expectation = expectation(description: "Event detected")
        var eventDetected = false

        monitor.startMonitoring(path: tempDir.path) { path in
            eventDetected = true
            expectation.fulfill()
        }

        // Modify file
        let testFile = tempDir.appendingPathComponent("test.txt")
        try "content".write(to: testFile, atomically: true, encoding: .utf8)

        await fulfillment(of: [expectation], timeout: 5.0)
        XCTAssertTrue(eventDetected)
    }

    // Test: Debounce behavior
    func testDebounceMultipleEvents() async throws {
        var callCount = 0
        let expectation = expectation(description: "Debounced")

        monitor.debounceInterval = 2.0
        monitor.startMonitoring(path: tempDir.path) { _ in
            callCount += 1
            expectation.fulfill()
        }

        // Rapid modifications
        let testFile = tempDir.appendingPathComponent("test.txt")
        for i in 0..<10 {
            try "\(i)".write(to: testFile, atomically: true, encoding: .utf8)
            try await Task.sleep(nanoseconds: 100_000_000) // 0.1s
        }

        await fulfillment(of: [expectation], timeout: 5.0)
        XCTAssertEqual(callCount, 1, "Should only trigger once after debounce")
    }

    // Test: Ignores excluded paths
    func testIgnoresExcludedPaths() async throws {
        monitor.excludedPaths = ["Bounces", "Freeze Files"]
        var triggeredForExcluded = false

        monitor.startMonitoring(path: tempDir.path) { path in
            if path.contains("Bounces") {
                triggeredForExcluded = true
            }
        }

        let bouncesDir = tempDir.appendingPathComponent("Bounces")
        try FileManager.default.createDirectory(at: bouncesDir, withIntermediateDirectories: true)
        let testFile = bouncesDir.appendingPathComponent("bounce.wav")
        try Data().write(to: testFile)

        try await Task.sleep(nanoseconds: 3_000_000_000)
        XCTAssertFalse(triggeredForExcluded)
    }

    // Test: Handles rapid start/stop
    func testStartStopCycles() throws {
        for _ in 0..<100 {
            monitor.startMonitoring(path: tempDir.path) { _ in }
            monitor.stopMonitoring()
        }
        // Should not crash or leak
    }

    // Test: Multiple paths simultaneously
    func testMonitorsMultiplePaths() async throws {
        let dir1 = tempDir.appendingPathComponent("project1")
        let dir2 = tempDir.appendingPathComponent("project2")
        try FileManager.default.createDirectory(at: dir1, withIntermediateDirectories: true)
        try FileManager.default.createDirectory(at: dir2, withIntermediateDirectories: true)

        var events: Set<String> = []
        let expectation = expectation(description: "Both paths monitored")
        expectation.expectedFulfillmentCount = 2

        monitor.startMonitoring(paths: [dir1.path, dir2.path]) { path in
            events.insert(path)
            expectation.fulfill()
        }

        try "test".write(to: dir1.appendingPathComponent("file1.txt"), atomically: true, encoding: .utf8)
        try "test".write(to: dir2.appendingPathComponent("file2.txt"), atomically: true, encoding: .utf8)

        await fulfillment(of: [expectation], timeout: 5.0)
        XCTAssertEqual(events.count, 2)
    }
}
```

**Test Checklist**:
- [ ] `testDetectsFileModification` - Basic event detection
- [ ] `testDebounceMultipleEvents` - Debouncing works correctly
- [ ] `testIgnoresExcludedPaths` - .oxenignore patterns respected
- [ ] `testStartStopCycles` - No memory leaks
- [ ] `testMonitorsMultiplePaths` - Multi-project support
- [ ] Event latency < 100ms
- [ ] CPU usage < 1% when idle
- [ ] No dropped events under load

#### 1.2.2 PowerManager Tests

**File**: `OxVCS-LaunchAgent/Tests/PowerManagerTests.swift`

```swift
import XCTest
@testable import OxVCS_LaunchAgent

class PowerManagerTests: XCTestCase {
    var powerManager: PowerManager!
    var mockCommitter: MockDraftCommitter!

    override func setUp() {
        mockCommitter = MockDraftCommitter()
        powerManager = PowerManager(committer: mockCommitter)
    }

    // Test: Sleep notification triggers commit
    func testSleepNotificationTriggersCommit() async throws {
        let expectation = expectation(description: "Emergency commit triggered")
        mockCommitter.onCommit = { reason in
            XCTAssertTrue(reason.contains("sleep"))
            expectation.fulfill()
        }

        powerManager.registerPowerNotifications()

        // Simulate sleep notification
        NotificationCenter.default.post(
            name: NSWorkspace.willSleepNotification,
            object: nil
        )

        await fulfillment(of: [expectation], timeout: 2.0)
    }

    // Test: Power off notification triggers commit
    func testPowerOffTriggersCommit() async throws {
        let expectation = expectation(description: "Emergency commit on poweroff")
        mockCommitter.onCommit = { reason in
            XCTAssertTrue(reason.contains("power"))
            expectation.fulfill()
        }

        powerManager.registerPowerNotifications()

        NotificationCenter.default.post(
            name: NSWorkspace.willPowerOffNotification,
            object: nil
        )

        await fulfillment(of: [expectation], timeout: 2.0)
    }

    // Test: Commit completes before sleep (timeout check)
    func testCommitCompletesWithinTimeout() async throws {
        mockCommitter.commitDuration = 0.5 // 500ms commit
        powerManager.commitTimeout = 5.0 // 5s timeout

        let expectation = expectation(description: "Commit completes")
        mockCommitter.onCommit = { _ in
            expectation.fulfill()
        }

        powerManager.registerPowerNotifications()
        NotificationCenter.default.post(
            name: NSWorkspace.willSleepNotification,
            object: nil
        )

        await fulfillment(of: [expectation], timeout: 6.0)
    }

    // Test: Handles multiple rapid sleep/wake cycles
    func testMultipleSleepWakeCycles() async throws {
        var commitCount = 0
        mockCommitter.onCommit = { _ in commitCount += 1 }

        powerManager.registerPowerNotifications()

        for _ in 0..<10 {
            NotificationCenter.default.post(
                name: NSWorkspace.willSleepNotification,
                object: nil
            )
            try await Task.sleep(nanoseconds: 100_000_000)
            NotificationCenter.default.post(
                name: NSWorkspace.didWakeNotification,
                object: nil
            )
        }

        try await Task.sleep(nanoseconds: 1_000_000_000)
        XCTAssertEqual(commitCount, 10)
    }
}

class MockDraftCommitter: DraftCommitterProtocol {
    var commitDuration: TimeInterval = 0.1
    var onCommit: ((String) -> Void)?

    func forceCommit(message: String) async throws {
        try await Task.sleep(nanoseconds: UInt64(commitDuration * 1_000_000_000))
        onCommit?(message)
    }
}
```

**Test Checklist**:
- [ ] `testSleepNotificationTriggersCommit` - Sleep detection works
- [ ] `testPowerOffTriggersCommit` - Power off detection works
- [ ] `testCommitCompletesWithinTimeout` - Timeout handling
- [ ] `testMultipleSleepWakeCycles` - Stability under cycles
- [ ] Emergency commits complete in < 5s
- [ ] No commit data loss on abrupt sleep

#### 1.2.3 DraftCommitter Tests

**File**: `OxVCS-LaunchAgent/Tests/DraftCommitterTests.swift`

```swift
import XCTest
@testable import OxVCS_LaunchAgent

class DraftCommitterTests: XCTestCase {
    var committer: DraftCommitter!
    var mockOxenService: MockOxenService!
    var tempProjectPath: URL!

    override func setUp() async throws {
        mockOxenService = MockOxenService()
        committer = DraftCommitter(oxenService: mockOxenService)
        tempProjectPath = FileManager.default.temporaryDirectory
            .appendingPathComponent(UUID().uuidString)
        try FileManager.default.createDirectory(at: tempProjectPath, withIntermediateDirectories: true)
    }

    // Test: Creates draft commit with correct branch
    func testCreatesDraftCommitOnDraftBranch() async throws {
        try await committer.commitDraft(projectPath: tempProjectPath.path)

        XCTAssertEqual(mockOxenService.lastCommitBranch, "draft")
        XCTAssertTrue(mockOxenService.lastCommitMessage.contains("Auto-save"))
    }

    // Test: Includes timestamp in commit message
    func testIncludesTimestampInMessage() async throws {
        let before = Date()
        try await committer.commitDraft(projectPath: tempProjectPath.path)
        let after = Date()

        let message = mockOxenService.lastCommitMessage
        // Verify timestamp is present and reasonable
        XCTAssertTrue(message.contains("2025") || message.contains("2026"))
    }

    // Test: Handles commit failures gracefully
    func testHandlesCommitFailure() async throws {
        mockOxenService.shouldFail = true

        do {
            try await committer.commitDraft(projectPath: tempProjectPath.path)
            XCTFail("Should throw error")
        } catch {
            XCTAssertTrue(error is OxenError)
        }
    }

    // Test: Skips commit if no changes
    func testSkipsCommitIfNoChanges() async throws {
        mockOxenService.hasChanges = false

        try await committer.commitDraft(projectPath: tempProjectPath.path)

        XCTAssertEqual(mockOxenService.commitCount, 0)
    }

    // Test: Concurrent commit requests are queued
    func testConcurrentCommitsAreQueued() async throws {
        let commitCount = 10
        let expectations = (0..<commitCount).map { expectation(description: "Commit \($0)") }

        await withTaskGroup(of: Void.self) { group in
            for i in 0..<commitCount {
                group.addTask {
                    try? await self.committer.commitDraft(projectPath: self.tempProjectPath.path)
                    expectations[i].fulfill()
                }
            }
        }

        await fulfillment(of: expectations, timeout: 10.0)
        XCTAssertEqual(mockOxenService.commitCount, commitCount)
    }
}

class MockOxenService: OxenServiceProtocol {
    var lastCommitMessage = ""
    var lastCommitBranch = ""
    var commitCount = 0
    var shouldFail = false
    var hasChanges = true

    func commit(message: String, branch: String) async throws {
        guard !shouldFail else {
            throw OxenError.commitFailed
        }
        guard hasChanges else { return }

        lastCommitMessage = message
        lastCommitBranch = branch
        commitCount += 1

        try await Task.sleep(nanoseconds: 100_000_000) // Simulate work
    }

    func hasUncommittedChanges() -> Bool {
        return hasChanges
    }
}
```

**Test Checklist**:
- [ ] `testCreatesDraftCommitOnDraftBranch` - Correct branch usage
- [ ] `testIncludesTimestampInMessage` - Timestamp format
- [ ] `testHandlesCommitFailure` - Error recovery
- [ ] `testSkipsCommitIfNoChanges` - Optimization
- [ ] `testConcurrentCommitsAreQueued` - Thread safety
- [ ] Commit latency < 100ms (excluding Oxen operations)

#### 1.2.4 LockManager Tests (Expand Existing)

**File**: `OxVCS-LaunchAgent/Tests/LockManagerTests.swift`

Expand existing tests to cover:

```swift
// Test: Lock timeout enforcement
func testLockTimeoutEnforcement() async throws {
    let lockManager = LockManager(timeout: 2.0)

    try lockManager.acquireLock(user: "user1", project: testProject)

    try await Task.sleep(nanoseconds: 3_000_000_000) // 3s

    // Should auto-release after timeout
    let canAcquire = try lockManager.acquireLock(user: "user2", project: testProject)
    XCTAssertTrue(canAcquire)
}

// Test: Lock persistence across daemon restarts
func testLockPersistenceAcrossRestarts() throws {
    let lockManager1 = LockManager()
    try lockManager1.acquireLock(user: "user1", project: testProject)

    // Simulate daemon restart
    let lockManager2 = LockManager()

    XCTAssertTrue(lockManager2.isLocked(project: testProject))
    XCTAssertEqual(lockManager2.lockHolder(project: testProject), "user1")
}

// Test: File permission enforcement
func testFilePermissionEnforcement() throws {
    try lockManager.acquireLock(user: "user1", project: testProject)
    lockManager.enforcePermissions(project: testProject)

    let projectFile = testProject.appendingPathComponent("projectData")
    let attributes = try FileManager.default.attributesOfItem(atPath: projectFile.path)
    let permissions = attributes[.posixPermissions] as! NSNumber

    // Should be read-only
    XCTAssertEqual(permissions.intValue & 0o200, 0)
}
```

**Additional Test Checklist**:
- [ ] Lock timeout enforcement
- [ ] Lock persistence
- [ ] File permission enforcement
- [ ] Lock stealing with force flag
- [ ] Remote lock synchronization

#### 1.2.5 IPCService Tests

**File**: `OxVCS-LaunchAgent/Tests/IPCServiceTests.swift`

```swift
import XCTest
@testable import OxVCS_LaunchAgent

class IPCServiceTests: XCTestCase {
    var service: IPCService!
    var mockCommitter: MockDraftCommitter!

    override func setUp() {
        mockCommitter = MockDraftCommitter()
        service = IPCService(committer: mockCommitter)
    }

    // Test: XPC connection establishment
    func testXPCConnectionEstablishment() async throws {
        let connection = NSXPCConnection(serviceName: "com.oxenvcs.agent")
        connection.remoteObjectInterface = NSXPCInterface(with: OxenVCSServiceProtocol.self)
        connection.resume()

        let proxy = connection.remoteObjectProxyWithErrorHandler { error in
            XCTFail("Connection error: \(error)")
        } as? OxenVCSServiceProtocol

        XCTAssertNotNil(proxy)
        connection.invalidate()
    }

    // Test: Execute commit via IPC
    func testExecuteCommitViaIPC() async throws {
        let expectation = expectation(description: "IPC commit")

        service.executeCommit(message: "Test commit") { success, error in
            XCTAssertTrue(success)
            XCTAssertNil(error)
            expectation.fulfill()
        }

        await fulfillment(of: [expectation], timeout: 5.0)
    }

    // Test: IPC error propagation
    func testIPCErrorPropagation() async throws {
        mockCommitter.shouldFail = true
        let expectation = expectation(description: "IPC error")

        service.executeCommit(message: "Test") { success, error in
            XCTAssertFalse(success)
            XCTAssertNotNil(error)
            expectation.fulfill()
        }

        await fulfillment(of: [expectation], timeout: 5.0)
    }

    // Test: Concurrent IPC requests
    func testConcurrentIPCRequests() async throws {
        let requestCount = 20
        let expectations = (0..<requestCount).map { expectation(description: "Request \($0)") }

        for i in 0..<requestCount {
            service.executeCommit(message: "Commit \(i)") { success, error in
                XCTAssertTrue(success)
                expectations[i].fulfill()
            }
        }

        await fulfillment(of: expectations, timeout: 30.0)
    }
}
```

**Test Checklist**:
- [ ] XPC connection establishment
- [ ] Execute commit via IPC
- [ ] Stage path via IPC
- [ ] Error propagation
- [ ] Concurrent IPC requests
- [ ] Connection recovery after crash

---

### 1.3 UI Application Unit Tests (New Tests Required)

**Status**: <5% coverage (only MockXPCClient)
**Estimated Time**: 4-5 days

#### 1.3.1 ViewModel Tests

**File**: `OxVCS-App/Tests/CommitViewModelTests.swift`

```swift
import XCTest
@testable import OxVCS_App

@MainActor
class CommitViewModelTests: XCTestCase {
    var viewModel: CommitViewModel!
    var mockService: MockOxenService!

    override func setUp() {
        mockService = MockOxenService()
        viewModel = CommitViewModel(oxenService: mockService)
    }

    // Test: Load commit history
    func testLoadCommitHistory() async throws {
        mockService.mockCommits = [
            Commit(id: "abc123", message: "Test commit", author: "user", date: Date()),
            Commit(id: "def456", message: "Another commit", author: "user", date: Date())
        ]

        await viewModel.loadHistory()

        XCTAssertEqual(viewModel.commits.count, 2)
        XCTAssertEqual(viewModel.commits[0].id, "abc123")
    }

    // Test: Create milestone commit
    func testCreateMilestoneCommit() async throws {
        let metadata = CommitMetadata(bpm: 120, sampleRate: 48000, key: "C")

        try await viewModel.createMilestoneCommit(
            message: "Mix v1",
            metadata: metadata
        )

        XCTAssertTrue(mockService.lastCommitMessage.contains("Mix v1"))
        XCTAssertTrue(mockService.lastCommitMessage.contains("BPM: 120"))
    }

    // Test: Rollback to commit
    func testRollbackToCommit() async throws {
        let commit = Commit(id: "abc123", message: "Old version", author: "user", date: Date())

        try await viewModel.rollback(to: commit)

        XCTAssertEqual(mockService.lastCheckoutCommit, "abc123")
    }

    // Test: Error handling
    func testErrorHandling() async {
        mockService.shouldFail = true

        await viewModel.loadHistory()

        XCTAssertTrue(viewModel.hasError)
        XCTAssertNotNil(viewModel.errorMessage)
    }

    // Test: Loading state management
    func testLoadingStateManagement() async {
        XCTAssertFalse(viewModel.isLoading)

        let task = Task {
            await viewModel.loadHistory()
        }

        // Should be loading during operation
        try? await Task.sleep(nanoseconds: 10_000_000)
        XCTAssertTrue(viewModel.isLoading)

        await task.value
        XCTAssertFalse(viewModel.isLoading)
    }
}
```

**Additional ViewModel Tests Needed**:
- [ ] ProjectBrowserViewModel
- [ ] MergeHelperViewModel
- [ ] LockManagementViewModel
- [ ] SettingsViewModel

#### 1.3.2 Service Tests

**File**: `OxVCS-App/Tests/OxenServiceTests.swift`

```swift
import XCTest
@testable import OxVCS_App

class OxenServiceTests: XCTestCase {
    var service: OxenService!
    var tempDir: URL!

    override func setUp() async throws {
        tempDir = FileManager.default.temporaryDirectory
            .appendingPathComponent(UUID().uuidString)
        try FileManager.default.createDirectory(at: tempDir, withIntermediateDirectories: true)
        service = OxenService()
    }

    // Test: Initialize repository
    func testInitializeRepository() throws {
        try service.initRepository(at: tempDir)

        let oxenDir = tempDir.appendingPathComponent(".oxen")
        XCTAssertTrue(FileManager.default.fileExists(atPath: oxenDir.path))
    }

    // Test: Detect Logic Pro project
    func testDetectLogicProProject() throws {
        // Create fake Logic Pro structure
        let projectDir = tempDir.appendingPathComponent("Test.logicx")
        try FileManager.default.createDirectory(at: projectDir, withIntermediateDirectories: true)
        try Data().write(to: projectDir.appendingPathComponent("projectData"))

        let isLogicProject = service.isLogicProProject(at: projectDir)
        XCTAssertTrue(isLogicProject)
    }

    // Test: Generate .oxenignore
    func testGenerateOxenignore() throws {
        try service.generateOxenignore(at: tempDir)

        let oxenignorePath = tempDir.appendingPathComponent(".oxenignore")
        XCTAssertTrue(FileManager.default.fileExists(atPath: oxenignorePath.path))

        let content = try String(contentsOf: oxenignorePath)
        XCTAssertTrue(content.contains("Bounces/"))
        XCTAssertTrue(content.contains("Freeze Files/"))
    }

    // Test: Get commit list
    func testGetCommitList() throws {
        try service.initRepository(at: tempDir)
        try service.commit(message: "Initial commit")

        let commits = try service.getCommits()
        XCTAssertGreaterThanOrEqual(commits.count, 1)
    }

    // Test: Checkout commit
    func testCheckoutCommit() throws {
        try service.initRepository(at: tempDir)
        let testFile = tempDir.appendingPathComponent("test.txt")
        try "v1".write(to: testFile, atomically: true, encoding: .utf8)
        try service.add(files: [testFile])
        let commit1 = try service.commit(message: "Version 1")

        try "v2".write(to: testFile, atomically: true, encoding: .utf8)
        try service.add(files: [testFile])
        try service.commit(message: "Version 2")

        try service.checkout(commit: commit1)

        let content = try String(contentsOf: testFile)
        XCTAssertEqual(content, "v1")
    }
}
```

**Test Checklist**:
- [ ] Repository initialization
- [ ] Logic Pro project detection
- [ ] .oxenignore generation
- [ ] Commit creation
- [ ] Commit listing
- [ ] Checkout/rollback
- [ ] Branch operations
- [ ] Lock operations

---

## Phase 2: Integration Tests (Priority: Critical)

**Estimated Time**: 3-4 days

### 2.1 End-to-End Workflow Tests

**File**: `tests/integration/e2e_workflow_tests.rs`

```rust
#[cfg(test)]
mod e2e_tests {
    use std::process::Command;
    use std::path::PathBuf;

    #[test]
    fn test_complete_draft_commit_workflow() {
        // 1. Setup test project
        let test_project = setup_test_logic_project();

        // 2. Initialize repository via CLI
        let output = Command::new("./target/release/oxenvcs-cli")
            .args(&["init", test_project.to_str().unwrap()])
            .output()
            .expect("Failed to execute init");
        assert!(output.status.success());

        // 3. Start LaunchAgent (simulated)
        // In real test, would launchctl load the agent

        // 4. Modify project file
        modify_logic_project(&test_project);

        // 5. Wait for debounce period
        std::thread::sleep(std::time::Duration::from_secs(35));

        // 6. Verify draft commit created
        let output = Command::new("./target/release/oxenvcs-cli")
            .args(&["log", "--branch=draft"])
            .current_dir(&test_project)
            .output()
            .expect("Failed to execute log");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Auto-save"));
    }

    #[test]
    fn test_milestone_commit_workflow() {
        let test_project = setup_test_logic_project();

        // Initialize
        Command::new("./target/release/oxenvcs-cli")
            .args(&["init", test_project.to_str().unwrap()])
            .output()
            .expect("Failed to init");

        // Modify project
        modify_logic_project(&test_project);

        // Create milestone commit via UI (simulated via CLI)
        let output = Command::new("./target/release/oxenvcs-cli")
            .args(&[
                "commit",
                "--message=Mix v1",
                "--bpm=120",
                "--sample-rate=48000",
                "--key=C"
            ])
            .current_dir(&test_project)
            .output()
            .expect("Failed to commit");

        assert!(output.status.success());

        // Verify metadata in commit
        let output = Command::new("./target/release/oxenvcs-cli")
            .args(&["log", "-n", "1"])
            .current_dir(&test_project)
            .output()
            .expect("Failed to log");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("BPM: 120"));
        assert!(stdout.contains("Sample Rate: 48kHz"));
    }

    #[test]
    fn test_rollback_workflow() {
        let test_project = setup_test_logic_project();

        // Initialize and make two commits
        Command::new("./target/release/oxenvcs-cli")
            .args(&["init", test_project.to_str().unwrap()])
            .output()
            .expect("Failed to init");

        modify_logic_project(&test_project);
        let commit1 = create_commit(&test_project, "Version 1");

        modify_logic_project(&test_project);
        create_commit(&test_project, "Version 2");

        // Rollback to commit1
        let output = Command::new("./target/release/oxenvcs-cli")
            .args(&["checkout", &commit1])
            .current_dir(&test_project)
            .output()
            .expect("Failed to checkout");

        assert!(output.status.success());

        // Verify project state matches commit1
        // (implementation specific verification)
    }

    fn setup_test_logic_project() -> PathBuf {
        // Create a minimal Logic Pro project structure
        let temp_dir = std::env::temp_dir();
        let project = temp_dir.join(format!("TestProject_{}.logicx", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&project).unwrap();

        // Create projectData file
        std::fs::write(project.join("projectData"), b"fake project data").unwrap();

        // Create Audio Files directory
        std::fs::create_dir_all(project.join("Audio Files")).unwrap();

        project
    }

    fn modify_logic_project(project: &PathBuf) {
        let project_data = project.join("projectData");
        let mut content = std::fs::read(&project_data).unwrap();
        content.push(0x00); // Modify binary data
        std::fs::write(&project_data, content).unwrap();
    }

    fn create_commit(project: &PathBuf, message: &str) -> String {
        let output = Command::new("./target/release/oxenvcs-cli")
            .args(&["commit", "--message", message])
            .current_dir(project)
            .output()
            .expect("Failed to commit");

        // Extract commit ID from output
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .find(|l| l.contains("commit"))
            .unwrap()
            .split_whitespace()
            .nth(1)
            .unwrap()
            .to_string()
    }
}
```

**Test Checklist**:
- [ ] Complete draft commit workflow
- [ ] Milestone commit workflow
- [ ] Rollback workflow
- [ ] Power management workflow (sleep/wake)
- [ ] Multi-project monitoring
- [ ] Lock acquisition and release
- [ ] Remote push/pull workflow

---

### 2.2 Component Communication Tests

**File**: `tests/integration/ipc_communication_tests.swift`

```swift
import XCTest
@testable import OxVCS_App
@testable import OxVCS_LaunchAgent

class IPCCommunicationTests: XCTestCase {
    var app: OxVCS_App.Application!
    var agent: OxVCS_LaunchAgent.Agent!

    override func setUp() async throws {
        // Start LaunchAgent
        agent = OxVCS_LaunchAgent.Agent()
        try agent.start()

        // Launch app
        app = OxVCS_App.Application()
        try await app.connect()
    }

    override func tearDown() async throws {
        agent.stop()
    }

    // Test: App can trigger commit via IPC
    func testAppTriggersCommitViaIPC() async throws {
        let expectation = expectation(description: "IPC commit")

        try await app.requestCommit(message: "Test") { result in
            switch result {
            case .success:
                expectation.fulfill()
            case .failure(let error):
                XCTFail("Commit failed: \(error)")
            }
        }

        await fulfillment(of: [expectation], timeout: 5.0)
    }

    // Test: Agent notifies app of draft commits
    func testAgentNotifiesAppOfDraftCommits() async throws {
        let expectation = expectation(description: "Draft notification")

        app.onDraftCommit = { commit in
            XCTAssertTrue(commit.branch == "draft")
            expectation.fulfill()
        }

        // Trigger draft commit from agent
        try await agent.createDraftCommit()

        await fulfillment(of: [expectation], timeout: 5.0)
    }

    // Test: Lock status synchronization
    func testLockStatusSynchronization() async throws {
        let project = createTestProject()

        // Acquire lock from app
        try await app.acquireLock(project: project.path, user: "testuser")

        // Verify agent sees lock
        try await Task.sleep(nanoseconds: 500_000_000) // 0.5s for propagation
        let isLocked = agent.isProjectLocked(project.path)
        XCTAssertTrue(isLocked)
    }

    // Test: IPC reconnection after agent restart
    func testIPCReconnectionAfterAgentRestart() async throws {
        // Initial connection
        XCTAssertTrue(app.isConnected)

        // Restart agent
        agent.stop()
        try await Task.sleep(nanoseconds: 1_000_000_000)
        try agent.start()

        // Verify reconnection
        try await Task.sleep(nanoseconds: 2_000_000_000)
        XCTAssertTrue(app.isConnected)
    }
}
```

**Test Checklist**:
- [ ] App triggers commit via IPC
- [ ] Agent notifies app of draft commits
- [ ] Lock status synchronization
- [ ] IPC reconnection after crash
- [ ] Concurrent IPC requests from app
- [ ] IPC message ordering

---

### 2.3 Real Logic Pro Project Tests

**Test with actual .logicx projects**
**Estimated Time**: 2 days

```swift
class RealLogicProProjectTests: XCTestCase {
    var testProjects: [LogicProProject] = []

    override func setUp() async throws {
        // Load test fixtures
        testProjects = [
            loadProject("TinyProject.logicx"),      // < 100MB
            loadProject("MediumProject.logicx"),    // 1-5GB
            loadProject("LargeProject.logicx")      // 10-50GB
        ]
    }

    // Test: Initialize tiny project
    func testInitializeTinyProject() throws {
        let project = testProjects[0]
        try OxenService.shared.initRepository(at: project.url)

        // Verify .oxen directory
        XCTAssertTrue(project.hasOxenRepository())

        // Verify .oxenignore
        let oxenignore = try String(contentsOf: project.oxenignoreURL)
        XCTAssertTrue(oxenignore.contains("Bounces/"))
    }

    // Test: Commit medium project
    func testCommitMediumProject() async throws {
        let project = testProjects[1]
        try OxenService.shared.initRepository(at: project.url)

        let startTime = Date()
        try await OxenService.shared.commit(message: "Initial commit")
        let duration = Date().timeIntervalSince(startTime)

        // Should complete in reasonable time
        XCTAssertLessThan(duration, 60.0) // < 1 minute for 5GB
    }

    // Test: Rollback large project
    func testRollbackLargeProject() async throws {
        let project = testProjects[2]
        try OxenService.shared.initRepository(at: project.url)

        // Create two versions
        try await OxenService.shared.commit(message: "Version 1")
        project.modify() // Make some changes
        try await OxenService.shared.commit(message: "Version 2")

        // Rollback to version 1
        let commits = try OxenService.shared.getCommits()
        let version1 = commits.first { $0.message.contains("Version 1") }!

        let startTime = Date()
        try await OxenService.shared.checkout(commit: version1.id)
        let duration = Date().timeIntervalSince(startTime)

        // Should complete in reasonable time
        XCTAssertLessThan(duration, 300.0) // < 5 minutes for 50GB

        // Verify project state
        XCTAssertTrue(project.verifyIntegrity())
    }

    // Test: Detect corrupted project
    func testDetectCorruptedProject() throws {
        let corruptedProject = loadProject("CorruptedProject.logicx")

        XCTAssertThrowsError(try OxenService.shared.initRepository(at: corruptedProject.url)) { error in
            XCTAssertTrue(error is ProjectValidationError)
        }
    }

    // Test: Open project in Logic Pro after rollback
    func testOpenProjectInLogicAfterRollback() async throws {
        let project = testProjects[1]
        try OxenService.shared.initRepository(at: project.url)

        try await OxenService.shared.commit(message: "Version 1")
        project.modify()
        try await OxenService.shared.commit(message: "Version 2")

        // Rollback
        let commits = try OxenService.shared.getCommits()
        let version1 = commits.first { $0.message.contains("Version 1") }!
        try await OxenService.shared.checkout(commit: version1.id)

        // Attempt to open in Logic Pro
        // NOTE: This requires Logic Pro to be installed
        let success = try await openInLogicPro(project: project)
        XCTAssertTrue(success)
    }

    func openInLogicPro(project: LogicProProject) async throws -> Bool {
        let task = Process()
        task.executableURL = URL(fileURLWithPath: "/Applications/Logic Pro.app/Contents/MacOS/Logic Pro")
        task.arguments = [project.url.path]

        try task.run()

        // Wait a few seconds for Logic to open
        try await Task.sleep(nanoseconds: 10_000_000_000)

        // Check if Logic Pro is running
        let running = NSWorkspace.shared.runningApplications
            .contains { $0.bundleIdentifier == "com.apple.logic10" }

        // Close Logic Pro
        if running {
            let logicApp = NSWorkspace.shared.runningApplications
                .first { $0.bundleIdentifier == "com.apple.logic10" }
            logicApp?.terminate()
        }

        return running
    }
}
```

**Test Checklist**:
- [ ] Initialize tiny project (< 100MB)
- [ ] Initialize medium project (1-5GB)
- [ ] Initialize large project (10-50GB)
- [ ] Commit tiny project
- [ ] Commit medium project
- [ ] Commit large project
- [ ] Rollback tiny project
- [ ] Rollback medium project
- [ ] Rollback large project
- [ ] Detect corrupted project
- [ ] Open rolled-back project in Logic Pro
- [ ] Verify project integrity after rollback
- [ ] Test with various Logic Pro versions

---

## Phase 3: System Tests (Priority: High)

**Estimated Time**: 5-7 days

### 3.1 Long-Running Stability Tests

**Goal**: Verify daemon stability over extended periods

```bash
#!/bin/bash
# File: tests/system/long_running_test.sh

# Setup
PROJECT_DIR="$HOME/TestProjects/LongRunningTest.logicx"
LOG_FILE="$HOME/oxenvcs-stability-test.log"

echo "Starting 24-hour stability test..." | tee -a "$LOG_FILE"
echo "Project: $PROJECT_DIR" | tee -a "$LOG_FILE"
echo "Start time: $(date)" | tee -a "$LOG_FILE"

# Initialize project
oxenvcs-cli init "$PROJECT_DIR"

# Load LaunchAgent
launchctl load -w ~/Library/LaunchAgents/com.oxenvcs.agent.plist

# Monitor for 24 hours
START_TIME=$(date +%s)
DURATION=$((24 * 60 * 60)) # 24 hours
MODIFICATION_INTERVAL=300   # Modify every 5 minutes

while true; do
    CURRENT_TIME=$(date +%s)
    ELAPSED=$((CURRENT_TIME - START_TIME))

    if [ $ELAPSED -ge $DURATION ]; then
        echo "Test completed successfully!" | tee -a "$LOG_FILE"
        break
    fi

    # Modify project
    echo "Modification at $(date)" | tee -a "$LOG_FILE"
    echo "random data" >> "$PROJECT_DIR/test_file_$(date +%s).txt"

    # Check daemon status
    if ! launchctl list | grep -q com.oxenvcs.agent; then
        echo "ERROR: Daemon crashed at $(date)" | tee -a "$LOG_FILE"
        exit 1
    fi

    # Check memory usage
    MEMORY=$(ps -o rss= -p $(pgrep OxVCS-LaunchAgent))
    echo "Memory usage: $MEMORY KB" | tee -a "$LOG_FILE"

    if [ $MEMORY -gt 500000 ]; then  # > 500MB
        echo "WARNING: High memory usage at $(date)" | tee -a "$LOG_FILE"
    fi

    # Check CPU usage
    CPU=$(ps -o %cpu= -p $(pgrep OxVCS-LaunchAgent))
    echo "CPU usage: $CPU%" | tee -a "$LOG_FILE"

    # Wait before next modification
    sleep $MODIFICATION_INTERVAL
done

# Verify all changes were committed
COMMIT_COUNT=$(cd "$PROJECT_DIR" && oxenvcs-cli log --oneline | wc -l)
EXPECTED_COMMITS=$((DURATION / MODIFICATION_INTERVAL))

echo "Commits created: $COMMIT_COUNT" | tee -a "$LOG_FILE"
echo "Expected commits: ~$EXPECTED_COMMITS" | tee -a "$LOG_FILE"

if [ $COMMIT_COUNT -lt $((EXPECTED_COMMITS - 5)) ]; then
    echo "ERROR: Missing commits!" | tee -a "$LOG_FILE"
    exit 1
fi

echo "Test passed!" | tee -a "$LOG_FILE"
```

**Test Variations**:
- [ ] 8-hour continuous monitoring
- [ ] 24-hour continuous monitoring
- [ ] 72-hour continuous monitoring (weekend test)
- [ ] Monitor 3 projects simultaneously
- [ ] Monitor 10 projects simultaneously
- [ ] High-frequency modifications (every 30s)
- [ ] Low-frequency modifications (every hour)

**Success Criteria**:
- [ ] Daemon never crashes
- [ ] Memory usage stable (< 200MB)
- [ ] CPU usage < 5% average
- [ ] All modifications captured in commits
- [ ] No missed FSEvents
- [ ] No IPC connection drops

---

### 3.2 Power Management Tests

**Test power cycle scenarios**

```swift
class PowerManagementSystemTests: XCTestCase {

    // Test: Commit on sleep
    func testCommitOnSleep() async throws {
        let project = createTestProject()
        try OxenService.shared.initRepository(at: project.url)

        // Start monitoring
        let monitor = FSEventsMonitor()
        monitor.startMonitoring(path: project.url.path)

        // Modify project
        try "test".write(to: project.url.appendingPathComponent("test.txt"), atomically: true, encoding: .utf8)

        // Simulate sleep
        NotificationCenter.default.post(name: NSWorkspace.willSleepNotification, object: nil)

        // Wait for commit
        try await Task.sleep(nanoseconds: 2_000_000_000)

        // Verify commit created
        let commits = try OxenService.shared.getCommits()
        XCTAssertTrue(commits.first?.message.contains("sleep") ?? false)
    }

    // Manual test: Actual sleep/wake cycle
    func testActualSleepWakeCycle() async throws {
        // This test requires manual intervention
        print("1. Start test")
        print("2. Modify test project")
        print("3. Put Mac to sleep")
        print("4. Wake Mac")
        print("5. Verify commit was created")

        let project = createTestProject()
        try OxenService.shared.initRepository(at: project.url)

        print("Project ready at: \(project.url.path)")
        print("Press any key to continue after sleep/wake cycle...")

        // Wait for manual intervention
        _ = readLine()

        // Verify commit
        let commits = try OxenService.shared.getCommits()
        XCTAssertGreaterThan(commits.count, 0)
    }
}
```

**Manual Test Procedures**:

1. **Sleep/Wake Test**
   - [ ] Start LaunchAgent
   - [ ] Open Logic Pro project
   - [ ] Make changes
   - [ ] Put Mac to sleep (⌘⌥Power)
   - [ ] Wait 30 seconds
   - [ ] Wake Mac
   - [ ] Verify emergency commit created
   - [ ] Verify no data loss

2. **Power Off Test**
   - [ ] Start LaunchAgent
   - [ ] Open Logic Pro project
   - [ ] Make changes
   - [ ] Initiate shutdown
   - [ ] Verify emergency commit triggered
   - [ ] Cancel shutdown
   - [ ] Verify commit completed

3. **Battery Critical Test** (MacBook only)
   - [ ] Let battery drain to < 10%
   - [ ] Make project changes
   - [ ] Wait for low battery warning
   - [ ] Verify emergency commit
   - [ ] Plug in power

4. **Crash Recovery Test**
   - [ ] Force kill LaunchAgent (`kill -9`)
   - [ ] Verify uncommitted changes detected on restart
   - [ ] Verify recovery commit created

---

### 3.3 Multi-User Lock Tests

**Test collaboration scenarios**

```swift
class MultiUserLockTests: XCTestCase {
    var user1Service: OxenService!
    var user2Service: OxenService!
    var testProject: URL!

    override func setUp() async throws {
        testProject = createSharedTestProject()

        // Simulate two users
        user1Service = OxenService(username: "user1")
        user2Service = OxenService(username: "user2")
    }

    // Test: Exclusive lock acquisition
    func testExclusiveLockAcquisition() async throws {
        // User 1 acquires lock
        try await user1Service.acquireLock(project: testProject.path)

        // User 2 attempts to acquire lock
        XCTAssertThrowsError(try await user2Service.acquireLock(project: testProject.path)) { error in
            XCTAssertTrue(error is LockError)
        }
    }

    // Test: Lock release
    func testLockRelease() async throws {
        try await user1Service.acquireLock(project: testProject.path)
        try await user1Service.releaseLock(project: testProject.path)

        // User 2 should now be able to acquire
        XCTAssertNoThrow(try await user2Service.acquireLock(project: testProject.path))
    }

    // Test: Lock timeout
    func testLockTimeout() async throws {
        user1Service.lockTimeout = 5.0 // 5 seconds

        try await user1Service.acquireLock(project: testProject.path)

        // Wait for timeout
        try await Task.sleep(nanoseconds: 6_000_000_000)

        // User 2 should be able to acquire
        XCTAssertNoThrow(try await user2Service.acquireLock(project: testProject.path))
    }

    // Test: Lock stealing with force flag
    func testLockStealingWithForce() async throws {
        try await user1Service.acquireLock(project: testProject.path)

        // User 2 steals lock with force
        try await user2Service.acquireLock(project: testProject.path, force: true)

        // User 1 should no longer hold lock
        XCTAssertFalse(user1Service.holdsLock(project: testProject.path))
    }

    // Test: File permissions enforcement
    func testFilePermissionsEnforcement() async throws {
        try await user1Service.acquireLock(project: testProject.path)

        // Verify project is read-only for user2
        let projectFile = testProject.appendingPathComponent("projectData")

        XCTAssertThrowsError(try user2Service.write(to: projectFile)) { error in
            XCTAssertTrue(error is PermissionError)
        }
    }
}
```

**Manual Multi-User Test**:
1. [ ] Setup shared network drive
2. [ ] User 1 acquires lock
3. [ ] User 2 attempts to open project
4. [ ] Verify User 2 sees lock dialog
5. [ ] User 1 releases lock
6. [ ] User 2 acquires lock
7. [ ] Verify User 1 sees lock dialog

---

### 3.4 Performance Tests

**Measure system performance under load**

```swift
class PerformanceTests: XCTestCase {

    // Test: Commit performance scaling
    func testCommitPerformanceScaling() throws {
        let projectSizes = [
            ("tiny", 100_000_000),      // 100 MB
            ("small", 500_000_000),     // 500 MB
            ("medium", 2_000_000_000),  // 2 GB
            ("large", 10_000_000_000)   // 10 GB
        ]

        for (name, size) in projectSizes {
            let project = createTestProject(size: size)
            try OxenService.shared.initRepository(at: project.url)

            measure {
                try! OxenService.shared.commit(message: "Performance test - \(name)")
            }
        }
    }

    // Test: FSEvents latency
    func testFSEventsLatency() async throws {
        let monitor = FSEventsMonitor()
        var detectionTime: TimeInterval = 0
        let expectation = expectation(description: "Event detected")

        let testFile = tempDir.appendingPathComponent("test.txt")

        monitor.startMonitoring(path: tempDir.path) { _ in
            detectionTime = Date().timeIntervalSince(modificationTime)
            expectation.fulfill()
        }

        let modificationTime = Date()
        try "content".write(to: testFile, atomically: true, encoding: .utf8)

        await fulfillment(of: [expectation], timeout: 5.0)

        XCTAssertLessThan(detectionTime, 0.1) // < 100ms
    }

    // Test: Memory usage under load
    func testMemoryUsageUnderLoad() async throws {
        let monitor = FSEventsMonitor()
        monitor.startMonitoring(path: tempDir.path)

        let initialMemory = getMemoryUsage()

        // Create 1000 file modifications
        for i in 0..<1000 {
            let file = tempDir.appendingPathComponent("file_\(i).txt")
            try "content".write(to: file, atomically: true, encoding: .utf8)
            try await Task.sleep(nanoseconds: 10_000_000) // 10ms
        }

        try await Task.sleep(nanoseconds: 5_000_000_000) // Wait 5s

        let finalMemory = getMemoryUsage()
        let memoryGrowth = finalMemory - initialMemory

        XCTAssertLessThan(memoryGrowth, 50_000_000) // < 50MB growth
    }

    func getMemoryUsage() -> Int {
        var info = task_vm_info_data_t()
        var count = mach_msg_type_number_t(MemoryLayout<task_vm_info_data_t>.size) / 4

        let result = withUnsafeMutablePointer(to: &info) {
            $0.withMemoryRebound(to: integer_t.self, capacity: 1) {
                task_info(mach_task_self_, task_flavor_t(TASK_VM_INFO), $0, &count)
            }
        }

        guard result == KERN_SUCCESS else { return 0 }
        return Int(info.phys_footprint)
    }
}
```

**Performance Benchmarks**:
- [ ] Single file add: < 10ms
- [ ] Small project commit (100MB): < 5s
- [ ] Medium project commit (2GB): < 30s
- [ ] Large project commit (10GB): < 2min
- [ ] FSEvents detection latency: < 100ms
- [ ] IPC round-trip: < 50ms
- [ ] Memory usage idle: < 100MB
- [ ] Memory usage under load: < 200MB
- [ ] CPU usage idle: < 1%
- [ ] CPU usage active: < 10%

---

## Phase 4: User Acceptance Testing (Priority: Medium)

**Estimated Time**: 1 week

### 4.1 Beta User Testing

**Recruit 3-5 Logic Pro users for real-world testing**

**Test Plan for Beta Users**:

```markdown
# Beta Test Instructions - Oxen-VCS for Logic Pro

Thank you for participating in beta testing!

## Setup (Day 1)
1. Install Oxen-VCS application
2. Complete initialization wizard
3. Select an active Logic Pro project (< 2GB recommended)
4. Complete initial commit

## Week 1: Normal Usage
- Use Logic Pro as you normally would
- The system will auto-save your work in the background
- Try to notice any performance impact

**Daily Survey Questions**:
- Did you notice any performance issues?
- Did the daemon interfere with your workflow?
- Rate your confidence in the auto-save system (1-10)

## Week 2: Advanced Features
- Create 2-3 milestone commits with descriptive messages
- Try rolling back to a previous version
- Open the rolled-back project in Logic Pro

**Survey Questions**:
- Was the rollback process intuitive?
- Did the rolled-back project open correctly in Logic?
- Would you trust this system for production work?

## Week 3: Stress Testing
- Work on a larger project (5-10GB)
- Test power management (sleep/wake cycles)
- Try leaving the system running overnight

**Survey Questions**:
- Did you encounter any crashes or errors?
- How was the performance with larger projects?
- Any features you wish existed?

## Bug Reporting
Please report any issues to: [GitHub Issues URL]

Include:
- macOS version
- Logic Pro version
- Project size
- Steps to reproduce
- Console logs (if available)
```

**Success Criteria**:
- [ ] 90%+ users complete full 3-week test
- [ ] No data loss incidents
- [ ] Average satisfaction rating > 8/10
- [ ] < 5 critical bugs reported
- [ ] Performance acceptable to users

---

### 4.2 Workflow Validation Tests

**Validate key user workflows**

1. **New Project Workflow**
   - [ ] User creates new Logic Pro project
   - [ ] User initializes Oxen-VCS
   - [ ] First commit created automatically
   - [ ] User continues working normally

2. **Existing Project Migration**
   - [ ] User has existing Logic Pro project
   - [ ] User initializes Oxen-VCS
   - [ ] Full project state captured in initial commit
   - [ ] No data loss during migration

3. **Daily Work Workflow**
   - [ ] User opens project
   - [ ] Makes changes throughout day
   - [ ] Draft commits created automatically
   - [ ] User creates milestone commit at end of day
   - [ ] No noticeable performance impact

4. **Rollback Workflow**
   - [ ] User realizes mistake was made
   - [ ] Opens Oxen-VCS app
   - [ ] Browses commit history
   - [ ] Rolls back to previous version
   - [ ] Opens project in Logic Pro successfully
   - [ ] Continues working from that point

5. **Collaboration Workflow** (Phase 3)
   - [ ] User A acquires lock
   - [ ] User A makes changes
   - [ ] User A releases lock
   - [ ] User B acquires lock
   - [ ] User B makes changes
   - [ ] Both users sync successfully

---

## Phase 5: Stress & Edge Case Testing (Priority: Medium)

**Estimated Time**: 3-4 days

### 5.1 Stress Tests

```bash
#!/bin/bash
# File: tests/stress/rapid_modification_test.sh

PROJECT_DIR="$HOME/TestProjects/StressTest.logicx"

# Initialize project
oxenvcs-cli init "$PROJECT_DIR"

# Rapid modifications (10 per second for 1 minute)
for i in {1..600}; do
    echo "Modification $i" >> "$PROJECT_DIR/stress_test.txt"
    sleep 0.1
done

# Verify system handled load
if launchctl list | grep -q com.oxenvcs.agent; then
    echo "Daemon survived stress test"
else
    echo "ERROR: Daemon crashed"
    exit 1
fi

# Check commit count
COMMIT_COUNT=$(cd "$PROJECT_DIR" && oxenvcs-cli log --oneline | wc -l)
echo "Commits created: $COMMIT_COUNT"

# Should have debounced to ~2 commits (30s debounce)
if [ $COMMIT_COUNT -ge 1 ] && [ $COMMIT_COUNT -le 5 ]; then
    echo "Debouncing worked correctly"
else
    echo "WARNING: Unexpected commit count"
fi
```

**Stress Test Scenarios**:
- [ ] 600 modifications in 1 minute
- [ ] 10,000 file changes in project
- [ ] 100GB project commit
- [ ] 1000 commits in project history
- [ ] 10 simultaneous users
- [ ] Network interruption during push
- [ ] Disk full during commit
- [ ] Low memory conditions (< 1GB available)

---

### 5.2 Edge Case Tests

```swift
class EdgeCaseTests: XCTestCase {

    // Test: Project with spaces in path
    func testProjectWithSpacesInPath() throws {
        let projectPath = "/Users/test/Logic Projects/My Project With Spaces.logicx"
        let project = createTestProject(at: projectPath)

        XCTAssertNoThrow(try OxenService.shared.initRepository(at: project.url))
        XCTAssertNoThrow(try OxenService.shared.commit(message: "Test"))
    }

    // Test: Project with special characters
    func testProjectWithSpecialCharacters() throws {
        let projectPath = "/Users/test/Project-[Final]_v2.0_(2025).logicx"
        let project = createTestProject(at: projectPath)

        XCTAssertNoThrow(try OxenService.shared.initRepository(at: project.url))
    }

    // Test: Project on external drive
    func testProjectOnExternalDrive() throws {
        // Requires external drive mounted
        let externalDrive = URL(fileURLWithPath: "/Volumes/External/Project.logicx")
        guard FileManager.default.fileExists(atPath: "/Volumes/External") else {
            throw XCTSkip("External drive not available")
        }

        let project = createTestProject(at: externalDrive)
        XCTAssertNoThrow(try OxenService.shared.initRepository(at: project.url))
    }

    // Test: Project with symbolic links
    func testProjectWithSymbolicLinks() throws {
        let realPath = tempDir.appendingPathComponent("RealProject.logicx")
        let symlinkPath = tempDir.appendingPathComponent("LinkedProject.logicx")

        let project = createTestProject(at: realPath)
        try FileManager.default.createSymbolicLink(at: symlinkPath, withDestinationURL: realPath)

        XCTAssertNoThrow(try OxenService.shared.initRepository(at: symlinkPath))
    }

    // Test: Very long commit message
    func testVeryLongCommitMessage() throws {
        let longMessage = String(repeating: "A", count: 10000)
        try OxenService.shared.initRepository(at: testProject.url)

        XCTAssertNoThrow(try OxenService.shared.commit(message: longMessage))
    }

    // Test: Commit with no changes
    func testCommitWithNoChanges() throws {
        try OxenService.shared.initRepository(at: testProject.url)
        try OxenService.shared.commit(message: "Initial")

        // Attempt commit with no changes
        XCTAssertThrowsError(try OxenService.shared.commit(message: "Empty")) { error in
            XCTAssertTrue(error is OxenError)
        }
    }

    // Test: Rollback to non-existent commit
    func testRollbackToNonExistentCommit() throws {
        try OxenService.shared.initRepository(at: testProject.url)

        XCTAssertThrowsError(try OxenService.shared.checkout(commit: "fake_commit_id")) { error in
            XCTAssertTrue(error is OxenError)
        }
    }

    // Test: Initialize already initialized project
    func testInitializeAlreadyInitializedProject() throws {
        try OxenService.shared.initRepository(at: testProject.url)

        XCTAssertThrowsError(try OxenService.shared.initRepository(at: testProject.url)) { error in
            XCTAssertTrue(error is OxenError)
        }
    }
}
```

**Edge Cases to Test**:
- [ ] Project path with spaces
- [ ] Project path with special characters (@, #, $, etc.)
- [ ] Project on external drive
- [ ] Project on network share
- [ ] Project with symbolic links
- [ ] Very long file paths (> 255 chars)
- [ ] Project with 10,000+ audio files
- [ ] Commit message with emoji
- [ ] Commit message with unicode
- [ ] Empty commit attempt
- [ ] Rollback to non-existent commit
- [ ] Double initialization
- [ ] Lock acquisition without network
- [ ] Daemon start with no projects

---

## Test Execution Timeline

### Week 1: Unit Tests
- **Days 1-2**: Rust CLI validation on macOS
- **Days 3-4**: LaunchAgent unit tests
- **Days 5**: UI App unit tests

### Week 2: Integration Tests
- **Days 1-2**: E2E workflow tests
- **Days 3-4**: IPC communication tests
- **Day 5**: Real Logic Pro project tests

### Week 3: System Tests
- **Days 1-2**: Long-running stability tests (8-24 hours)
- **Day 3**: Power management tests
- **Days 4-5**: Multi-user and performance tests

### Week 4 (Optional): UAT & Stress
- **Days 1-3**: Beta user testing setup and monitoring
- **Days 4-5**: Stress and edge case testing

---

## Test Reporting

### Daily Test Report Template

```markdown
# Oxen-VCS Test Report - [Date]

## Environment
- macOS Version:
- Xcode Version:
- Logic Pro Version:
- Oxen CLI Version:

## Tests Executed
- Total Tests:
- Passed:
- Failed:
- Skipped:

## Failures
| Test Name | Component | Error | Severity |
|-----------|-----------|-------|----------|
| | | | |

## Performance Metrics
- Memory Usage:
- CPU Usage:
- Commit Latency:
- FSEvents Latency:

## Blockers
- [ ] List any blockers

## Notes
- Any observations or concerns

## Next Steps
- [ ] Action items for tomorrow
```

### Bug Report Template

```markdown
# Bug Report

## Title
[Short description]

## Environment
- macOS Version:
- Logic Pro Version:
- Project Size:

## Steps to Reproduce
1.
2.
3.

## Expected Behavior
[What should happen]

## Actual Behavior
[What actually happened]

## Logs
```
[Paste relevant logs]
```

## Screenshots
[If applicable]

## Severity
- [ ] Critical (data loss, crash)
- [ ] High (major functionality broken)
- [ ] Medium (minor functionality issue)
- [ ] Low (cosmetic, performance)

## Priority
- [ ] P0 (blocks release)
- [ ] P1 (should fix before release)
- [ ] P2 (can fix after release)
- [ ] P3 (nice to have)
```

---

## Success Criteria Summary

### Must Pass (P0) - MVP Readiness
- [ ] All Rust unit tests pass on macOS
- [ ] LaunchAgent unit tests > 70% coverage
- [ ] UI App unit tests > 50% coverage
- [ ] End-to-end draft commit workflow works
- [ ] End-to-end milestone commit workflow works
- [ ] End-to-end rollback workflow works
- [ ] Real Logic Pro projects initialize successfully
- [ ] Real Logic Pro projects commit successfully
- [ ] Real Logic Pro projects rollback successfully
- [ ] Rolled-back projects open in Logic Pro
- [ ] 8-hour stability test passes
- [ ] No data loss in any test
- [ ] Performance within acceptable ranges

### Should Pass (P1) - Production Ready
- [ ] All unit tests > 80% coverage
- [ ] 24-hour stability test passes
- [ ] Multi-project monitoring works
- [ ] Power management tests pass
- [ ] Lock management works correctly
- [ ] IPC communication reliable
- [ ] Performance benchmarks met
- [ ] Beta users report positive experience

### Nice to Have (P2) - Polish
- [ ] 72-hour stability test passes
- [ ] Stress tests all pass
- [ ] All edge cases handled
- [ ] Multi-user collaboration tested
- [ ] Memory usage optimized
- [ ] All performance targets exceeded

---

## Appendix: Test Execution Commands

### Build and Test Commands

```bash
# Build Rust CLI
cd OxVCS-CLI-Wrapper
cargo build --release
cargo test --release

# Run specific Rust test
cargo test test_oxen_subprocess_wrapper -- --exact --nocapture

# Build Swift LaunchAgent
cd ../OxVCS-LaunchAgent
xcodebuild -scheme OxVCS-LaunchAgent -configuration Release build

# Run Swift tests
xcodebuild test -scheme OxVCS-LaunchAgent

# Build UI App
cd ../OxVCS-App
xcodebuild -scheme OxVCS -configuration Release build

# Run UI tests
xcodebuild test -scheme OxVCS

# Run integration tests
cd ../tests/integration
cargo test --test e2e_workflow_tests
```

### Daemon Management

```bash
# Load daemon
launchctl load -w ~/Library/LaunchAgents/com.oxenvcs.agent.plist

# Unload daemon
launchctl unload ~/Library/LaunchAgents/com.oxenvcs.agent.plist

# Check daemon status
launchctl list | grep com.oxenvcs

# View daemon logs
log show --predicate 'process == "OxVCS-LaunchAgent"' --last 1h --style syslog

# Tail daemon logs in real-time
log stream --predicate 'process == "OxVCS-LaunchAgent"'
```

### Performance Monitoring

```bash
# Monitor memory usage
while true; do
    ps -o pid,rss,vsz,command -p $(pgrep OxVCS-LaunchAgent)
    sleep 5
done

# Monitor CPU usage
top -pid $(pgrep OxVCS-LaunchAgent)

# Memory leaks detection
leaks --atExit -- ./OxVCS-LaunchAgent

# Instruments profiling
instruments -t "Time Profiler" -D ~/profile.trace ./OxVCS-LaunchAgent
```

---

## Document Control

**Version**: 1.0
**Last Updated**: 2025-10-29
**Author**: Claude
**Status**: Ready for Execution

**Change Log**:
- 2025-10-29: Initial test plan created

**Next Review**: After Phase 1 completion

---

*This test plan should be updated as tests are executed and new requirements are discovered.*
