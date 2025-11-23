import Foundation
import Network

/// Monitors network connectivity and triggers queue synchronization when network returns
/// Ensures offline operations are synchronized automatically after reconnection
@available(macOS 10.14, *)
public class NetworkMonitor {

    // MARK: - Properties

    private let monitor: NWPathMonitor
    private let queue: DispatchQueue
    private var reconnectHandler: (() async -> Void)?
    private var isMonitoring = false
    private var wasDisconnected = false
    private var lastPath: NWPath?

    private let cliPath: String

    // MARK: - Initialization

    public init(cliPath: String = "/usr/local/bin/auxin") {
        self.cliPath = cliPath
        self.monitor = NWPathMonitor()
        self.queue = DispatchQueue(label: "com.auxin.network-monitor", qos: .utility)
    }

    deinit {
        stopMonitoring()
    }

    // MARK: - Public Interface

    /// Start monitoring network connectivity
    /// - Parameter reconnectHandler: Async closure to call when network reconnects (optional, for custom handling)
    public func startMonitoring(reconnectHandler: (() async -> Void)? = nil) {
        guard !isMonitoring else {
            print("Network monitoring already active")
            return
        }

        self.reconnectHandler = reconnectHandler
        self.isMonitoring = true

        monitor.pathUpdateHandler = { [weak self] path in
            self?.handlePathUpdate(path)
        }

        monitor.start(queue: queue)

        print("✓ Network monitoring started")
        print("  - Will sync queue on reconnect")
        print("  - Monitoring: WiFi, Ethernet, Cellular")
    }

    /// Stop monitoring network connectivity
    public func stopMonitoring() {
        guard isMonitoring else { return }

        monitor.cancel()
        reconnectHandler = nil
        isMonitoring = false
        lastPath = nil
        wasDisconnected = false

        print("Network monitoring stopped")
    }

    // MARK: - Private Implementation

    private func handlePathUpdate(_ path: NWPath) {
        let previousPath = lastPath
        lastPath = path

        let statusDescription = pathStatusDescription(path.status)

        // Check for reconnection
        if path.status == .satisfied {
            // Network is available
            if wasDisconnected || previousPath?.status != .satisfied {
                // We just reconnected
                print("\n📶 Network reconnected (\(statusDescription))")
                print("  Interface: \(interfaceDescription(path))")

                wasDisconnected = false

                // Trigger queue synchronization
                Task {
                    await handleNetworkReconnect()
                }
            }
        } else {
            // Network is unavailable
            if previousPath?.status == .satisfied || previousPath == nil {
                // We just disconnected
                print("\n⚠️  Network disconnected (\(statusDescription))")
                wasDisconnected = true
            }
        }
    }

    private func handleNetworkReconnect() async {
        print("Checking for pending sync operations...")

        // First, call any custom handler
        if let handler = reconnectHandler {
            await handler()
        }

        // Then perform queue sync
        await performQueueSync()
    }

    /// Synchronize the offline queue using the CLI
    private func performQueueSync() async {
        print("🔄 Syncing offline queue...")

        let task = Process()
        task.executableURL = URL(fileURLWithPath: cliPath)
        task.arguments = ["queue", "sync"]

        let outputPipe = Pipe()
        let errorPipe = Pipe()
        task.standardOutput = outputPipe
        task.standardError = errorPipe

        do {
            try task.run()
            task.waitUntilExit()

            let outputData = outputPipe.fileHandleForReading.readDataToEndOfFile()
            let errorData = errorPipe.fileHandleForReading.readDataToEndOfFile()
            let output = String(data: outputData, encoding: .utf8) ?? ""
            let error = String(data: errorData, encoding: .utf8) ?? ""

            if task.terminationStatus == 0 {
                // Parse output to report sync results
                if output.contains("No pending operations") {
                    print("✓ No operations pending sync")
                } else if output.contains("synced") {
                    print("✓ Queue synchronized successfully")
                    if !output.isEmpty {
                        // Print relevant lines from output
                        let lines = output.components(separatedBy: .newlines)
                        for line in lines where line.contains("✓") || line.contains("synced") {
                            print("  \(line)")
                        }
                    }
                } else {
                    print("✓ Sync completed")
                }
            } else {
                print("⚠️  Queue sync failed with exit code: \(task.terminationStatus)")
                if !error.isEmpty {
                    print("  Error: \(error)")
                }
                // Don't retry immediately - will try again on next reconnect
            }
        } catch {
            print("⚠️  Failed to launch queue sync: \(error.localizedDescription)")
        }
    }

    // MARK: - Status Helpers

    private func pathStatusDescription(_ status: NWPath.Status) -> String {
        switch status {
        case .satisfied:
            return "Connected"
        case .unsatisfied:
            return "No Connection"
        case .requiresConnection:
            return "Requires Connection"
        @unknown default:
            return "Unknown"
        }
    }

    private func interfaceDescription(_ path: NWPath) -> String {
        var interfaces: [String] = []

        if path.usesInterfaceType(.wifi) {
            interfaces.append("WiFi")
        }
        if path.usesInterfaceType(.wiredEthernet) {
            interfaces.append("Ethernet")
        }
        if path.usesInterfaceType(.cellular) {
            interfaces.append("Cellular")
        }
        if path.usesInterfaceType(.loopback) {
            interfaces.append("Loopback")
        }

        if interfaces.isEmpty {
            return "Unknown"
        }

        return interfaces.joined(separator: ", ")
    }

    // MARK: - Status Queries

    /// Check if network is currently available
    public var isNetworkAvailable: Bool {
        return lastPath?.status == .satisfied
    }

    /// Check if we're using an expensive interface (cellular)
    public var isExpensiveConnection: Bool {
        return lastPath?.isExpensive ?? false
    }

    /// Check if we're using a constrained interface (low data mode)
    public var isConstrainedConnection: Bool {
        return lastPath?.isConstrained ?? false
    }

    /// Get current network status description
    public var statusDescription: String {
        guard let path = lastPath else {
            return "Not monitoring"
        }

        var status = pathStatusDescription(path.status)

        if path.status == .satisfied {
            status += " via \(interfaceDescription(path))"

            if path.isExpensive {
                status += " (Expensive)"
            }
            if path.isConstrained {
                status += " (Constrained)"
            }
        }

        return status
    }

    /// Get statistics for daemon status display
    public func getStatistics() -> [String: Any] {
        return [
            "isMonitoring": isMonitoring,
            "isAvailable": isNetworkAvailable,
            "isExpensive": isExpensiveConnection,
            "isConstrained": isConstrainedConnection,
            "status": statusDescription,
            "wasDisconnected": wasDisconnected
        ]
    }
}

// MARK: - Testing Utilities

@available(macOS 10.14, *)
extension NetworkMonitor {

    /// Simulate a reconnection event for testing
    public func simulateReconnect() async {
        print("\n[TEST] Simulating network reconnection")
        await handleNetworkReconnect()
    }

    /// Test queue sync without waiting for reconnect
    public func testQueueSync() async {
        print("\n[TEST] Testing queue sync")
        await performQueueSync()
    }

    /// Manually trigger sync (useful for CLI commands)
    public func manualSync() async {
        print("Manual network sync triggered")
        await performQueueSync()
    }
}
