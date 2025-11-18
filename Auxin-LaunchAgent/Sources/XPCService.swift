import Foundation

// MARK: - XPC Protocol

/// Protocol for communication between UI app and background daemon
/// All methods are async and return results via completion handlers
@objc public protocol OxenDaemonXPCProtocol {

    /// Initialize an Oxen repository for a Logic Pro project
    /// - Parameters:
    ///   - projectPath: Absolute path to .logicx project
    ///   - reply: Completion handler with success status and error message
    func initializeProject(
        _ projectPath: String,
        withReply reply: @escaping (Bool, String?) -> Void
    )

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

    /// Acquire an exclusive lock for a project
    /// - Parameters:
    ///   - projectPath: Path to project
    ///   - timeoutHours: Lock timeout in hours (default: 24)
    ///   - reply: Success status and error message
    func acquireLock(
        for projectPath: String,
        timeoutHours: Int,
        withReply reply: @escaping (Bool, String?) -> Void
    )

    /// Release a lock for a project
    /// - Parameters:
    ///   - projectPath: Path to project
    ///   - reply: Success status and error message
    func releaseLock(
        for projectPath: String,
        withReply reply: @escaping (Bool, String?) -> Void
    )

    /// Force-break a lock (admin operation)
    /// - Parameters:
    ///   - projectPath: Path to project
    ///   - reply: Success status and error message
    func forceBreakLock(
        for projectPath: String,
        withReply reply: @escaping (Bool, String?) -> Void
    )

    /// Get lock information for a project
    /// - Parameters:
    ///   - projectPath: Path to project
    ///   - reply: Lock info dictionary or nil if not locked
    func getLockInfo(
        for projectPath: String,
        withReply reply: @escaping ([String: Any]?) -> Void
    )

    /// Get current configuration settings
    /// - Parameter reply: Configuration dictionary
    func getConfiguration(
        withReply reply: @escaping ([String: Any]) -> Void
    )

    /// Set debounce time for auto-commits
    /// - Parameters:
    ///   - seconds: Debounce time in seconds
    ///   - reply: Success status
    func setDebounceTime(
        _ seconds: Int,
        withReply reply: @escaping (Bool) -> Void
    )

    /// Set default lock timeout
    /// - Parameters:
    ///   - hours: Timeout in hours
    ///   - reply: Success status
    func setLockTimeout(
        _ hours: Int,
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

    // Configuration settings
    private var debounceTime: Int {
        get { UserDefaults.standard.integer(forKey: "debounceTime") == 0 ? 30 : UserDefaults.standard.integer(forKey: "debounceTime") }
        set { UserDefaults.standard.set(newValue, forKey: "debounceTime") }
    }

    private var lockTimeout: Int {
        get { UserDefaults.standard.integer(forKey: "lockTimeout") == 0 ? 24 : UserDefaults.standard.integer(forKey: "lockTimeout") }
        set { UserDefaults.standard.set(newValue, forKey: "lockTimeout") }
    }

    // MARK: - Initialization

    public init(orchestrator: CommitOrchestrator) {
        self.orchestrator = orchestrator
        self.listener = NSXPCListener(machServiceName: "com.auxin.daemon.xpc")

        super.init()

        self.listener.delegate = self
    }

    /// Start listening for XPC connections
    public func start() {
        listener.resume()
        print("✓ XPC service started")
        print("  Mach service: com.auxin.daemon.xpc")
    }

    /// Stop the XPC service
    public func stop() {
        listener.suspend()
        print("XPC service stopped")
    }

    // MARK: - Protocol Implementation

    public func initializeProject(
        _ projectPath: String,
        withReply reply: @escaping (Bool, String?) -> Void
    ) {
        print("XPC: Initialize project: \(projectPath)")

        // Validate path
        guard FileManager.default.fileExists(atPath: projectPath) else {
            reply(false, "Project not found at path")
            return
        }

        // Detect project type
        guard let projectType = ProjectType.detect(from: projectPath) else {
            let supportedExtensions = ProjectType.supportedExtensions.map { ".\($0)" }.joined(separator: ", ")
            reply(false, "Unsupported project type. Supported types: \(supportedExtensions)")
            return
        }

        print("  Detected project type: \(projectType.displayName)")

        // Check if already initialized
        let oxenDir: String
        if projectType.isFolderBased {
            oxenDir = (projectPath as NSString).appendingPathComponent(".oxen")
        } else {
            // For file-based projects, .oxen is in the same directory
            oxenDir = (URL(fileURLWithPath: projectPath).deletingLastPathComponent()
                .appendingPathComponent(".oxen")).path
        }

        if FileManager.default.fileExists(atPath: oxenDir) {
            print("✓ Project already initialized, registering for monitoring")

            // Verify it's a valid Oxen repository
            let configPath = (oxenDir as NSString).appendingPathComponent("config.toml")
            guard FileManager.default.fileExists(atPath: configPath) else {
                print("⚠️  .oxen directory exists but appears corrupted")
                reply(false, "Project appears to have a corrupted Oxen repository. Please remove the .oxen directory and try again.")
                return
            }

            orchestrator.registerProject(projectPath)
            reply(true, "already_initialized")
            return
        }

        // Call CLI to initialize the Oxen repository
        let task = Process()
        task.executableURL = URL(fileURLWithPath: "/usr/local/bin/auxin")
        task.arguments = ["init", projectType.cliArgument, projectPath]

        let outputPipe = Pipe()
        let errorPipe = Pipe()
        task.standardOutput = outputPipe
        task.standardError = errorPipe

        task.terminationHandler = { [weak self] process in
            let outputData = outputPipe.fileHandleForReading.readDataToEndOfFile()
            let errorData = errorPipe.fileHandleForReading.readDataToEndOfFile()
            let output = String(data: outputData, encoding: .utf8) ?? ""
            let error = String(data: errorData, encoding: .utf8) ?? ""

            if process.terminationStatus == 0 {
                print("✓ Project initialized successfully")
                print("  Output: \(output)")

                // Register for monitoring after successful initialization
                self?.orchestrator.registerProject(projectPath)
                reply(true, nil)
            } else {
                print("✗ Failed to initialize project")
                print("  Error: \(error)")
                reply(false, "Failed to initialize: \(error.isEmpty ? "Unknown error" : error)")
            }
        }

        do {
            try task.run()
        } catch {
            print("✗ Failed to launch CLI: \(error)")
            reply(false, "Failed to launch CLI tool: \(error.localizedDescription)")
        }
    }

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

        // Detect project type
        guard let projectType = ProjectType.detect(from: projectPath) else {
            let supportedExtensions = ProjectType.supportedExtensions.map { ".\($0)" }.joined(separator: ", ")
            reply(false, "Unsupported project type. Supported types: \(supportedExtensions)")
            return
        }

        print("  Detected project type: \(projectType.displayName)")

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

    public func acquireLock(
        for projectPath: String,
        timeoutHours: Int,
        withReply reply: @escaping (Bool, String?) -> Void
    ) {
        print("XPC: Acquire lock for: \(projectPath)")

        let success = LockManager.shared.acquireLock(projectPath: projectPath, timeoutHours: timeoutHours)
        if success {
            reply(true, nil)
        } else {
            if let lock = LockManager.shared.getLockInfo(projectPath: projectPath) {
                reply(false, "Project is already locked by \(lock.lockedBy)")
            } else {
                reply(false, "Failed to acquire lock")
            }
        }
    }

    public func releaseLock(
        for projectPath: String,
        withReply reply: @escaping (Bool, String?) -> Void
    ) {
        print("XPC: Release lock for: \(projectPath)")

        let success = LockManager.shared.releaseLock(projectPath: projectPath)
        if success {
            reply(true, nil)
        } else {
            reply(false, "Failed to release lock (not locked or owned by someone else)")
        }
    }

    public func forceBreakLock(
        for projectPath: String,
        withReply reply: @escaping (Bool, String?) -> Void
    ) {
        print("XPC: Force-break lock for: \(projectPath)")

        let success = LockManager.shared.forceBreakLock(projectPath: projectPath)
        if success {
            reply(true, nil)
        } else {
            reply(false, "Failed to break lock (project not locked)")
        }
    }

    public func getLockInfo(
        for projectPath: String,
        withReply reply: @escaping ([String: Any]?) -> Void
    ) {
        guard let lock = LockManager.shared.getLockInfo(projectPath: projectPath) else {
            reply(nil)
            return
        }

        let formatter = ISO8601DateFormatter()
        let lockInfo: [String: Any] = [
            "projectPath": lock.projectPath,
            "lockedBy": lock.lockedBy,
            "lockId": lock.lockId,
            "acquiredAt": formatter.string(from: lock.acquiredAt),
            "expiresAt": formatter.string(from: lock.expiresAt),
            "isExpired": lock.isExpired,
            "remainingHours": lock.remainingHours,
            "isLocked": true
        ]

        reply(lockInfo)
    }

    public func getConfiguration(
        withReply reply: @escaping ([String: Any]) -> Void
    ) {
        let config: [String: Any] = [
            "debounceTime": debounceTime,
            "lockTimeout": lockTimeout
        ]
        reply(config)
    }

    public func setDebounceTime(
        _ seconds: Int,
        withReply reply: @escaping (Bool) -> Void
    ) {
        guard seconds >= 5 && seconds <= 300 else {
            print("XPC: Invalid debounce time: \(seconds) (must be 5-300 seconds)")
            reply(false)
            return
        }

        print("XPC: Set debounce time to: \(seconds) seconds")
        debounceTime = seconds

        // Note: This change will apply to newly registered projects
        // Existing monitors would need to be recreated to use the new value
        reply(true)
    }

    public func setLockTimeout(
        _ hours: Int,
        withReply reply: @escaping (Bool) -> Void
    ) {
        guard hours >= 1 && hours <= 168 else {
            print("XPC: Invalid lock timeout: \(hours) (must be 1-168 hours)")
            reply(false)
            return
        }

        print("XPC: Set lock timeout to: \(hours) hours")
        lockTimeout = hours

        // This will apply to newly acquired locks
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
        let connection = NSXPCConnection(machServiceName: "com.auxin.daemon.xpc")

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
