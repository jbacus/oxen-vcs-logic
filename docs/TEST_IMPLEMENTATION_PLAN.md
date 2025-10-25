# Test Implementation Plan

## Overview

This document provides a phased, prioritized plan for implementing the testing strategy outlined in `TESTING_STRATEGY.md`. The plan is designed to deliver incremental value, with each phase building on previous work.

**Total Estimated Effort**: 3-4 weeks (1 developer)
**Success Metrics**: 70-80% code coverage, CI/CD pipeline operational, all critical paths tested

---

## Phase 0: Foundation (Week 0 - 2 days)

**Goal**: Set up test infrastructure and CI/CD pipeline

### Deliverables

#### 0.1 GitHub Actions CI/CD Pipeline
**File**: `.github/workflows/test.yml`
**Effort**: 4 hours

```yaml
name: Test Suite

on:
  push:
    branches: [ main, develop, "claude/*" ]
  pull_request:
    branches: [ main, develop ]

jobs:
  rust-tests:
    name: Rust Tests (CLI Wrapper)
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true
          components: rustfmt, clippy

      - name: Cache cargo dependencies
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            OxVCS-CLI-Wrapper/target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Check formatting
        working-directory: OxVCS-CLI-Wrapper
        run: cargo fmt -- --check

      - name: Run clippy
        working-directory: OxVCS-CLI-Wrapper
        run: cargo clippy -- -D warnings

      - name: Run tests
        working-directory: OxVCS-CLI-Wrapper
        run: cargo test --verbose

      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin

      - name: Generate coverage
        working-directory: OxVCS-CLI-Wrapper
        run: cargo tarpaulin --out Xml --output-dir coverage

      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: ./OxVCS-CLI-Wrapper/coverage/cobertura.xml
          flags: rust-cli

  swift-daemon-tests:
    name: Swift Tests (LaunchAgent)
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v3

      - name: Select Xcode
        run: sudo xcode-select -s /Applications/Xcode.app

      - name: Run tests
        working-directory: OxVCS-LaunchAgent
        run: swift test --enable-code-coverage

      - name: Generate coverage report
        working-directory: OxVCS-LaunchAgent
        run: |
          xcrun llvm-cov export \
            -format="lcov" \
            .build/debug/OxVCS-LaunchAgentPackageTests.xctest/Contents/MacOS/OxVCS-LaunchAgentPackageTests \
            -instr-profile .build/debug/codecov/default.profdata > coverage.lcov

      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: ./OxVCS-LaunchAgent/coverage.lcov
          flags: swift-daemon

  swift-app-tests:
    name: Swift Tests (App)
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v3

      - name: Select Xcode
        run: sudo xcode-select -s /Applications/Xcode.app

      - name: Run tests
        working-directory: OxVCS-App
        run: swift test --enable-code-coverage

      - name: Generate coverage report
        working-directory: OxVCS-App
        run: |
          xcrun llvm-cov export \
            -format="lcov" \
            .build/debug/OxVCS-AppPackageTests.xctest/Contents/MacOS/OxVCS-AppPackageTests \
            -instr-profile .build/debug/codecov/default.profdata > coverage.lcov

      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: ./OxVCS-App/coverage.lcov
          flags: swift-app

  report-coverage:
    name: Coverage Report
    runs-on: ubuntu-latest
    needs: [rust-tests, swift-daemon-tests, swift-app-tests]
    steps:
      - name: Download coverage reports
        uses: actions/download-artifact@v3

      - name: Display coverage summary
        run: echo "Coverage reports uploaded to Codecov"
```

**Acceptance Criteria**:
- [ ] CI runs on every commit
- [ ] Tests run for all three components
- [ ] Coverage reports generated
- [ ] Status badges displayed

#### 0.2 Test Utilities and Fixtures
**Files**:
- `OxVCS-CLI-Wrapper/tests/common/mod.rs`
- `OxVCS-LaunchAgent/Tests/TestUtils/TestFixtures.swift`
- `OxVCS-App/Tests/TestUtils/MockXPCClient.swift`

**Effort**: 4 hours

See "Test Utility Templates" section below for complete code.

**Acceptance Criteria**:
- [ ] Shared test fixtures for Logic Pro projects
- [ ] Mock implementations for XPC
- [ ] Temporary directory management
- [ ] Cleanup utilities

#### 0.3 Documentation
**Files**:
- Update `CONTRIBUTING.md` with testing guidelines
- Add testing section to `README.md`

**Effort**: 2 hours

**Acceptance Criteria**:
- [ ] Clear instructions for running tests
- [ ] Documentation on writing new tests
- [ ] Coverage requirements documented

---

## Phase 1: Critical Path Testing (Week 1 - 3 days)

**Goal**: Test the most critical functionality to prevent data loss and system instability

**Priority**: Highest
**Estimated Effort**: 3 days

### 1.1 Lock Manager Tests (Already Complete ✅)

The Lock Manager already has comprehensive tests in `OxVCS-LaunchAgent/Tests/LockManagerTests.swift` with 14 test cases. No additional work needed.

### 1.2 Oxen Operations Tests (Rust)

**File**: `OxVCS-CLI-Wrapper/tests/oxen_ops_tests.rs`
**Effort**: 6 hours
**Coverage Target**: 80%

```rust
use oxvcs_cli_wrapper::oxen_ops;
use tempfile::TempDir;
use std::path::Path;

mod common;

#[tokio::test]
async fn test_init_creates_oxen_repository() {
    let temp = TempDir::new().unwrap();
    let project_path = temp.path().join("TestProject.logicx");
    std::fs::create_dir_all(&project_path).unwrap();

    let result = oxen_ops::init(&project_path).await;
    assert!(result.is_ok());

    // Verify .oxen directory exists
    assert!(project_path.join(".oxen").exists());
}

#[tokio::test]
async fn test_init_fails_on_invalid_path() {
    let result = oxen_ops::init(Path::new("/nonexistent/path")).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_add_stages_files() {
    let temp = common::TestFixture::new();
    oxen_ops::init(&temp.project_path).await.unwrap();

    // Create a test file
    temp.create_audio_file("test.wav", 1);

    let result = oxen_ops::add(&temp.project_path, ".").await;
    assert!(result.is_ok());

    // Verify file is staged
    let status = oxen_ops::status(&temp.project_path).await.unwrap();
    assert!(status.staged_files.len() > 0);
}

#[tokio::test]
async fn test_create_commit() {
    let temp = common::TestFixture::new();
    oxen_ops::init(&temp.project_path).await.unwrap();
    temp.create_audio_file("test.wav", 1);
    oxen_ops::add(&temp.project_path, ".").await.unwrap();

    let commit = oxen_ops::create_commit(
        &temp.project_path,
        "Test commit message"
    ).await.unwrap();

    assert!(!commit.hash.is_empty());
    assert_eq!(commit.message, "Test commit message");
}

#[tokio::test]
async fn test_get_history() {
    let temp = common::TestFixture::new();
    oxen_ops::init(&temp.project_path).await.unwrap();

    // Create multiple commits
    for i in 0..3 {
        temp.create_audio_file(&format!("test{}.wav", i), 1);
        oxen_ops::add(&temp.project_path, ".").await.unwrap();
        oxen_ops::create_commit(&temp.project_path, &format!("Commit {}", i)).await.unwrap();
    }

    let history = oxen_ops::get_history(&temp.project_path, None).await.unwrap();
    assert_eq!(history.len(), 3);
    assert_eq!(history[0].message, "Commit 2");
    assert_eq!(history[2].message, "Commit 0");
}

#[tokio::test]
async fn test_restore_reverts_files() {
    let temp = common::TestFixture::new();
    oxen_ops::init(&temp.project_path).await.unwrap();

    // Initial commit
    temp.create_audio_file("test.wav", 1);
    oxen_ops::add(&temp.project_path, ".").await.unwrap();
    let first_commit = oxen_ops::create_commit(&temp.project_path, "First").await.unwrap();

    // Modify and commit
    std::fs::write(temp.project_path.join("Media/test.wav"), b"modified").unwrap();
    oxen_ops::add(&temp.project_path, ".").await.unwrap();
    oxen_ops::create_commit(&temp.project_path, "Second").await.unwrap();

    // Restore to first commit
    let result = oxen_ops::restore(&temp.project_path, &first_commit.hash).await;
    assert!(result.is_ok());

    // Verify file content restored
    let content = std::fs::read(temp.project_path.join("Media/test.wav")).unwrap();
    assert_ne!(content, b"modified");
}

#[tokio::test]
async fn test_status_shows_untracked_files() {
    let temp = common::TestFixture::new();
    oxen_ops::init(&temp.project_path).await.unwrap();

    temp.create_audio_file("untracked.wav", 1);

    let status = oxen_ops::status(&temp.project_path).await.unwrap();
    assert!(status.untracked_files.iter().any(|f| f.contains("untracked.wav")));
}

#[tokio::test]
async fn test_concurrent_operations_are_safe() {
    let temp = common::TestFixture::new();
    oxen_ops::init(&temp.project_path).await.unwrap();

    // Spawn multiple concurrent add operations
    let mut handles = vec![];
    for i in 0..10 {
        let path = temp.project_path.clone();
        let handle = tokio::spawn(async move {
            temp.create_audio_file(&format!("file{}.wav", i), 1);
            oxen_ops::add(&path, ".").await
        });
        handles.push(handle);
    }

    // All should complete without errors
    for handle in handles {
        assert!(handle.await.unwrap().is_ok());
    }
}
```

**Test Cases**:
- [x] Repository initialization
- [x] File staging (add)
- [x] Commit creation
- [x] Commit history retrieval
- [x] File restoration (rollback)
- [x] Status checking
- [x] Error handling (invalid paths)
- [x] Concurrent operations

**Acceptance Criteria**:
- [ ] All oxen_ops.rs functions tested
- [ ] Error cases covered
- [ ] 80%+ code coverage
- [ ] Tests pass in CI

### 1.3 Logic Project Detection Tests (Rust)

**File**: `OxVCS-CLI-Wrapper/tests/logic_project_tests.rs`
**Effort**: 3 hours
**Coverage Target**: 85%

```rust
use oxvcs_cli_wrapper::logic_project::{is_logic_project, get_project_info, ProjectInfo};
use tempfile::TempDir;
use std::fs;

#[test]
fn test_detects_valid_logic_project() {
    let temp = TempDir::new().unwrap();
    let project = temp.path().join("Test.logicx");

    // Create valid Logic Pro structure
    fs::create_dir_all(&project).unwrap();
    fs::create_dir_all(project.join("Alternatives")).unwrap();

    assert!(is_logic_project(&project).unwrap());
}

#[test]
fn test_rejects_directory_without_alternatives() {
    let temp = TempDir::new().unwrap();
    let project = temp.path().join("Test.logicx");
    fs::create_dir_all(&project).unwrap();
    // Missing Alternatives directory

    assert!(!is_logic_project(&project).unwrap());
}

#[test]
fn test_rejects_non_logicx_extension() {
    let temp = TempDir::new().unwrap();
    let project = temp.path().join("Test.logic"); // Wrong extension
    fs::create_dir_all(&project).unwrap();
    fs::create_dir_all(project.join("Alternatives")).unwrap();

    assert!(!is_logic_project(&project).unwrap());
}

#[test]
fn test_extracts_project_metadata() {
    let temp = TempDir::new().unwrap();
    let project = temp.path().join("Test.logicx");
    fs::create_dir_all(&project).unwrap();
    fs::create_dir_all(project.join("Alternatives")).unwrap();

    // Create projectData with metadata
    let xml = r#"
    <?xml version="1.0" encoding="UTF-8"?>
    <project>
        <tempo>128</tempo>
        <sampleRate>48000</sampleRate>
        <key>Am</key>
    </project>
    "#;
    fs::write(project.join("projectData"), xml).unwrap();

    let info = get_project_info(&project).unwrap();
    assert_eq!(info.bpm, Some(128));
    assert_eq!(info.sample_rate, Some(48000));
    assert_eq!(info.key_signature, Some("Am".to_string()));
}

#[test]
fn test_handles_missing_metadata_gracefully() {
    let temp = TempDir::new().unwrap();
    let project = temp.path().join("Test.logicx");
    fs::create_dir_all(&project).unwrap();
    fs::create_dir_all(project.join("Alternatives")).unwrap();

    let info = get_project_info(&project).unwrap();
    assert_eq!(info.bpm, None);
    assert_eq!(info.sample_rate, None);
}

#[test]
fn test_handles_malformed_xml() {
    let temp = TempDir::new().unwrap();
    let project = temp.path().join("Test.logicx");
    fs::create_dir_all(&project).unwrap();
    fs::create_dir_all(project.join("Alternatives")).unwrap();

    fs::write(project.join("projectData"), "not valid xml").unwrap();

    // Should not crash, return None for metadata
    let info = get_project_info(&project).unwrap();
    assert!(info.bpm.is_none());
}
```

**Test Cases**:
- [x] Valid Logic Pro project detection
- [x] Invalid project rejection
- [x] Metadata extraction (BPM, sample rate, key)
- [x] Missing metadata handling
- [x] Malformed XML handling
- [x] Edge cases (symlinks, permissions)

**Acceptance Criteria**:
- [ ] All is_logic_project and get_project_info branches tested
- [ ] Error handling verified
- [ ] 85%+ code coverage

### 1.4 Commit Metadata Tests (Rust)

**File**: `OxVCS-CLI-Wrapper/tests/commit_metadata_tests.rs`
**Effort**: 2 hours

```rust
use oxvcs_cli_wrapper::commit_metadata::CommitMetadata;

#[test]
fn test_formats_commit_message_with_all_metadata() {
    let metadata = CommitMetadata {
        bpm: Some(120),
        sample_rate: Some(48000),
        key_signature: Some("C".to_string()),
        tags: vec!["mix".to_string(), "final".to_string()],
        notes: Some("Ready for mastering".to_string()),
    };

    let message = metadata.format_commit_message("Milestone commit");

    assert!(message.contains("Milestone commit"));
    assert!(message.contains("BPM: 120"));
    assert!(message.contains("Sample Rate: 48000"));
    assert!(message.contains("Key: C"));
    assert!(message.contains("Tags: mix, final"));
    assert!(message.contains("Ready for mastering"));
}

#[test]
fn test_formats_commit_message_with_minimal_metadata() {
    let metadata = CommitMetadata {
        bpm: None,
        sample_rate: None,
        key_signature: None,
        tags: vec![],
        notes: None,
    };

    let message = metadata.format_commit_message("Simple commit");

    assert_eq!(message.trim(), "Simple commit");
}

#[test]
fn test_parses_commit_message() {
    let commit_msg = r#"
    Mix version 3

    BPM: 140
    Sample Rate: 96000
    Key: Dm
    Tags: mix, draft

    Notes: Needs vocal editing
    "#;

    let metadata = CommitMetadata::parse(commit_msg).unwrap();

    assert_eq!(metadata.bpm, Some(140));
    assert_eq!(metadata.sample_rate, Some(96000));
    assert_eq!(metadata.key_signature, Some("Dm".to_string()));
    assert_eq!(metadata.tags.len(), 2);
    assert!(metadata.notes.is_some());
}

#[test]
fn test_serializes_to_json() {
    let metadata = CommitMetadata {
        bpm: Some(128),
        sample_rate: Some(44100),
        key_signature: Some("G".to_string()),
        tags: vec!["demo".to_string()],
        notes: None,
    };

    let json = serde_json::to_string(&metadata).unwrap();
    assert!(json.contains("\"bpm\":128"));
}
```

**Acceptance Criteria**:
- [ ] Metadata formatting tested
- [ ] Metadata parsing tested
- [ ] JSON serialization tested
- [ ] 90%+ code coverage

### 1.5 Power Management Tests (Swift)

**File**: `OxVCS-LaunchAgent/Tests/PowerManagementTests.swift`
**Effort**: 4 hours

```swift
import XCTest
@testable import OxVCS_LaunchAgent

final class PowerManagementTests: XCTestCase {
    var powerManager: PowerManagement!
    var testProject: URL!
    var commitCalled: Bool = false

    override func setUp() {
        super.setUp()
        testProject = TestFixtures.createLogicProject()
        commitCalled = false

        powerManager = PowerManagement()
        powerManager.onPowerEvent = { [weak self] event in
            if event == .willSleep {
                self?.commitCalled = true
            }
        }
    }

    override func tearDown() {
        TestFixtures.cleanup(testProject)
        super.tearDown()
    }

    func testWillSleepNotificationTriggersCallback() {
        let expectation = self.expectation(description: "Power callback")

        powerManager.onPowerEvent = { event in
            XCTAssertEqual(event, .willSleep)
            expectation.fulfill()
        }

        powerManager.start()

        // Simulate system sleep notification
        NotificationCenter.default.post(
            name: NSWorkspace.willSleepNotification,
            object: nil
        )

        wait(for: [expectation], timeout: 1.0)
    }

    func testWillPowerOffNotificationTriggersCallback() {
        let expectation = self.expectation(description: "Shutdown callback")

        powerManager.onPowerEvent = { event in
            XCTAssertEqual(event, .willPowerOff)
            expectation.fulfill()
        }

        powerManager.start()

        // Simulate shutdown notification
        NotificationCenter.default.post(
            name: NSWorkspace.willPowerOffNotification,
            object: nil
        )

        wait(for: [expectation], timeout: 1.0)
    }

    func testDidWakeNotificationReceived() {
        let expectation = self.expectation(description: "Wake callback")

        powerManager.onPowerEvent = { event in
            if event == .didWake {
                expectation.fulfill()
            }
        }

        powerManager.start()

        NotificationCenter.default.post(
            name: NSWorkspace.didWakeNotification,
            object: nil
        )

        wait(for: [expectation], timeout: 1.0)
    }

    func testStopRemovesObservers() {
        powerManager.start()
        powerManager.stop()

        // Post notification after stop
        NotificationCenter.default.post(
            name: NSWorkspace.willSleepNotification,
            object: nil
        )

        // Callback should NOT be called
        XCTAssertFalse(commitCalled)
    }
}
```

**Acceptance Criteria**:
- [ ] Sleep notification handling tested
- [ ] Shutdown notification handling tested
- [ ] Wake notification handling tested
- [ ] Observer cleanup tested
- [ ] 75%+ code coverage

---

## Phase 2: Core Component Testing (Week 1-2 - 4 days)

**Goal**: Comprehensive unit tests for all core modules

**Priority**: High
**Estimated Effort**: 4 days

### 2.1 FSEvents Monitor Tests (Swift)

**File**: `OxVCS-LaunchAgent/Tests/FSEventsMonitorTests.swift`
**Effort**: 6 hours

```swift
import XCTest
@testable import OxVCS_LaunchAgent

final class FSEventsMonitorTests: XCTestCase {
    var monitor: FSEventsMonitor!
    var testDirectory: URL!
    var callbackCount: Int = 0

    override func setUp() {
        super.setUp()
        testDirectory = TestFixtures.createTempDirectory()
        callbackCount = 0

        monitor = FSEventsMonitor(projectPaths: [testDirectory.path])
    }

    override func tearDown() {
        monitor.stop()
        TestFixtures.cleanup(testDirectory)
        super.tearDown()
    }

    func testDetectsFileCreation() {
        let expectation = self.expectation(description: "File creation detected")

        monitor.onFileChanged = { path in
            XCTAssertTrue(path.contains(self.testDirectory.path))
            expectation.fulfill()
        }

        monitor.start()

        // Create a file
        let testFile = testDirectory.appendingPathComponent("test.txt")
        try! "test content".write(to: testFile, atomically: true, encoding: .utf8)

        wait(for: [expectation], timeout: 3.0)
    }

    func testDetectsFileModification() {
        let testFile = testDirectory.appendingPathComponent("test.txt")
        try! "initial".write(to: testFile, atomically: true, encoding: .utf8)

        let expectation = self.expectation(description: "File modification detected")

        monitor.onFileChanged = { path in
            expectation.fulfill()
        }

        monitor.start()

        // Modify the file
        try! "modified".write(to: testFile, atomically: true, encoding: .utf8)

        wait(for: [expectation], timeout: 3.0)
    }

    func testDebouncesPrevientsMultipleCallbacks() {
        let expectation = self.expectation(description: "Single callback after debounce")
        expectation.expectedFulfillmentCount = 1
        expectation.assertForOverFulfill = true

        monitor.debounceInterval = 1.0 // 1 second debounce
        monitor.onFileChanged = { _ in
            expectation.fulfill()
        }

        monitor.start()

        // Make multiple rapid changes
        let testFile = testDirectory.appendingPathComponent("test.txt")
        for i in 0..<10 {
            try! "content \(i)".write(to: testFile, atomically: true, encoding: .utf8)
            usleep(100_000) // 100ms between writes
        }

        // Should only get ONE callback after debounce
        wait(for: [expectation], timeout: 5.0)
    }

    func testIgnoresSystemFiles() {
        let expectation = self.expectation(description: "System files ignored")
        expectation.isInverted = true

        monitor.onFileChanged = { path in
            // Should not be called for system files
            expectation.fulfill()
        }

        monitor.start()

        // Write .DS_Store (should be ignored)
        let dsStore = testDirectory.appendingPathComponent(".DS_Store")
        try! Data().write(to: dsStore)

        wait(for: [expectation], timeout: 2.0)
    }

    func testIgnoresOxenDirectory() {
        let expectation = self.expectation(description: ".oxen ignored")
        expectation.isInverted = true

        monitor.onFileChanged = { _ in
            expectation.fulfill()
        }

        monitor.start()

        // Create .oxen directory and file
        let oxenDir = testDirectory.appendingPathComponent(".oxen")
        try! FileManager.default.createDirectory(at: oxenDir, withIntermediateDirectories: true)
        try! "data".write(to: oxenDir.appendingPathComponent("index"), atomically: true, encoding: .utf8)

        wait(for: [expectation], timeout: 2.0)
    }

    func testMonitorsMultipleProjects() {
        let project1 = TestFixtures.createTempDirectory()
        let project2 = TestFixtures.createTempDirectory()

        monitor = FSEventsMonitor(projectPaths: [project1.path, project2.path])

        let expectation1 = self.expectation(description: "Project 1 change")
        let expectation2 = self.expectation(description: "Project 2 change")

        var changedPaths: Set<String> = []
        monitor.onFileChanged = { path in
            changedPaths.insert(path)
            if changedPaths.count == 2 {
                expectation1.fulfill()
                expectation2.fulfill()
            }
        }

        monitor.start()

        // Change files in both projects
        try! "test1".write(to: project1.appendingPathComponent("file1.txt"), atomically: true, encoding: .utf8)
        try! "test2".write(to: project2.appendingPathComponent("file2.txt"), atomically: true, encoding: .utf8)

        wait(for: [expectation1, expectation2], timeout: 5.0)

        TestFixtures.cleanup(project1)
        TestFixtures.cleanup(project2)
    }

    func testStopHaltsMonitoring() {
        let expectation = self.expectation(description: "No callbacks after stop")
        expectation.isInverted = true

        monitor.onFileChanged = { _ in
            expectation.fulfill()
        }

        monitor.start()
        monitor.stop()

        // Make changes after stopping
        let testFile = testDirectory.appendingPathComponent("test.txt")
        try! "content".write(to: testFile, atomically: true, encoding: .utf8)

        wait(for: [expectation], timeout: 2.0)
    }
}
```

**Acceptance Criteria**:
- [ ] File creation/modification detection
- [ ] Debounce mechanism tested
- [ ] System file filtering tested
- [ ] Multi-project monitoring tested
- [ ] Start/stop tested
- [ ] 80%+ code coverage

### 2.2 Commit Orchestrator Tests (Swift)

**File**: `OxVCS-LaunchAgent/Tests/CommitOrchestratorTests.swift`
**Effort**: 6 hours

```swift
import XCTest
@testable import OxVCS_LaunchAgent

final class CommitOrchestratorTests: XCTestCase {
    var orchestrator: CommitOrchestrator!
    var testProject: URL!
    var mockCLI: MockCLIWrapper!

    override func setUp() {
        super.setUp()
        testProject = TestFixtures.createLogicProject()
        mockCLI = MockCLIWrapper()
        orchestrator = CommitOrchestrator(cliWrapper: mockCLI)
    }

    override func tearDown() {
        TestFixtures.cleanup(testProject)
        super.tearDown()
    }

    func testSchedulesCommitAfterFileChange() {
        orchestrator.handleFileChange(at: testProject.path)

        XCTAssertTrue(orchestrator.hasPendingCommit(for: testProject.path))
    }

    func testDebouncesMergesMultipleChanges() {
        // Multiple rapid changes
        for _ in 0..<5 {
            orchestrator.handleFileChange(at: testProject.path)
            usleep(100_000) // 100ms
        }

        // Should only schedule ONE commit
        XCTAssertEqual(orchestrator.pendingCommitsCount(), 1)
    }

    func testExecutesCommitAfterDebounce() async {
        let expectation = self.expectation(description: "Commit executed")

        mockCLI.onCommit = {
            expectation.fulfill()
        }

        orchestrator.debounceInterval = 1.0 // 1 second
        orchestrator.handleFileChange(at: testProject.path)

        await fulfillment(of: [expectation], timeout: 3.0)
        XCTAssertTrue(mockCLI.commitCalled)
    }

    func testHandlesMultipleProjectsIndependently() async {
        let project2 = TestFixtures.createLogicProject()
        defer { TestFixtures.cleanup(project2) }

        orchestrator.handleFileChange(at: testProject.path)
        orchestrator.handleFileChange(at: project2.path)

        // Should have two separate pending commits
        XCTAssertTrue(orchestrator.hasPendingCommit(for: testProject.path))
        XCTAssertTrue(orchestrator.hasPendingCommit(for: project2.path))
    }

    func testCancelsCommitOnStop() {
        orchestrator.handleFileChange(at: testProject.path)
        XCTAssertTrue(orchestrator.hasPendingCommit(for: testProject.path))

        orchestrator.cancelPendingCommits()

        XCTAssertFalse(orchestrator.hasPendingCommit(for: testProject.path))
    }

    func testPowerEventTriggersImmediateCommit() async {
        let expectation = self.expectation(description: "Immediate commit")

        mockCLI.onCommit = {
            expectation.fulfill()
        }

        orchestrator.handleFileChange(at: testProject.path)

        // Trigger power event (should commit immediately, no debounce)
        orchestrator.handlePowerEvent(.willSleep)

        await fulfillment(of: [expectation], timeout: 2.0)
    }
}

// Mock CLI Wrapper for testing
class MockCLIWrapper: CLIWrapperProtocol {
    var commitCalled = false
    var onCommit: (() -> Void)?

    func createAutoCommit(at path: String) async throws {
        commitCalled = true
        onCommit?()
    }
}
```

**Acceptance Criteria**:
- [ ] Debounce logic tested
- [ ] Multi-project handling tested
- [ ] Power event handling tested
- [ ] Commit execution tested
- [ ] 75%+ code coverage

### 2.3 Draft Manager Tests (Rust)

**File**: `OxVCS-CLI-Wrapper/tests/draft_manager_tests.rs`
**Effort**: 4 hours

```rust
use oxvcs_cli_wrapper::draft_manager::{DraftManager, create_draft_branch, switch_to_draft, prune_old_drafts};
use tempfile::TempDir;

mod common;

#[tokio::test]
async fn test_create_draft_branch() {
    let temp = common::TestFixture::new();
    oxen_ops::init(&temp.project_path).await.unwrap();

    let branch_name = create_draft_branch(&temp.project_path).await.unwrap();

    assert!(branch_name.starts_with("draft-"));
    assert!(branch_name.contains("-auto"));
}

#[tokio::test]
async fn test_switch_to_draft() {
    let temp = common::TestFixture::new();
    oxen_ops::init(&temp.project_path).await.unwrap();

    let branch = create_draft_branch(&temp.project_path).await.unwrap();
    let result = switch_to_draft(&temp.project_path, &branch).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_prune_old_drafts() {
    let temp = common::TestFixture::new();
    oxen_ops::init(&temp.project_path).await.unwrap();

    // Create multiple draft branches
    for i in 0..5 {
        let branch = format!("draft-2024-01-{:02}-auto", i+1);
        create_draft_branch(&temp.project_path).await.unwrap();
    }

    // Prune keeping only 3 most recent
    prune_old_drafts(&temp.project_path, 3).await.unwrap();

    let branches = list_branches(&temp.project_path).await.unwrap();
    let draft_count = branches.iter().filter(|b| b.starts_with("draft-")).count();
    assert_eq!(draft_count, 3);
}
```

**Acceptance Criteria**:
- [ ] Draft branch creation tested
- [ ] Branch switching tested
- [ ] Pruning logic tested
- [ ] 70%+ code coverage

---

## Phase 3: UI Component Testing (Week 2-3 - 4 days)

**Goal**: Test ViewModels and UI logic

### 3.1 ViewModel Tests (Swift)

**Files**:
- `OxVCS-App/Tests/ViewModels/ProjectListViewModelTests.swift` (4 hours)
- `OxVCS-App/Tests/ViewModels/ProjectDetailViewModelTests.swift` (4 hours)

```swift
// ProjectListViewModelTests.swift
import XCTest
@testable import OxVCS_App

final class ProjectListViewModelTests: XCTestCase {
    var viewModel: ProjectListViewModel!
    var mockXPC: MockOxenDaemonXPCClient!

    override func setUp() {
        super.setUp()
        mockXPC = MockOxenDaemonXPCClient()
        viewModel = ProjectListViewModel(xpcClient: mockXPC)
    }

    func testLoadProjectsFromDaemon() async {
        mockXPC.mockProjects = [
            Project(path: "/path/1", name: "Project 1", status: .monitoring),
            Project(path: "/path/2", name: "Project 2", status: .idle)
        ]

        await viewModel.loadProjects()

        XCTAssertEqual(viewModel.projects.count, 2)
        XCTAssertEqual(viewModel.projects[0].name, "Project 1")
    }

    func testRefreshUpdatesStatus() async {
        mockXPC.mockProjects = [
            Project(path: "/path/1", name: "Project 1", status: .idle)
        ]

        await viewModel.loadProjects()
        XCTAssertEqual(viewModel.projects[0].status, .idle)

        // Update mock
        mockXPC.mockProjects[0].status = .committing
        await viewModel.refresh()

        XCTAssertEqual(viewModel.projects[0].status, .committing)
    }

    func testAddProjectCallsDaemon() async {
        await viewModel.addProject(path: "/path/to/project")

        XCTAssertTrue(mockXPC.addProjectCalled)
        XCTAssertEqual(mockXPC.lastAddedPath, "/path/to/project")
    }

    func testHandlesLoadError() async {
        mockXPC.shouldFail = true

        await viewModel.loadProjects()

        XCTAssertTrue(viewModel.hasError)
        XCTAssertNotNil(viewModel.errorMessage)
    }
}
```

**Acceptance Criteria**:
- [ ] ProjectListViewModel: 80%+ coverage
- [ ] ProjectDetailViewModel: 80%+ coverage
- [ ] All async operations tested
- [ ] Error handling tested

### 3.2 Model Tests (Swift)

**File**: `OxVCS-App/Tests/Models/ProjectTests.swift`
**Effort**: 2 hours

```swift
import XCTest
@testable import OxVCS_App

final class ProjectTests: XCTestCase {
    func testProjectInitialization() {
        let project = Project(
            path: "/Users/test/MyProject.logicx",
            name: "MyProject",
            status: .monitoring
        )

        XCTAssertEqual(project.name, "MyProject")
        XCTAssertEqual(project.status, .monitoring)
    }

    func testProjectDecodingFromJSON() throws {
        let json = """
        {
            "path": "/path/to/project",
            "name": "Test Project",
            "status": "idle"
        }
        """

        let data = json.data(using: .utf8)!
        let project = try JSONDecoder().decode(Project.self, from: data)

        XCTAssertEqual(project.name, "Test Project")
        XCTAssertEqual(project.status, .idle)
    }

    func testProjectEncodingToJSON() throws {
        let project = Project(path: "/path", name: "Test", status: .monitoring)

        let data = try JSONEncoder().encode(project)
        let json = String(data: data, encoding: .utf8)!

        XCTAssertTrue(json.contains("\"name\":\"Test\""))
    }
}
```

**Acceptance Criteria**:
- [ ] All model properties tested
- [ ] Codable conformance tested
- [ ] 90%+ coverage

### 3.3 XPC Client Tests (Swift)

**File**: `OxVCS-App/Tests/Services/OxenDaemonXPCClientTests.swift`
**Effort**: 4 hours

```swift
import XCTest
@testable import OxVCS_App

final class OxenDaemonXPCClientTests: XCTestCase {
    var client: OxenDaemonXPCClient!
    var mockConnection: MockXPCConnection!

    override func setUp() {
        super.setUp()
        mockConnection = MockXPCConnection()
        client = OxenDaemonXPCClient(connection: mockConnection)
    }

    func testGetMonitoredProjects() async throws {
        mockConnection.mockResponse = [
            ["path": "/path/1", "name": "Project 1", "status": "monitoring"]
        ]

        let projects = try await client.getMonitoredProjects()

        XCTAssertEqual(projects.count, 1)
        XCTAssertEqual(projects[0].name, "Project 1")
    }

    func testAddProjectSendsCorrectMessage() async throws {
        try await client.addProject("/path/to/project")

        XCTAssertTrue(mockConnection.messageSent)
        XCTAssertEqual(mockConnection.lastMessage, "addProject")
    }

    func testHandlesConnectionError() async {
        mockConnection.shouldFail = true

        do {
            _ = try await client.getMonitoredProjects()
            XCTFail("Should have thrown error")
        } catch {
            // Expected
        }
    }
}
```

**Acceptance Criteria**:
- [ ] All XPC methods tested
- [ ] Error handling tested
- [ ] Mock XPC connection used
- [ ] 70%+ coverage

---

## Phase 4: Integration and E2E Tests (Week 3-4 - 3 days)

**Goal**: Test cross-component interactions and complete workflows

### 4.1 Integration Tests

**File**: `Tests/Integration/WorkflowIntegrationTests.swift`
**Effort**: 8 hours

```swift
import XCTest

final class WorkflowIntegrationTests: XCTestCase {
    func testCompleteAutoCommitWorkflow() async throws {
        // 1. Initialize project
        let project = TestFixtures.createLogicProject()
        try await CLIWrapper.init(project.path)

        // 2. Start daemon
        let daemon = TestDaemon()
        daemon.start()
        daemon.addProject(project.path)

        // 3. Modify project
        let audioFile = project.appendingPathComponent("Media/new.wav")
        try Data(count: 1024).write(to: audioFile)

        // 4. Wait for FSEvents + debounce + commit
        try await Task.sleep(nanoseconds: 5_000_000_000)

        // 5. Verify commit created
        let commits = try await CLIWrapper.getHistory(project.path)
        XCTAssertTrue(commits.contains { $0.message.contains("Auto-draft") })

        TestFixtures.cleanup(project)
    }

    func testCompleteMilestoneCommitWorkflow() async throws {
        let project = TestFixtures.createLogicProject()
        try await CLIWrapper.init(project.path)

        // Create milestone via UI
        let metadata = CommitMetadata(bpm: 120, sampleRate: 48000, key: "C")
        try await CLIWrapper.createMilestoneCommit(
            at: project.path,
            message: "Final mix",
            metadata: metadata
        )

        // Verify commit has metadata
        let commits = try await CLIWrapper.getHistory(project.path)
        let milestone = commits.first!
        XCTAssertTrue(milestone.message.contains("Final mix"))
        XCTAssertTrue(milestone.message.contains("BPM: 120"))

        TestFixtures.cleanup(project)
    }

    func testCompleteRollbackWorkflow() async throws {
        let project = TestFixtures.createLogicProject()
        try await CLIWrapper.init(project.path)

        // Initial commit
        let file1 = project.appendingPathComponent("Media/v1.wav")
        try "version 1".data(using: .utf8)!.write(to: file1)
        try await CLIWrapper.add(project.path, ".")
        let commit1 = try await CLIWrapper.commit(project.path, "Version 1")

        // Second commit
        try "version 2".data(using: .utf8)!.write(to: file1)
        try await CLIWrapper.add(project.path, ".")
        try await CLIWrapper.commit(project.path, "Version 2")

        // Rollback
        try await CLIWrapper.restore(project.path, commit1.hash)

        // Verify restored
        let content = try String(contentsOf: file1)
        XCTAssertEqual(content, "version 1")

        TestFixtures.cleanup(project)
    }
}
```

**Acceptance Criteria**:
- [ ] Auto-commit workflow tested end-to-end
- [ ] Milestone commit workflow tested
- [ ] Rollback workflow tested
- [ ] Lock workflow tested
- [ ] All integration tests pass

### 4.2 Performance Tests

**File**: `OxVCS-CLI-Wrapper/benches/commit_bench.rs`
**Effort**: 4 hours

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use oxvcs_cli_wrapper::oxen_ops;
use tempfile::TempDir;

fn benchmark_commit_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("commit_performance");

    for size_mb in [1, 10, 100, 500, 1000] {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}MB", size_mb)),
            &size_mb,
            |b, &size| {
                b.iter(|| {
                    let temp = create_test_project_with_size(size);
                    oxen_ops::init(&temp.path()).await.unwrap();
                    oxen_ops::add(&temp.path(), ".").await.unwrap();
                    oxen_ops::create_commit(&temp.path(), "Bench commit").await.unwrap();
                });
            }
        );
    }

    group.finish();
}

fn benchmark_history_retrieval(c: &mut Criterion) {
    let temp = create_test_project();

    // Create 100 commits
    for i in 0..100 {
        create_commit(&temp, &format!("Commit {}", i));
    }

    c.bench_function("get_history_100_commits", |b| {
        b.iter(|| {
            oxen_ops::get_history(black_box(&temp.path()), None).await.unwrap()
        });
    });
}

criterion_group!(benches, benchmark_commit_sizes, benchmark_history_retrieval);
criterion_main!(benches);
```

**Acceptance Criteria**:
- [ ] Commit performance benchmarked (1GB in <10s)
- [ ] History retrieval benchmarked
- [ ] Lock acquisition benchmarked (<100ms)
- [ ] Baseline established for regression detection

---

## Phase 5: CI/CD Enhancement (Week 4 - 1 day)

**Goal**: Enhance CI/CD with advanced features

### 5.1 Coverage Enforcement

**File**: `.github/workflows/coverage.yml`
**Effort**: 2 hours

```yaml
name: Coverage Check

on:
  pull_request:
    branches: [ main, develop ]

jobs:
  check-coverage:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v3
        with:
          fetch-depth: 0  # Fetch full history for comparison

      - name: Get base coverage
        run: |
          # Download coverage from base branch
          gh run download -n coverage-report -b ${{ github.base_ref }}
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}

      - name: Generate current coverage
        run: |
          cd OxVCS-CLI-Wrapper && cargo tarpaulin --out Json
          cd ../OxVCS-LaunchAgent && swift test --enable-code-coverage
          # ... generate reports

      - name: Compare coverage
        run: |
          python scripts/compare_coverage.py base_coverage.json current_coverage.json

      - name: Fail if coverage decreased
        run: |
          if [ $COVERAGE_DECREASED -eq 1 ]; then
            echo "❌ Coverage decreased. Please add tests."
            exit 1
          fi
```

### 5.2 Performance Regression Detection

**File**: `.github/workflows/performance.yml`
**Effort**: 3 hours

```yaml
name: Performance Tests

on:
  pull_request:
    branches: [ main ]

jobs:
  benchmark:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v3

      - name: Run benchmarks
        working-directory: OxVCS-CLI-Wrapper
        run: cargo bench -- --save-baseline pr-${{ github.event.pull_request.number }}

      - name: Compare with main
        run: |
          cargo bench --baseline main -- --load-baseline pr-${{ github.event.pull_request.number }}

      - name: Comment PR
        uses: actions/github-script@v6
        with:
          script: |
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: '📊 Performance benchmark results:\n' + benchmarkResults
            })
```

**Acceptance Criteria**:
- [ ] Coverage tracking implemented
- [ ] Coverage ratcheting enforced (no decrease)
- [ ] Performance benchmarks run on PRs
- [ ] Regression alerts configured

---

## Test Utility Templates

### Rust Test Utilities

**File**: `OxVCS-CLI-Wrapper/tests/common/mod.rs`

```rust
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use std::fs;

pub struct TestFixture {
    pub temp_dir: TempDir,
    pub project_path: PathBuf,
}

impl TestFixture {
    pub fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().join("TestProject.logicx");

        // Create Logic Pro project structure
        fs::create_dir_all(&project_path).unwrap();
        fs::create_dir_all(project_path.join("Alternatives")).unwrap();
        fs::create_dir_all(project_path.join("Media")).unwrap();

        Self { temp_dir, project_path }
    }

    pub fn create_audio_file(&self, name: &str, size_mb: usize) -> PathBuf {
        let media = self.project_path.join("Media");
        let file_path = media.join(name);
        let data = vec![0u8; size_mb * 1024 * 1024];
        fs::write(&file_path, data).unwrap();
        file_path
    }

    pub fn create_project_data(&self, bpm: u32, sample_rate: u32) {
        let xml = format!(r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <project>
            <tempo>{}</tempo>
            <sampleRate>{}</sampleRate>
        </project>
        "#, bpm, sample_rate);

        fs::write(self.project_path.join("projectData"), xml).unwrap();
    }
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        // Cleanup happens automatically with TempDir
    }
}

/// Create a test project with specified size
pub fn create_test_project_with_size(size_mb: usize) -> TempDir {
    let fixture = TestFixture::new();
    fixture.create_audio_file("large.wav", size_mb);
    fixture.temp_dir
}
```

### Swift Test Utilities

**File**: `OxVCS-LaunchAgent/Tests/TestUtils/TestFixtures.swift`

```swift
import Foundation

public struct TestFixtures {
    /// Creates a temporary Logic Pro project with realistic structure
    public static func createLogicProject(
        name: String = "TestProject",
        bpm: Int = 120,
        sampleRate: Int = 48000,
        audioFileSizeMB: Int = 1
    ) -> URL {
        let tempDir = FileManager.default.temporaryDirectory
            .appendingPathComponent(UUID().uuidString)
        try! FileManager.default.createDirectory(at: tempDir, withIntermediateDirectories: true)

        let projectPackage = tempDir.appendingPathComponent("\(name).logicx")
        try! FileManager.default.createDirectory(at: projectPackage, withIntermediateDirectories: true)

        // Create required directories
        let alternatives = projectPackage.appendingPathComponent("Alternatives")
        try! FileManager.default.createDirectory(at: alternatives, withIntermediateDirectories: true)

        let media = projectPackage.appendingPathComponent("Media")
        try! FileManager.default.createDirectory(at: media, withIntermediateDirectories: true)

        // Create dummy audio file
        let audioFile = media.appendingPathComponent("Audio.wav")
        let audioData = Data(count: audioFileSizeMB * 1024 * 1024)
        try! audioData.write(to: audioFile)

        // Create projectData
        let projectData = projectPackage.appendingPathComponent("projectData")
        let xml = """
        <?xml version="1.0" encoding="UTF-8"?>
        <project>
            <tempo>\(bpm)</tempo>
            <sampleRate>\(sampleRate)</sampleRate>
        </project>
        """
        try! xml.write(to: projectData, atomically: true, encoding: .utf8)

        return projectPackage
    }

    /// Creates a temporary directory
    public static func createTempDirectory() -> URL {
        let tempDir = FileManager.default.temporaryDirectory
            .appendingPathComponent(UUID().uuidString)
        try! FileManager.default.createDirectory(at: tempDir, withIntermediateDirectories: true)
        return tempDir
    }

    /// Cleans up temporary test projects
    public static func cleanup(_ projectURL: URL) {
        try? FileManager.default.removeItem(at: projectURL.deletingLastPathComponent())
    }
}
```

**File**: `OxVCS-App/Tests/TestUtils/MockXPCClient.swift`

```swift
import Foundation
@testable import OxVCS_App

public class MockOxenDaemonXPCClient: OxenDaemonXPCClientProtocol {
    public var mockProjects: [Project] = []
    public var mockCommits: [CommitInfo] = []
    public var addProjectCalled = false
    public var lastAddedPath: String?
    public var shouldFail = false
    public var commitCalled = false

    public init() {}

    public func getMonitoredProjects() async throws -> [Project] {
        if shouldFail {
            throw NSError(domain: "TestError", code: -1)
        }
        return mockProjects
    }

    public func addProject(_ path: String) async throws {
        if shouldFail {
            throw NSError(domain: "TestError", code: -1)
        }
        addProjectCalled = true
        lastAddedPath = path
        mockProjects.append(Project(path: path, name: URL(fileURLWithPath: path).lastPathComponent))
    }

    public func createMilestoneCommit(_ message: String, metadata: CommitMetadata) async throws {
        if shouldFail {
            throw NSError(domain: "TestError", code: -1)
        }
        commitCalled = true
        let commit = CommitInfo(hash: UUID().uuidString, message: message, timestamp: Date())
        mockCommits.append(commit)
    }

    public func getCommitHistory(for path: String) async throws -> [CommitInfo] {
        if shouldFail {
            throw NSError(domain: "TestError", code: -1)
        }
        return mockCommits
    }
}
```

---

## Summary and Timeline

### Effort Summary

| Phase | Focus | Effort | Priority |
|-------|-------|--------|----------|
| Phase 0 | Foundation & CI/CD | 2 days | Critical |
| Phase 1 | Critical Path Tests | 3 days | High |
| Phase 2 | Core Components | 4 days | High |
| Phase 3 | UI Components | 4 days | Medium |
| Phase 4 | Integration & E2E | 3 days | Medium |
| Phase 5 | CI/CD Enhancement | 1 day | Low |
| **Total** | | **17 days** | |

### Week-by-Week Breakdown

**Week 0 (Days 1-2)**: Foundation
- Set up CI/CD pipeline
- Create test utilities and fixtures
- Update documentation

**Week 1 (Days 3-7)**: Critical Path + Core
- Lock Manager (already done ✅)
- Oxen operations tests
- Logic project detection tests
- Commit metadata tests
- Power management tests

**Week 2 (Days 8-12)**: Core + UI
- FSEvents monitor tests
- Commit orchestrator tests
- Draft manager tests
- ViewModel tests (begin)

**Week 3 (Days 13-17)**: UI + Integration
- Complete ViewModel tests
- Model tests
- XPC client tests
- Integration tests

**Week 4 (Days 18-19)**: Polish
- Performance benchmarks
- E2E tests
- CI/CD enhancements
- Documentation updates

### Success Criteria

**Completion Checklist**:
- [ ] CI/CD pipeline operational
- [ ] All Phase 1 tests implemented and passing
- [ ] All Phase 2 tests implemented and passing
- [ ] Code coverage ≥70% overall
- [ ] Critical paths have ≥90% coverage
- [ ] Performance benchmarks established
- [ ] Integration tests cover key workflows
- [ ] Documentation updated

**Quality Metrics**:
- [ ] 0 test failures
- [ ] <5% flaky tests
- [ ] All new code has accompanying tests
- [ ] Coverage does not decrease (ratcheting)

---

## Getting Started

To begin implementation, start with **Phase 0**:

1. Create `.github/workflows/test.yml`
2. Set up test utilities in each component
3. Run initial CI build to verify setup
4. Proceed to Phase 1 critical path tests

Refer to `TESTING_STRATEGY.md` for detailed testing philosophy and patterns.
