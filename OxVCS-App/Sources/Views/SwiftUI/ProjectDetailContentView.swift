import SwiftUI

struct ProjectDetailContentView: View {
    let project: Project
    @State private var commits: [CommitInfo] = []
    @State private var isLoading = false
    @State private var showingMilestoneCommit = false

    var body: some View {
        VStack(spacing: 0) {
            // Header
            ProjectHeaderView(project: project)
                .padding()
                .background(Color(nsColor: .controlBackgroundColor))

            Divider()

            // Commit history
            if isLoading {
                Spacer()
                ProgressView("Loading commits...")
                Spacer()
            } else if commits.isEmpty {
                Spacer()
                Text("No commits yet")
                    .foregroundColor(.secondary)
                Spacer()
            } else {
                List(commits, id: \.id) { commit in
                    CommitRowView(commit: commit, project: project)
                }
                .listStyle(.inset)
            }
        }
        .navigationTitle(project.displayName)
        .navigationSubtitle(project.path)
        .toolbar {
            ToolbarItemGroup(placement: .primaryAction) {
                Button {
                    openInLogic()
                } label: {
                    Label("Open in Logic Pro", systemImage: "music.note")
                }
                .buttonStyle(.bordered)

                Button("Create Milestone") {
                    showingMilestoneCommit = true
                }
                .buttonStyle(.borderedProminent)
            }
        }
        .sheet(isPresented: $showingMilestoneCommit) {
            MilestoneCommitView(project: project)
        }
        .onAppear {
            loadCommits()
        }
    }

    private func loadCommits() {
        isLoading = true

        OxenDaemonXPCClient.shared.getCommitHistory(path: project.path, limit: 50) { commitData in
            DispatchQueue.main.async {
                // Parse commit history from XPC response
                var loadedCommits: [CommitInfo] = []

                for commit in commitData {
                    guard let id = commit["id"] as? String,
                          let message = commit["message"] as? String,
                          let timestamp = commit["timestamp"] as? Date,
                          let author = commit["author"] as? String else {
                        continue
                    }

                    // Parse metadata if available
                    var metadata: CommitMetadata? = nil
                    if let metadataDict = commit["metadata"] as? [String: Any] {
                        metadata = CommitMetadata(
                            bpm: metadataDict["bpm"] as? Double,
                            sampleRate: metadataDict["sample_rate"] as? Int,
                            keySignature: metadataDict["key_signature"] as? String,
                            timeSignature: metadataDict["time_signature"] as? String,
                            tags: metadataDict["tags"] as? [String]
                        )
                    }

                    loadedCommits.append(CommitInfo(
                        id: id,
                        message: message,
                        timestamp: timestamp,
                        author: author,
                        metadata: metadata
                    ))
                }

                commits = loadedCommits
                isLoading = false
            }
        }
    }

    private func openInLogic() {
        let projectURL = URL(fileURLWithPath: project.path)
        NSWorkspace.shared.open(projectURL)
    }
}

struct ProjectHeaderView: View {
    let project: Project

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                VStack(alignment: .leading, spacing: 4) {
                    Text(project.displayName)
                        .font(.title2)
                        .fontWeight(.semibold)

                    Text(project.path)
                        .font(.caption)
                        .foregroundColor(.secondary)
                }

                Spacer()

                if project.isLocked {
                    VStack(alignment: .trailing, spacing: 2) {
                        HStack {
                            Image(systemName: "lock.fill")
                                .foregroundColor(.orange)
                            Text("Locked")
                                .fontWeight(.medium)
                        }
                        if let lockedBy = project.lockedBy {
                            Text("by \(lockedBy)")
                                .font(.caption)
                                .foregroundColor(.secondary)
                        }
                    }
                    .padding(8)
                    .background(Color.orange.opacity(0.1))
                    .cornerRadius(8)
                }
            }

            HStack(spacing: 20) {
                StatView(label: "Commits", value: "\(project.commitCount)")
                if let lastCommit = project.lastCommit {
                    StatView(
                        label: "Last Commit",
                        value: lastCommit.formatted(date: .abbreviated, time: .omitted)
                    )
                }
            }
        }
    }
}

struct StatView: View {
    let label: String
    let value: String

    var body: some View {
        VStack(alignment: .leading, spacing: 2) {
            Text(label)
                .font(.caption)
                .foregroundColor(.secondary)
            Text(value)
                .font(.body)
                .fontWeight(.medium)
        }
    }
}

struct CommitRowView: View {
    let commit: CommitInfo
    let project: Project

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Text(commit.message)
                    .font(.body)
                    .fontWeight(.medium)

                Spacer()

                Button("Rollback") {
                    // TODO: Rollback to this commit
                }
                .buttonStyle(.bordered)
                .controlSize(.small)
            }

            Text(commit.timestamp.formatted())
                .font(.caption)
                .foregroundColor(.secondary)

            if let metadata = commit.metadata {
                HStack(spacing: 12) {
                    if let bpm = metadata.bpm {
                        Label("\(Int(bpm)) BPM", systemImage: "metronome")
                            .font(.caption)
                    }
                    if let sampleRate = metadata.sampleRate {
                        Label("\(sampleRate/1000)kHz", systemImage: "waveform")
                            .font(.caption)
                    }
                    if let key = metadata.keySignature {
                        Label(key, systemImage: "music.note")
                            .font(.caption)
                    }
                }
                .foregroundColor(.secondary)
            }
        }
        .padding(.vertical, 4)
    }
}

struct MergeHelperView: View {
    let project: Project
    @Environment(\.dismiss) var dismiss

    var body: some View {
        VStack {
            Text("Merge Helper")
                .font(.title)
                .padding()

            Text("Merge helper functionality coming soon...")
                .foregroundColor(.secondary)

            Spacer()

            Button("Close") {
                dismiss()
            }
            .padding()
        }
        .frame(width: 400, height: 300)
    }
}
