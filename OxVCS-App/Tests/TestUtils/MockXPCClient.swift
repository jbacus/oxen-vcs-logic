import Foundation

// MARK: - Mock Types

/// Mock project for testing
public struct MockProject: Codable {
    public let path: String
    public let name: String
    public var status: ProjectStatus

    public init(path: String, name: String, status: ProjectStatus = .idle) {
        self.path = path
        self.name = name
        self.status = status
    }
}

/// Project status enum
public enum ProjectStatus: String, Codable {
    case idle
    case monitoring
    case committing
    case locked
}

/// Mock commit info for testing
public struct MockCommitInfo: Codable {
    public let hash: String
    public let message: String
    public let timestamp: Date
    public let author: String?
    public let metadata: MockCommitMetadata?

    public init(
        hash: String,
        message: String,
        timestamp: Date = Date(),
        author: String? = nil,
        metadata: MockCommitMetadata? = nil
    ) {
        self.hash = hash
        self.message = message
        self.timestamp = timestamp
        self.author = author
        self.metadata = metadata
    }
}

/// Mock commit metadata
public struct MockCommitMetadata: Codable {
    public let bpm: Int?
    public let sampleRate: Int?
    public let keySignature: String?
    public let tags: [String]

    public init(
        bpm: Int? = nil,
        sampleRate: Int? = nil,
        keySignature: String? = nil,
        tags: [String] = []
    ) {
        self.bpm = bpm
        self.sampleRate = sampleRate
        self.keySignature = keySignature
        self.tags = tags
    }
}

// MARK: - Mock XPC Client Protocol

/// Protocol for XPC daemon client (allows mocking)
public protocol OxenDaemonXPCClientProtocol {
    func getMonitoredProjects() async throws -> [MockProject]
    func addProject(_ path: String) async throws
    func removeProject(_ path: String) async throws
    func createMilestoneCommit(
        at path: String,
        message: String,
        metadata: MockCommitMetadata?
    ) async throws
    func getCommitHistory(for path: String, limit: Int?) async throws -> [MockCommitInfo]
    func rollback(project path: String, to commitHash: String) async throws
    func acquireLock(for path: String) async throws -> Bool
    func releaseLock(for path: String) async throws
    func getLockStatus(for path: String) async throws -> LockStatus?
    func pauseMonitoring() async throws
    func resumeMonitoring() async throws
    func getDaemonStatus() async throws -> DaemonStatus
}

/// Lock status information
public struct LockStatus: Codable {
    public let isLocked: Bool
    public let owner: String?
    public let acquiredAt: Date?

    public init(isLocked: Bool, owner: String? = nil, acquiredAt: Date? = nil) {
        self.isLocked = isLocked
        self.owner = owner
        self.acquiredAt = acquiredAt
    }
}

/// Daemon status information
public struct DaemonStatus: Codable {
    public var isRunning: Bool
    public var monitoredProjectsCount: Int
    public var isMonitoring: Bool

    public init(isRunning: Bool, monitoredProjectsCount: Int, isMonitoring: Bool) {
        self.isRunning = isRunning
        self.monitoredProjectsCount = monitoredProjectsCount
        self.isMonitoring = isMonitoring
    }
}

// MARK: - Mock Implementation

/// Mock XPC client for testing UI components without a real daemon
public class MockOxenDaemonXPCClient: OxenDaemonXPCClientProtocol {
    // MARK: - Mock Data

    public var mockProjects: [MockProject] = []
    public var mockCommits: [String: [MockCommitInfo]] = [:] // projectPath -> commits
    public var mockLocks: [String: LockStatus] = [:] // projectPath -> lockStatus
    public var mockDaemonStatus = DaemonStatus(
        isRunning: true,
        monitoredProjectsCount: 0,
        isMonitoring: true
    )

    // MARK: - Call Tracking

    public var addProjectCalled = false
    public var removeProjectCalled = false
    public var createCommitCalled = false
    public var rollbackCalled = false
    public var acquireLockCalled = false
    public var releaseLockCalled = false
    public var pauseMonitoringCalled = false
    public var resumeMonitoringCalled = false

    public var lastAddedPath: String?
    public var lastRemovedPath: String?
    public var lastCommitMessage: String?
    public var lastRollbackHash: String?

    // MARK: - Error Simulation

    public var shouldFail = false
    public var failureError: Error = NSError(
        domain: "MockXPCClient",
        code: -1,
        userInfo: [NSLocalizedDescriptionKey: "Mock failure"]
    )

    // MARK: - Initialization

    public init() {}

    // MARK: - Protocol Implementation

    public func getMonitoredProjects() async throws -> [MockProject] {
        if shouldFail {
            throw failureError
        }
        return mockProjects
    }

    public func addProject(_ path: String) async throws {
        if shouldFail {
            throw failureError
        }

        addProjectCalled = true
        lastAddedPath = path

        // Add to mock projects if not already present
        if !mockProjects.contains(where: { $0.path == path }) {
            let name = URL(fileURLWithPath: path).lastPathComponent
            mockProjects.append(MockProject(path: path, name: name, status: .monitoring))
            mockDaemonStatus.monitoredProjectsCount = mockProjects.count
        }
    }

    public func removeProject(_ path: String) async throws {
        if shouldFail {
            throw failureError
        }

        removeProjectCalled = true
        lastRemovedPath = path

        mockProjects.removeAll { $0.path == path }
        mockDaemonStatus.monitoredProjectsCount = mockProjects.count
    }

    public func createMilestoneCommit(
        at path: String,
        message: String,
        metadata: MockCommitMetadata?
    ) async throws {
        if shouldFail {
            throw failureError
        }

        createCommitCalled = true
        lastCommitMessage = message

        let commit = MockCommitInfo(
            hash: UUID().uuidString,
            message: message,
            timestamp: Date(),
            author: "Test User",
            metadata: metadata
        )

        if mockCommits[path] == nil {
            mockCommits[path] = []
        }
        mockCommits[path]?.insert(commit, at: 0) // Most recent first
    }

    public func getCommitHistory(for path: String, limit: Int? = nil) async throws -> [MockCommitInfo] {
        if shouldFail {
            throw failureError
        }

        let commits = mockCommits[path] ?? []

        if let limit = limit {
            return Array(commits.prefix(limit))
        }

        return commits
    }

    public func rollback(project path: String, to commitHash: String) async throws {
        if shouldFail {
            throw failureError
        }

        rollbackCalled = true
        lastRollbackHash = commitHash

        // In a real implementation, this would restore files
        // For mock, just verify the commit exists
        guard let commits = mockCommits[path],
              commits.contains(where: { $0.hash == commitHash }) else {
            throw NSError(
                domain: "MockXPCClient",
                code: -2,
                userInfo: [NSLocalizedDescriptionKey: "Commit not found"]
            )
        }
    }

    public func acquireLock(for path: String) async throws -> Bool {
        if shouldFail {
            throw failureError
        }

        acquireLockCalled = true

        // Check if already locked
        if let lockStatus = mockLocks[path], lockStatus.isLocked {
            return false
        }

        // Acquire lock
        mockLocks[path] = LockStatus(
            isLocked: true,
            owner: "test-user",
            acquiredAt: Date()
        )

        // Update project status
        if let index = mockProjects.firstIndex(where: { $0.path == path }) {
            mockProjects[index].status = .locked
        }

        return true
    }

    public func releaseLock(for path: String) async throws {
        if shouldFail {
            throw failureError
        }

        releaseLockCalled = true

        mockLocks[path] = LockStatus(isLocked: false)

        // Update project status
        if let index = mockProjects.firstIndex(where: { $0.path == path }) {
            mockProjects[index].status = .monitoring
        }
    }

    public func getLockStatus(for path: String) async throws -> LockStatus? {
        if shouldFail {
            throw failureError
        }

        return mockLocks[path]
    }

    public func pauseMonitoring() async throws {
        if shouldFail {
            throw failureError
        }

        pauseMonitoringCalled = true
        mockDaemonStatus.isMonitoring = false

        // Update all project statuses
        for i in 0..<mockProjects.count {
            mockProjects[i].status = .idle
        }
    }

    public func resumeMonitoring() async throws {
        if shouldFail {
            throw failureError
        }

        resumeMonitoringCalled = true
        mockDaemonStatus.isMonitoring = true

        // Update all project statuses
        for i in 0..<mockProjects.count {
            mockProjects[i].status = .monitoring
        }
    }

    public func getDaemonStatus() async throws -> DaemonStatus {
        if shouldFail {
            throw failureError
        }

        return mockDaemonStatus
    }

    // MARK: - Test Helpers

    /// Reset all mock data and call tracking
    public func reset() {
        mockProjects = []
        mockCommits = [:]
        mockLocks = [:]
        mockDaemonStatus = DaemonStatus(
            isRunning: true,
            monitoredProjectsCount: 0,
            isMonitoring: true
        )

        addProjectCalled = false
        removeProjectCalled = false
        createCommitCalled = false
        rollbackCalled = false
        acquireLockCalled = false
        releaseLockCalled = false
        pauseMonitoringCalled = false
        resumeMonitoringCalled = false

        lastAddedPath = nil
        lastRemovedPath = nil
        lastCommitMessage = nil
        lastRollbackHash = nil

        shouldFail = false
    }

    /// Set up mock data for a typical testing scenario
    public func setupTypicalScenario() {
        let project1 = MockProject(
            path: "/Users/test/Project1.logicx",
            name: "Project1",
            status: .monitoring
        )
        let project2 = MockProject(
            path: "/Users/test/Project2.logicx",
            name: "Project2",
            status: .idle
        )

        mockProjects = [project1, project2]

        // Add some commits for project1
        mockCommits[project1.path] = [
            MockCommitInfo(
                hash: "abc123",
                message: "Initial commit",
                timestamp: Date().addingTimeInterval(-3600),
                metadata: MockCommitMetadata(bpm: 120, sampleRate: 48000, keySignature: "C")
            ),
            MockCommitInfo(
                hash: "def456",
                message: "Added drums",
                timestamp: Date().addingTimeInterval(-1800),
                metadata: MockCommitMetadata(bpm: 128, sampleRate: 48000)
            ),
            MockCommitInfo(
                hash: "ghi789",
                message: "Final mix",
                timestamp: Date(),
                metadata: MockCommitMetadata(bpm: 128, sampleRate: 48000, tags: ["mix", "final"])
            ),
        ]

        mockDaemonStatus.monitoredProjectsCount = mockProjects.count
    }
}
