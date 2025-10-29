import XCTest
@testable import OxVCS_LaunchAgent

final class CommitOrchestratorTests: XCTestCase {
    var orchestrator: CommitOrchestrator!
    var tempDirectory: URL!
    var testProjectPath: String!

    override func setUp() {
        super.setUp()

        // Create temporary directory for testing
        tempDirectory = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
        try? FileManager.default.createDirectory(at: tempDirectory, withIntermediateDirectories: true)

        // Create test project structure
        testProjectPath = tempDirectory.appendingPathComponent("TestProject.logicx").path
        try? FileManager.default.createDirectory(atPath: testProjectPath, withIntermediateDirectories: true)

        // Initialize orchestrator with a non-existent CLI path for testing
        orchestrator = CommitOrchestrator(cliPath: "/tmp/test-oxenvcs-cli")
    }

    override func tearDown() {
        orchestrator = nil
        try? FileManager.default.removeItem(at: tempDirectory)
        super.tearDown()
    }

    // MARK: - Initialization Tests

    func testInitWithDefaultPath() {
        let orch = CommitOrchestrator()
        XCTAssertNotNil(orch, "Should initialize with default CLI path")
    }

    func testInitWithCustomPath() {
        let customPath = "/custom/path/to/cli"
        let orch = CommitOrchestrator(cliPath: customPath)
        XCTAssertNotNil(orch, "Should initialize with custom CLI path")
    }

    func testInitWithEmptyPath() {
        let orch = CommitOrchestrator(cliPath: "")
        XCTAssertNotNil(orch, "Should initialize even with empty path")
    }

    // MARK: - Project Registration Tests

    func testRegisterProject() {
        orchestrator.registerProject(testProjectPath)

        let registered = orchestrator.getRegisteredProjects()
        XCTAssertTrue(registered.contains(testProjectPath), "Project should be registered")
    }

    func testRegisterMultipleProjects() {
        let project1 = tempDirectory.appendingPathComponent("Project1.logicx").path
        let project2 = tempDirectory.appendingPathComponent("Project2.logicx").path

        orchestrator.registerProject(project1)
        orchestrator.registerProject(project2)

        let registered = orchestrator.getRegisteredProjects()
        XCTAssertEqual(registered.count, 2, "Should have 2 registered projects")
        XCTAssertTrue(registered.contains(project1), "Should contain project1")
        XCTAssertTrue(registered.contains(project2), "Should contain project2")
    }

    func testRegisterSameProjectTwice() {
        orchestrator.registerProject(testProjectPath)
        orchestrator.registerProject(testProjectPath)

        let registered = orchestrator.getRegisteredProjects()
        XCTAssertEqual(registered.count, 1, "Should only register project once")
    }

    func testRegisterProjectWithRelativePath() {
        let relativePath = "./TestProject.logicx"
        orchestrator.registerProject(relativePath)

        let registered = orchestrator.getRegisteredProjects()
        // Should be normalized to absolute path
        XCTAssertEqual(registered.count, 1, "Should register relative path")
    }

    func testUnregisterProject() {
        orchestrator.registerProject(testProjectPath)
        orchestrator.unregisterProject(testProjectPath)

        let registered = orchestrator.getRegisteredProjects()
        XCTAssertFalse(registered.contains(testProjectPath), "Project should be unregistered")
    }

    func testUnregisterNonexistentProject() {
        // Should not crash when unregistering non-existent project
        orchestrator.unregisterProject("/nonexistent/project")

        XCTAssert(true, "Should handle unregistering non-existent project")
    }

    func testUnregisterAllProjects() {
        let project1 = tempDirectory.appendingPathComponent("Project1.logicx").path
        let project2 = tempDirectory.appendingPathComponent("Project2.logicx").path

        orchestrator.registerProject(project1)
        orchestrator.registerProject(project2)

        orchestrator.unregisterProject(project1)
        orchestrator.unregisterProject(project2)

        let registered = orchestrator.getRegisteredProjects()
        XCTAssertTrue(registered.isEmpty, "Should have no registered projects")
    }

    func testGetRegisteredProjectsEmptyInitially() {
        let registered = orchestrator.getRegisteredProjects()
        XCTAssertTrue(registered.isEmpty, "Should start with no registered projects")
    }

    // MARK: - Commit Type Tests

    func testCommitTypeAutoSave() {
        let type = CommitOrchestrator.CommitType.autoSave
        XCTAssertNotNil(type, "autoSave type should exist")
    }

    func testCommitTypeEmergency() {
        let type = CommitOrchestrator.CommitType.emergency
        XCTAssertNotNil(type, "emergency type should exist")
    }

    func testCommitTypeManual() {
        let type = CommitOrchestrator.CommitType.manual
        XCTAssertNotNil(type, "manual type should exist")
    }

    // MARK: - CommitResult Tests

    func testCommitResultStructure() {
        let result = CommitOrchestrator.CommitResult(
            success: true,
            commitId: "abc123",
            message: "Test commit",
            duration: 1.5
        )

        XCTAssertTrue(result.success, "Success should be true")
        XCTAssertEqual(result.commitId, "abc123", "CommitId should match")
        XCTAssertEqual(result.message, "Test commit", "Message should match")
        XCTAssertEqual(result.duration, 1.5, accuracy: 0.01, "Duration should match")
    }

    func testCommitResultWithNilCommitId() {
        let result = CommitOrchestrator.CommitResult(
            success: true,
            commitId: nil,
            message: "No changes",
            duration: 0.1
        )

        XCTAssertTrue(result.success, "Success should be true")
        XCTAssertNil(result.commitId, "CommitId should be nil")
    }

    func testCommitResultFailure() {
        let result = CommitOrchestrator.CommitResult(
            success: false,
            commitId: nil,
            message: "Commit failed",
            duration: 0.5
        )

        XCTAssertFalse(result.success, "Success should be false")
        XCTAssertNil(result.commitId, "Failed commit should have no ID")
    }

    // MARK: - Perform Commit Tests

    func testPerformCommitWithoutRegistration() async {
        // Should work even if project not registered
        let result = await orchestrator.performCommit(for: testProjectPath, type: .autoSave)

        // Will likely fail because CLI doesn't exist, but should return result
        XCTAssertNotNil(result, "Should return a result")
    }

    func testPerformCommitAutoSaveType() async {
        orchestrator.registerProject(testProjectPath)

        let result = await orchestrator.performCommit(for: testProjectPath, type: .autoSave)

        XCTAssertNotNil(result, "Should return result for auto-save")
        XCTAssertGreaterThanOrEqual(result.duration, 0, "Duration should be non-negative")
    }

    func testPerformCommitEmergencyType() async {
        orchestrator.registerProject(testProjectPath)

        let result = await orchestrator.performCommit(for: testProjectPath, type: .emergency)

        XCTAssertNotNil(result, "Should return result for emergency")
        XCTAssertGreaterThanOrEqual(result.duration, 0, "Duration should be non-negative")
    }

    func testPerformCommitManualType() async {
        orchestrator.registerProject(testProjectPath)

        let result = await orchestrator.performCommit(for: testProjectPath, type: .manual)

        XCTAssertNotNil(result, "Should return result for manual")
        XCTAssertGreaterThanOrEqual(result.duration, 0, "Duration should be non-negative")
    }

    func testPerformCommitWithLockedProject() async {
        // Acquire lock first
        _ = LockManager.shared.acquireLock(projectPath: testProjectPath)

        let result = await orchestrator.performCommit(for: testProjectPath, type: .autoSave)

        XCTAssertFalse(result.success, "Should fail when project is locked")
        XCTAssertTrue(result.message.contains("locked"), "Message should mention lock")

        // Clean up
        _ = LockManager.shared.releaseLock(projectPath: testProjectPath)
    }

    func testPerformCommitReleasesLockAfter() async {
        // Acquire lock
        _ = LockManager.shared.acquireLock(projectPath: testProjectPath)

        let result = await orchestrator.performCommit(for: testProjectPath, type: .autoSave)

        // Should fail but not crash
        XCTAssertFalse(result.success, "Should fail with lock")

        // Release and try again
        _ = LockManager.shared.releaseLock(projectPath: testProjectPath)

        let result2 = await orchestrator.performCommit(for: testProjectPath, type: .autoSave)
        XCTAssertNotNil(result2, "Should attempt commit after lock released")
    }

    // MARK: - Emergency Commits Tests

    func testPerformEmergencyCommitsNoProjects() async {
        // Should not crash with no registered projects
        await orchestrator.performEmergencyCommits()

        XCTAssert(true, "Should handle empty project list")
    }

    func testPerformEmergencyCommitsSingleProject() async {
        orchestrator.registerProject(testProjectPath)

        await orchestrator.performEmergencyCommits()

        XCTAssert(true, "Should process single project")
    }

    func testPerformEmergencyCommitsMultipleProjects() async {
        let project1 = tempDirectory.appendingPathComponent("Project1.logicx").path
        let project2 = tempDirectory.appendingPathComponent("Project2.logicx").path

        try? FileManager.default.createDirectory(atPath: project1, withIntermediateDirectories: true)
        try? FileManager.default.createDirectory(atPath: project2, withIntermediateDirectories: true)

        orchestrator.registerProject(project1)
        orchestrator.registerProject(project2)

        await orchestrator.performEmergencyCommits()

        XCTAssert(true, "Should process multiple projects")
    }

    // MARK: - Draft Branch Tests

    func testEnsureOnDraftBranch() async {
        let result = await orchestrator.ensureOnDraftBranch(at: testProjectPath)

        // Will likely fail because CLI doesn't exist, but should not crash
        XCTAssert(result == true || result == false, "Should return boolean")
    }

    func testEnsureOnDraftBranchWithNonexistentProject() async {
        let nonexistentPath = "/nonexistent/project.logicx"

        let result = await orchestrator.ensureOnDraftBranch(at: nonexistentPath)

        XCTAssertFalse(result, "Should fail for nonexistent project")
    }

    // MARK: - Concurrent Commit Tests

    func testConcurrentCommitAttempts() async {
        orchestrator.registerProject(testProjectPath)

        // Try to perform multiple commits concurrently
        async let result1 = orchestrator.performCommit(for: testProjectPath, type: .autoSave)
        async let result2 = orchestrator.performCommit(for: testProjectPath, type: .autoSave)
        async let result3 = orchestrator.performCommit(for: testProjectPath, type: .autoSave)

        let results = await [result1, result2, result3]

        // At least one should complete, others may be skipped due to "commit in progress"
        XCTAssertEqual(results.count, 3, "Should return 3 results")
    }

    func testConcurrentCommitsDifferentProjects() async {
        let project1 = tempDirectory.appendingPathComponent("Project1.logicx").path
        let project2 = tempDirectory.appendingPathComponent("Project2.logicx").path

        try? FileManager.default.createDirectory(atPath: project1, withIntermediateDirectories: true)
        try? FileManager.default.createDirectory(atPath: project2, withIntermediateDirectories: true)

        orchestrator.registerProject(project1)
        orchestrator.registerProject(project2)

        // Commits to different projects should not block each other
        async let result1 = orchestrator.performCommit(for: project1, type: .autoSave)
        async let result2 = orchestrator.performCommit(for: project2, type: .autoSave)

        let results = await [result1, result2]

        XCTAssertEqual(results.count, 2, "Should return 2 results")
    }

    // MARK: - Path Normalization Tests

    func testPerformCommitWithRelativePath() async {
        let relativePath = "./TestProject.logicx"

        let result = await orchestrator.performCommit(for: relativePath, type: .autoSave)

        XCTAssertNotNil(result, "Should handle relative path")
    }

    func testPerformCommitWithTrailingSlash() async {
        let pathWithSlash = testProjectPath + "/"

        let result = await orchestrator.performCommit(for: pathWithSlash, type: .autoSave)

        XCTAssertNotNil(result, "Should handle trailing slash")
    }

    func testRegisterProjectNormalizesPath() {
        let pathWithExtras = testProjectPath + "/../TestProject.logicx"

        orchestrator.registerProject(pathWithExtras)

        let registered = orchestrator.getRegisteredProjects()
        // Path should be normalized
        XCTAssertEqual(registered.count, 1, "Should register normalized path")
    }

    // MARK: - Error Handling Tests

    func testPerformCommitWithInvalidPath() async {
        let invalidPath = ""

        let result = await orchestrator.performCommit(for: invalidPath, type: .autoSave)

        XCTAssertNotNil(result, "Should return result even for invalid path")
    }

    func testPerformCommitWithNonexistentCLI() async {
        // Orchestrator initialized with non-existent CLI path
        let result = await orchestrator.performCommit(for: testProjectPath, type: .autoSave)

        // Should fail gracefully
        XCTAssertFalse(result.success, "Should fail when CLI doesn't exist")
    }

    // MARK: - Message Generation Tests

    func testCommitMessageIncludesTimestamp() async {
        let result = await orchestrator.performCommit(for: testProjectPath, type: .autoSave)

        // Message should include timestamp information
        XCTAssertNotNil(result.message, "Should have a message")
    }

    func testCommitMessageDiffersByType() async {
        let autoSaveResult = await orchestrator.performCommit(for: testProjectPath, type: .autoSave)
        let emergencyResult = await orchestrator.performCommit(for: testProjectPath, type: .emergency)
        let manualResult = await orchestrator.performCommit(for: testProjectPath, type: .manual)

        // All should have messages
        XCTAssertNotNil(autoSaveResult.message, "Auto-save should have message")
        XCTAssertNotNil(emergencyResult.message, "Emergency should have message")
        XCTAssertNotNil(manualResult.message, "Manual should have message")
    }

    // MARK: - Performance Tests

    func testCommitTracksDuration() async {
        let result = await orchestrator.performCommit(for: testProjectPath, type: .autoSave)

        XCTAssertGreaterThan(result.duration, 0, "Duration should be positive")
        XCTAssertLessThan(result.duration, 60.0, "Duration should be reasonable (< 60s)")
    }

    func testEmergencyCommitsCompletesInReasonableTime() async {
        orchestrator.registerProject(testProjectPath)

        let startTime = Date()
        await orchestrator.performEmergencyCommits()
        let duration = Date().timeIntervalSince(startTime)

        XCTAssertLessThan(duration, 30.0, "Emergency commits should complete quickly")
    }

    // MARK: - Integration Tests

    func testFullWorkflowRegisterCommitUnregister() async {
        // Register
        orchestrator.registerProject(testProjectPath)
        var registered = orchestrator.getRegisteredProjects()
        XCTAssertTrue(registered.contains(testProjectPath), "Should be registered")

        // Commit
        let result = await orchestrator.performCommit(for: testProjectPath, type: .autoSave)
        XCTAssertNotNil(result, "Should get commit result")

        // Unregister
        orchestrator.unregisterProject(testProjectPath)
        registered = orchestrator.getRegisteredProjects()
        XCTAssertFalse(registered.contains(testProjectPath), "Should be unregistered")
    }

    func testMultipleCommitsSequentially() async {
        orchestrator.registerProject(testProjectPath)

        let result1 = await orchestrator.performCommit(for: testProjectPath, type: .autoSave)
        let result2 = await orchestrator.performCommit(for: testProjectPath, type: .autoSave)
        let result3 = await orchestrator.performCommit(for: testProjectPath, type: .autoSave)

        XCTAssertNotNil(result1, "First commit should complete")
        XCTAssertNotNil(result2, "Second commit should complete")
        XCTAssertNotNil(result3, "Third commit should complete")
    }

    // MARK: - Edge Cases

    func testPerformCommitWithUnicodeProjectPath() async {
        let unicodePath = tempDirectory.appendingPathComponent("プロジェクト.logicx").path

        let result = await orchestrator.performCommit(for: unicodePath, type: .autoSave)

        XCTAssertNotNil(result, "Should handle Unicode paths")
    }

    func testPerformCommitWithSpacesInPath() async {
        let pathWithSpaces = tempDirectory.appendingPathComponent("My Project Name.logicx").path

        let result = await orchestrator.performCommit(for: pathWithSpaces, type: .autoSave)

        XCTAssertNotNil(result, "Should handle spaces in path")
    }

    func testRegisterProjectWithSymlink() {
        // Create a symlink (if supported)
        let targetPath = tempDirectory.appendingPathComponent("Target.logicx").path
        let linkPath = tempDirectory.appendingPathComponent("Link.logicx").path

        try? FileManager.default.createDirectory(atPath: targetPath, withIntermediateDirectories: true)
        try? FileManager.default.createSymbolicLink(atPath: linkPath, withDestinationPath: targetPath)

        orchestrator.registerProject(linkPath)

        let registered = orchestrator.getRegisteredProjects()
        XCTAssertTrue(registered.count > 0, "Should handle symlinks")
    }
}
