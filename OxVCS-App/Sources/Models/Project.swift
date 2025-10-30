import Foundation

struct Project: Identifiable, Codable, Hashable {
    let id: UUID
    let path: String
    let name: String
    let isMonitored: Bool
    let lastCommit: Date?
    let commitCount: Int
    let isLocked: Bool
    let lockedBy: String?

    var displayName: String {
        name.replacingOccurrences(of: ".logicx", with: "")
    }

    var directoryURL: URL {
        URL(fileURLWithPath: path)
    }
}

struct CommitInfo: Identifiable, Codable {
    let id: String  // commit hash
    let message: String
    let timestamp: Date
    let author: String
    let metadata: CommitMetadata?

    var shortHash: String {
        String(id.prefix(7))
    }

    var formattedDate: String {
        let formatter = DateFormatter()
        formatter.dateStyle = .medium
        formatter.timeStyle = .short
        return formatter.string(from: timestamp)
    }
}

struct CommitMetadata: Codable {
    let bpm: Double?
    let sampleRate: Int?
    let keySignature: String?
    let timeSignature: String?
    let tags: [String]?
}

struct DaemonStatus: Codable {
    let isRunning: Bool
    let monitoredProjectCount: Int
    let lastActivity: Date?
}

struct ProjectLock: Codable {
    let projectPath: String
    let lockedBy: String
    let lockId: String
    let acquiredAt: Date
    let expiresAt: Date

    var isExpired: Bool {
        Date() > expiresAt
    }

    var remainingTime: TimeInterval {
        expiresAt.timeIntervalSince(Date())
    }
}
