import Foundation

/// Orchestrates automatic commits using the Rust CLI wrapper
/// Handles both regular debounced commits and emergency power-event commits
public class CommitOrchestrator {

    // MARK: - Properties

    private let cliPath: String
    private var monitoredProjects: Set<String> = []
    private var isCommitting = false
    private let commitQueue = DispatchQueue(label: "com.oxen.logic.commit", qos: .userInitiated)

    public enum CommitType {
        case autoSave           // Regular auto-save after debounce
        case emergency          // Pre-sleep/shutdown commit
        case manual             // User-triggered commit
    }

    public struct CommitResult {
        public let success: Bool
        public let commitId: String?
        public let message: String
        public let duration: TimeInterval
    }

    // MARK: - Initialization

    /// Initialize with path to the Rust CLI binary
    /// - Parameter cliPath: Path to oxenvcs-cli binary
    public init(cliPath: String = "/usr/local/bin/oxenvcs-cli") {
        self.cliPath = cliPath
        verifyCliExists()
    }

    private func verifyCliExists() {
        if !FileManager.default.fileExists(atPath: cliPath) {
            print("⚠️  Warning: CLI binary not found at: \(cliPath)")
            print("   Expected location: \(cliPath)")
            print("   Make sure to build and install the Rust CLI wrapper")
        }
    }

    // MARK: - Project Registration

    /// Register a Logic Pro project for monitoring
    /// - Parameter projectPath: Path to .logicx project
    public func registerProject(_ projectPath: String) {
        let normalizedPath = (projectPath as NSString).standardizingPath
        monitoredProjects.insert(normalizedPath)
        print("Registered project: \(normalizedPath)")
    }

    /// Unregister a project from monitoring
    /// - Parameter projectPath: Path to .logicx project
    public func unregisterProject(_ projectPath: String) {
        let normalizedPath = (projectPath as NSString).standardizingPath
        monitoredProjects.remove(normalizedPath)
        print("Unregistered project: \(normalizedPath)")
    }

    /// Get all registered projects
    /// - Returns: Set of project paths
    public func getRegisteredProjects() -> Set<String> {
        return monitoredProjects
    }

    // MARK: - Commit Operations

    /// Perform an automatic commit for a project
    /// - Parameters:
    ///   - projectPath: Path to Logic Pro project
    ///   - type: Type of commit (autoSave, emergency, manual)
    /// - Returns: CommitResult with success status and details
    @discardableResult
    public func performCommit(
        for projectPath: String,
        type: CommitType = .autoSave
    ) async -> CommitResult {
        let startTime = Date()
        let normalizedPath = (projectPath as NSString).standardizingPath

        // Check for lock before committing
        if LockManager.shared.isLocked(projectPath: normalizedPath) {
            if let lock = LockManager.shared.getLockInfo(projectPath: normalizedPath) {
                print("🔒 Project is locked by \(lock.lockedBy)")
                return CommitResult(
                    success: false,
                    commitId: nil,
                    message: "Project is locked by \(lock.lockedBy). Lock expires in \(lock.remainingHours) hours.",
                    duration: 0
                )
            }
        }

        // Prevent concurrent commits
        guard !isCommitting else {
            print("⚠️  Commit already in progress, skipping")
            return CommitResult(
                success: false,
                commitId: nil,
                message: "Commit already in progress",
                duration: 0
            )
        }

        isCommitting = true
        defer { isCommitting = false }

        // Check if project has changes
        let hasChanges = await checkForChanges(at: normalizedPath)

        guard hasChanges else {
            print("No changes detected in \(projectPath)")
            return CommitResult(
                success: true,
                commitId: nil,
                message: "No changes to commit",
                duration: Date().timeIntervalSince(startTime)
            )
        }

        // Generate commit message based on type
        let message = generateCommitMessage(for: type)

        print("\(commitTypeIcon(type)) Creating \(commitTypeName(type))...")
        print("   Project: \(projectPath)")
        print("   Message: \(message)")

        // Execute commit via Rust CLI
        let result = await executeCommit(
            projectPath: normalizedPath,
            message: message
        )

        let duration = Date().timeIntervalSince(startTime)

        if result.success {
            print("✓ Commit successful (\(String(format: "%.2f", duration))s)")
            if let commitId = result.commitId {
                print("  Commit ID: \(commitId)")
            }
        } else {
            print("✗ Commit failed: \(result.message)")
        }

        return CommitResult(
            success: result.success,
            commitId: result.commitId,
            message: result.message,
            duration: duration
        )
    }

    /// Perform emergency commits for all registered projects
    /// Used before system sleep or shutdown
    public func performEmergencyCommits() async {
        print("\n⚠️  Emergency Commit - Processing \(monitoredProjects.count) projects")

        var successCount = 0
        var failureCount = 0

        for projectPath in monitoredProjects {
            let result = await performCommit(for: projectPath, type: .emergency)
            if result.success {
                successCount += 1
            } else if result.commitId != nil {
                // No changes is still a success
                successCount += 1
            } else {
                failureCount += 1
            }
        }

        print("\nEmergency commit summary:")
        print("  ✓ Success: \(successCount)")
        if failureCount > 0 {
            print("  ✗ Failed: \(failureCount)")
        }
    }

    // MARK: - Private Helpers

    private func checkForChanges(at projectPath: String) async -> Bool {
        let process = Process()
        process.executableURL = URL(fileURLWithPath: cliPath)
        process.arguments = ["status", "--project", projectPath, "--porcelain"]

        let pipe = Pipe()
        process.standardOutput = pipe
        process.standardError = pipe

        do {
            try process.run()
            process.waitUntilExit()

            let data = pipe.fileHandleForReading.readDataToEndOfFile()
            let output = String(data: data, encoding: .utf8) ?? ""

            // If there's any output, there are changes
            return !output.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty

        } catch {
            print("⚠️  Failed to check status: \(error)")
            return false
        }
    }

    private func executeCommit(
        projectPath: String,
        message: String
    ) async -> (success: Bool, commitId: String?, message: String) {
        // First, stage all changes
        let stageResult = await runCliCommand(
            arguments: ["add", "--project", projectPath, "--all"]
        )

        guard stageResult.success else {
            return (false, nil, "Failed to stage changes: \(stageResult.output)")
        }

        // Then commit
        let commitResult = await runCliCommand(
            arguments: ["commit", "--project", projectPath, "--message", message]
        )

        if commitResult.success {
            // Extract commit ID from output
            let commitId = extractCommitId(from: commitResult.output)
            return (true, commitId, "Commit successful")
        } else {
            return (false, nil, commitResult.output)
        }
    }

    private func runCliCommand(
        arguments: [String]
    ) async -> (success: Bool, output: String) {
        let process = Process()
        process.executableURL = URL(fileURLWithPath: cliPath)
        process.arguments = arguments

        let pipe = Pipe()
        process.standardOutput = pipe
        process.standardError = pipe

        do {
            try process.run()
            process.waitUntilExit()

            let data = pipe.fileHandleForReading.readDataToEndOfFile()
            let output = String(data: data, encoding: .utf8) ?? ""

            return (process.terminationStatus == 0, output)

        } catch {
            return (false, "Process execution failed: \(error)")
        }
    }

    private func extractCommitId(from output: String) -> String? {
        // Look for patterns like "Created commit abc123..." or "Commit: abc123"
        let patterns = [
            #"[Cc]ommit:?\s+([a-f0-9]+)"#,
            #"[Cc]reated commit\s+([a-f0-9]+)"#,
            #"^([a-f0-9]{7,40})$"#  // Full line is commit hash
        ]

        for pattern in patterns {
            if let regex = try? NSRegularExpression(pattern: pattern),
               let match = regex.firstMatch(
                in: output,
                range: NSRange(output.startIndex..., in: output)
               ),
               match.numberOfRanges > 1 {
                let range = Range(match.range(at: 1), in: output)
                if let range = range {
                    return String(output[range])
                }
            }
        }

        return nil
    }

    private func generateCommitMessage(for type: CommitType) -> String {
        let timestamp = ISO8601DateFormatter().string(from: Date())

        switch type {
        case .autoSave:
            return "Auto-save at \(formatTimestamp(Date()))"

        case .emergency:
            return "Emergency commit (pre-power event) at \(formatTimestamp(Date()))"

        case .manual:
            return "Manual commit at \(formatTimestamp(Date()))"
        }
    }

    private func formatTimestamp(_ date: Date) -> String {
        let formatter = DateFormatter()
        formatter.dateStyle = .short
        formatter.timeStyle = .medium
        return formatter.string(from: date)
    }

    private func commitTypeIcon(_ type: CommitType) -> String {
        switch type {
        case .autoSave: return "💾"
        case .emergency: return "⚠️"
        case .manual: return "✏️"
        }
    }

    private func commitTypeName(_ type: CommitType) -> String {
        switch type {
        case .autoSave: return "auto-save"
        case .emergency: return "emergency commit"
        case .manual: return "manual commit"
        }
    }

    // MARK: - Draft Branch Management

    /// Ensure project is on the draft branch
    /// - Parameter projectPath: Path to Logic Pro project
    /// - Returns: true if on draft branch or successfully switched
    public func ensureOnDraftBranch(at projectPath: String) async -> Bool {
        let normalizedPath = (projectPath as NSString).standardizingPath

        // Check if draft branch exists
        let branchCheckResult = await runCliCommand(
            arguments: ["branch", "--project", normalizedPath, "--list"]
        )

        let hasDraftBranch = branchCheckResult.output.contains("draft")

        if !hasDraftBranch {
            // Create draft branch
            print("Creating draft branch for: \(projectPath)")
            let createResult = await runCliCommand(
                arguments: ["branch", "--project", normalizedPath, "draft"]
            )

            guard createResult.success else {
                print("⚠️  Failed to create draft branch")
                return false
            }
        }

        // Switch to draft branch
        let checkoutResult = await runCliCommand(
            arguments: ["checkout", "--project", normalizedPath, "draft"]
        )

        if checkoutResult.success {
            print("✓ On draft branch")
            return true
        } else {
            print("⚠️  Failed to switch to draft branch")
            return false
        }
    }
}
