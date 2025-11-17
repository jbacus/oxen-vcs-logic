import Foundation
import AppKit
import IOKit
import IOKit.pwr_mgt
import IOKit.ps

/// Handles system power events and triggers emergency commits before sleep/shutdown
/// Ensures data safety by committing uncommitted changes when system power events occur
public class PowerManagement {

    // MARK: - Properties

    private var sleepObserver: NSObjectProtocol?
    private var wakeObserver: NSObjectProtocol?
    private var shutdownObserver: NSObjectProtocol?
    private var emergencyCommitHandler: (() async -> Void)?
    private var isMonitoring = false

    // IOKit power assertion to prevent sleep during emergency commits
    private var powerAssertion: IOPMAssertionID = IOPMAssertionID(0)

    public enum PowerEvent {
        case willSleep
        case didWake
        case willShutdown
        case willRestart
    }

    // MARK: - Initialization

    public init() {
        // Empty initializer
    }

    deinit {
        stopMonitoring()
    }

    // MARK: - Public Interface

    /// Start monitoring system power events
    /// - Parameter emergencyHandler: Async closure to call when emergency commit is needed
    public func startMonitoring(emergencyCommitHandler: @escaping () async -> Void) {
        guard !isMonitoring else {
            print("Power management already monitoring")
            return
        }

        self.emergencyCommitHandler = emergencyCommitHandler
        self.isMonitoring = true

        registerForPowerNotifications()

        print("✓ Power management monitoring started")
        print("  - Will commit before sleep")
        print("  - Will commit before shutdown")
        print("  - Will handle unexpected power loss")
    }

    /// Stop monitoring power events
    public func stopMonitoring() {
        guard isMonitoring else { return }

        unregisterFromPowerNotifications()

        emergencyCommitHandler = nil
        isMonitoring = false

        print("Power management monitoring stopped")
    }

    // MARK: - Private Implementation

    private func registerForPowerNotifications() {
        let workspace = NSWorkspace.shared.notificationCenter

        // System will sleep notification
        sleepObserver = workspace.addObserver(
            forName: NSWorkspace.willSleepNotification,
            object: nil,
            queue: .main
        ) { [weak self] notification in
            print("\n⚠️  System will sleep - triggering emergency commit")
            self?.handlePowerEvent(.willSleep)
        }

        // System did wake notification
        wakeObserver = workspace.addObserver(
            forName: NSWorkspace.didWakeNotification,
            object: nil,
            queue: .main
        ) { [weak self] notification in
            print("System woke from sleep")
            self?.handlePowerEvent(.didWake)
        }

        // System will power off notification
        shutdownObserver = workspace.addObserver(
            forName: NSWorkspace.willPowerOffNotification,
            object: nil,
            queue: .main
        ) { [weak self] notification in
            print("\n⚠️  System will shutdown - triggering emergency commit")
            self?.handlePowerEvent(.willShutdown)
        }

        print("✓ Registered for power notifications")
    }

    private func unregisterFromPowerNotifications() {
        let workspace = NSWorkspace.shared.notificationCenter

        if let observer = sleepObserver {
            workspace.removeObserver(observer)
            sleepObserver = nil
        }

        if let observer = wakeObserver {
            workspace.removeObserver(observer)
            wakeObserver = nil
        }

        if let observer = shutdownObserver {
            workspace.removeObserver(observer)
            shutdownObserver = nil
        }
    }

    private func handlePowerEvent(_ event: PowerEvent) {
        switch event {
        case .willSleep, .willShutdown, .willRestart:
            // Critical: Prevent sleep until commit completes
            preventSystemSleep()

            // Perform emergency commit synchronously
            Task {
                await performEmergencyCommit(for: event)
                allowSystemSleep()
            }

        case .didWake:
            // System has woken up - resume normal operation
            print("Resuming normal operation after wake")
        }
    }

    private func performEmergencyCommit(for event: PowerEvent) async {
        let eventName = eventDescription(for: event)
        print("Performing emergency commit for: \(eventName)")

        let startTime = Date()

        guard let handler = emergencyCommitHandler else {
            print("⚠️  No emergency commit handler registered")
            return
        }

        // Execute the emergency commit
        await handler()

        let duration = Date().timeIntervalSince(startTime)
        print("✓ Emergency commit completed in \(String(format: "%.2f", duration))s")
    }

    private func eventDescription(for event: PowerEvent) -> String {
        switch event {
        case .willSleep: return "System Sleep"
        case .willShutdown: return "System Shutdown"
        case .willRestart: return "System Restart"
        case .didWake: return "System Wake"
        }
    }

    // MARK: - Sleep Prevention

    /// Prevent system sleep while emergency commit is in progress
    private func preventSystemSleep() {
        let reason = "Oxen VCS emergency commit in progress" as CFString
        let assertionType = kIOPMAssertionTypePreventSystemSleep as CFString

        let result = IOPMAssertionCreateWithName(
            assertionType,
            IOPMAssertionLevel(kIOPMAssertionLevelOn),
            reason,
            &powerAssertion
        )

        if result == kIOReturnSuccess {
            print("✓ Preventing system sleep during commit")
        } else {
            print("⚠️  Failed to prevent system sleep (error: \(result))")
        }
    }

    /// Allow system sleep after emergency commit completes
    private func allowSystemSleep() {
        guard powerAssertion != 0 else { return }

        let result = IOPMAssertionRelease(powerAssertion)

        if result == kIOReturnSuccess {
            print("✓ System sleep prevention released")
            powerAssertion = 0
        } else {
            print("⚠️  Failed to release sleep prevention (error: \(result))")
        }
    }

    // MARK: - Battery Status Monitoring

    /// Check if system is running on battery power
    /// - Returns: true if on battery, false if on AC power
    public static func isOnBatteryPower() -> Bool {
        let snapshot = IOPSCopyPowerSourcesInfo().takeRetainedValue()
        let sources = IOPSCopyPowerSourcesList(snapshot).takeRetainedValue() as Array

        for source in sources {
            if let description = IOPSGetPowerSourceDescription(snapshot, source).takeUnretainedValue() as? [String: Any] {
                if let powerSourceState = description[kIOPSPowerSourceStateKey] as? String {
                    return powerSourceState == kIOPSBatteryPowerValue
                }
            }
        }

        return false
    }

    /// Get current battery level percentage
    /// - Returns: Battery level 0-100, or nil if not available
    public static func batteryLevel() -> Int? {
        let snapshot = IOPSCopyPowerSourcesInfo().takeRetainedValue()
        let sources = IOPSCopyPowerSourcesList(snapshot).takeRetainedValue() as Array

        for source in sources {
            if let description = IOPSGetPowerSourceDescription(snapshot, source).takeUnretainedValue() as? [String: Any] {
                if let currentCapacity = description[kIOPSCurrentCapacityKey] as? Int,
                   let maxCapacity = description[kIOPSMaxCapacityKey] as? Int,
                   maxCapacity > 0 {
                    return (currentCapacity * 100) / maxCapacity
                }
            }
        }

        return nil
    }

    /// Check if battery level is critically low
    /// - Parameter threshold: Battery percentage threshold (default: 10%)
    /// - Returns: true if battery is below threshold
    public static func isBatteryLow(threshold: Int = 10) -> Bool {
        guard let level = batteryLevel() else {
            return false
        }
        return level < threshold
    }

    // MARK: - System Load Monitoring

    /// Check if system is under high load (to avoid commits during intensive tasks)
    /// - Returns: true if system load is high
    public static func isSystemBusy() -> Bool {
        var loadavg = [Double](repeating: 0, count: 3)
        let result = getloadavg(&loadavg, 3)

        guard result > 0 else {
            return false
        }

        // Consider system busy if 1-minute load average is above number of CPUs
        let cpuCount = ProcessInfo.processInfo.activeProcessorCount
        return loadavg[0] > Double(cpuCount)
    }

    // MARK: - Emergency Commit Policy

    /// Determine if an emergency commit should be performed based on system state
    /// - Returns: true if emergency commit should proceed
    public static func shouldPerformEmergencyCommit() -> Bool {
        // Always commit on shutdown
        // For sleep, check battery and load

        if isOnBatteryPower() {
            if let batteryLevel = batteryLevel() {
                print("Battery level: \(batteryLevel)%")

                // If battery is very low, skip commit to preserve power
                if batteryLevel < 5 {
                    print("⚠️  Battery critically low - skipping commit")
                    return false
                }
            }
        }

        if isSystemBusy() {
            print("System is busy - commit may be slower")
            // Still proceed, but user is warned
        }

        return true
    }
}

// MARK: - Testing Utilities

extension PowerManagement {

    /// Simulate a power event for testing purposes
    /// - Parameter event: The power event to simulate
    public func simulateEvent(_ event: PowerEvent) {
        print("\n[TEST] Simulating power event: \(eventDescription(for: event))")
        handlePowerEvent(event)
    }

    /// Test the emergency commit flow without actual power events
    public func testEmergencyCommit() async {
        print("\n[TEST] Testing emergency commit flow")
        await performEmergencyCommit(for: .willSleep)
    }
}
