import Foundation

/// Manages the lifecycle of the Oxen VCS daemon using launchctl
/// Handles registration, activation, and status monitoring of the LaunchAgent
@available(macOS 13.0, *)
public class ServiceManager {

    // MARK: - Properties

    private static let serviceName = "com.oxen.logic.daemon"
    private let plistPath: String

    public enum ServiceError: Error, LocalizedError {
        case registrationFailed(String)
        case unregistrationFailed(String)
        case statusCheckFailed(String)
        case notAuthorized
        case alreadyRegistered
        case notRegistered
        case plistNotFound

        public var errorDescription: String? {
            switch self {
            case .registrationFailed(let reason):
                return "Failed to register service: \(reason)"
            case .unregistrationFailed(let reason):
                return "Failed to unregister service: \(reason)"
            case .statusCheckFailed(let reason):
                return "Failed to check service status: \(reason)"
            case .notAuthorized:
                return "Not authorized to manage services"
            case .alreadyRegistered:
                return "Service is already registered"
            case .notRegistered:
                return "Service is not registered"
            case .plistNotFound:
                return "Service plist not found. Please run the install.sh script first."
            }
        }
    }

    // MARK: - Initialization

    public init() {
        let launchAgentsDir = NSString(string: "~/Library/LaunchAgents").expandingTildeInPath
        self.plistPath = "\(launchAgentsDir)/\(ServiceManager.serviceName).plist"
    }

    // MARK: - Service Management

    /// Register and start the LaunchAgent service
    /// - Throws: ServiceError if registration fails
    public func register() throws {
        // Check if plist exists
        guard FileManager.default.fileExists(atPath: plistPath) else {
            throw ServiceError.plistNotFound
        }

        // Check if already loaded
        if isServiceLoaded() {
            print("Service is already loaded and running")
            return
        }

        print("Registering LaunchAgent service...")

        // Use launchctl to load the service
        let task = Process()
        task.executableURL = URL(fileURLWithPath: "/bin/launchctl")
        task.arguments = ["load", plistPath]

        let pipe = Pipe()
        task.standardError = pipe

        do {
            try task.run()
            task.waitUntilExit()

            if task.terminationStatus == 0 {
                print("✓ Service registered successfully")
                print("The daemon will start automatically on login")
            } else {
                let data = pipe.fileHandleForReading.readDataToEndOfFile()
                let error = String(data: data, encoding: .utf8) ?? "Unknown error"
                throw ServiceError.registrationFailed("launchctl error: \(error)")
            }
        } catch let error as ServiceError {
            throw error
        } catch {
            throw ServiceError.registrationFailed(error.localizedDescription)
        }
    }

    /// Unregister the LaunchAgent service
    /// - Throws: ServiceError if unregistration fails
    public func unregister() throws {
        guard isServiceLoaded() else {
            print("Service is not loaded")
            return
        }

        print("Unregistering LaunchAgent service...")

        // Use launchctl to unload the service
        let task = Process()
        task.executableURL = URL(fileURLWithPath: "/bin/launchctl")
        task.arguments = ["unload", plistPath]

        let pipe = Pipe()
        task.standardError = pipe

        do {
            try task.run()
            task.waitUntilExit()

            if task.terminationStatus == 0 {
                print("✓ Service unregistered successfully")
            } else {
                let data = pipe.fileHandleForReading.readDataToEndOfFile()
                let error = String(data: data, encoding: .utf8) ?? "Unknown error"
                throw ServiceError.unregistrationFailed("launchctl error: \(error)")
            }
        } catch let error as ServiceError {
            throw error
        } catch {
            throw ServiceError.unregistrationFailed(error.localizedDescription)
        }
    }

    /// Check if the service is currently loaded with launchctl
    /// - Returns: true if loaded
    private func isServiceLoaded() -> Bool {
        let task = Process()
        task.executableURL = URL(fileURLWithPath: "/bin/launchctl")
        task.arguments = ["list", ServiceManager.serviceName]

        let pipe = Pipe()
        task.standardOutput = pipe
        task.standardError = pipe

        do {
            try task.run()
            task.waitUntilExit()
            return task.terminationStatus == 0
        } catch {
            return false
        }
    }

    /// Check if the service is currently running
    /// - Returns: true if enabled and running
    public func isRunning() -> Bool {
        return isServiceLoaded()
    }

    /// Get a human-readable status string
    /// - Returns: Status description
    public func getStatusDescription() -> String {
        let plistExists = FileManager.default.fileExists(atPath: plistPath)
        let serviceLoaded = isServiceLoaded()

        if !plistExists {
            return "✗ Service configuration not found (run install.sh)"
        } else if serviceLoaded {
            return "✓ Enabled and running"
        } else {
            return "○ Not loaded (run: oxvcs-daemon --install)"
        }
    }

    // MARK: - Installation Helpers

    /// Install the service with user-friendly feedback
    /// - Returns: true if successful, false otherwise
    public func install() -> Bool {
        do {
            try register()
            return true
        } catch ServiceError.notAuthorized {
            print("\nNext steps:")
            print("1. Open System Settings")
            print("2. Navigate to General → Login Items & Extensions")
            print("3. Enable 'Oxen VCS Daemon'")
            print("4. Run this command again")
            return false
        } catch {
            print("Installation failed: \(error.localizedDescription)")
            return false
        }
    }

    /// Uninstall the service
    /// - Returns: true if successful, false otherwise
    public func uninstall() -> Bool {
        do {
            try unregister()
            return true
        } catch {
            print("Uninstallation failed: \(error.localizedDescription)")
            return false
        }
    }

    // MARK: - Status Display

    /// Print detailed service status information
    public func printStatus() {
        print("Oxen VCS Daemon Status")
        print("=====================")
        print("Service: \(ServiceManager.serviceName)")
        print("Status: \(getStatusDescription())")
        print("Plist: \(plistPath)")

        if isRunning() {
            print("\nThe daemon is monitoring your Logic Pro projects")
            print("Auto-commits will be created after 30 seconds of inactivity")
        } else {
            print("\nThe daemon is not running")
            if FileManager.default.fileExists(atPath: plistPath) {
                print("Run: oxvcs-daemon --install")
            } else {
                print("Run: ./install.sh (to install the plist and daemon)")
            }
        }
    }

    // MARK: - Configuration

    /// Verify that the plist file exists and is valid
    /// - Returns: true if valid, false otherwise
    public static func verifyConfiguration() -> Bool {
        let possiblePaths = [
            "~/Library/LaunchAgents/\(serviceName).plist",
            "/Library/LaunchAgents/\(serviceName).plist",
        ]

        for path in possiblePaths {
            let expandedPath = NSString(string: path).expandingTildeInPath
            if FileManager.default.fileExists(atPath: expandedPath) {
                print("✓ Found plist at: \(expandedPath)")
                return true
            }
        }

        print("✗ No plist found at expected locations")
        print("Expected locations:")
        possiblePaths.forEach { print("  - \($0)") }
        return false
    }

    /// Install the plist to the correct location
    /// - Parameter sourcePath: Path to the plist file
    /// - Throws: Error if copy fails
    public static func installPlist(from sourcePath: String) throws {
        let targetDir = NSString(string: "~/Library/LaunchAgents").expandingTildeInPath
        let targetPath = "\(targetDir)/\(serviceName).plist"

        // Create LaunchAgents directory if it doesn't exist
        try FileManager.default.createDirectory(
            atPath: targetDir,
            withIntermediateDirectories: true,
            attributes: nil
        )

        // Copy plist
        try FileManager.default.copyItem(atPath: sourcePath, toPath: targetPath)

        // Set permissions
        try FileManager.default.setAttributes(
            [.posixPermissions: 0o644],
            ofItemAtPath: targetPath
        )

        print("✓ Installed plist to: \(targetPath)")
    }
}

// MARK: - Command Line Interface

@available(macOS 13.0, *)
extension ServiceManager {

    /// Handle command line arguments for service management
    /// - Parameter args: Command line arguments
    public static func handleCommand(_ args: [String]) {
        let manager = ServiceManager()

        guard args.count > 1 else {
            printUsage()
            return
        }

        let command = args[1]

        switch command {
        case "--install", "install":
            print("Installing Oxen VCS Daemon...")
            if manager.install() {
                print("\n✓ Installation complete")
            } else {
                print("\n✗ Installation failed")
                exit(1)
            }

        case "--uninstall", "uninstall":
            print("Uninstalling Oxen VCS Daemon...")
            if manager.uninstall() {
                print("\n✓ Uninstallation complete")
            } else {
                print("\n✗ Uninstallation failed")
                exit(1)
            }

        case "--status", "status":
            manager.printStatus()

        case "--verify", "verify":
            print("Verifying service configuration...")
            if verifyConfiguration() {
                print("\n✓ Configuration is valid")
            } else {
                print("\n✗ Configuration is invalid")
                exit(1)
            }

        default:
            print("Unknown command: \(command)")
            printUsage()
            exit(1)
        }
    }

    private static func printUsage() {
        print("""
        Oxen VCS Service Manager

        Usage:
          oxvcs-daemon --install      Install and register the daemon
          oxvcs-daemon --uninstall    Uninstall and stop the daemon
          oxvcs-daemon --status       Show daemon status
          oxvcs-daemon --verify       Verify configuration
          oxvcs-daemon --daemon       Run as daemon (internal use)

        The daemon provides:
          • Automatic file system monitoring
          • Auto-commits after inactivity
          • Power management (pre-sleep commits)
          • XPC communication for UI
        """)
    }
}
