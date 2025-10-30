import Foundation

/// Manages exclusive file locks for Logic Pro projects
/// Lock files are stored in .oxen/locks.json
class LockManager {
    static let shared = LockManager()

    private let fileManager = FileManager.default
    private let lockFileName = "locks.json"
    private let lockQueue = DispatchQueue(label: "com.oxenvcs.lockmanager", attributes: .concurrent)

    private init() {}

    // MARK: - Public API

    /// Attempts to acquire a lock for a project
    /// Returns true if lock acquired, false if already locked by someone else
    func acquireLock(projectPath: String, timeoutHours: Int = 24) -> Bool {
        return lockQueue.sync(flags: .barrier) {
            // Check if project is already locked
            if let existingLock = readLock(projectPath: projectPath) {
                if existingLock.isExpired {
                    // Lock has expired, remove it and acquire new lock
                    NSLog("[LockManager] Lock expired for \(projectPath), removing stale lock")
                    _ = releaseLock(projectPath: projectPath)
                } else {
                    NSLog("[LockManager] Project already locked by \(existingLock.lockedBy)")
                    return false
                }
            }

            // Create new lock
            let lock = ProjectLock(
                projectPath: projectPath,
                lockedBy: getCurrentUserIdentifier(),
                lockId: UUID().uuidString,
                acquiredAt: Date(),
                expiresAt: Date().addingTimeInterval(TimeInterval(timeoutHours * 3600))
            )

            return writeLock(lock, projectPath: projectPath)
        }
    }

    /// Releases a lock for a project
    /// Returns true if lock was released, false if lock doesn't exist or belongs to someone else
    func releaseLock(projectPath: String) -> Bool {
        guard let existingLock = readLock(projectPath: projectPath) else {
            return false
        }

        // Verify lock belongs to current user (or is expired)
        if existingLock.lockedBy != getCurrentUserIdentifier() && !existingLock.isExpired {
            NSLog("[LockManager] Cannot release lock owned by \(existingLock.lockedBy)")
            return false
        }

        let lockFilePath = getLockFilePath(projectPath: projectPath)
        do {
            try fileManager.removeItem(atPath: lockFilePath)
            NSLog("[LockManager] Lock released for \(projectPath)")
            return true
        } catch {
            NSLog("[LockManager] Failed to remove lock file: \(error)")
            return false
        }
    }

    /// Force-releases a lock (admin operation)
    func forceBreakLock(projectPath: String) -> Bool {
        let lockFilePath = getLockFilePath(projectPath: projectPath)

        guard fileManager.fileExists(atPath: lockFilePath) else {
            return false
        }

        do {
            try fileManager.removeItem(atPath: lockFilePath)
            NSLog("[LockManager] Lock force-broken for \(projectPath)")
            return true
        } catch {
            NSLog("[LockManager] Failed to force-break lock: \(error)")
            return false
        }
    }

    /// Checks if a project is currently locked
    func isLocked(projectPath: String) -> Bool {
        if let lock = readLock(projectPath: projectPath) {
            return !lock.isExpired
        }
        return false
    }

    /// Gets information about the current lock
    func getLockInfo(projectPath: String) -> ProjectLock? {
        return readLock(projectPath: projectPath)
    }

    /// Cleans up expired locks across all projects
    func cleanupExpiredLocks(baseDir: String) {
        // Find all .oxen directories
        let enumerator = fileManager.enumerator(atPath: baseDir)
        while let path = enumerator?.nextObject() as? String {
            if path.hasSuffix(".oxen/\(lockFileName)") {
                let fullPath = (baseDir as NSString).appendingPathComponent(path)
                if let lock = readLockFromFile(fullPath), lock.isExpired {
                    try? fileManager.removeItem(atPath: fullPath)
                    NSLog("[LockManager] Cleaned up expired lock at \(fullPath)")
                }
            }
        }
    }

    // MARK: - Private Methods

    private func getLockFilePath(projectPath: String) -> String {
        let oxenDir = (projectPath as NSString).appendingPathComponent(".oxen")
        return (oxenDir as NSString).appendingPathComponent(lockFileName)
    }

    private func readLock(projectPath: String) -> ProjectLock? {
        let lockFilePath = getLockFilePath(projectPath: projectPath)
        return readLockFromFile(lockFilePath)
    }

    private func readLockFromFile(_ path: String) -> ProjectLock? {
        guard fileManager.fileExists(atPath: path),
              let data = try? Data(contentsOf: URL(fileURLWithPath: path)) else {
            return nil
        }

        let decoder = JSONDecoder()
        decoder.dateDecodingStrategy = .iso8601

        guard let lock = try? decoder.decode(ProjectLock.self, from: data) else {
            return nil
        }
        return lock
    }

    private func writeLock(_ lock: ProjectLock, projectPath: String) -> Bool {
        let lockFilePath = getLockFilePath(projectPath: projectPath)
        let oxenDir = (lockFilePath as NSString).deletingLastPathComponent

        // Ensure .oxen directory exists
        if !fileManager.fileExists(atPath: oxenDir) {
            do {
                try fileManager.createDirectory(atPath: oxenDir, withIntermediateDirectories: true)
            } catch {
                NSLog("[LockManager] Failed to create .oxen directory: \(error)")
                return false
            }
        }

        // Write lock file
        do {
            let encoder = JSONEncoder()
            encoder.dateEncodingStrategy = .iso8601
            encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
            let data = try encoder.encode(lock)
            try data.write(to: URL(fileURLWithPath: lockFilePath))
            NSLog("[LockManager] Lock acquired for \(projectPath) by \(lock.lockedBy)")
            return true
        } catch {
            NSLog("[LockManager] Failed to write lock file: \(error)")
            return false
        }
    }

    private func getCurrentUserIdentifier() -> String {
        let username = ProcessInfo.processInfo.userName
        let hostname = ProcessInfo.processInfo.hostName
        return "\(username)@\(hostname)"
    }
}

// MARK: - ProjectLock Model

struct ProjectLock: Codable {
    let projectPath: String
    let lockedBy: String
    let lockId: String
    let acquiredAt: Date
    let expiresAt: Date

    var isExpired: Bool {
        Date() > expiresAt
    }

    var remainingTime: TimeInterval {
        expiresAt.timeIntervalSince(Date())
    }

    var remainingHours: Int {
        Int(remainingTime / 3600)
    }
}
