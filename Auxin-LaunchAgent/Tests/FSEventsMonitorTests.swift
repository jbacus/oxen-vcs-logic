import XCTest
@testable import Auxin_LaunchAgent

/// Tests for FSEventsMonitor
@available(macOS 10.15, *)
final class FSEventsMonitorTests: XCTestCase {

    var tempDir: URL!
    var monitor: FSEventsMonitor!

    override func setUpWithError() throws {
        try super.setUpWithError()
        tempDir = FileManager.default.temporaryDirectory
            .appendingPathComponent(UUID().uuidString)
        try FileManager.default.createDirectory(at: tempDir, withIntermediateDirectories: true)
        monitor = FSEventsMonitor(debounceThreshold: 0.5, projectType: .logicPro)
    }

    override func tearDownWithError() throws {
        monitor?.stop()
        monitor = nil
        if let tempDir = tempDir {
            try? FileManager.default.removeItem(at: tempDir)
        }
        try super.tearDownWithError()
    }

    // MARK: - Lifecycle Tests

    func testMonitorCanStartWithValidPath() async throws {
        try await monitor.start(watchingPath: tempDir.path)
        XCTAssertTrue(monitor.isActive())
    }

    func testMonitorThrowsErrorForEmptyPath() async {
        do {
            try await monitor.start(watchingPath: "")
            XCTFail("Should throw error for empty path")
        } catch {
            XCTAssertNotNil(error)
        }
    }

    func testMonitorCanStopAfterStarting() async throws {
        try await monitor.start(watchingPath: tempDir.path)
        monitor.stop()
        XCTAssertFalse(monitor.isActive())
    }

    // MARK: - Debounce Tests

    func testDebounceCallbackInvokedAfterThreshold() async throws {
        let expectation = XCTestExpectation(description: "Commit callback")
        var callbackInvoked = false

        monitor.setCommitCallback { path in
            callbackInvoked = true
            expectation.fulfill()
        }

        try await monitor.start(watchingPath: tempDir.path)
        let testFile = tempDir.appendingPathComponent("test.txt")
        try "Hello".write(to: testFile, atomically: true, encoding: .utf8)

        await fulfillment(of: [expectation], timeout: 2.0)
        XCTAssertTrue(callbackInvoked)
    }
}
