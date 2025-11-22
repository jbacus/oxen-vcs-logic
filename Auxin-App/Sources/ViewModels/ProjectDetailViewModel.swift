import Foundation
import Combine

class ProjectDetailViewModel: ObservableObject {
    @Published var project: Project
    @Published var commits: [CommitInfo] = []
    @Published var isLoading = false
    @Published var errorMessage: String?

    private let xpcClient: AuxinXPCClient

    init(project: Project, xpcClient: AuxinXPCClient = OxenDaemonXPCClient.shared) {
        self.project = project
        self.xpcClient = xpcClient
    }

    // MARK: - Public Methods

    @MainActor
    func loadCommitHistory(limit: Int = 100) async {
        isLoading = true
        errorMessage = nil

        let rawCommits = await xpcClient.getCommitHistory(path: project.path, limit: limit)

        let commits = rawCommits.compactMap { dict -> CommitInfo? in
            // Try both "id" and "hash" keys for backwards compatibility
            guard let id = (dict["id"] as? String) ?? (dict["hash"] as? String),
                  let message = dict["message"] as? String else {
                print("ProjectDetailVM: Skipping commit - missing id or message: \(dict)")
                return nil
            }

            // Timestamp and author are optional - use defaults if not present
            let timestamp = dict["timestamp"] as? Date ?? Date()
            let author = dict["author"] as? String ?? "Unknown"

            let metadata: CommitMetadata?
            if let metaDict = dict["metadata"] as? [String: Any] {
                metadata = CommitMetadata(
                    bpm: metaDict["bpm"] as? Double,
                    sampleRate: metaDict["sample_rate"] as? Int,
                    keySignature: metaDict["key_signature"] as? String,
                    timeSignature: metaDict["time_signature"] as? String,
                    tags: metaDict["tags"] as? [String]
                )
            } else {
                metadata = nil
            }

            return CommitInfo(
                id: id,
                message: message,
                timestamp: timestamp,
                author: author,
                metadata: metadata
            )
        }

        self.commits = commits
        self.isLoading = false
    }

    @MainActor
    func restoreToCommit(_ commit: CommitInfo) async -> Bool {
        let success = await xpcClient.restoreProject(path: project.path, commitHash: commit.id)
        return success
    }

    // Callback-based version for backwards compatibility with existing views
    func restoreToCommit(_ commit: CommitInfo, completion: @escaping (Bool) -> Void) {
        Task { @MainActor in
            let success = await restoreToCommit(commit)
            completion(success)
        }
    }

    @MainActor
    func createMilestoneCommit(message: String, metadata: CommitMetadata?) async -> Bool {
        var metaDict: [String: Any] = [:]
        if let metadata = metadata {
            if let bpm = metadata.bpm { metaDict["bpm"] = bpm }
            if let sampleRate = metadata.sampleRate { metaDict["sample_rate"] = sampleRate }
            if let keySignature = metadata.keySignature { metaDict["key_signature"] = keySignature }
            if let timeSignature = metadata.timeSignature { metaDict["time_signature"] = timeSignature }
            if let tags = metadata.tags { metaDict["tags"] = tags }
        }

        let success = await xpcClient.commitProject(path: project.path, message: message, metadata: metaDict)
        if success {
            await loadCommitHistory()
        }
        return success
    }

    // Callback-based version for backwards compatibility with existing views
    func createMilestoneCommit(message: String, metadata: CommitMetadata?, completion: @escaping (Bool) -> Void) {
        Task { @MainActor in
            let success = await createMilestoneCommit(message: message, metadata: metadata)
            completion(success)
        }
    }
}
