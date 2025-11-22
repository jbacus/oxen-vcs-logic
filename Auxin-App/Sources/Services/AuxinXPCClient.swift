import Foundation

/// A protocol that defines the asynchronous interface for the XPC client.
/// This allows for a mock implementation to be used in tests.
protocol AuxinXPCClient {
    func getMonitoredProjects() async -> [String]
    func registerProject(path: String) async -> Bool
    func getCommitHistory(path: String, limit: Int) async -> [[String: Any]]
    func getLockInfo(for path: String) async -> [String: Any]?
    func ping() async -> Bool
    func commitProject(path: String, message: String?, metadata: [String: Any]?) async -> Bool
    func restoreProject(path: String, commitHash: String) async -> Bool
}

/// An extension to provide an async interface to the callback-based XPC client.
extension OxenDaemonXPCClient: AuxinXPCClient {
    func getMonitoredProjects() async -> [String] {
        return await withCheckedContinuation { continuation in
            getMonitoredProjects { projects in
                continuation.resume(returning: projects)
            }
        }
    }

    func registerProject(path: String) async -> Bool {
        return await withCheckedContinuation { continuation in
            registerProject(path: path) { success in
                continuation.resume(returning: success)
            }
        }
    }

    func getCommitHistory(path: String, limit: Int) async -> [[String: Any]] {
        return await withCheckedContinuation { continuation in
            getCommitHistory(path: path, limit: limit) { commits in
                continuation.resume(returning: commits)
            }
        }
    }

    func getLockInfo(for path: String) async -> [String: Any]? {
        return await withCheckedContinuation { continuation in
            getLockInfo(for: path) { lockInfo in
                continuation.resume(returning: lockInfo)
            }
        }
    }

    func ping() async -> Bool {
        return await withCheckedContinuation { continuation in
            ping { isRunning in
                continuation.resume(returning: isRunning)
            }
        }
    }

    func commitProject(path: String, message: String?, metadata: [String: Any]?) async -> Bool {
        return await withCheckedContinuation { continuation in
            commitProject(path: path, message: message, metadata: metadata) { success in
                continuation.resume(returning: success)
            }
        }
    }

    func restoreProject(path: String, commitHash: String) async -> Bool {
        return await withCheckedContinuation { continuation in
            restoreProject(path: path, commitHash: commitHash) { success in
                continuation.resume(returning: success)
            }
        }
    }
}
