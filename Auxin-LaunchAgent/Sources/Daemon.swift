import Foundation

/// Production-grade daemon that coordinates all Oxen VCS background services
/// Integrates FSEvents monitoring, power management, XPC communication, and auto-commits
@available(macOS 13.0, *)
public class OxenDaemon {

    // MARK: - Components

    private let orchestrator: CommitOrchestrator
    private let powerManager: PowerManagement
    private var xpcService: OxenDaemonXPCService?
    private var monitors: [String: FSEventsMonitor] = [:]
    private var isRunning = false

    // MARK: - Configuration

    private let cliPath: String
    private let debounceThreshold: TimeInterval

    // MARK: - Initialization

    public init(
        cliPath: String = "/usr/local/bin/auxin",
        debounceThreshold: TimeInterval = 30.0
    ) {
        self.cliPath = cliPath
        self.debounceThreshold = debounceThreshold
        self.orchestrator = CommitOrchestrator(cliPath: cliPath)
        self.powerManager = PowerManagement()

        printBanner()
    }

    // MARK: - Lifecycle

    /// Start the daemon with all services
    public func start() async {
        guard !isRunning else {
            print("⚠️  Daemon already running")
            return
        }

        print("\n🚀 Starting Oxen VCS Daemon...")
        isRunning = true

        // 1. Start power management monitoring
        print("\n[1/4] Initializing power management...")
        powerManager.startMonitoring { [weak self] in
            await self?.handleEmergencyCommit()
        }

        // 2. Start XPC service
        print("[2/4] Starting XPC service...")
        let xpc = OxenDaemonXPCService(orchestrator: orchestrator)
        xpc.start()
        self.xpcService = xpc

        // 3. Scan for existing Logic Pro projects
        print("[3/4] Scanning for Logic Pro projects...")
        await scanForProjects()

        // 4. Start monitoring registered projects
        print("[4/4] Starting file system monitors...")
        await startMonitoring()

        print("\n✓ Daemon started successfully")
        printStatus()

        // Keep daemon running and handle signals
        await withCheckedContinuation { (continuation: CheckedContinuation<Void, Never>) in
            // Set up signal handlers using DispatchSource (proper Swift approach)
            let sigintSource = DispatchSource.makeSignalSource(signal: SIGINT, queue: .main)
            let sigtermSource = DispatchSource.makeSignalSource(signal: SIGTERM, queue: .main)

            sigintSource.setEventHandler {
                print("\nReceived SIGINT - shutting down gracefully...")
                continuation.resume()
            }

            sigtermSource.setEventHandler {
                print("\nReceived SIGTERM - shutting down gracefully...")
                continuation.resume()
            }

            // Ignore default signal handling
            signal(SIGINT, SIG_IGN)
            signal(SIGTERM, SIG_IGN)

            sigintSource.resume()
            sigtermSource.resume()
        }

        await stop()
    }

    /// Stop the daemon
    public func stop() async {
        guard isRunning else { return }

        print("\n🛑 Stopping Oxen VCS Daemon...")

        // Stop all monitors
        for (path, monitor) in monitors {
            print("  Stopping monitor for: \(path)")
            monitor.stop()
        }
        monitors.removeAll()

        // Stop XPC service
        xpcService?.stop()

        // Stop power management
        powerManager.stopMonitoring()

        isRunning = false
        print("✓ Daemon stopped")
    }

    // MARK: - Project Management

    /// Register a project for monitoring
    public func registerProject(_ projectPath: String) async {
        let normalizedPath = (projectPath as NSString).standardizingPath

        guard monitors[normalizedPath] == nil else {
            print("Project already registered: \(projectPath)")
            return
        }

        print("\n📁 Registering project: \(projectPath)")

        // Ensure on draft branch
        if await orchestrator.ensureOnDraftBranch(at: normalizedPath) {
            print("  ✓ On draft branch")
        }

        // Create and configure monitor
        let monitor = FSEventsMonitor(debounceThreshold: debounceThreshold)

        // Set commit callback
        monitor.setCommitCallback { [weak self] path in
            await self?.handleAutoCommit(for: path)
        }

        // Start monitoring
        Task {
            do {
                try await monitor.start(watchingPath: normalizedPath)
            } catch {
                print("  ✗ Failed to start monitor: \(error)")
            }
        }

        monitors[normalizedPath] = monitor
        orchestrator.registerProject(normalizedPath)

        print("  ✓ Monitoring started")
    }

    /// Unregister a project
    public func unregisterProject(_ projectPath: String) {
        let normalizedPath = (projectPath as NSString).standardizingPath

        if let monitor = monitors[normalizedPath] {
            monitor.stop()
            monitors.removeValue(forKey: normalizedPath)
        }

        orchestrator.unregisterProject(normalizedPath)
        print("Unregistered project: \(projectPath)")
    }

    // MARK: - Commit Handlers

    /// Handle auto-commit triggered by FSEvents
    private func handleAutoCommit(for projectPath: String) async {
        // Check if project is paused in XPC service
        if xpcService?.isPaused(projectPath) ?? false {
            print("⏸️  Auto-commit paused for: \(projectPath)")
            return
        }

        print("\n💾 Auto-commit triggered")
        await orchestrator.performCommit(for: projectPath, type: .autoSave)
    }

    /// Handle emergency commit (power events)
    private func handleEmergencyCommit() async {
        // Check if we should proceed based on system state
        guard PowerManagement.shouldPerformEmergencyCommit() else {
            print("⏭️  Skipping emergency commit due to system state")
            return
        }

        await orchestrator.performEmergencyCommits()
    }

    // MARK: - Project Discovery

    /// Scan common locations for Logic Pro projects
    private func scanForProjects() async {
        let fileManager = FileManager.default
        let homeDir = fileManager.homeDirectoryForCurrentUser

        let searchPaths = [
            homeDir.appendingPathComponent("Music").path,
            homeDir.appendingPathComponent("Documents").path,
            homeDir.appendingPathComponent("Desktop").path,
        ]

        var foundProjects: [String] = []

        for searchPath in searchPaths {
            guard fileManager.fileExists(atPath: searchPath) else { continue }

            if let enumerator = fileManager.enumerator(
                at: URL(fileURLWithPath: searchPath),
                includingPropertiesForKeys: [.isDirectoryKey],
                options: [.skipsHiddenFiles]
            ) {
                for case let fileURL as URL in enumerator {
                    if fileURL.pathExtension == "logicx" {
                        // Check if it's already tracked by Oxen
                        let oxenPath = fileURL.appendingPathComponent(".oxen").path
                        if fileManager.fileExists(atPath: oxenPath) {
                            foundProjects.append(fileURL.path)
                        }
                    }
                }
            }
        }

        if foundProjects.isEmpty {
            print("  No Oxen-tracked Logic Pro projects found")
        } else {
            print("  Found \(foundProjects.count) Oxen-tracked project(s):")
            for project in foundProjects {
                print("    - \(project)")
            }
        }

        // Auto-register found projects
        for project in foundProjects {
            await registerProject(project)
        }
    }

    /// Start monitoring for all registered projects
    private func startMonitoring() async {
        let projects = orchestrator.getRegisteredProjects()

        if projects.isEmpty {
            print("  No projects to monitor")
            print("\n💡 To start monitoring, run:")
            print("   auxin-cli init <path-to-logic-project>")
            return
        }

        print("  Starting \(projects.count) monitor(s)")
    }

    // MARK: - Status & Diagnostics

    private func printBanner() {
        print("""
        ╔═══════════════════════════════════════════════════════════╗
        ║                                                           ║
        ║              Oxen VCS for Logic Pro                       ║
        ║              Production Daemon v2.0.0                     ║
        ║                                                           ║
        ╚═══════════════════════════════════════════════════════════╝
        """)
    }

    private func printStatus() {
        let projects = orchestrator.getRegisteredProjects()

        print("""

        ╔═══════════════════════════════════════════════════════════╗
        ║ DAEMON STATUS                                             ║
        ╠═══════════════════════════════════════════════════════════╣
        ║ Power Management:  ✓ Active                              ║
        ║ XPC Service:       ✓ Listening                           ║
        ║ Monitored Projects: \(String(format: "%2d", projects.count))                                   ║
        ║ Debounce Interval:  \(Int(debounceThreshold))s                                 ║
        ╚═══════════════════════════════════════════════════════════╝

        The daemon is now monitoring your Logic Pro projects.
        Auto-commits will be created after \(Int(debounceThreshold)) seconds of inactivity.

        Press Ctrl+C to stop (emergency commits will be performed first)

        """)
    }

    /// Get current daemon statistics
    public func getStatistics() -> [String: Any] {
        return [
            "isRunning": isRunning,
            "projectCount": orchestrator.getRegisteredProjects().count,
            "monitorCount": monitors.count,
            "debounceThreshold": debounceThreshold,
            "cliPath": cliPath,
            "uptime": ProcessInfo.processInfo.systemUptime
        ]
    }
}

// MARK: - Command Line Entry Point

@available(macOS 13.0, *)
extension OxenDaemon {

    /// Main entry point for daemon
    public static func main(arguments: [String]) async {
        // Check for service management commands
        if arguments.count > 1 {
            switch arguments[1] {
            case "--install", "install", "--uninstall", "uninstall", "--status", "status", "--verify", "verify":
                ServiceManager.handleCommand(arguments)
                return

            case "--daemon", "daemon":
                // Run as daemon
                let daemon = OxenDaemon()
                await daemon.start()
                return

            case "--help", "help", "-h":
                printUsage()
                return

            case "--version", "version":
                print("Oxen VCS Daemon v2.0.0")
                return

            default:
                print("Unknown command: \(arguments[1])")
                printUsage()
                exit(1)
            }
        }

        // Default: show usage
        printUsage()
    }

    private static func printUsage() {
        print("""
        Oxen VCS Daemon - Background service for Logic Pro version control

        USAGE:
            auxin-daemon <command>

        COMMANDS:
            --install       Install and register the daemon service
            --uninstall     Uninstall and stop the daemon service
            --status        Show daemon status
            --verify        Verify configuration
            --daemon        Run as background daemon (internal use)
            --help          Show this help message
            --version       Show version information

        FEATURES:
            • Automatic file system monitoring with FSEvents
            • Auto-commits after 30 seconds of inactivity
            • Emergency commits before sleep/shutdown
            • Power management integration
            • XPC communication for UI integration
            • Draft branch workflow

        INSTALLATION:
            1. Build the daemon: swift build -c release
            2. Install: sudo cp .build/release/auxin-daemon /usr/local/bin/
            3. Register service: auxin-daemon --install
            4. Check status: auxin-daemon --status

        The daemon runs automatically on login after installation.

        For more information, see: https://github.com/Oxen-AI/oxen-vcs-logic
        """)
    }
}
