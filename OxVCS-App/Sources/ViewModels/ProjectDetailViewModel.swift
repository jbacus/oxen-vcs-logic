import Foundation
import Combine

class ProjectDetailViewModel: ObservableObject {
    @Published var project: Project
    @Published var commits: [CommitInfo] = []
    @Published var isLoading = false
    @Published var errorMessage: String?

    private let xpcClient = OxenDaemonXPCClient.shared

    init(project: Project) {
        self.project = project
        loadCommitHistory()
    }

    // MARK: - Public Methods

    func loadCommitHistory(limit: Int = 100) {
        isLoading = true
        errorMessage = nil

        xpcClient.getCommitHistory(path: project.path, limit: limit) { [weak self] rawCommits in
            guard let self = self else { return }

            let commits = rawCommits.compactMap { dict -> CommitInfo? in
                guard let id = dict["hash"] as? String,
                      let message = dict["message"] as? String,
                      let timestamp = dict["timestamp"] as? Date,
                      let author = dict["author"] as? String else {
                    return nil
                }

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

            DispatchQueue.main.async {
                self.commits = commits
                self.isLoading = false
            }
        }
    }

    func restoreToCommit(_ commit: CommitInfo, completion: @escaping (Bool) -> Void) {
        xpcClient.restoreProject(path: project.path, commitHash: commit.id) { success in
            completion(success)
        }
    }

    func createMilestoneCommit(message: String, metadata: CommitMetadata?, completion: @escaping (Bool) -> Void) {
        var metaDict: [String: Any] = [:]
        if let metadata = metadata {
            if let bpm = metadata.bpm { metaDict["bpm"] = bpm }
            if let sampleRate = metadata.sampleRate { metaDict["sample_rate"] = sampleRate }
            if let keySignature = metadata.keySignature { metaDict["key_signature"] = keySignature }
            if let timeSignature = metadata.timeSignature { metaDict["time_signature"] = timeSignature }
            if let tags = metadata.tags { metaDict["tags"] = tags }
        }

        xpcClient.commitProject(path: project.path, message: message, metadata: metaDict) { [weak self] success in
            if success {
                self?.loadCommitHistory()
            }
            completion(success)
        }
    }
}
