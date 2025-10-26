import Foundation

// MARK: - XPC Protocol (must match server definition)

/// Protocol for communication between UI app and background daemon
@objc protocol OxenDaemonXPCProtocol {
    func registerProject(_ projectPath: String, withReply reply: @escaping (Bool, String?) -> Void)
    func unregisterProject(_ projectPath: String, withReply reply: @escaping (Bool, String?) -> Void)
    func getMonitoredProjects(withReply reply: @escaping ([String]) -> Void)
    func commitProject(_ projectPath: String, message: String?, withReply reply: @escaping (String?, String?) -> Void)
    func getStatus(withReply reply: @escaping ([String: Any]) -> Void)
    func getCommitHistory(for projectPath: String, limit: Int, withReply reply: @escaping ([[String: Any]]) -> Void)
    func restoreProject(_ projectPath: String, toCommit commitId: String, withReply reply: @escaping (Bool, String?) -> Void)
    func pauseMonitoring(for projectPath: String, withReply reply: @escaping (Bool) -> Void)
    func resumeMonitoring(for projectPath: String, withReply reply: @escaping (Bool) -> Void)
    func ping(withReply reply: @escaping (Bool) -> Void)
    func acquireLock(for projectPath: String, timeoutHours: Int, withReply reply: @escaping (Bool, String?) -> Void)
    func releaseLock(for projectPath: String, withReply reply: @escaping (Bool, String?) -> Void)
    func forceBreakLock(for projectPath: String, withReply reply: @escaping (Bool, String?) -> Void)
    func getLockInfo(for projectPath: String, withReply reply: @escaping ([String: Any]?) -> Void)
}

// MARK: - XPC Client

/// XPC client for communicating with OxVCS LaunchAgent daemon
class OxenDaemonXPCClient {
    static let shared = OxenDaemonXPCClient()

    private let connection: NSXPCConnection
    private let serviceName = "com.oxen.logic.daemon.xpc"

    init() {
        connection = NSXPCConnection(machServiceName: serviceName, options: [])
        connection.remoteObjectInterface = NSXPCInterface(with: OxenDaemonXPCProtocol.self)
        connection.resume()
    }

    deinit {
        connection.invalidate()
    }

    private func getProxy() -> OxenDaemonXPCProtocol? {
        return connection.remoteObjectProxyWithErrorHandler { error in
            print("XPC Error: \(error)")
        } as? OxenDaemonXPCProtocol
    }

    // MARK: - Public API

    func ping(completion: @escaping (Bool) -> Void) {
        guard let proxy = getProxy() else {
            completion(false)
            return
        }
        proxy.ping(withReply: { success in
            completion(success)
        })
    }

    func registerProject(path: String, completion: @escaping (Bool) -> Void) {
        guard let proxy = getProxy() else {
            completion(false)
            return
        }
        proxy.registerProject(path, withReply: { success, error in
            if let error = error {
                print("XPC registerProject error: \(error)")
            }
            completion(success)
        })
    }

    func getMonitoredProjects(completion: @escaping ([String]) -> Void) {
        guard let proxy = getProxy() else {
            completion([])
            return
        }
        proxy.getMonitoredProjects(withReply: { projects in
            completion(projects)
        })
    }

    func commitProject(path: String, message: String?, metadata: [String: Any]?, completion: @escaping (Bool) -> Void) {
        guard let proxy = getProxy() else {
            completion(false)
            return
        }
        proxy.commitProject(path, message: message, withReply: { commitId, error in
            if let error = error {
                print("XPC commitProject error: \(error)")
                completion(false)
            } else {
                completion(true)
            }
        })
    }

    func getCommitHistory(path: String, limit: Int, completion: @escaping ([[String: Any]]) -> Void) {
        guard let proxy = getProxy() else {
            completion([])
            return
        }
        proxy.getCommitHistory(for: path, limit: limit, withReply: { commits in
            completion(commits)
        })
    }

    func restoreProject(path: String, commitHash: String, completion: @escaping (Bool) -> Void) {
        guard let proxy = getProxy() else {
            completion(false)
            return
        }
        proxy.restoreProject(path, toCommit: commitHash, withReply: { success, error in
            if let error = error {
                print("XPC restoreProject error: \(error)")
            }
            completion(success)
        })
    }

    func getStatus(completion: @escaping ([String: Any]) -> Void) {
        guard let proxy = getProxy() else {
            completion([:])
            return
        }
        proxy.getStatus(withReply: { status in
            completion(status)
        })
    }

    func pauseMonitoring(for path: String, completion: @escaping (Bool) -> Void) {
        guard let proxy = getProxy() else {
            completion(false)
            return
        }
        proxy.pauseMonitoring(for: path, withReply: { success in
            completion(success)
        })
    }

    func resumeMonitoring(for path: String, completion: @escaping (Bool) -> Void) {
        guard let proxy = getProxy() else {
            completion(false)
            return
        }
        proxy.resumeMonitoring(for: path, withReply: { success in
            completion(success)
        })
    }
}
