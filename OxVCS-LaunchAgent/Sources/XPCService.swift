import Foundation

// MARK: - XPC Protocol

/// Protocol for communication between UI app and background daemon
/// All methods are async and return results via completion handlers
@objc public protocol OxenDaemonXPCProtocol {

    /// Register a Logic Pro project for automatic monitoring
    /// - Parameters:
    ///   - projectPath: Absolute path to .logicx project
    ///   - reply: Completion handler with success status and error message
    func registerProject(
        _ projectPath: String,
        withReply reply: @escaping (Bool, String?) -> Void
    )

    /// Unregister a project from monitoring
    /// - Parameters:
    ///   - projectPath: Absolute path to project
    ///   - reply: Completion handler
    func unregisterProject(
        _ projectPath: String,
        withReply reply: @escaping (Bool, String?) -> Void
    )

    /// Get list of currently monitored projects
    /// - Parameter reply: Completion handler with array of project paths
    func getMonitoredProjects(
        withReply reply: @escaping ([String]) -> Void
    )

    /// Trigger a manual commit for a project
    /// - Parameters:
    ///   - projectPath: Path to project
    ///   - message: Optional custom commit message
    ///   - reply: Completion handler with commit ID and error
    func commitProject(
        _ projectPath: String,
        message: String?,
        withReply reply: @escaping (String?, String?) -> Void
    )

    /// Get the current status of the daemon
    /// - Parameter reply: Status dictionary with keys: isRunning, projectCount, lastCommit
    func getStatus(
        withReply reply: @escaping ([String: Any]) -> Void
    )

    /// Get commit history for a project
    /// - Parameters:
    ///   - projectPath: Path to project
    ///   - limit: Maximum number of commits to return
    ///   - reply: Array of commit dictionaries
    func getCommitHistory(
        for projectPath: String,
        limit: Int,
        withReply reply: @escaping ([[String: Any]]) -> Void
    )

    /// Restore a project to a specific commit
    /// - Parameters:
    ///   - projectPath: Path to project
    ///   - commitId: Commit hash to restore
    ///   - reply: Success status and error message
    func restoreProject(
        _ projectPath: String,
        toCommit commitId: String,
        withReply reply: @escaping (Bool, String?) -> Void
    )

    /// Pause automatic commits for a project
    /// - Parameters:
    ///   - projectPath: Path to project
    ///   - reply: Success status
    func pauseMonitoring(
        for projectPath: String,
        withReply reply: @escaping (Bool) -> Void
    )

    /// Resume automatic commits for a project
    /// - Parameters:
    ///   - projectPath: Path to project
    ///   - reply: Success status
    func resumeMonitoring(
        for projectPath: String,
        withReply reply: @escaping (Bool) -> Void
    )

    /// Perform a health check on the daemon
    /// - Parameter reply: Health status (true = healthy)
    func ping(
        withReply reply: @escaping (Bool) -> Void
    )
}

// MARK: - XPC Service Implementation

/// Server-side implementation of the XPC service
/// Runs in the background daemon and handles requests from the UI
public class OxenDaemonXPCService: NSObject, OxenDaemonXPCProtocol {

    // MARK: - Properties

    private let orchestrator: CommitOrchestrator
    private let listener: NSXPCListener
    private var pausedProjects: Set<String> = []

    // MARK: - Initialization

    public init(orchestrator: CommitOrchestrator) {
        self.orchestrator = orchestrator
        self.listener = NSXPCListener(machServiceName: "com.oxen.logic.daemon.xpc")

        super.init()

        self.listener.delegate = self
    }

    /// Start listening for XPC connections
    public func start() {
        listener.resume()
        print("✓ XPC service started")
        print("  Mach service: com.oxen.logic.daemon.xpc")
    }

    /// Stop the XPC service
    public func stop() {
        listener.suspend()
        print("XPC service stopped")
    }

    // MARK: - Protocol Implementation

    public func registerProject(
        _ projectPath: String,
        withReply reply: @escaping (Bool, String?) -> Void
    ) {
        print("XPC: Register project: \(projectPath)")

        // Validate path
        guard FileManager.default.fileExists(atPath: projectPath) else {
            reply(false, "Project not found at path")
            return
        }

        guard projectPath.hasSuffix(".logicx") else {
            reply(false, "Invalid Logic Pro project (must be .logicx)")
            return
        }

        orchestrator.registerProject(projectPath)
        reply(true, nil)
    }

    public func unregisterProject(
        _ projectPath: String,
        withReply reply: @escaping (Bool, String?) -> Void
    ) {
        print("XPC: Unregister project: \(projectPath)")
        orchestrator.unregisterProject(projectPath)
        pausedProjects.remove(projectPath)
        reply(true, nil)
    }

    public func getMonitoredProjects(
        withReply reply: @escaping ([String]) -> Void
    ) {
        let projects = Array(orchestrator.getRegisteredProjects())
        reply(projects)
    }

    public func commitProject(
        _ projectPath: String,
        message: String?,
        withReply reply: @escaping (String?, String?) -> Void
    ) {
        print("XPC: Manual commit for: \(projectPath)")

        Task {
            let result = await orchestrator.performCommit(
                for: projectPath,
                type: .manual
            )

            if result.success {
                reply(result.commitId, nil)
            } else {
                reply(nil, result.message)
            }
        }
    }

    public func getStatus(
        withReply reply: @escaping ([String: Any]) -> Void
    ) {
        let projects = orchestrator.getRegisteredProjects()

        let status: [String: Any] = [
            "isRunning": true,
            "projectCount": projects.count,
            "pausedCount": pausedProjects.count,
            "version": "2.0.0",
            "uptime": ProcessInfo.processInfo.systemUptime
        ]

        reply(status)
    }

    public func getCommitHistory(
        for projectPath: String,
        limit: Int,
        withReply reply: @escaping ([[String: Any]]) -> Void
    ) {
        // This would call the Rust CLI to get commit history
        // For now, return empty array as placeholder
        print("XPC: Get commit history for: \(projectPath) (limit: \(limit))")
        reply([])
    }

    public func restoreProject(
        _ projectPath: String,
        toCommit commitId: String,
        withReply reply: @escaping (Bool, String?) -> Void
    ) {
        print("XPC: Restore project: \(projectPath) to commit: \(commitId)")
        // This would call the Rust CLI to restore
        reply(false, "Not implemented yet")
    }

    public func pauseMonitoring(
        for projectPath: String,
        withReply reply: @escaping (Bool) -> Void
    ) {
        print("XPC: Pause monitoring for: \(projectPath)")
        pausedProjects.insert(projectPath)
        reply(true)
    }

    public func resumeMonitoring(
        for projectPath: String,
        withReply reply: @escaping (Bool) -> Void
    ) {
        print("XPC: Resume monitoring for: \(projectPath)")
        pausedProjects.remove(projectPath)
        reply(true)
    }

    public func ping(withReply reply: @escaping (Bool) -> Void) {
        reply(true)
    }

    // MARK: - Helpers

    /// Check if a project is currently paused
    /// - Parameter projectPath: Path to check
    /// - Returns: true if paused
    public func isPaused(_ projectPath: String) -> Bool {
        return pausedProjects.contains(projectPath)
    }
}

// MARK: - XPC Listener Delegate

extension OxenDaemonXPCService: NSXPCListenerDelegate {

    public func listener(
        _ listener: NSXPCListener,
        shouldAcceptNewConnection newConnection: NSXPCConnection
    ) -> Bool {
        print("XPC: New connection request")

        // Set up the connection
        newConnection.exportedInterface = NSXPCInterface(with: OxenDaemonXPCProtocol.self)
        newConnection.exportedObject = self

        // Handle connection lifecycle
        newConnection.invalidationHandler = {
            print("XPC: Connection invalidated")
        }

        newConnection.interruptionHandler = {
            print("XPC: Connection interrupted")
        }

        newConnection.resume()

        print("✓ XPC connection accepted")
        return true
    }
}

// MARK: - XPC Client

/// Client-side XPC connection for UI app to communicate with daemon
public class OxenDaemonXPCClient {

    private var connection: NSXPCConnection?

    public init() {
        setupConnection()
    }

    deinit {
        connection?.invalidate()
    }

    private func setupConnection() {
        let connection = NSXPCConnection(machServiceName: "com.oxen.logic.daemon.xpc")

        connection.remoteObjectInterface = NSXPCInterface(with: OxenDaemonXPCProtocol.self)

        connection.invalidationHandler = {
            print("XPC Client: Connection invalidated")
        }

        connection.interruptionHandler = {
            print("XPC Client: Connection interrupted - will reconnect")
        }

        connection.resume()
        self.connection = connection

        print("✓ XPC client connected")
    }

    /// Get the remote proxy object
    /// - Returns: Proxy conforming to OxenDaemonXPCProtocol
    public func getProxy() -> OxenDaemonXPCProtocol? {
        return connection?.remoteObjectProxy as? OxenDaemonXPCProtocol
    }

    /// Get proxy with error handler
    /// - Parameter errorHandler: Called if connection fails
    /// - Returns: Proxy object
    public func getProxy(
        errorHandler: @escaping (Error) -> Void
    ) -> OxenDaemonXPCProtocol? {
        return connection?.remoteObjectProxyWithErrorHandler { error in
            errorHandler(error)
        } as? OxenDaemonXPCProtocol
    }

    /// Test connection to daemon
    /// - Parameter completion: Called with success status
    public func testConnection(completion: @escaping (Bool) -> Void) {
        guard let proxy = getProxy() else {
            completion(false)
            return
        }

        proxy.ping { isAlive in
            completion(isAlive)
        }
    }
}
