import Foundation

/// XPC client for communicating with OxVCS LaunchAgent daemon
class OxenDaemonXPCClient {
    static let shared = OxenDaemonXPCClient()

    private let connection: NSXPCConnection
    private let serviceName = "com.oxenvcs.daemon"

    init() {
        connection = NSXPCConnection(machServiceName: serviceName, options: [])
        connection.remoteObjectInterface = NSXPCInterface(with: OxenDaemonProtocol.self)
        connection.resume()
    }

    deinit {
        connection.invalidate()
    }

    private func getProxy() -> OxenDaemonProtocol? {
        return connection.remoteObjectProxyWithErrorHandler { error in
            print("XPC Error: \(error)")
        } as? OxenDaemonProtocol
    }

    // MARK: - Public API

    func ping(completion: @escaping (Bool) -> Void) {
        guard let proxy = getProxy() else {
            completion(false)
            return
        }
        proxy.ping { success in
            completion(success)
        }
    }

    func registerProject(path: String, completion: @escaping (Bool) -> Void) {
        guard let proxy = getProxy() else {
            completion(false)
            return
        }
        proxy.registerProject(path: path) { success in
            completion(success)
        }
    }

    func getMonitoredProjects(completion: @escaping ([String]) -> Void) {
        guard let proxy = getProxy() else {
            completion([])
            return
        }
        proxy.getMonitoredProjects { projects in
            completion(projects)
        }
    }

    func commitProject(path: String, message: String?, metadata: [String: Any]?, completion: @escaping (Bool) -> Void) {
        guard let proxy = getProxy() else {
            completion(false)
            return
        }
        proxy.commitProject(path: path, message: message, metadata: metadata) { success in
            completion(success)
        }
    }

    func getCommitHistory(path: String, limit: Int, completion: @escaping ([[String: Any]]) -> Void) {
        guard let proxy = getProxy() else {
            completion([])
            return
        }
        proxy.getCommitHistory(path: path, limit: limit) { commits in
            completion(commits)
        }
    }

    func restoreProject(path: String, commitHash: String, completion: @escaping (Bool) -> Void) {
        guard let proxy = getProxy() else {
            completion(false)
            return
        }
        proxy.restoreProject(path: path, commitHash: commitHash) { success in
            completion(success)
        }
    }

    func getStatus(path: String, completion: @escaping ([String: Any]?) -> Void) {
        guard let proxy = getProxy() else {
            completion(nil)
            return
        }
        proxy.getStatus(path: path) { status in
            completion(status)
        }
    }

    func pauseMonitoring(completion: @escaping (Bool) -> Void) {
        guard let proxy = getProxy() else {
            completion(false)
            return
        }
        proxy.pauseMonitoring { success in
            completion(success)
        }
    }

    func resumeMonitoring(completion: @escaping (Bool) -> Void) {
        guard let proxy = getProxy() else {
            completion(false)
            return
        }
        proxy.resumeMonitoring { success in
            completion(success)
        }
    }
}

// MARK: - XPC Protocol

@objc protocol OxenDaemonProtocol {
    func ping(reply: @escaping (Bool) -> Void)
    func registerProject(path: String, reply: @escaping (Bool) -> Void)
    func getMonitoredProjects(reply: @escaping ([String]) -> Void)
    func commitProject(path: String, message: String?, metadata: [String: Any]?, reply: @escaping (Bool) -> Void)
    func getCommitHistory(path: String, limit: Int, reply: @escaping ([[String: Any]]) -> Void)
    func restoreProject(path: String, commitHash: String, reply: @escaping (Bool) -> Void)
    func getStatus(path: String, reply: @escaping ([String: Any]?) -> Void)
    func pauseMonitoring(reply: @escaping (Bool) -> Void)
    func resumeMonitoring(reply: @escaping (Bool) -> Void)
}
