import XCTest
@testable import OxVCS_LaunchAgent

final class LockManagerTests: XCTestCase {
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

        let oxenDir = (testProjectPath as NSString).appendingPathComponent(".oxen")
        try? FileManager.default.createDirectory(atPath: oxenDir, withIntermediateDirectories: true)
    }

    override func tearDown() {
        // Clean up temporary directory
        try? FileManager.default.removeItem(at: tempDirectory)
        super.tearDown()
    }

    // MARK: - Lock Acquisition Tests

    func testAcquireLock_Success() {
        let success = LockManager.shared.acquireLock(projectPath: testProjectPath)
        XCTAssertTrue(success, "Should successfully acquire lock on unlocked project")
    }

    func testAcquireLock_AlreadyLocked() {
        // First acquisition should succeed
        let firstAcquire = LockManager.shared.acquireLock(projectPath: testProjectPath)
        XCTAssertTrue(firstAcquire, "First lock acquisition should succeed")

        // Second acquisition should fail
        let secondAcquire = LockManager.shared.acquireLock(projectPath: testProjectPath)
        XCTAssertFalse(secondAcquire, "Should not acquire lock when already locked")
    }

    func testAcquireLock_CustomTimeout() {
        let success = LockManager.shared.acquireLock(projectPath: testProjectPath, timeoutHours: 48)
        XCTAssertTrue(success, "Should acquire lock with custom timeout")

        if let lock = LockManager.shared.getLockInfo(projectPath: testProjectPath) {
            let expectedExpiration = Date().addingTimeInterval(48 * 3600)
            let timeDifference = abs(lock.expiresAt.timeIntervalSince(expectedExpiration))
            XCTAssertLessThan(timeDifference, 2, "Lock expiration should be approximately 48 hours from now")
        } else {
            XCTFail("Should have lock info after acquiring lock")
        }
    }

    // MARK: - Lock Release Tests

    func testReleaseLock_Success() {
        // Acquire lock first
        _ = LockManager.shared.acquireLock(projectPath: testProjectPath)

        // Release should succeed
        let success = LockManager.shared.releaseLock(projectPath: testProjectPath)
        XCTAssertTrue(success, "Should successfully release owned lock")
    }

    func testReleaseLock_NotLocked() {
        // Try to release non-existent lock
        let success = LockManager.shared.releaseLock(projectPath: testProjectPath)
        XCTAssertFalse(success, "Should fail to release when no lock exists")
    }

    func testReleaseLock_ReacquireAfterRelease() {
        // Acquire, release, then acquire again
        _ = LockManager.shared.acquireLock(projectPath: testProjectPath)
        _ = LockManager.shared.releaseLock(projectPath: testProjectPath)

        let secondAcquire = LockManager.shared.acquireLock(projectPath: testProjectPath)
        XCTAssertTrue(secondAcquire, "Should be able to reacquire lock after releasing")
    }

    // MARK: - Lock Status Tests

    func testIsLocked_UnlockedProject() {
        let isLocked = LockManager.shared.isLocked(projectPath: testProjectPath)
        XCTAssertFalse(isLocked, "New project should not be locked")
    }

    func testIsLocked_LockedProject() {
        _ = LockManager.shared.acquireLock(projectPath: testProjectPath)

        let isLocked = LockManager.shared.isLocked(projectPath: testProjectPath)
        XCTAssertTrue(isLocked, "Project should be locked after acquiring lock")
    }

    func testGetLockInfo_NoLock() {
        let lockInfo = LockManager.shared.getLockInfo(projectPath: testProjectPath)
        XCTAssertNil(lockInfo, "Should return nil for unlocked project")
    }

    func testGetLockInfo_WithLock() {
        _ = LockManager.shared.acquireLock(projectPath: testProjectPath)

        let lockInfo = LockManager.shared.getLockInfo(projectPath: testProjectPath)
        XCTAssertNotNil(lockInfo, "Should return lock info for locked project")

        if let lock = lockInfo {
            XCTAssertEqual(lock.projectPath, testProjectPath)
            XCTAssertFalse(lock.isExpired, "Newly created lock should not be expired")
            XCTAssertGreaterThan(lock.remainingHours, 0, "Lock should have remaining time")
        }
    }

    // MARK: - Force Break Tests

    func testForceBreakLock_Success() {
        // Acquire lock first
        _ = LockManager.shared.acquireLock(projectPath: testProjectPath)

        // Force break should succeed
        let success = LockManager.shared.forceBreakLock(projectPath: testProjectPath)
        XCTAssertTrue(success, "Should successfully force-break lock")

        // Verify lock is removed
        let isLocked = LockManager.shared.isLocked(projectPath: testProjectPath)
        XCTAssertFalse(isLocked, "Project should not be locked after force-break")
    }

    func testForceBreakLock_NoLock() {
        let success = LockManager.shared.forceBreakLock(projectPath: testProjectPath)
        XCTAssertFalse(success, "Should fail to force-break when no lock exists")
    }

    // MARK: - Expiration Tests

    func testExpiredLock_AutomaticallyRemoved() {
        // Create a lock with very short timeout (1 second = 1/3600 hours)
        let success = LockManager.shared.acquireLock(projectPath: testProjectPath, timeoutHours: 0)
        XCTAssertTrue(success, "Should acquire lock even with 0 timeout")

        // Wait a moment for expiration
        sleep(1)

        // Try to acquire again - should succeed because previous lock is expired
        let secondAcquire = LockManager.shared.acquireLock(projectPath: testProjectPath)
        XCTAssertTrue(secondAcquire, "Should be able to acquire lock after previous one expired")
    }

    // MARK: - Lock File Persistence Tests

    func testLockFilePersistence() {
        // Acquire lock
        _ = LockManager.shared.acquireLock(projectPath: testProjectPath)

        // Verify lock file exists
        let lockFilePath = (testProjectPath as NSString).appendingPathComponent(".oxen/locks.json")
        let lockFileExists = FileManager.default.fileExists(atPath: lockFilePath)
        XCTAssertTrue(lockFileExists, "Lock file should exist on disk")

        // Release lock
        _ = LockManager.shared.releaseLock(projectPath: testProjectPath)

        // Verify lock file is removed
        let lockFileExistsAfterRelease = FileManager.default.fileExists(atPath: lockFilePath)
        XCTAssertFalse(lockFileExistsAfterRelease, "Lock file should be removed after release")
    }

    func testLockFileFormat() throws {
        // Acquire lock
        _ = LockManager.shared.acquireLock(projectPath: testProjectPath, timeoutHours: 24)

        // Read lock file
        let lockFilePath = (testProjectPath as NSString).appendingPathComponent(".oxen/locks.json")
        let data = try Data(contentsOf: URL(fileURLWithPath: lockFilePath))
        let lock = try JSONDecoder().decode(ProjectLock.self, from: data)

        // Verify lock structure
        XCTAssertEqual(lock.projectPath, testProjectPath)
        XCTAssertFalse(lock.lockId.isEmpty)
        XCTAssertFalse(lock.lockedBy.isEmpty)
        XCTAssertGreaterThan(lock.expiresAt, lock.acquiredAt)
    }

    // MARK: - Concurrent Access Tests

    func testConcurrentLockAttempts() {
        let expectation = XCTestExpectation(description: "Concurrent lock attempts")
        var successCount = 0
        var failureCount = 0
        let totalAttempts = 10

        DispatchQueue.concurrentPerform(iterations: totalAttempts) { _ in
            if LockManager.shared.acquireLock(projectPath: testProjectPath) {
                successCount += 1
            } else {
                failureCount += 1
            }
        }

        // Only one should succeed
        XCTAssertEqual(successCount, 1, "Only one concurrent lock attempt should succeed")
        XCTAssertEqual(failureCount, totalAttempts - 1, "Other attempts should fail")

        expectation.fulfill()
        wait(for: [expectation], timeout: 5)
    }

    // MARK: - User Identifier Tests

    func testLockOwnerIdentifier() {
        _ = LockManager.shared.acquireLock(projectPath: testProjectPath)

        if let lock = LockManager.shared.getLockInfo(projectPath: testProjectPath) {
            let expectedFormat = "\(ProcessInfo.processInfo.userName)@\(ProcessInfo.processInfo.hostName)"
            XCTAssertEqual(lock.lockedBy, expectedFormat, "Lock owner should be in format user@hostname")
        } else {
            XCTFail("Should have lock info")
        }
    }
}
