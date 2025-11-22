import XCTest
@testable import Auxin_LaunchAgent

@available(macOS 10.14, *)
final class NetworkMonitorTests: XCTestCase {

    var monitor: NetworkMonitor!

    override func setUp() {
        super.setUp()
        monitor = NetworkMonitor(cliPath: "/usr/local/bin/auxin")
    }

    override func tearDown() {
        monitor.stopMonitoring()
        monitor = nil
        super.tearDown()
    }

    // MARK: - Initialization Tests

    func testMonitorInitialization() {
        XCTAssertNotNil(monitor)
        XCTAssertEqual(monitor.statusDescription, "Not monitoring")
    }

    // MARK: - Monitoring Lifecycle Tests

    func testStartMonitoring() {
        monitor.startMonitoring()

        // Give time for the monitor to initialize
        let expectation = XCTestExpectation(description: "Monitor started")
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) {
            expectation.fulfill()
        }
        wait(for: [expectation], timeout: 1.0)

        // After starting, should have a status (not "Not monitoring")
        XCTAssertNotEqual(monitor.statusDescription, "Not monitoring")
    }

    func testStopMonitoring() {
        monitor.startMonitoring()

        let expectation = XCTestExpectation(description: "Monitor started")
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) {
            self.monitor.stopMonitoring()
            expectation.fulfill()
        }
        wait(for: [expectation], timeout: 1.0)

        // After stopping, status should be "Not monitoring"
        XCTAssertEqual(monitor.statusDescription, "Not monitoring")
    }

    func testDoubleStartPrevented() {
        monitor.startMonitoring()
        monitor.startMonitoring() // Should print warning but not crash

        let expectation = XCTestExpectation(description: "Double start handled")
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) {
            expectation.fulfill()
        }
        wait(for: [expectation], timeout: 1.0)
    }

    // MARK: - Status Query Tests

    func testGetStatistics() {
        let stats = monitor.getStatistics()

        XCTAssertNotNil(stats["isMonitoring"])
        XCTAssertNotNil(stats["isAvailable"])
        XCTAssertNotNil(stats["isExpensive"])
        XCTAssertNotNil(stats["isConstrained"])
        XCTAssertNotNil(stats["status"])
        XCTAssertNotNil(stats["wasDisconnected"])

        // Before starting, should not be monitoring
        XCTAssertFalse(stats["isMonitoring"] as! Bool)
    }

    func testNetworkAvailabilityQuery() {
        // Before starting monitoring
        XCTAssertFalse(monitor.isNetworkAvailable)
    }

    func testExpensiveConnectionQuery() {
        // Before starting monitoring
        XCTAssertFalse(monitor.isExpensiveConnection)
    }

    func testConstrainedConnectionQuery() {
        // Before starting monitoring
        XCTAssertFalse(monitor.isConstrainedConnection)
    }

    // MARK: - Custom Handler Tests

    func testCustomReconnectHandler() {
        var handlerCalled = false
        let expectation = XCTestExpectation(description: "Handler setup")

        monitor.startMonitoring {
            handlerCalled = true
        }

        DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) {
            expectation.fulfill()
        }
        wait(for: [expectation], timeout: 1.0)

        // Handler should not be called just from starting
        // (only called on reconnection)
        // We just verify it was accepted
        XCTAssertNotNil(monitor)
    }

    // MARK: - Simulated Events Tests

    func testSimulateReconnect() async {
        // Test that simulate reconnect doesn't crash
        await monitor.simulateReconnect()
    }

    func testTestQueueSync() async {
        // Test that test queue sync doesn't crash
        await monitor.testQueueSync()
    }

    func testManualSync() async {
        // Test that manual sync doesn't crash
        await monitor.manualSync()
    }

    // MARK: - Integration Tests

    func testMonitorWithDaemonIntegration() {
        // Verify monitor can be created with daemon's CLI path
        let daemonMonitor = NetworkMonitor(cliPath: "/usr/local/bin/auxin")
        XCTAssertNotNil(daemonMonitor)

        daemonMonitor.startMonitoring()

        let expectation = XCTestExpectation(description: "Monitor started")
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.2) {
            let stats = daemonMonitor.getStatistics()
            XCTAssertTrue(stats["isMonitoring"] as! Bool)
            daemonMonitor.stopMonitoring()
            expectation.fulfill()
        }
        wait(for: [expectation], timeout: 1.0)
    }

    // MARK: - Status Description Tests

    func testStatusDescriptionNotMonitoring() {
        XCTAssertEqual(monitor.statusDescription, "Not monitoring")
    }

    func testStatusDescriptionAfterStart() {
        monitor.startMonitoring()

        let expectation = XCTestExpectation(description: "Monitor started")
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.2) {
            let status = self.monitor.statusDescription
            // Should contain status information
            XCTAssertNotEqual(status, "Not monitoring")
            expectation.fulfill()
        }
        wait(for: [expectation], timeout: 1.0)
    }

    // MARK: - Multiple Monitor Tests

    func testMultipleMonitorsIndependent() {
        let monitor1 = NetworkMonitor(cliPath: "/usr/local/bin/auxin")
        let monitor2 = NetworkMonitor(cliPath: "/usr/local/bin/auxin")

        monitor1.startMonitoring()

        let expectation = XCTestExpectation(description: "Monitors independent")
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) {
            let stats1 = monitor1.getStatistics()
            let stats2 = monitor2.getStatistics()

            // Monitor 1 should be monitoring, monitor 2 should not
            XCTAssertTrue(stats1["isMonitoring"] as! Bool)
            XCTAssertFalse(stats2["isMonitoring"] as! Bool)

            monitor1.stopMonitoring()
            monitor2.stopMonitoring()
            expectation.fulfill()
        }
        wait(for: [expectation], timeout: 1.0)
    }
}
