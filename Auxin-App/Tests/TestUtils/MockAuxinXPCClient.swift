import Foundation
@testable import Auxin_App

class MockAuxinXPCClient: AuxinXPCClient {
    var monitoredProjects: [String] = []
    var registeredProjects: [String] = []
    var commitHistory: [[String: Any]] = []
    var lockInfo: [String: Any]? = nil
    var pingResult: Bool = true
    var commitResult: Bool = true
    var restoreResult: Bool = true

    var lastCommitPath: String?
    var lastCommitMessage: String?
    var lastCommitMetadata: [String: Any]?
    var lastRestorePath: String?
    var lastRestoreCommitHash: String?

    var shouldThrowError = false

    func getMonitoredProjects() async -> [String] {
        if shouldThrowError {
            return []
        }
        return monitoredProjects
    }

    func registerProject(path: String) async -> Bool {
        if shouldThrowError {
            return false
        }
        registeredProjects.append(path)
        return true
    }

    func getCommitHistory(path: String, limit: Int) async -> [[String: Any]] {
        if shouldThrowError {
            return []
        }
        return commitHistory
    }

    func getLockInfo(for path: String) async -> [String: Any]? {
        if shouldThrowError {
            return nil
        }
        return lockInfo
    }

    func ping() async -> Bool {
        if shouldThrowError {
            return false
        }
        return pingResult
    }

    func commitProject(path: String, message: String?, metadata: [String: Any]?) async -> Bool {
        if shouldThrowError {
            return false
        }
        lastCommitPath = path
        lastCommitMessage = message
        lastCommitMetadata = metadata
        return commitResult
    }

    func restoreProject(path: String, commitHash: String) async -> Bool {
        if shouldThrowError {
            return false
        }
        lastRestorePath = path
        lastRestoreCommitHash = commitHash
        return restoreResult
    }
}
