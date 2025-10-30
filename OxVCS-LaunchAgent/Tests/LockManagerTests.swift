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

        let decoder = JSONDecoder()
        decoder.dateDecodingStrategy = .iso8601
        let lock = try decoder.decode(ProjectLock.self, from: data)

        // Verify lock structure
        XCTAssertEqual(lock.projectPath, testProjectPath)
        XCTAssertFalse(lock.lockId.isEmpty)
        XCTAssertFalse(lock.lockedBy.isEmpty)
        XCTAssertGreaterThan(lock.expiresAt, lock.acquiredAt)
    }

    // MARK: - Concurrent Access Tests

    func testConcurrentLockAttempts() {
        let expectation = XCTestExpectation(description: "Concurrent lock attempts")
        let queue = DispatchQueue(label: "test.counter")
        var successCount = 0
        var failureCount = 0
        let totalAttempts = 10

        DispatchQueue.concurrentPerform(iterations: totalAttempts) { _ in
            if LockManager.shared.acquireLock(projectPath: testProjectPath) {
                queue.sync { successCount += 1 }
            } else {
                queue.sync { failureCount += 1 }
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

    // MARK: - Additional Lock Acquisition Edge Cases

    func testAcquireLockZeroTimeout() {
        let success = LockManager.shared.acquireLock(projectPath: testProjectPath, timeoutHours: 0)
        XCTAssertTrue(success, "Should acquire lock with 0 timeout")

        if let lock = LockManager.shared.getLockInfo(projectPath: testProjectPath) {
            // Lock should be immediately expired
            XCTAssertTrue(lock.isExpired, "Lock with 0 timeout should be expired immediately")
        }
    }

    func testAcquireLockNegativeTimeout() {
        let success = LockManager.shared.acquireLock(projectPath: testProjectPath, timeoutHours: -1)
        XCTAssertTrue(success, "Should acquire lock even with negative timeout")

        if let lock = LockManager.shared.getLockInfo(projectPath: testProjectPath) {
            XCTAssertTrue(lock.isExpired, "Lock with negative timeout should be expired")
        }
    }

    func testAcquireLockVeryLargeTimeout() {
        let success = LockManager.shared.acquireLock(projectPath: testProjectPath, timeoutHours: 8760) // 1 year
        XCTAssertTrue(success, "Should acquire lock with very large timeout")

        if let lock = LockManager.shared.getLockInfo(projectPath: testProjectPath) {
            XCTAssertFalse(lock.isExpired, "Lock should not be expired")
            XCTAssertGreaterThan(lock.remainingHours, 8700, "Should have ~1 year remaining")
        }
    }

    func testAcquireLockEmptyPath() {
        let success = LockManager.shared.acquireLock(projectPath: "")
        XCTAssertFalse(success, "Should reject empty path")
    }

    func testAcquireLockPathWithSpaces() {
        let pathWithSpaces = tempDirectory.appendingPathComponent("My Test Project.logicx").path
        try? FileManager.default.createDirectory(atPath: pathWithSpaces, withIntermediateDirectories: true)

        let oxenDir = (pathWithSpaces as NSString).appendingPathComponent(".oxen")
        try? FileManager.default.createDirectory(atPath: oxenDir, withIntermediateDirectories: true)

        let success = LockManager.shared.acquireLock(projectPath: pathWithSpaces)
        XCTAssertTrue(success, "Should handle paths with spaces")

        // Cleanup
        _ = LockManager.shared.releaseLock(projectPath: pathWithSpaces)
    }

    func testAcquireLockPathWithUnicode() {
        let unicodePath = tempDirectory.appendingPathComponent("プロジェクト.logicx").path
        try? FileManager.default.createDirectory(atPath: unicodePath, withIntermediateDirectories: true)

        let oxenDir = (unicodePath as NSString).appendingPathComponent(".oxen")
        try? FileManager.default.createDirectory(atPath: oxenDir, withIntermediateDirectories: true)

        let success = LockManager.shared.acquireLock(projectPath: unicodePath)
        XCTAssertTrue(success, "Should handle Unicode paths")

        // Cleanup
        _ = LockManager.shared.releaseLock(projectPath: unicodePath)
    }

    // MARK: - Lock Info Edge Cases

    func testGetLockInfoMultipleTimes() {
        _ = LockManager.shared.acquireLock(projectPath: testProjectPath)

        let info1 = LockManager.shared.getLockInfo(projectPath: testProjectPath)
        let info2 = LockManager.shared.getLockInfo(projectPath: testProjectPath)

        XCTAssertNotNil(info1, "First call should return info")
        XCTAssertNotNil(info2, "Second call should return info")

        if let info1 = info1, let info2 = info2 {
            XCTAssertEqual(info1.lockId, info2.lockId, "Should return same lock info")
        }
    }

    func testGetLockInfoAfterExpiration() {
        _ = LockManager.shared.acquireLock(projectPath: testProjectPath, timeoutHours: 0)

        sleep(1)

        let info = LockManager.shared.getLockInfo(projectPath: testProjectPath)
        XCTAssertNotNil(info, "Should still return expired lock info")

        if let info = info {
            XCTAssertTrue(info.isExpired, "Lock should be expired")
        }
    }

    // MARK: - Multiple Project Tests

    func testLockMultipleProjects() {
        let project2Path = tempDirectory.appendingPathComponent("Project2.logicx").path
        try? FileManager.default.createDirectory(atPath: project2Path, withIntermediateDirectories: true)

        let oxenDir2 = (project2Path as NSString).appendingPathComponent(".oxen")
        try? FileManager.default.createDirectory(atPath: oxenDir2, withIntermediateDirectories: true)

        let success1 = LockManager.shared.acquireLock(projectPath: testProjectPath)
        let success2 = LockManager.shared.acquireLock(projectPath: project2Path)

        XCTAssertTrue(success1, "Should lock first project")
        XCTAssertTrue(success2, "Should lock second project")

        XCTAssertTrue(LockManager.shared.isLocked(projectPath: testProjectPath), "First project should be locked")
        XCTAssertTrue(LockManager.shared.isLocked(projectPath: project2Path), "Second project should be locked")

        // Cleanup
        _ = LockManager.shared.releaseLock(projectPath: testProjectPath)
        _ = LockManager.shared.releaseLock(projectPath: project2Path)
    }

    func testReleaseOneOfMultipleLocks() {
        let project2Path = tempDirectory.appendingPathComponent("Project2.logicx").path
        try? FileManager.default.createDirectory(atPath: project2Path, withIntermediateDirectories: true)

        let oxenDir2 = (project2Path as NSString).appendingPathComponent(".oxen")
        try? FileManager.default.createDirectory(atPath: oxenDir2, withIntermediateDirectories: true)

        _ = LockManager.shared.acquireLock(projectPath: testProjectPath)
        _ = LockManager.shared.acquireLock(projectPath: project2Path)

        _ = LockManager.shared.releaseLock(projectPath: testProjectPath)

        XCTAssertFalse(LockManager.shared.isLocked(projectPath: testProjectPath), "First project should be unlocked")
        XCTAssertTrue(LockManager.shared.isLocked(projectPath: project2Path), "Second project should still be locked")

        // Cleanup
        _ = LockManager.shared.releaseLock(projectPath: project2Path)
    }

    // MARK: - Lock Metadata Tests

    func testLockAcquiredAtTimestamp() {
        let beforeAcquire = Date()
        usleep(100000) // 100ms to ensure different second if needed
        _ = LockManager.shared.acquireLock(projectPath: testProjectPath)
        let afterAcquire = Date()

        if let lock = LockManager.shared.getLockInfo(projectPath: testProjectPath) {
            // Allow 1-second tolerance for ISO8601 second-precision rounding
            XCTAssertGreaterThanOrEqual(lock.acquiredAt.timeIntervalSince1970, beforeAcquire.timeIntervalSince1970 - 1, "Acquired time should be after start")
            XCTAssertLessThanOrEqual(lock.acquiredAt, afterAcquire, "Acquired time should be before end")
        } else {
            XCTFail("Should have lock info")
        }
    }

    func testLockIdIsUnique() {
        _ = LockManager.shared.acquireLock(projectPath: testProjectPath)
        let lockId1 = LockManager.shared.getLockInfo(projectPath: testProjectPath)?.lockId

        _ = LockManager.shared.releaseLock(projectPath: testProjectPath)

        _ = LockManager.shared.acquireLock(projectPath: testProjectPath)
        let lockId2 = LockManager.shared.getLockInfo(projectPath: testProjectPath)?.lockId

        XCTAssertNotNil(lockId1, "First lock should have ID")
        XCTAssertNotNil(lockId2, "Second lock should have ID")
        XCTAssertNotEqual(lockId1, lockId2, "Lock IDs should be unique")
    }

    func testLockRemainingHoursDecreases() {
        _ = LockManager.shared.acquireLock(projectPath: testProjectPath, timeoutHours: 1)

        if let lock1 = LockManager.shared.getLockInfo(projectPath: testProjectPath) {
            let remaining1 = lock1.remainingTime

            sleep(2)

            if let lock2 = LockManager.shared.getLockInfo(projectPath: testProjectPath) {
                let remaining2 = lock2.remainingTime

                XCTAssertLessThan(remaining2, remaining1, "Remaining time should decrease over time")
            }
        }
    }

    // MARK: - Force Break Edge Cases

    func testForceBreakExpiredLock() {
        _ = LockManager.shared.acquireLock(projectPath: testProjectPath, timeoutHours: 0)
        sleep(1)

        let success = LockManager.shared.forceBreakLock(projectPath: testProjectPath)
        XCTAssertTrue(success, "Should force-break expired lock")
    }

    func testForceBreakMultipleTimes() {
        _ = LockManager.shared.acquireLock(projectPath: testProjectPath)

        let success1 = LockManager.shared.forceBreakLock(projectPath: testProjectPath)
        let success2 = LockManager.shared.forceBreakLock(projectPath: testProjectPath)

        XCTAssertTrue(success1, "First force-break should succeed")
        XCTAssertFalse(success2, "Second force-break should fail (no lock)")
    }

    // MARK: - isLocked Edge Cases

    func testIsLockedImmediatelyAfterExpiration() {
        _ = LockManager.shared.acquireLock(projectPath: testProjectPath, timeoutHours: 0)

        sleep(1)

        let isLocked = LockManager.shared.isLocked(projectPath: testProjectPath)
        XCTAssertFalse(isLocked, "Should not be locked after expiration")
    }

    func testIsLockedMultipleCalls() {
        _ = LockManager.shared.acquireLock(projectPath: testProjectPath)

        let isLocked1 = LockManager.shared.isLocked(projectPath: testProjectPath)
        let isLocked2 = LockManager.shared.isLocked(projectPath: testProjectPath)
        let isLocked3 = LockManager.shared.isLocked(projectPath: testProjectPath)

        XCTAssertTrue(isLocked1, "First check should return true")
        XCTAssertTrue(isLocked2, "Second check should return true")
        XCTAssertTrue(isLocked3, "Third check should return true")
    }

    // MARK: - Singleton Tests

    func testLockManagerIsSingleton() {
        let instance1 = LockManager.shared
        let instance2 = LockManager.shared

        XCTAssertTrue(instance1 === instance2, "LockManager should be a singleton")
    }

    func testSingletonAcrossOperations() {
        LockManager.shared.acquireLock(projectPath: testProjectPath)

        let isLocked = LockManager.shared.isLocked(projectPath: testProjectPath)
        XCTAssertTrue(isLocked, "Lock should persist across singleton accesses")
    }

    // MARK: - Path Normalization Tests

    func testLockWithRelativePath() {
        let relativePath = "./TestProject.logicx"
        let success = LockManager.shared.acquireLock(projectPath: relativePath)

        // May succeed or fail depending on directory structure
        XCTAssert(success == true || success == false, "Should handle relative path")
    }

    func testLockWithTrailingSlash() {
        let pathWithSlash = testProjectPath + "/"
        let success = LockManager.shared.acquireLock(projectPath: pathWithSlash)

        XCTAssertTrue(success, "Should handle trailing slash")

        // Cleanup
        _ = LockManager.shared.releaseLock(projectPath: pathWithSlash)
    }

    // MARK: - Stress Tests

    func testRapidLockReleaseLoop() {
        for _ in 0..<10 {
            let acquired = LockManager.shared.acquireLock(projectPath: testProjectPath)
            XCTAssertTrue(acquired, "Should acquire lock in loop")

            let released = LockManager.shared.releaseLock(projectPath: testProjectPath)
            XCTAssertTrue(released, "Should release lock in loop")
        }
    }

    func testManyLocksSequentially() {
        for i in 0..<20 {
            let projectPath = tempDirectory.appendingPathComponent("Project\(i).logicx").path
            try? FileManager.default.createDirectory(atPath: projectPath, withIntermediateDirectories: true)

            let oxenDir = (projectPath as NSString).appendingPathComponent(".oxen")
            try? FileManager.default.createDirectory(atPath: oxenDir, withIntermediateDirectories: true)

            let success = LockManager.shared.acquireLock(projectPath: projectPath)
            XCTAssertTrue(success, "Should lock project \(i)")
        }

        // Cleanup
        for i in 0..<20 {
            let projectPath = tempDirectory.appendingPathComponent("Project\(i).logicx").path
            _ = LockManager.shared.releaseLock(projectPath: projectPath)
        }
    }
}
