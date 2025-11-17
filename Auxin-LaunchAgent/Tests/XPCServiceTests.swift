import XCTest
@testable import Auxin_LaunchAgent

/// Mock XPC service implementation for testing
class MockXPCService: NSObject, OxenDaemonXPCProtocol {
    var initializeProjectCalled = false
    var registerProjectCalled = false
    var unregisterProjectCalled = false
    var getMonitoredProjectsCalled = false
    var commitProjectCalled = false
    var getStatusCalled = false
    var getCommitHistoryCalled = false
    var restoreProjectCalled = false
    var pauseMonitoringCalled = false
    var resumeMonitoringCalled = false
    var pingCalled = false

    var lastProjectPath: String?
    var lastMessage: String?
    var lastCommitId: String?
    var lastLimit: Int?

    func initializeProject(_ projectPath: String, withReply reply: @escaping (Bool, String?) -> Void) {
        initializeProjectCalled = true
        lastProjectPath = projectPath
        reply(true, nil)
    }

    func registerProject(_ projectPath: String, withReply reply: @escaping (Bool, String?) -> Void) {
        registerProjectCalled = true
        lastProjectPath = projectPath
        reply(true, nil)
    }

    func unregisterProject(_ projectPath: String, withReply reply: @escaping (Bool, String?) -> Void) {
        unregisterProjectCalled = true
        lastProjectPath = projectPath
        reply(true, nil)
    }

    func getMonitoredProjects(withReply reply: @escaping ([String]) -> Void) {
        getMonitoredProjectsCalled = true
        reply(["/test/project1.logicx", "/test/project2.logicx"])
    }

    func commitProject(_ projectPath: String, message: String?, withReply reply: @escaping (String?, String?) -> Void) {
        commitProjectCalled = true
        lastProjectPath = projectPath
        lastMessage = message
        reply("abc123", nil)
    }

    func getStatus(withReply reply: @escaping ([String : Any]) -> Void) {
        getStatusCalled = true
        reply([
            "isRunning": true,
            "projectCount": 2,
            "lastCommit": "2025-10-28T10:00:00Z"
        ])
    }

    func getCommitHistory(for projectPath: String, limit: Int, withReply reply: @escaping ([[String : Any]]) -> Void) {
        getCommitHistoryCalled = true
        lastProjectPath = projectPath
        lastLimit = limit
        reply([
            ["id": "abc123", "message": "Test commit", "timestamp": "2025-10-28"],
            ["id": "def456", "message": "Another commit", "timestamp": "2025-10-27"]
        ])
    }

    func restoreProject(_ projectPath: String, toCommit commitId: String, withReply reply: @escaping (Bool, String?) -> Void) {
        restoreProjectCalled = true
        lastProjectPath = projectPath
        lastCommitId = commitId
        reply(true, nil)
    }

    func pauseMonitoring(for projectPath: String, withReply reply: @escaping (Bool) -> Void) {
        pauseMonitoringCalled = true
        lastProjectPath = projectPath
        reply(true)
    }

    func resumeMonitoring(for projectPath: String, withReply reply: @escaping (Bool) -> Void) {
        resumeMonitoringCalled = true
        lastProjectPath = projectPath
        reply(true)
    }

    func ping(withReply reply: @escaping (Bool) -> Void) {
        pingCalled = true
        reply(true)
    }

    // MARK: - Lock Management Methods

    func acquireLock(
        for projectPath: String,
        timeoutHours: Int,
        withReply reply: @escaping (Bool, String?) -> Void
    ) {
        lastProjectPath = projectPath
        reply(true, nil)
    }

    func releaseLock(
        for projectPath: String,
        withReply reply: @escaping (Bool, String?) -> Void
    ) {
        lastProjectPath = projectPath
        reply(true, nil)
    }

    func forceBreakLock(
        for projectPath: String,
        withReply reply: @escaping (Bool, String?) -> Void
    ) {
        lastProjectPath = projectPath
        reply(true, nil)
    }

    func getLockInfo(
        for projectPath: String,
        withReply reply: @escaping ([String: Any]?) -> Void
    ) {
        lastProjectPath = projectPath
        reply(nil)
    }

    // MARK: - Configuration Methods

    func getConfiguration(
        withReply reply: @escaping ([String: Any]) -> Void
    ) {
        reply(["debounceTime": 30, "lockTimeout": 24])
    }

    func setDebounceTime(
        _ seconds: Int,
        withReply reply: @escaping (Bool) -> Void
    ) {
        reply(true)
    }

    func setLockTimeout(
        _ hours: Int,
        withReply reply: @escaping (Bool) -> Void
    ) {
        reply(true)
    }
}

final class XPCServiceTests: XCTestCase {
    var mockService: MockXPCService!

    override func setUp() {
        super.setUp()
        mockService = MockXPCService()
    }

    override func tearDown() {
        mockService = nil
        super.tearDown()
    }

    // MARK: - Protocol Conformance Tests

    func testMockServiceConformsToProtocol() {
        XCTAssertTrue(mockService is OxenDaemonXPCProtocol, "Mock should conform to protocol")
    }

    // MARK: - Initialize Project Tests

    func testInitializeProject() {
        let expectation = XCTestExpectation(description: "Initialize should complete")

        mockService.initializeProject("/test/project.logicx") { success, error in
            XCTAssertTrue(success, "Should succeed")
            XCTAssertNil(error, "Should have no error")
            expectation.fulfill()
        }

        wait(for: [expectation], timeout: 1.0)
        XCTAssertTrue(mockService.initializeProjectCalled, "Method should be called")
        XCTAssertEqual(mockService.lastProjectPath, "/test/project.logicx", "Should store path")
    }

    func testInitializeProjectWithEmptyPath() {
        let expectation = XCTestExpectation(description: "Should handle empty path")

        mockService.initializeProject("") { success, error in
            expectation.fulfill()
        }

        wait(for: [expectation], timeout: 1.0)
        XCTAssertEqual(mockService.lastProjectPath, "", "Should accept empty path")
    }

    // MARK: - Register Project Tests

    func testRegisterProject() {
        let expectation = XCTestExpectation(description: "Register should complete")

        mockService.registerProject("/test/project.logicx") { success, error in
            XCTAssertTrue(success, "Should succeed")
            XCTAssertNil(error, "Should have no error")
            expectation.fulfill()
        }

        wait(for: [expectation], timeout: 1.0)
        XCTAssertTrue(mockService.registerProjectCalled, "Method should be called")
    }

    func testRegisterMultipleProjects() {
        let expectation1 = XCTestExpectation(description: "Register project 1")
        let expectation2 = XCTestExpectation(description: "Register project 2")

        mockService.registerProject("/project1.logicx") { _, _ in expectation1.fulfill() }
        mockService.registerProject("/project2.logicx") { _, _ in expectation2.fulfill() }

        wait(for: [expectation1, expectation2], timeout: 1.0)
    }

    // MARK: - Unregister Project Tests

    func testUnregisterProject() {
        let expectation = XCTestExpectation(description: "Unregister should complete")

        mockService.unregisterProject("/test/project.logicx") { success, error in
            XCTAssertTrue(success, "Should succeed")
            expectation.fulfill()
        }

        wait(for: [expectation], timeout: 1.0)
        XCTAssertTrue(mockService.unregisterProjectCalled, "Method should be called")
    }

    // MARK: - Get Monitored Projects Tests

    func testGetMonitoredProjects() {
        let expectation = XCTestExpectation(description: "Get projects should complete")

        mockService.getMonitoredProjects { projects in
            XCTAssertEqual(projects.count, 2, "Should return 2 projects")
            XCTAssertTrue(projects.contains("/test/project1.logicx"), "Should contain project1")
            XCTAssertTrue(projects.contains("/test/project2.logicx"), "Should contain project2")
            expectation.fulfill()
        }

        wait(for: [expectation], timeout: 1.0)
        XCTAssertTrue(mockService.getMonitoredProjectsCalled, "Method should be called")
    }

    func testGetMonitoredProjectsMultipleCalls() {
        let expectation1 = XCTestExpectation(description: "First call")
        let expectation2 = XCTestExpectation(description: "Second call")

        mockService.getMonitoredProjects { _ in expectation1.fulfill() }
        mockService.getMonitoredProjects { _ in expectation2.fulfill() }

        wait(for: [expectation1, expectation2], timeout: 1.0)
    }

    // MARK: - Commit Project Tests

    func testCommitProject() {
        let expectation = XCTestExpectation(description: "Commit should complete")

        mockService.commitProject("/test/project.logicx", message: "Test commit") { commitId, error in
            XCTAssertEqual(commitId, "abc123", "Should return commit ID")
            XCTAssertNil(error, "Should have no error")
            expectation.fulfill()
        }

        wait(for: [expectation], timeout: 1.0)
        XCTAssertTrue(mockService.commitProjectCalled, "Method should be called")
        XCTAssertEqual(mockService.lastMessage, "Test commit", "Should store message")
    }

    func testCommitProjectWithNilMessage() {
        let expectation = XCTestExpectation(description: "Commit with nil message")

        mockService.commitProject("/test/project.logicx", message: nil) { commitId, error in
            XCTAssertNotNil(commitId, "Should still return commit ID")
            expectation.fulfill()
        }

        wait(for: [expectation], timeout: 1.0)
        XCTAssertNil(mockService.lastMessage, "Message should be nil")
    }

    func testCommitProjectWithEmptyMessage() {
        let expectation = XCTestExpectation(description: "Commit with empty message")

        mockService.commitProject("/test/project.logicx", message: "") { commitId, error in
            expectation.fulfill()
        }

        wait(for: [expectation], timeout: 1.0)
        XCTAssertEqual(mockService.lastMessage, "", "Should accept empty message")
    }

    // MARK: - Get Status Tests

    func testGetStatus() {
        let expectation = XCTestExpectation(description: "Get status should complete")

        mockService.getStatus { status in
            XCTAssertTrue(status["isRunning"] as? Bool == true, "Should be running")
            XCTAssertEqual(status["projectCount"] as? Int, 2, "Should have 2 projects")
            XCTAssertNotNil(status["lastCommit"], "Should have last commit timestamp")
            expectation.fulfill()
        }

        wait(for: [expectation], timeout: 1.0)
        XCTAssertTrue(mockService.getStatusCalled, "Method should be called")
    }

    func testGetStatusMultipleTimes() {
        let expectation1 = XCTestExpectation(description: "First status call")
        let expectation2 = XCTestExpectation(description: "Second status call")

        mockService.getStatus { _ in expectation1.fulfill() }
        mockService.getStatus { _ in expectation2.fulfill() }

        wait(for: [expectation1, expectation2], timeout: 1.0)
    }

    // MARK: - Get Commit History Tests

    func testGetCommitHistory() {
        let expectation = XCTestExpectation(description: "Get history should complete")

        mockService.getCommitHistory(for: "/test/project.logicx", limit: 10) { commits in
            XCTAssertEqual(commits.count, 2, "Should return 2 commits")
            XCTAssertEqual(commits[0]["id"] as? String, "abc123", "First commit ID should match")
            XCTAssertEqual(commits[1]["id"] as? String, "def456", "Second commit ID should match")
            expectation.fulfill()
        }

        wait(for: [expectation], timeout: 1.0)
        XCTAssertTrue(mockService.getCommitHistoryCalled, "Method should be called")
        XCTAssertEqual(mockService.lastLimit, 10, "Should store limit")
    }

    func testGetCommitHistoryWithZeroLimit() {
        let expectation = XCTestExpectation(description: "Get history with zero limit")

        mockService.getCommitHistory(for: "/test/project.logicx", limit: 0) { _ in
            expectation.fulfill()
        }

        wait(for: [expectation], timeout: 1.0)
        XCTAssertEqual(mockService.lastLimit, 0, "Should accept zero limit")
    }

    func testGetCommitHistoryWithLargeLimit() {
        let expectation = XCTestExpectation(description: "Get history with large limit")

        mockService.getCommitHistory(for: "/test/project.logicx", limit: 1000) { _ in
            expectation.fulfill()
        }

        wait(for: [expectation], timeout: 1.0)
        XCTAssertEqual(mockService.lastLimit, 1000, "Should accept large limit")
    }

    // MARK: - Restore Project Tests

    func testRestoreProject() {
        let expectation = XCTestExpectation(description: "Restore should complete")

        mockService.restoreProject("/test/project.logicx", toCommit: "abc123") { success, error in
            XCTAssertTrue(success, "Should succeed")
            XCTAssertNil(error, "Should have no error")
            expectation.fulfill()
        }

        wait(for: [expectation], timeout: 1.0)
        XCTAssertTrue(mockService.restoreProjectCalled, "Method should be called")
        XCTAssertEqual(mockService.lastCommitId, "abc123", "Should store commit ID")
    }

    func testRestoreProjectWithShortHash() {
        let expectation = XCTestExpectation(description: "Restore with short hash")

        mockService.restoreProject("/test/project.logicx", toCommit: "abc123") { _, _ in
            expectation.fulfill()
        }

        wait(for: [expectation], timeout: 1.0)
        XCTAssertEqual(mockService.lastCommitId, "abc123", "Should accept short hash")
    }

    func testRestoreProjectWithFullHash() {
        let expectation = XCTestExpectation(description: "Restore with full hash")
        let fullHash = "abc123def456abc123def456abc123def456abcd"

        mockService.restoreProject("/test/project.logicx", toCommit: fullHash) { _, _ in
            expectation.fulfill()
        }

        wait(for: [expectation], timeout: 1.0)
        XCTAssertEqual(mockService.lastCommitId, fullHash, "Should accept full hash")
    }

    // MARK: - Pause Monitoring Tests

    func testPauseMonitoring() {
        let expectation = XCTestExpectation(description: "Pause should complete")

        mockService.pauseMonitoring(for: "/test/project.logicx") { success in
            XCTAssertTrue(success, "Should succeed")
            expectation.fulfill()
        }

        wait(for: [expectation], timeout: 1.0)
        XCTAssertTrue(mockService.pauseMonitoringCalled, "Method should be called")
    }

    // MARK: - Resume Monitoring Tests

    func testResumeMonitoring() {
        let expectation = XCTestExpectation(description: "Resume should complete")

        mockService.resumeMonitoring(for: "/test/project.logicx") { success in
            XCTAssertTrue(success, "Should succeed")
            expectation.fulfill()
        }

        wait(for: [expectation], timeout: 1.0)
        XCTAssertTrue(mockService.resumeMonitoringCalled, "Method should be called")
    }

    // MARK: - Ping Tests

    func testPing() {
        let expectation = XCTestExpectation(description: "Ping should complete")

        mockService.ping { isAlive in
            XCTAssertTrue(isAlive, "Should be alive")
            expectation.fulfill()
        }

        wait(for: [expectation], timeout: 1.0)
        XCTAssertTrue(mockService.pingCalled, "Method should be called")
    }

    func testPingMultipleTimes() {
        let expectation1 = XCTestExpectation(description: "First ping")
        let expectation2 = XCTestExpectation(description: "Second ping")
        let expectation3 = XCTestExpectation(description: "Third ping")

        mockService.ping { _ in expectation1.fulfill() }
        mockService.ping { _ in expectation2.fulfill() }
        mockService.ping { _ in expectation3.fulfill() }

        wait(for: [expectation1, expectation2, expectation3], timeout: 1.0)
    }

    // MARK: - Workflow Integration Tests

    func testFullWorkflowInitializeRegisterCommit() {
        let initExpectation = XCTestExpectation(description: "Initialize")
        let registerExpectation = XCTestExpectation(description: "Register")
        let commitExpectation = XCTestExpectation(description: "Commit")

        let projectPath = "/test/project.logicx"

        mockService.initializeProject(projectPath) { success, _ in
            XCTAssertTrue(success, "Initialize should succeed")
            initExpectation.fulfill()
        }

        mockService.registerProject(projectPath) { success, _ in
            XCTAssertTrue(success, "Register should succeed")
            registerExpectation.fulfill()
        }

        mockService.commitProject(projectPath, message: "Initial commit") { commitId, error in
            XCTAssertNotNil(commitId, "Should return commit ID")
            XCTAssertNil(error, "Should have no error")
            commitExpectation.fulfill()
        }

        wait(for: [initExpectation, registerExpectation, commitExpectation], timeout: 1.0)
    }

    func testFullWorkflowPauseResume() {
        let pauseExpectation = XCTestExpectation(description: "Pause")
        let resumeExpectation = XCTestExpectation(description: "Resume")

        let projectPath = "/test/project.logicx"

        mockService.pauseMonitoring(for: projectPath) { success in
            XCTAssertTrue(success, "Pause should succeed")
            pauseExpectation.fulfill()
        }

        mockService.resumeMonitoring(for: projectPath) { success in
            XCTAssertTrue(success, "Resume should succeed")
            resumeExpectation.fulfill()
        }

        wait(for: [pauseExpectation, resumeExpectation], timeout: 1.0)
    }

    // MARK: - Error Handling Tests

    func testAllMethodsHandleInvalidPaths() {
        let invalidPath = ""
        var completionCount = 0
        let totalMethods = 7

        mockService.initializeProject(invalidPath) { _, _ in completionCount += 1 }
        mockService.registerProject(invalidPath) { _, _ in completionCount += 1 }
        mockService.unregisterProject(invalidPath) { _, _ in completionCount += 1 }
        mockService.commitProject(invalidPath, message: nil) { _, _ in completionCount += 1 }
        mockService.restoreProject(invalidPath, toCommit: "abc") { _, _ in completionCount += 1 }
        mockService.pauseMonitoring(for: invalidPath) { _ in completionCount += 1 }
        mockService.resumeMonitoring(for: invalidPath) { _ in completionCount += 1 }

        // Give time for async completions
        let expectation = XCTestExpectation(description: "All methods complete")
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) {
            expectation.fulfill()
        }

        wait(for: [expectation], timeout: 1.0)
        XCTAssertEqual(completionCount, totalMethods, "All methods should complete")
    }

    // MARK: - Concurrent Access Tests

    func testConcurrentRegisterCalls() {
        let expectation = XCTestExpectation(description: "Concurrent registrations")
        expectation.expectedFulfillmentCount = 5

        for i in 0..<5 {
            mockService.registerProject("/project\(i).logicx") { _, _ in
                expectation.fulfill()
            }
        }

        wait(for: [expectation], timeout: 1.0)
    }

    func testConcurrentStatusChecks() {
        let expectation = XCTestExpectation(description: "Concurrent status checks")
        expectation.expectedFulfillmentCount = 10

        for _ in 0..<10 {
            mockService.getStatus { _ in
                expectation.fulfill()
            }
        }

        wait(for: [expectation], timeout: 1.0)
    }

    // MARK: - Performance Tests

    func testGetStatusPerformance() {
        measure {
            let expectation = XCTestExpectation(description: "Status check")

            mockService.getStatus { _ in
                expectation.fulfill()
            }

            wait(for: [expectation], timeout: 1.0)
        }
    }

    func testGetMonitoredProjectsPerformance() {
        measure {
            let expectation = XCTestExpectation(description: "Get projects")

            mockService.getMonitoredProjects { _ in
                expectation.fulfill()
            }

            wait(for: [expectation], timeout: 1.0)
        }
    }
}
