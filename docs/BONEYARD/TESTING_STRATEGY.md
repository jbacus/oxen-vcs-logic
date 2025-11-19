# Testing Strategy for Oxen-VCS Logic Pro

## Table of Contents
- [1. Overview](#1-overview)
- [2. Testing Objectives](#2-testing-objectives)
- [3. Testing Levels](#3-testing-levels)
- [4. Component-Specific Strategies](#4-component-specific-strategies)
- [5. Test Infrastructure](#5-test-infrastructure)
- [6. CI/CD Strategy](#6-cicd-strategy)
- [7. Coverage Targets](#7-coverage-targets)
- [8. Quality Gates](#8-quality-gates)
- [9. Testing Challenges](#9-testing-challenges)

## 1. Overview

Oxen-VCS for Logic Pro is a native macOS version control system with three primary components:
- **Auxin-CLI-Wrapper** (Rust): High-performance FFI wrapper for Oxen.ai operations
- **Auxin-LaunchAgent** (Swift): Background daemon for file monitoring and auto-commits
- **Auxin-App** (Swift/AppKit): Native macOS UI application

This testing strategy provides a comprehensive approach to ensuring reliability, performance, and correctness across all components.

### Current State
- **Total Production Code**: ~5,500 lines
- **Test Coverage**: <5% (only LockManager has tests)
- **CI/CD**: None
- **Automated Testing**: Manual only

### Target State
- **Test Coverage**: 70-80% overall
- **CI/CD**: Automated testing on all PRs and commits
- **Test Automation**: Continuous testing with fast feedback
- **Quality Gates**: Enforced coverage and quality standards

## 2. Testing Objectives

### Primary Objectives
1. **Prevent Data Loss**: Ensure no Logic Pro project corruption or file loss
2. **Ensure System Stability**: Prevent daemon crashes and resource leaks
3. **Validate Core Workflows**: Verify all version control operations work correctly
4. **Maintain Performance**: Ensure background operations don't impact DAW performance
5. **Cross-Platform Compatibility**: Validate behavior across macOS versions

### Secondary Objectives
1. Enable safe refactoring through comprehensive test coverage
2. Document expected behavior through test cases
3. Catch regressions early in development cycle
4. Provide confidence for production deployment

## 3. Testing Levels

### 3.1 Unit Tests

**Purpose**: Test individual functions and classes in isolation

**Scope**:
- Individual Rust modules (logic_project, oxen_ops, commit_metadata, etc.)
- Individual Swift classes (FSEventsMonitor, CommitOrchestrator, ViewModels, etc.)
- Pure business logic without external dependencies

**Tools**:
- Rust: `cargo test` with built-in test framework
- Swift: `XCTest` framework
- Mocking: Protocol-based mocking for Swift, trait-based for Rust

**Coverage Target**: 80-90% for business logic modules

**Example Test Cases**:
```swift
// Swift Unit Test Example
func testCommitMetadataExtraction() {
    let metadata = CommitMetadata(bpm: 120, sampleRate: 48000, key: "C Major")
    XCTAssertEqual(metadata.bpm, 120)
    XCTAssertEqual(metadata.formattedDescription, "120 BPM, 48kHz, C Major")
}
```

```rust
// Rust Unit Test Example
#[test]
fn test_is_logic_project_valid() {
    let temp_dir = create_temp_logic_project();
    assert!(is_logic_project(&temp_dir).unwrap());
}
```

### 3.2 Integration Tests

**Purpose**: Test interactions between components

**Scope**:
- CLI wrapper calling Oxen.ai library
- Daemon XPC communication with UI app
- File system monitoring triggering commits
- Lock manager coordinating between processes

**Tools**:
- Rust: Integration tests in `tests/` directory
- Swift: XCTest with XPC test harness
- Test doubles: In-memory file systems, mock XPC connections

**Coverage Target**: 60-70% of integration points

**Example Test Cases**:
```swift
// Integration Test: FSEvents → CommitOrchestrator
func testFileChangeTriggersCommit() {
    let monitor = FSEventsMonitor(projectPaths: [testProjectPath])
    let orchestrator = CommitOrchestrator()

    // Write file to trigger event
    writeTestFile(to: testProjectPath)

    // Wait for debounce
    wait(seconds: 3)

    // Verify commit was created
    let commits = getCommitHistory(testProjectPath)
    XCTAssertTrue(commits.contains { $0.message.contains("Auto-draft") })
}
```

### 3.3 End-to-End Tests

**Purpose**: Test complete user workflows from UI to file system

**Scope**:
- Complete commit workflow (UI → Daemon → CLI → Oxen)
- Complete rollback workflow
- Complete lock acquisition and release workflow
- Project initialization wizard
- Merge protocol workflow

**Tools**:
- Swift: XCTest with UI testing (XCUITest)
- Automation: AppleScript or UI automation for Logic Pro interaction
- Fixtures: Pre-built Logic Pro project templates

**Coverage Target**: All critical user paths (8-10 workflows)

**Example Test Cases**:
```swift
// E2E Test: Complete Rollback Workflow
func testCompleteRollbackWorkflow() {
    // 1. Create initial commit
    createMilestoneCommit(message: "Initial state", bpm: 120)

    // 2. Make changes
    modifyProjectFile(testProjectPath)

    // 3. Create second commit
    createMilestoneCommit(message: "Modified state", bpm: 140)

    // 4. Rollback to first commit
    performRollback(to: firstCommitHash)

    // 5. Verify project state restored
    let projectInfo = getProjectInfo(testProjectPath)
    XCTAssertEqual(projectInfo.bpm, 120)
}
```

### 3.4 Performance Tests

**Purpose**: Ensure system performance meets requirements

**Scope**:
- Daemon CPU/memory usage during file monitoring
- Commit performance with large projects (>1GB)
- FSEvents debounce effectiveness
- Lock acquisition latency

**Tools**:
- Rust: `criterion` benchmarking framework
- Swift: `XCTMetric` for performance testing
- Profiling: Instruments.app for deep analysis

**Performance Requirements**:
- Daemon CPU usage: <5% average, <20% peak
- Daemon memory usage: <100MB
- Commit time: <10s for 1GB project
- Lock acquisition: <100ms
- FSEvents latency: <500ms after debounce

**Example Test Cases**:
```rust
// Rust Benchmark Example
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_commit_large_project(c: &mut Criterion) {
    let large_project = create_test_project_1gb();
    c.bench_function("commit 1GB project", |b| {
        b.iter(|| {
            commit_project(black_box(&large_project))
        });
    });
}
```

### 3.5 Stress and Reliability Tests

**Purpose**: Test system behavior under extreme conditions

**Scope**:
- Concurrent commits from multiple processes
- Rapid file changes (thousands per second)
- Disk space exhaustion
- Network interruption during Oxen operations
- System sleep/wake cycles
- Daemon crash recovery

**Tools**:
- Custom stress test scripts
- XCTest with repeated operations
- Network link conditioner (Apple Developer Tools)

**Example Test Cases**:
```swift
// Stress Test: Concurrent Lock Attempts
func testConcurrentLockAcquisition() {
    let concurrentAttempts = 100
    let expectations = (0..<concurrentAttempts).map { _ in expectation() }

    DispatchQueue.concurrentPerform(iterations: concurrentAttempts) { index in
        let result = lockManager.acquireLock(for: testProjectPath)
        // Exactly one should succeed
        expectations[index].fulfill()
    }

    waitForExpectations(timeout: 10)
    XCTAssertEqual(successfulAcquisitions, 1)
}
```

### 3.6 Regression Tests

**Purpose**: Prevent previously fixed bugs from reappearing

**Scope**:
- All reported bugs should have a corresponding test
- Critical bug fixes should have dedicated test cases
- Edge cases discovered during manual testing

**Strategy**:
- Create test case for each bug before fixing
- Maintain regression test suite
- Run regression tests on every build

## 4. Component-Specific Strategies

### 4.1 Auxin-CLI-Wrapper (Rust)

**Testing Priorities**:
1. Logic Pro project detection (`logic_project.rs`)
2. Oxen operations wrapper (`oxen_ops.rs`)
3. Commit metadata parsing (`commit_metadata.rs`)
4. Draft manager workflow (`draft_manager.rs`)

**Unit Tests**:
```rust
// tests/unit/logic_project_tests.rs
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_detect_valid_logic_project() {
        let temp = TempDir::new().unwrap();
        create_mock_logic_project(&temp.path());
        assert!(is_logic_project(&temp.path()).unwrap());
    }

    #[test]
    fn test_reject_non_logic_directory() {
        let temp = TempDir::new().unwrap();
        assert!(!is_logic_project(&temp.path()).unwrap());
    }

    #[test]
    fn test_extract_project_metadata() {
        let temp = create_mock_logic_project_with_metadata();
        let info = get_project_info(&temp.path()).unwrap();
        assert_eq!(info.bpm, Some(120));
        assert_eq!(info.sample_rate, Some(48000));
    }
}
```

**Integration Tests**:
```rust
// tests/integration/oxen_workflow_tests.rs
#[tokio::test]
async fn test_complete_commit_workflow() {
    let temp = TempDir::new().unwrap();
    let project_path = create_mock_logic_project(&temp.path());

    // Initialize repository
    oxen_ops::init(&project_path).await.unwrap();

    // Add files
    oxen_ops::add(&project_path, ".").await.unwrap();

    // Create commit
    let metadata = CommitMetadata::new(120, 48000, "C Major");
    let commit = oxen_ops::create_commit(&project_path, &metadata).await.unwrap();

    // Verify commit exists
    let history = oxen_ops::get_history(&project_path).await.unwrap();
    assert!(history.iter().any(|c| c.id == commit.id));
}
```

**Test Coverage Goals**:
- Unit tests: 85% coverage
- Integration tests: 70% coverage
- Focus: Error handling, edge cases, Oxen.ai library interaction

### 4.2 Auxin-LaunchAgent (Swift Daemon)

**Testing Priorities**:
1. File system monitoring (`FSEventsMonitor.swift`)
2. Auto-commit orchestration (`CommitOrchestrator.swift`)
3. Power management (`PowerManagement.swift`)
4. Lock manager (`LockManager.swift` - already has tests)
5. XPC service (`XPCService.swift`)

**Unit Tests**:
```swift
// Tests/FSEventsMonitorTests.swift
final class FSEventsMonitorTests: XCTestCase {
    var monitor: FSEventsMonitor!
    var testDirectory: URL!

    override func setUp() {
        super.setUp()
        testDirectory = createTempDirectory()
        monitor = FSEventsMonitor(projectPaths: [testDirectory.path])
    }

    func testDebouncePreventsDuplicateEvents() {
        let expectation = self.expectation(description: "Single callback")
        expectation.expectedFulfillmentCount = 1

        monitor.onFileChanged = { path in
            expectation.fulfill()
        }

        monitor.start()

        // Trigger multiple rapid changes
        for i in 0..<10 {
            writeFile(to: testDirectory, content: "Change \(i)")
            usleep(100_000) // 100ms between changes
        }

        // Should only get one callback after debounce
        wait(for: [expectation], timeout: 5.0)
    }

    func testIgnoresSystemFiles() {
        let expectation = self.expectation(description: "No callback for system files")
        expectation.isInverted = true

        monitor.onFileChanged = { path in
            expectation.fulfill()
        }

        monitor.start()

        // Write .DS_Store (should be ignored)
        writeFile(to: testDirectory.appendingPathComponent(".DS_Store"))

        wait(for: [expectation], timeout: 2.0)
    }
}
```

**Integration Tests**:
```swift
// Tests/DaemonIntegrationTests.swift
final class DaemonIntegrationTests: XCTestCase {
    func testPowerEventTriggersCommit() {
        let daemon = Daemon()
        let testProject = createTestLogicProject()
        daemon.addProject(testProject)

        // Simulate system sleep notification
        NotificationCenter.default.post(
            name: NSWorkspace.willSleepNotification,
            object: nil
        )

        // Wait for commit to complete
        wait(seconds: 2)

        // Verify commit was created
        let commits = getCommitHistory(testProject)
        XCTAssertTrue(commits.last?.message.contains("Power-safe commit"))
    }
}
```

**Test Coverage Goals**:
- Unit tests: 80% coverage (focus on business logic)
- Integration tests: 60% coverage (focus on component interaction)
- Mock XPC communication for testing without full system
- Test all power management scenarios (sleep, shutdown, wake)

### 4.3 Auxin-App (Swift AppKit UI)

**Testing Priorities**:
1. ViewModels (`ProjectListViewModel.swift`, `ProjectDetailViewModel.swift`)
2. Models (`Project.swift`)
3. XPC Client (`OxenDaemonXPCClient.swift`)
4. UI interactions (Views)

**Unit Tests (ViewModels)**:
```swift
// Tests/ViewModels/ProjectListViewModelTests.swift
final class ProjectListViewModelTests: XCTestCase {
    var viewModel: ProjectListViewModel!
    var mockXPCClient: MockOxenDaemonXPCClient!

    override func setUp() {
        super.setUp()
        mockXPCClient = MockOxenDaemonXPCClient()
        viewModel = ProjectListViewModel(xpcClient: mockXPCClient)
    }

    func testLoadProjectsFromDaemon() async {
        // Setup mock data
        mockXPCClient.mockProjects = [
            Project(path: "/path/to/project1", name: "Project 1"),
            Project(path: "/path/to/project2", name: "Project 2")
        ]

        // Load projects
        await viewModel.loadProjects()

        // Verify
        XCTAssertEqual(viewModel.projects.count, 2)
        XCTAssertEqual(viewModel.projects[0].name, "Project 1")
    }

    func testRefreshUpdatesProjectStatus() async {
        mockXPCClient.mockProjects = [
            Project(path: "/path/to/project1", name: "Project 1", status: .idle)
        ]

        await viewModel.loadProjects()
        XCTAssertEqual(viewModel.projects[0].status, .idle)

        // Update mock to return different status
        mockXPCClient.mockProjects[0].status = .monitoring
        await viewModel.refresh()

        XCTAssertEqual(viewModel.projects[0].status, .monitoring)
    }
}
```

**UI Tests**:
```swift
// Tests/UI/ProjectWizardUITests.swift
final class ProjectWizardUITests: XCTestCase {
    var app: XCUIApplication!

    override func setUp() {
        super.setUp()
        app = XCUIApplication()
        app.launch()
    }

    func testProjectInitializationWizard() {
        // Open wizard
        app.menuItems["File"].click()
        app.menuItems["Add Project..."].click()

        // Select project directory
        let browserButton = app.buttons["Browse..."]
        XCTAssertTrue(browserButton.exists)
        browserButton.click()

        // (File picker interaction would need additional setup)

        // Initialize repository
        let initButton = app.buttons["Initialize Repository"]
        XCTAssertTrue(initButton.exists)
        initButton.click()

        // Verify success message
        XCTAssertTrue(app.staticTexts["Repository initialized successfully"].exists)
    }
}
```

**Test Coverage Goals**:
- ViewModels: 80% coverage (critical business logic)
- Models: 90% coverage (data structures)
- Views: 40-50% coverage (basic interaction tests)
- XPC Client: 70% coverage (mocked daemon responses)

### 4.4 Cross-Component Integration Tests

**Purpose**: Test full system integration across all three components

**Test Scenarios**:
1. **End-to-End Commit Flow**:
   - UI initiates milestone commit → XPC → Daemon → CLI → Oxen → Verify in UI
2. **Auto-Commit Flow**:
   - File change → FSEvents → Daemon → CLI → Verify commit created
3. **Rollback Flow**:
   - UI requests rollback → XPC → Daemon → CLI → Verify files restored
4. **Lock Flow**:
   - UI requests lock → XPC → Daemon → Verify lock file → Release → Verify removed
5. **Multi-Project Orchestration**:
   - Monitor 3 projects → Make changes to all → Verify commits debounced correctly

**Example Test**:
```swift
// Tests/Integration/EndToEndWorkflowTests.swift
final class EndToEndWorkflowTests: XCTestCase {
    func testCompleteAutoCommitWorkflow() async throws {
        // 1. Start daemon
        let daemon = Daemon()
        daemon.start()

        // 2. Add project via UI
        let project = createTestLogicProject()
        let app = AuxinApp()
        await app.addProject(project)

        // 3. Modify project file
        modifyAudioFile(in: project)

        // 4. Wait for FSEvents + debounce
        try await Task.sleep(nanoseconds: 3_000_000_000) // 3 seconds

        // 5. Verify commit created via CLI
        let commits = try await CLIWrapper.getHistory(project)
        XCTAssertTrue(commits.contains { $0.message.contains("Auto-draft") })

        // 6. Verify UI reflects new commit
        let viewModel = ProjectDetailViewModel(project: project)
        await viewModel.refresh()
        XCTAssertTrue(viewModel.commits.count > 0)
    }
}
```

## 5. Test Infrastructure

### 5.1 Test Utilities and Helpers

**Shared Test Utilities** (`Tests/TestUtils/`):

```swift
// Tests/TestUtils/TestFixtures.swift
struct TestFixtures {
    /// Creates a temporary Logic Pro project with realistic structure
    static func createLogicProject(
        name: String = "TestProject",
        bpm: Int = 120,
        sampleRate: Int = 48000,
        audioFiles: Int = 5
    ) -> URL {
        let tempDir = FileManager.default.temporaryDirectory
            .appendingPathComponent(UUID().uuidString)
        try! FileManager.default.createDirectory(at: tempDir, withIntermediateDirectories: true)

        // Create .logicx package structure
        let projectPackage = tempDir.appendingPathComponent("\(name).logicx")
        try! FileManager.default.createDirectory(at: projectPackage, withIntermediateDirectories: true)

        // Create Alternatives directory
        let alternatives = projectPackage.appendingPathComponent("Alternatives")
        try! FileManager.default.createDirectory(at: alternatives, withIntermediateDirectories: true)

        // Create dummy audio files
        let media = projectPackage.appendingPathComponent("Media")
        try! FileManager.default.createDirectory(at: media, withIntermediateDirectories: true)

        for i in 0..<audioFiles {
            let audioFile = media.appendingPathComponent("Audio_\(i).wav")
            let dummyData = Data(repeating: 0, count: 1024 * 1024) // 1MB dummy
            try! dummyData.write(to: audioFile)
        }

        // Create projectData file (XML)
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

    /// Cleans up temporary test projects
    static func cleanup(_ projectURL: URL) {
        try? FileManager.default.removeItem(at: projectURL.deletingLastPathComponent())
    }
}
```

```swift
// Tests/TestUtils/MockXPCClient.swift
class MockOxenDaemonXPCClient: OxenDaemonXPCClientProtocol {
    var mockProjects: [Project] = []
    var mockCommits: [CommitInfo] = []
    var addProjectCalled = false
    var commitCalled = false

    func getMonitoredProjects() async throws -> [Project] {
        return mockProjects
    }

    func addProject(_ path: String) async throws {
        addProjectCalled = true
        mockProjects.append(Project(path: path, name: URL(fileURLWithPath: path).lastPathComponent))
    }

    func createMilestoneCommit(_ message: String, metadata: CommitMetadata) async throws {
        commitCalled = true
        let commit = CommitInfo(hash: UUID().uuidString, message: message, timestamp: Date())
        mockCommits.append(commit)
    }
}
```

```rust
// Auxin-CLI-Wrapper/tests/common/mod.rs
use std::path::{Path, PathBuf};
use tempfile::TempDir;

pub struct TestFixture {
    pub temp_dir: TempDir,
    pub project_path: PathBuf,
}

impl TestFixture {
    pub fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().join("TestProject.logicx");
        std::fs::create_dir_all(&project_path).unwrap();

        // Create minimal Logic Pro project structure
        let alternatives = project_path.join("Alternatives");
        std::fs::create_dir_all(&alternatives).unwrap();

        Self { temp_dir, project_path }
    }

    pub fn create_audio_file(&self, name: &str, size_mb: usize) -> PathBuf {
        let media = self.project_path.join("Media");
        std::fs::create_dir_all(&media).unwrap();
        let file_path = media.join(name);
        let data = vec![0u8; size_mb * 1024 * 1024];
        std::fs::write(&file_path, data).unwrap();
        file_path
    }
}
```

### 5.2 Test Data Management

**Approach**:
- Use temporary directories for all file system tests
- Clean up after each test (setUp/tearDown)
- Use realistic but minimal test data (small audio files, simple XML)
- Version control test fixtures in `Tests/Fixtures/` for complex scenarios

**Example Fixtures Directory**:
```
Tests/
├── Fixtures/
│   ├── SampleProjects/
│   │   ├── MinimalProject.logicx/    # Smallest valid project
│   │   ├── StandardProject.logicx/   # Typical project structure
│   │   └── LargeProject.logicx/      # Large project for perf tests
│   ├── CommitHistories/
│   │   └── sample-history.json       # Mock commit history
│   └── LockFiles/
│       └── sample-lock.json          # Example lock manifest
```

### 5.3 Continuous Integration Configuration

**GitHub Actions Workflow** (`.github/workflows/test.yml`):

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

      - name: Cache cargo dependencies
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            Auxin-CLI-Wrapper/target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Run cargo test
        working-directory: Auxin-CLI-Wrapper
        run: cargo test --verbose

      - name: Run cargo clippy
        working-directory: Auxin-CLI-Wrapper
        run: cargo clippy -- -D warnings

      - name: Check formatting
        working-directory: Auxin-CLI-Wrapper
        run: cargo fmt -- --check

  swift-daemon-tests:
    name: Swift Tests (LaunchAgent Daemon)
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v3

      - name: Select Xcode version
        run: sudo xcode-select -s /Applications/Xcode_15.0.app

      - name: Run swift test
        working-directory: Auxin-LaunchAgent
        run: swift test --enable-code-coverage

      - name: Generate coverage report
        working-directory: Auxin-LaunchAgent
        run: |
          xcrun llvm-cov export \
            -format="lcov" \
            .build/debug/Auxin-LaunchAgentPackageTests.xctest/Contents/MacOS/Auxin-LaunchAgentPackageTests \
            -instr-profile .build/debug/codecov/default.profdata > coverage.lcov

      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v3
        with:
          files: ./Auxin-LaunchAgent/coverage.lcov
          flags: daemon

  swift-app-tests:
    name: Swift Tests (AppKit UI)
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v3

      - name: Select Xcode version
        run: sudo xcode-select -s /Applications/Xcode_15.0.app

      - name: Run swift test
        working-directory: Auxin-App
        run: swift test --enable-code-coverage

      - name: Generate coverage report
        working-directory: Auxin-App
        run: |
          xcrun llvm-cov export \
            -format="lcov" \
            .build/debug/Auxin-AppPackageTests.xctest/Contents/MacOS/Auxin-AppPackageTests \
            -instr-profile .build/debug/codecov/default.profdata > coverage.lcov

      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v3
        with:
          files: ./Auxin-App/coverage.lcov
          flags: app

  integration-tests:
    name: Integration Tests (All Components)
    runs-on: macos-latest
    needs: [rust-tests, swift-daemon-tests, swift-app-tests]
    steps:
      - uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Build CLI wrapper
        working-directory: Auxin-CLI-Wrapper
        run: cargo build --release

      - name: Build daemon
        working-directory: Auxin-LaunchAgent
        run: swift build

      - name: Build app
        working-directory: Auxin-App
        run: swift build

      - name: Run integration tests
        run: |
          # Integration tests would be in a separate test suite
          # This is a placeholder for end-to-end testing
          echo "Integration tests would run here"
```

### 5.4 Code Coverage Tools

**Setup Coverage Tracking**:

For Swift:
```bash
# Run tests with coverage
swift test --enable-code-coverage

# Generate HTML report
xcrun llvm-cov show \
  .build/debug/YourPackageTests.xctest/Contents/MacOS/YourPackageTests \
  -instr-profile=.build/debug/codecov/default.profdata \
  -format=html \
  -output-dir=coverage-report
```

For Rust:
```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Run with coverage
cargo tarpaulin --out Html --output-dir coverage-report
```

**Coverage Reporting**:
- Use Codecov.io for unified coverage reporting
- Display coverage badges in README
- Set coverage thresholds in CI (fail if below threshold)

## 6. CI/CD Strategy

### 6.1 Build Pipeline

**Stage 1: Fast Feedback (< 5 minutes)**
- Linting (cargo clippy, swiftlint)
- Code formatting checks (cargo fmt, swiftformat)
- Unit tests (fast tests only)
- Type checking

**Stage 2: Comprehensive Testing (< 15 minutes)**
- Full unit test suite
- Integration tests
- Code coverage generation

**Stage 3: Extended Testing (< 30 minutes)**
- Performance benchmarks
- Stress tests
- End-to-end workflows

**Stage 4: Release Validation (manual trigger)**
- Full system tests on clean macOS VM
- Installation testing
- Upgrade path testing

### 6.2 Quality Gates

**Pull Request Requirements**:
- ✅ All tests pass
- ✅ Code coverage doesn't decrease (ratcheting)
- ✅ No clippy/swiftlint warnings
- ✅ Code formatted correctly
- ✅ At least one test added for new functionality

**Main Branch Protection**:
- Require PR reviews (1-2 reviewers)
- Require status checks to pass
- No direct pushes to main
- Require up-to-date branches

### 6.3 Automated Testing Triggers

- **On every commit**: Linting, formatting, fast unit tests
- **On PR**: Full test suite, coverage report
- **On main branch**: Full suite + performance tests
- **Nightly**: Extended stress tests, compatibility tests
- **Weekly**: Full integration tests across macOS versions

## 7. Coverage Targets

### Component-Level Targets

| Component | Unit Test Coverage | Integration Coverage | Notes |
|-----------|-------------------|---------------------|-------|
| Auxin-CLI-Wrapper | 80% | 70% | Focus on oxen_ops.rs, logic_project.rs |
| Auxin-LaunchAgent | 75% | 60% | Mock FSEvents, XPC for isolation |
| Auxin-App | 70% | 50% | ViewModels critical, Views basic |
| Overall | 75% | 60% | Weighted by component importance |

### Critical Path Requirements

**Must Have 90%+ Coverage**:
- Lock manager (already has good tests)
- Commit creation and metadata
- Power management commit triggers
- File system change detection
- Project state restoration (rollback)

**Must Have 80%+ Coverage**:
- Draft branch management
- XPC communication
- Project initialization
- Settings persistence

**Can Have 60%+ Coverage**:
- UI views (basic smoke tests)
- Logging and diagnostics
- Settings UI

### Exemptions

**OK to exclude from coverage**:
- Generated code (protobuf, etc.)
- Third-party code
- Trivial getters/setters
- Deprecated code paths

## 8. Quality Gates

### Pre-Commit Checks (Local)

**Git Pre-Commit Hook** (`.git/hooks/pre-commit`):
```bash
#!/bin/bash

echo "Running pre-commit checks..."

# Rust formatting
cd Auxin-CLI-Wrapper
if ! cargo fmt -- --check; then
    echo "❌ Rust code not formatted. Run: cargo fmt"
    exit 1
fi

# Rust linting
if ! cargo clippy -- -D warnings; then
    echo "❌ Clippy warnings found"
    exit 1
fi

# Swift tests (fast unit tests only)
cd ../Auxin-LaunchAgent
if ! swift test --filter ".*UnitTests"; then
    echo "❌ Swift unit tests failed"
    exit 1
fi

echo "✅ Pre-commit checks passed"
```

### CI Quality Gates

**Required for PR Merge**:
1. All tests pass (100% pass rate required)
2. Code coverage ≥ baseline (no decrease)
3. No critical/high severity warnings
4. Performance benchmarks within 10% of baseline
5. All files formatted correctly

**Recommended for PR Merge** (warnings, not blockers):
1. Code coverage < 70% overall
2. New code not covered by tests
3. Performance degradation 5-10%
4. TODO/FIXME comments added

### Release Quality Gates

**Required for Release**:
1. All CI checks pass on release branch
2. Integration tests pass on clean macOS
3. Performance tests meet all SLAs
4. Manual testing checklist completed
5. Documentation updated
6. CHANGELOG.md updated

## 9. Testing Challenges

### 9.1 File System Testing

**Challenge**: Tests interact with real file system, potential for:
- Test pollution (leftover files)
- Race conditions (FSEvents timing)
- Platform-specific behaviors

**Solutions**:
- Use `TempDir` (Rust) and `FileManager.default.temporaryDirectory` (Swift)
- Clean up in `tearDown()` methods
- Use unique identifiers (UUID) for test directories
- Add delays/expectations for async file operations
- Mock FSEvents where possible

### 9.2 Background Daemon Testing

**Challenge**: LaunchAgent runs as persistent service:
- Difficult to start/stop in tests
- XPC communication requires daemon running
- SMAppService registration requires elevated permissions

**Solutions**:
- Protocol-based mocking for XPC (don't require real daemon)
- Create lightweight test daemon that doesn't register with launchd
- Use in-memory event bus instead of XPC for unit tests
- Integration tests run daemon in foreground mode

**Example**:
```swift
// Testable daemon that doesn't require launchd
class TestDaemon: DaemonProtocol {
    var isRunning = false

    func start() {
        isRunning = true
        // Don't call SMAppService.register()
        // Just start internal components
    }
}
```

### 9.3 UI Testing Limitations

**Challenge**: AppKit UI testing is less mature than UIKit:
- XCUITest support limited
- Accessibility identifiers needed
- Asynchronous UI updates

**Solutions**:
- Focus testing on ViewModels (testable business logic)
- Use protocol-based architecture for mocking
- Basic smoke tests for Views (can they render?)
- Manual testing checklist for complex UI flows

### 9.4 Oxen.ai Library Integration

**Challenge**: Tests depend on external Oxen library:
- Network operations (push/pull)
- Large binary handling
- Oxen.ai service availability

**Solutions**:
- Mock Oxen operations at CLI wrapper boundary
- Use local-only Oxen operations (no network)
- Create stub implementations for unit tests
- Integration tests use real Oxen but local repos only

**Example**:
```rust
// Mock trait for testing
#[cfg(test)]
pub trait OxenOps {
    fn init(&self, path: &Path) -> Result<()>;
    fn add(&self, path: &Path, files: &str) -> Result<()>;
    // ...
}

// Real implementation
pub struct RealOxenOps;
impl OxenOps for RealOxenOps {
    fn init(&self, path: &Path) -> Result<()> {
        liboxen::init(path) // Real Oxen call
    }
}

// Test mock
#[cfg(test)]
pub struct MockOxenOps {
    pub init_called: bool,
}
impl OxenOps for MockOxenOps {
    fn init(&self, path: &Path) -> Result<()> {
        self.init_called = true;
        Ok(())
    }
}
```

### 9.5 macOS Version Compatibility

**Challenge**: Need to test across macOS versions:
- FSEvents API changes
- SMAppService availability (macOS 13+)
- AppKit behavior differences

**Solutions**:
- CI matrix testing across macOS versions
- Conditional compilation for version-specific features
- Fallback implementations for older macOS
- Document minimum macOS version requirements

### 9.6 Performance Test Variability

**Challenge**: Performance tests can be flaky:
- CI runners have variable performance
- Background processes affect timing
- Non-deterministic behavior

**Solutions**:
- Use relative performance (% change vs baseline)
- Allow variance thresholds (±10%)
- Run performance tests multiple times, use median
- Dedicated performance test environment
- Track trends over time vs absolute values

### 9.7 Test Data Size

**Challenge**: Realistic Logic Pro projects are large (>1GB):
- Slow to create in tests
- Consume disk space
- Slow test execution

**Solutions**:
- Use minimal realistic projects (structure correct, small files)
- Lazy-load large test fixtures (download on demand)
- Separate fast tests from slow tests
- Use binary file stubs (headers only, no real audio data)

**Example**:
```swift
// Create minimal but realistic Logic project
func createMinimalLogicProject() -> URL {
    let project = tempDir.appendingPathComponent("Test.logicx")
    // Create structure...

    // Tiny stub audio files (100KB instead of 100MB)
    let audioStub = Data(count: 100_000)
    try! audioStub.write(to: project.appendingPathComponent("Media/Audio.wav"))

    return project
}
```

## 10. Test Maintenance Strategy

### 10.1 Test Ownership

- Each component owner responsible for maintaining tests
- Test failures block merges (enforce fixing)
- Regular test review during sprint planning
- Flaky tests tracked and prioritized for fixing

### 10.2 Test Debt Management

- Tag technical debt tests with `// TODO: Improve`
- Track test coverage in each sprint
- Allocate 10-15% of sprint capacity to test improvement
- Quarterly test suite health review

### 10.3 Test Documentation

- Document complex test scenarios
- Include "why" not just "what" in test names
- Maintain test architecture decision records
- Keep this testing strategy document updated

---

## Appendix A: Test Naming Conventions

### Swift Tests
```swift
// Pattern: test_{function}_{scenario}_{expectedResult}
func testLockAcquisition_WhenAvailable_Succeeds()
func testLockAcquisition_WhenAlreadyLocked_Fails()
func testCommitOrchestrator_OnFileChange_CreatesAutoCommit()
```

### Rust Tests
```rust
// Pattern: test_{function}_{scenario}
#[test]
fn test_is_logic_project_valid_structure()
#[test]
fn test_is_logic_project_missing_alternatives()
#[test]
fn test_commit_metadata_extraction_complete()
```

## Appendix B: Test Templates

See `TEST_IMPLEMENTATION_PLAN.md` for specific test templates and examples.

## Appendix C: Manual Testing Checklist

**Pre-Release Manual Testing**:
- [ ] Install on clean macOS (no prior Oxen)
- [ ] Initialize new Logic Pro project
- [ ] Verify FSEvents monitoring starts
- [ ] Make file changes, verify auto-commits
- [ ] Test system sleep/wake cycle
- [ ] Create milestone commit with metadata
- [ ] Perform rollback to previous state
- [ ] Verify Logic Pro can open rolled-back project
- [ ] Test lock acquisition/release
- [ ] Test multi-project monitoring
- [ ] Uninstall and verify cleanup

---

## Summary

This testing strategy provides a comprehensive framework for ensuring the quality and reliability of Oxen-VCS for Logic Pro. Key priorities:

1. **Establish CI/CD pipeline** to automate testing
2. **Achieve 70-80% code coverage** across all components
3. **Focus on critical paths** (data loss prevention, system stability)
4. **Create robust test utilities** for realistic test scenarios
5. **Address platform-specific challenges** (file system, daemon, macOS)

Implementation should proceed according to the **TEST_IMPLEMENTATION_PLAN.md** document, which provides a phased approach to building out the test suite.
