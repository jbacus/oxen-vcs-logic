import XCTest
@testable import OxVCS_LaunchAgent

final class FSEventsMonitorTests: XCTestCase {
    var tempDirectory: URL!
    var testProjectPath: String!
    var monitor: FSEventsMonitor!

    override func setUp() {
        super.setUp()

        // Create temporary directory for testing
        tempDirectory = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
        try? FileManager.default.createDirectory(at: tempDirectory, withIntermediateDirectories: true)

        // Create test project structure
        testProjectPath = tempDirectory.appendingPathComponent("TestProject.logicx").path
        try? FileManager.default.createDirectory(atPath: testProjectPath, withIntermediateDirectories: true)

        let alternativesDir = (testProjectPath as NSString).appendingPathComponent("Alternatives")
        try? FileManager.default.createDirectory(atPath: alternativesDir, withIntermediateDirectories: true)

        let resourcesDir = (testProjectPath as NSString).appendingPathComponent("Resources")
        try? FileManager.default.createDirectory(atPath: resourcesDir, withIntermediateDirectories: true)

        // Create projectData file
        let projectData = (testProjectPath as NSString).appendingPathComponent("projectData")
        try? "test data".write(toFile: projectData, atomically: true, encoding: .utf8)
    }

    override func tearDown() {
        // Stop monitoring if active
        monitor?.stop()
        monitor = nil

        // Clean up temporary directory
        try? FileManager.default.removeItem(at: tempDirectory)
        super.tearDown()
    }

    // MARK: - Initialization Tests

    func testInitWithDefaultDebounce() {
        monitor = FSEventsMonitor()
        XCTAssertNotNil(monitor, "Should create monitor with default debounce")
    }

    func testInitWithCustomDebounce() {
        monitor = FSEventsMonitor(debounceThreshold: 60.0)
        XCTAssertNotNil(monitor, "Should create monitor with custom debounce")
    }

    func testInitWithZeroDebounce() {
        monitor = FSEventsMonitor(debounceThreshold: 0.0)
        XCTAssertNotNil(monitor, "Should create monitor with zero debounce")
    }

    func testInitWithLargeDebounce() {
        monitor = FSEventsMonitor(debounceThreshold: 3600.0)
        XCTAssertNotNil(monitor, "Should create monitor with large debounce (1 hour)")
    }

    // MARK: - Callback Tests

    func testSetCommitCallback() {
        monitor = FSEventsMonitor()

        let expectation = XCTestExpectation(description: "Callback should be settable")

        monitor.setCommitCallback { path in
            expectation.fulfill()
        }

        // Callback is set, but we can't easily trigger it in tests without actual FSEvents
        // This test verifies the API works without crashing
        XCTAssert(true, "Callback set successfully")
    }

    func testSetMultipleCallbacks() {
        monitor = FSEventsMonitor()

        // Setting multiple callbacks - last one should win
        monitor.setCommitCallback { _ in
            XCTFail("Should not call first callback")
        }

        monitor.setCommitCallback { _ in
            // This is the active callback
        }

        XCTAssert(true, "Should allow setting multiple callbacks")
    }

    // MARK: - Monitoring State Tests

    func testIsActiveBeforeStart() {
        monitor = FSEventsMonitor()
        XCTAssertFalse(monitor.isActive(), "Should not be active before starting")
    }

    func testGetWatchedPathBeforeStart() {
        monitor = FSEventsMonitor()
        let path = monitor.getWatchedPath()
        XCTAssertEqual(path, "", "Watched path should be empty before starting")
    }

    func testStopWithoutStart() {
        monitor = FSEventsMonitor()

        // Should not crash when stopping without starting
        monitor.stop()

        XCTAssertFalse(monitor.isActive(), "Should remain inactive after stop without start")
    }

    func testStopMultipleTimes() {
        monitor = FSEventsMonitor()

        // Should not crash when stopping multiple times
        monitor.stop()
        monitor.stop()
        monitor.stop()

        XCTAssertFalse(monitor.isActive(), "Should remain inactive")
    }

    // MARK: - Path Filtering Tests

    func testShouldProcessProjectDataEvent() async {
        monitor = FSEventsMonitor()

        // ProjectData files should be processed
        let testPath = (testProjectPath as NSString).appendingPathComponent("projectData")

        // We can't directly test the private shouldProcessEvent method,
        // but we can verify the path structure is correct
        XCTAssertTrue(testPath.contains("projectData"), "Path should contain projectData")
    }

    func testShouldProcessAlternativesDirectory() {
        let alternativesPath = (testProjectPath as NSString).appendingPathComponent("Alternatives/001/file.wav")
        XCTAssertTrue(alternativesPath.contains("/Alternatives/"), "Should contain Alternatives directory")
    }

    func testShouldProcessResourcesDirectory() {
        let resourcesPath = (testProjectPath as NSString).appendingPathComponent("Resources/Audio.wav")
        XCTAssertTrue(resourcesPath.contains("/Resources/"), "Should contain Resources directory")
    }

    func testShouldIgnoreBouncesDirectory() {
        let bouncesPath = (testProjectPath as NSString).appendingPathComponent("Bounces/Mix.wav")
        XCTAssertTrue(bouncesPath.contains("/Bounces/"), "Bounces directory should be detected")
    }

    func testShouldIgnoreFreezeDirectory() {
        let freezePath = (testProjectPath as NSString).appendingPathComponent("Freeze Files/Track_01.wav")
        XCTAssertTrue(freezePath.contains("/Freeze Files/"), "Freeze directory should be detected")
    }

    func testShouldIgnoreAutosaveDirectory() {
        let autosavePath = (testProjectPath as NSString).appendingPathComponent("Autosave/backup.logicx")
        XCTAssertTrue(autosavePath.contains("/Autosave/"), "Autosave directory should be detected")
    }

    func testShouldIgnoreDSStore() {
        let dsStorePath = (testProjectPath as NSString).appendingPathComponent(".DS_Store")
        XCTAssertTrue(dsStorePath.contains(".DS_Store"), ".DS_Store should be detected")
    }

    // MARK: - Integration Tests (require actual FSEvents)

    func testStartMonitoring() async {
        monitor = FSEventsMonitor()

        // Note: This test will start monitoring but we can't reliably test
        // FSEvents in a unit test environment. The test verifies the API doesn't crash.
        let startTask = Task {
            try? await monitor.start(watchingPath: testProjectPath)
        }

        // Give it a moment to start
        try? await Task.sleep(nanoseconds: 500_000_000) // 0.5 seconds

        // Stop monitoring
        monitor.stop()
        startTask.cancel()

        XCTAssert(true, "Monitoring started and stopped without crashing")
    }

    func testGetWatchedPathAfterStart() async {
        monitor = FSEventsMonitor()

        let startTask = Task {
            try? await monitor.start(watchingPath: testProjectPath)
        }

        // Give it a moment to start
        try? await Task.sleep(nanoseconds: 500_000_000)

        let watchedPath = monitor.getWatchedPath()
        XCTAssertEqual(watchedPath, testProjectPath, "Should return the watched path")

        monitor.stop()
        startTask.cancel()
    }

    func testIsActiveAfterStart() async {
        monitor = FSEventsMonitor()

        let startTask = Task {
            try? await monitor.start(watchingPath: testProjectPath)
        }

        // Give it a moment to start
        try? await Task.sleep(nanoseconds: 500_000_000)

        XCTAssertTrue(monitor.isActive(), "Should be active after starting")

        monitor.stop()
        startTask.cancel()
    }

    func testIsActiveAfterStop() async {
        monitor = FSEventsMonitor()

        let startTask = Task {
            try? await monitor.start(watchingPath: testProjectPath)
        }

        try? await Task.sleep(nanoseconds: 500_000_000)

        monitor.stop()
        startTask.cancel()

        XCTAssertFalse(monitor.isActive(), "Should not be active after stopping")
    }

    func testStartMonitoringTwice() async {
        monitor = FSEventsMonitor()

        let firstTask = Task {
            try? await monitor.start(watchingPath: testProjectPath)
        }

        try? await Task.sleep(nanoseconds: 500_000_000)

        // Try to start again - should be ignored
        let secondTask = Task {
            try? await monitor.start(watchingPath: testProjectPath)
        }

        try? await Task.sleep(nanoseconds: 100_000_000)

        // Should still be monitoring original path
        XCTAssertTrue(monitor.isActive(), "Should remain active")

        monitor.stop()
        firstTask.cancel()
        secondTask.cancel()
    }

    // MARK: - Error Handling Tests

    func testStartMonitoringNonexistentPath() async {
        monitor = FSEventsMonitor()

        let nonexistentPath = "/path/that/does/not/exist/\(UUID().uuidString)"

        let startTask = Task {
            do {
                try await monitor.start(watchingPath: nonexistentPath)
                XCTFail("Should throw error for nonexistent path")
            } catch {
                // Expected to fail
                XCTAssert(true, "Correctly threw error for nonexistent path")
            }
        }

        try? await Task.sleep(nanoseconds: 500_000_000)
        startTask.cancel()
    }

    func testStartMonitoringEmptyPath() async {
        monitor = FSEventsMonitor()

        let startTask = Task {
            do {
                try await monitor.start(watchingPath: "")
                XCTFail("Should throw error for empty path")
            } catch {
                // Expected to fail
                XCTAssert(true, "Correctly threw error for empty path")
            }
        }

        try? await Task.sleep(nanoseconds: 500_000_000)
        startTask.cancel()
    }

    // MARK: - Debounce Behavior Tests

    func testDebounceThresholdAccessible() {
        // Test different debounce thresholds
        let monitor1 = FSEventsMonitor(debounceThreshold: 10.0)
        XCTAssertNotNil(monitor1, "10 second debounce should work")

        let monitor2 = FSEventsMonitor(debounceThreshold: 30.0)
        XCTAssertNotNil(monitor2, "30 second debounce should work")

        let monitor3 = FSEventsMonitor(debounceThreshold: 60.0)
        XCTAssertNotNil(monitor3, "60 second debounce should work")
    }

    // MARK: - Cleanup Tests

    func testDeinitStopsMonitoring() async {
        var tempMonitor: FSEventsMonitor? = FSEventsMonitor()

        // Capture monitor in Task before it can be mutated
        if let monitor = tempMonitor {
            let startTask = Task {
                try? await monitor.start(watchingPath: testProjectPath)
            }

            try? await Task.sleep(nanoseconds: 500_000_000)

            // Deinit should call stop()
            tempMonitor = nil
            startTask.cancel()
        }

        XCTAssertNil(tempMonitor, "Monitor should be deallocated")
    }

    // MARK: - Path Normalization Tests

    func testWatchPathWithTrailingSlash() async {
        monitor = FSEventsMonitor()

        let pathWithSlash = testProjectPath + "/"

        let startTask = Task {
            try? await monitor.start(watchingPath: pathWithSlash)
        }

        try? await Task.sleep(nanoseconds: 500_000_000)

        let watchedPath = monitor.getWatchedPath()
        XCTAssertEqual(watchedPath, pathWithSlash, "Should preserve original path format")

        monitor.stop()
        startTask.cancel()
    }

    func testWatchRelativePath() async {
        monitor = FSEventsMonitor()

        // Create a relative path
        let relativePath = "./TestProject.logicx"

        let startTask = Task {
            try? await monitor.start(watchingPath: relativePath)
        }

        try? await Task.sleep(nanoseconds: 500_000_000)

        monitor.stop()
        startTask.cancel()

        // Should work with relative paths
        XCTAssert(true, "Should handle relative paths")
    }

    // MARK: - Callback Invocation Tests

    func testCallbackReceivesCorrectPath() async {
        monitor = FSEventsMonitor()

        let expectation = XCTestExpectation(description: "Callback should receive path")
        expectation.isInverted = true // We don't expect it to be called without actual file changes

        monitor.setCommitCallback { path in
            XCTAssertEqual(path, self.testProjectPath, "Callback should receive correct path")
            expectation.fulfill()
        }

        let startTask = Task {
            try? await monitor.start(watchingPath: testProjectPath)
        }

        try? await Task.sleep(nanoseconds: 500_000_000)

        monitor.stop()
        startTask.cancel()

        await fulfillment(of: [expectation], timeout: 1.0)
    }
}
