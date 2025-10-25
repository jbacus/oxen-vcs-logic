import Foundation
import ServiceManagement

/// Manages the lifecycle of the Oxen VCS daemon using SMAppService
/// Handles registration, activation, and status monitoring of the LaunchAgent
@available(macOS 13.0, *)
public class ServiceManager {

    // MARK: - Properties

    private static let serviceName = "com.oxen.logic.daemon"
    private let service: SMAppService

    public enum ServiceError: Error, LocalizedError {
        case registrationFailed(String)
        case unregistrationFailed(String)
        case statusCheckFailed(String)
        case notAuthorized
        case alreadyRegistered
        case notRegistered

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
            }
        }
    }

    // MARK: - Initialization

    public init() {
        self.service = SMAppService.agent(plistName: ServiceManager.serviceName)
    }

    // MARK: - Service Management

    /// Register and start the LaunchAgent service
    /// - Throws: ServiceError if registration fails
    public func register() throws {
        let status = service.status

        switch status {
        case .enabled:
            print("Service is already enabled and running")
            return

        case .requiresApproval:
            print("⚠️  Service requires user approval in System Settings")
            print("Please go to: System Settings → General → Login Items & Extensions")
            print("Enable: Oxen VCS Daemon")
            throw ServiceError.notAuthorized

        case .notRegistered:
            print("Registering LaunchAgent service...")
            do {
                try service.register()
                print("✓ Service registered successfully")
                print("The daemon will start automatically on login")
            } catch {
                throw ServiceError.registrationFailed(error.localizedDescription)
            }

        case .notFound:
            throw ServiceError.registrationFailed("Service plist not found at expected location")

        @unknown default:
            throw ServiceError.statusCheckFailed("Unknown service status: \(status)")
        }
    }

    /// Unregister the LaunchAgent service
    /// - Throws: ServiceError if unregistration fails
    public func unregister() throws {
        let status = service.status

        guard status != .notRegistered else {
            print("Service is not registered")
            return
        }

        print("Unregistering LaunchAgent service...")
        do {
            try service.unregister()
            print("✓ Service unregistered successfully")
        } catch {
            throw ServiceError.unregistrationFailed(error.localizedDescription)
        }
    }

    /// Get the current status of the service
    /// - Returns: SMAppService.Status
    public func getStatus() -> SMAppService.Status {
        return service.status
    }

    /// Check if the service is currently running
    /// - Returns: true if enabled and running
    public func isRunning() -> Bool {
        return service.status == .enabled
    }

    /// Get a human-readable status string
    /// - Returns: Status description
    public func getStatusDescription() -> String {
        switch service.status {
        case .enabled:
            return "✓ Enabled and running"
        case .requiresApproval:
            return "⚠️  Requires approval in System Settings"
        case .notRegistered:
            return "○ Not registered"
        case .notFound:
            return "✗ Service configuration not found"
        @unknown default:
            return "? Unknown status"
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

        if isRunning() {
            print("\nThe daemon is monitoring your Logic Pro projects")
            print("Auto-commits will be created after 30 seconds of inactivity")
        } else if service.status == .requiresApproval {
            print("\n⚠️  Action Required:")
            print("Go to System Settings → General → Login Items")
            print("Enable 'Oxen VCS Daemon' to start automatic tracking")
        } else {
            print("\nThe daemon is not running")
            print("Run: oxvcs-daemon --install")
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
