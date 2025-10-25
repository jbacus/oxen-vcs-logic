import Foundation
import Combine

class ProjectListViewModel: ObservableObject {
    @Published var projects: [Project] = []
    @Published var isLoading = false
    @Published var errorMessage: String?
    @Published var daemonStatus: DaemonStatus?

    private let xpcClient = OxenDaemonXPCClient.shared
    private var refreshTimer: Timer?

    init() {
        startAutoRefresh()
        loadProjects()
        checkDaemonStatus()
    }

    deinit {
        refreshTimer?.invalidate()
    }

    // MARK: - Public Methods

    func loadProjects() {
        isLoading = true
        errorMessage = nil

        xpcClient.getMonitoredProjects { [weak self] projectPaths in
            guard let self = self else { return }

            // Convert paths to Project objects with additional metadata
            var loadedProjects: [Project] = []

            let group = DispatchGroup()

            for path in projectPaths {
                group.enter()
                self.loadProjectDetails(path: path) { project in
                    if let project = project {
                        loadedProjects.append(project)
                    }
                    group.leave()
                }
            }

            group.notify(queue: .main) {
                self.projects = loadedProjects.sorted { $0.name < $1.name }
                self.isLoading = false
            }
        }
    }

    func addProject(path: String, completion: @escaping (Bool) -> Void) {
        xpcClient.registerProject(path: path) { [weak self] success in
            if success {
                self?.loadProjects()
            }
            completion(success)
        }
    }

    func checkDaemonStatus() {
        xpcClient.ping { [weak self] isRunning in
            guard let self = self else { return }

            DispatchQueue.main.async {
                self.daemonStatus = DaemonStatus(
                    isRunning: isRunning,
                    monitoredProjectCount: self.projects.count,
                    lastActivity: Date()
                )
            }
        }
    }

    // MARK: - Private Methods

    private func loadProjectDetails(path: String, completion: @escaping (Project?) -> Void) {
        xpcClient.getStatus(path: path) { status in
            guard let status = status else {
                completion(nil)
                return
            }

            // Get commit history to determine last commit
            self.xpcClient.getCommitHistory(path: path, limit: 1) { commits in
                let lastCommit = commits.first?["timestamp"] as? Date
                let commitCount = status["commit_count"] as? Int ?? 0

                let project = Project(
                    id: UUID(),
                    path: path,
                    name: URL(fileURLWithPath: path).lastPathComponent,
                    isMonitored: true,
                    lastCommit: lastCommit,
                    commitCount: commitCount,
                    isLocked: status["is_locked"] as? Bool ?? false,
                    lockedBy: status["locked_by"] as? String
                )

                completion(project)
            }
        }
    }

    private func startAutoRefresh() {
        refreshTimer = Timer.scheduledTimer(withTimeInterval: 30, repeats: true) { [weak self] _ in
            self?.loadProjects()
            self?.checkDaemonStatus()
        }
    }
}
