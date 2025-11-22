import Foundation
import Combine

@MainActor
class ProjectListViewModel: ObservableObject {
    @Published var projects: [Project] = []
    @Published var isLoading = false
    @Published var errorMessage: String?
    @Published var daemonStatus: DaemonStatus?

    private var xpcClient: AuxinXPCClient
    private var refreshTimer: Timer?

    init(xpcClient: AuxinXPCClient = OxenDaemonXPCClient.shared) {
        self.xpcClient = xpcClient
        startAutoRefresh()
        
        Task {
            await loadProjects()
            await checkDaemonStatus()
        }
    }

    deinit {
        refreshTimer?.invalidate()
    }

    // MARK: - Public Methods

    func loadProjects() async {
        isLoading = true
        errorMessage = nil

        let projectPaths = await xpcClient.getMonitoredProjects()
        
        var loadedProjects = await withTaskGroup(of: Project?.self, returning: [Project].self) { group in
            for path in projectPaths {
                group.addTask {
                    return await self.loadProjectDetails(path: path)
                }
            }
            
            var projects: [Project] = []
            for await project in group {
                if let project = project {
                    projects.append(project)
                }
            }
            return projects
        }
        
        loadedProjects.sort { $0.name < $1.name }
        self.projects = loadedProjects
        self.isLoading = false
    }

    func addProject(path: String) async -> Bool {
        let success = await xpcClient.registerProject(path: path)
        if success {
            await loadProjects()
        }
        return success
    }

    func checkDaemonStatus() async {
        let isRunning = await xpcClient.ping()
        self.daemonStatus = DaemonStatus(
            isRunning: isRunning,
            monitoredProjectCount: self.projects.count,
            lastActivity: Date()
        )
    }

    // MARK: - Private Methods

    private func loadProjectDetails(path: String) async -> Project? {
        async let historyResult = xpcClient.getCommitHistory(path: path, limit: 1)
        async let lockInfoResult = xpcClient.getLockInfo(for: path)
        
        let (commits, lockInfo) = await (historyResult, lockInfoResult)

        let lastCommit = commits.first?["timestamp"] as? Date
        let commitCount = commits.first?["count"] as? Int ?? 0
        
        let isLocked = lockInfo?["isLocked"] as? Bool ?? false
        let lockedBy = lockInfo?["lockedBy"] as? String

        return Project(
            path: path,
            name: URL(fileURLWithPath: path).lastPathComponent,
            isMonitored: true,
            lastCommit: lastCommit,
            commitCount: commitCount,
            isLocked: isLocked,
            lockedBy: lockedBy
        )
    }

    private func startAutoRefresh() {
        refreshTimer = Timer.scheduledTimer(withTimeInterval: 30, repeats: true) { [weak self] _ in
            Task { [weak self] in
                await self?.loadProjects()
                await self?.checkDaemonStatus()
            }
        }
    }
}
