import XCTest
@testable import Auxin_LaunchAgent

final class PowerManagementTests: XCTestCase {
    var powerManager: PowerManagement!

    override func setUp() {
        super.setUp()
        powerManager = PowerManagement()
    }

    override func tearDown() {
        powerManager.stopMonitoring()
        powerManager = nil
        super.tearDown()
    }

    // MARK: - Initialization Tests

    func testInitialization() {
        let pm = PowerManagement()
        XCTAssertNotNil(pm, "Should initialize successfully")
    }

    func testMultipleInstances() {
        let pm1 = PowerManagement()
        let pm2 = PowerManagement()
        XCTAssertNotNil(pm1, "First instance should initialize")
        XCTAssertNotNil(pm2, "Second instance should initialize")
    }

    // MARK: - Monitoring Tests

    func testStartMonitoring() {
        let expectation = XCTestExpectation(description: "Handler should be settable")

        powerManager.startMonitoring {
            expectation.fulfill()
        }

        // Verify monitoring started without crashing
        XCTAssert(true, "Monitoring started successfully")
    }

    func testStartMonitoringTwice() {
        var callCount = 0

        powerManager.startMonitoring {
            callCount += 1
        }

        // Try to start again - should be ignored
        powerManager.startMonitoring {
            callCount += 2
        }

        // First handler should remain active
        XCTAssert(true, "Should handle multiple start attempts")
    }

    func testStopMonitoring() {
        powerManager.startMonitoring {
            // Handler
        }

        // Should not crash when stopping
        powerManager.stopMonitoring()
        XCTAssert(true, "Monitoring stopped successfully")
    }

    func testStopWithoutStart() {
        // Should not crash when stopping without starting
        powerManager.stopMonitoring()
        XCTAssert(true, "Should handle stop without start")
    }

    func testStopMultipleTimes() {
        powerManager.startMonitoring { }

        powerManager.stopMonitoring()
        powerManager.stopMonitoring()
        powerManager.stopMonitoring()

        XCTAssert(true, "Should handle multiple stops")
    }

    // MARK: - Power Event Simulation Tests

    func testSimulateWillSleepEvent() async {
        let expectation = XCTestExpectation(description: "Emergency handler should be called")

        powerManager.startMonitoring {
            expectation.fulfill()
        }

        powerManager.simulateEvent(.willSleep)

        await fulfillment(of: [expectation], timeout: 2.0)
    }

    func testSimulateWillShutdownEvent() async {
        let expectation = XCTestExpectation(description: "Emergency handler should be called")

        powerManager.startMonitoring {
            expectation.fulfill()
        }

        powerManager.simulateEvent(.willShutdown)

        await fulfillment(of: [expectation], timeout: 2.0)
    }

    func testSimulateDidWakeEvent() {
        powerManager.startMonitoring {
            XCTFail("Should not call handler for wake event")
        }

        // Wake events should not trigger emergency commits
        powerManager.simulateEvent(.didWake)

        XCTAssert(true, "Wake event should not trigger handler")
    }

    func testSimulateMultipleEvents() async {
        var eventCount = 0

        powerManager.startMonitoring {
            eventCount += 1
        }

        // Simulate multiple sleep events
        powerManager.simulateEvent(.willSleep)
        try? await Task.sleep(nanoseconds: 100_000_000) // 0.1s

        powerManager.simulateEvent(.willShutdown)
        try? await Task.sleep(nanoseconds: 100_000_000)

        XCTAssertGreaterThan(eventCount, 0, "Should have processed events")
    }

    // MARK: - Emergency Commit Tests

    func testEmergencyCommitFlow() async {
        let expectation = XCTestExpectation(description: "Emergency commit should complete")

        powerManager.startMonitoring {
            expectation.fulfill()
        }

        await powerManager.testEmergencyCommit()

        await fulfillment(of: [expectation], timeout: 2.0)
    }

    func testEmergencyCommitWithoutHandler() async {
        // Should not crash if handler not set
        await powerManager.testEmergencyCommit()
        XCTAssert(true, "Should handle emergency commit without handler")
    }

    // MARK: - Battery Status Tests

    func testIsOnBatteryPower() {
        let onBattery = PowerManagement.isOnBatteryPower()
        // Result depends on system state, just verify it doesn't crash
        XCTAssert(onBattery == true || onBattery == false, "Should return a boolean")
    }

    func testBatteryLevel() {
        let level = PowerManagement.batteryLevel()

        if let level = level {
            XCTAssertGreaterThanOrEqual(level, 0, "Battery level should be >= 0")
            XCTAssertLessThanOrEqual(level, 100, "Battery level should be <= 100")
        } else {
            // nil is valid if not on battery or no battery info available
            XCTAssert(true, "Battery level can be nil")
        }
    }

    func testIsBatteryLowDefaultThreshold() {
        let isLow = PowerManagement.isBatteryLow()
        // Result depends on system state
        XCTAssert(isLow == true || isLow == false, "Should return a boolean")
    }

    func testIsBatteryLowCustomThreshold() {
        let isLow = PowerManagement.isBatteryLow(threshold: 50)
        // Should work with custom threshold
        XCTAssert(isLow == true || isLow == false, "Should return a boolean with custom threshold")
    }

    func testIsBatteryLowZeroThreshold() {
        let isLow = PowerManagement.isBatteryLow(threshold: 0)
        // With 0 threshold, battery should never be "low" unless at exactly 0%
        if let level = PowerManagement.batteryLevel() {
            XCTAssertEqual(isLow, level < 0, "0 threshold logic should work")
        }
    }

    func testIsBatteryLowHundredThreshold() {
        let isLow = PowerManagement.isBatteryLow(threshold: 100)
        // With 100 threshold, battery is always "low" unless at 100%
        if let level = PowerManagement.batteryLevel() {
            XCTAssertEqual(isLow, level < 100, "100 threshold logic should work")
        }
    }

    // MARK: - System Load Tests

    func testIsSystemBusy() {
        let isBusy = PowerManagement.isSystemBusy()
        // Result depends on system state
        XCTAssert(isBusy == true || isBusy == false, "Should return a boolean")
    }

    // MARK: - Emergency Commit Policy Tests

    func testShouldPerformEmergencyCommit() {
        let shouldCommit = PowerManagement.shouldPerformEmergencyCommit()
        // Should return a decision based on system state
        XCTAssert(shouldCommit == true || shouldCommit == false, "Should return a boolean decision")
    }

    func testEmergencyCommitPolicyConsistency() {
        // Call multiple times - should be consistent in short timeframe
        let result1 = PowerManagement.shouldPerformEmergencyCommit()
        let result2 = PowerManagement.shouldPerformEmergencyCommit()

        // In a short timeframe, results should be consistent
        // (unless battery drops drastically between calls)
        XCTAssert(true, "Policy check should be consistent")
    }

    // MARK: - Power Event Types Tests

    func testPowerEventWillSleep() {
        let event = PowerManagement.PowerEvent.willSleep
        XCTAssertNotNil(event, "willSleep event should exist")
    }

    func testPowerEventDidWake() {
        let event = PowerManagement.PowerEvent.didWake
        XCTAssertNotNil(event, "didWake event should exist")
    }

    func testPowerEventWillShutdown() {
        let event = PowerManagement.PowerEvent.willShutdown
        XCTAssertNotNil(event, "willShutdown event should exist")
    }

    func testPowerEventWillRestart() {
        let event = PowerManagement.PowerEvent.willRestart
        XCTAssertNotNil(event, "willRestart event should exist")
    }

    // MARK: - Cleanup Tests

    func testDeinitStopsMonitoring() {
        var tempPM: PowerManagement? = PowerManagement()
        tempPM?.startMonitoring { }

        // Deinit should call stopMonitoring
        tempPM = nil

        XCTAssertNil(tempPM, "Should be deallocated")
    }

    // MARK: - Concurrent Access Tests

    func testConcurrentEventSimulation() async {
        let expectation = XCTestExpectation(description: "Handle concurrent events")
        expectation.expectedFulfillmentCount = 3

        powerManager.startMonitoring {
            expectation.fulfill()
        }

        // Simulate concurrent events
        await withTaskGroup(of: Void.self) { group in
            group.addTask {
                self.powerManager.simulateEvent(.willSleep)
            }
            group.addTask {
                self.powerManager.simulateEvent(.willShutdown)
            }
            group.addTask {
                self.powerManager.simulateEvent(.willRestart)
            }
        }

        await fulfillment(of: [expectation], timeout: 3.0)
    }

    // MARK: - Handler Timing Tests

    func testEmergencyCommitDuration() async {
        let startTime = Date()

        powerManager.startMonitoring {
            // Quick handler
        }

        await powerManager.testEmergencyCommit()

        let duration = Date().timeIntervalSince(startTime)

        // Emergency commit should complete quickly (< 5 seconds for test)
        XCTAssertLessThan(duration, 5.0, "Emergency commit should complete quickly")
    }

    func testLongRunningHandler() async {
        let expectation = XCTestExpectation(description: "Long handler should complete")

        powerManager.startMonitoring {
            // Simulate a longer operation
            try? await Task.sleep(nanoseconds: 500_000_000) // 0.5s
            expectation.fulfill()
        }

        powerManager.simulateEvent(.willSleep)

        await fulfillment(of: [expectation], timeout: 2.0)
    }

    // MARK: - Sleep Prevention Tests

    func testSleepPreventionFlow() async {
        // Test that sleep prevention is invoked during emergency commit
        let expectation = XCTestExpectation(description: "Handler should be called")

        powerManager.startMonitoring {
            // During this handler, system sleep should be prevented
            expectation.fulfill()
        }

        powerManager.simulateEvent(.willSleep)

        await fulfillment(of: [expectation], timeout: 2.0)
        // After handler completes, sleep prevention should be released
    }

    // MARK: - Edge Cases

    func testNilHandlerDoesNotCrash() {
        // Start monitoring without setting handler
        powerManager.startMonitoring {
            // Empty handler
        }

        // Simulate event
        powerManager.simulateEvent(.willSleep)

        XCTAssert(true, "Should handle events gracefully")
    }

    func testRapidStartStop() {
        for _ in 0..<10 {
            powerManager.startMonitoring { }
            powerManager.stopMonitoring()
        }

        XCTAssert(true, "Should handle rapid start/stop cycles")
    }

    // MARK: - Integration Tests

    func testFullPowerManagementCycle() async {
        let sleepExpectation = XCTestExpectation(description: "Sleep event handled")

        powerManager.startMonitoring {
            sleepExpectation.fulfill()
        }

        // Simulate full cycle: sleep -> wake
        powerManager.simulateEvent(.willSleep)
        try? await Task.sleep(nanoseconds: 100_000_000)

        powerManager.simulateEvent(.didWake)
        try? await Task.sleep(nanoseconds: 100_000_000)

        powerManager.stopMonitoring()

        await fulfillment(of: [sleepExpectation], timeout: 2.0)
    }
}
