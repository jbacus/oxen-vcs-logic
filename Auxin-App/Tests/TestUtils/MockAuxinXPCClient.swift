import Foundation
@testable import Auxin_App

class MockAuxinXPCClient: AuxinXPCClient {
    var monitoredProjects: [String] = []
    var registeredProjects: [String] = []
    var commitHistory: [[String: Any]] = []
    var lockInfo: [String: Any]? = nil
    var pingResult: Bool = true

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
}
