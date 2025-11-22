import SwiftUI
import Foundation

struct ProjectDetailContentView: View {
    let project: Project
    @State private var commits: [CommitInfo] = []
    @State private var isLoading = false
    @State private var showingMilestoneCommit = false

    init(project: Project) {
        self.project = project
        print("✅ ProjectDetailContentView.init() called for: \(project.path)")
        NSLog("✅ ProjectDetailContentView.init() called for: %@", project.path)
    }

    var body: some View {
        VStack(spacing: 0) {
            // Header
            ProjectHeaderView(project: project)
                .padding(12)
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
                    openInApplication()
                } label: {
                    Label("Open in \(project.projectType.displayName)", systemImage: project.iconName)
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
            let log = "[ProjectDetailContentView] onAppear: \(project.path)\n"
            try? log.appendToFile(at: "/tmp/auxin-app-debug.log")
            loadCommits()
        }
        .onChange(of: project.path) { oldPath, newPath in
            let log = "[ProjectDetailContentView] onChange: \(oldPath) -> \(newPath)\n"
            try? log.appendToFile(at: "/tmp/auxin-app-debug.log")
            loadCommits()
        }
    }

    private func loadCommits() {
        let log = "[ProjectDetailContentView] loadCommits: \(project.path)\n"
        try? log.appendToFile(at: "/tmp/auxin-app-debug.log")
        isLoading = true

        OxenDaemonXPCClient.shared.getCommitHistory(path: project.path, limit: 50) { commitData in
            let log2 = "[ProjectDetailContentView] Got \(commitData.count) commits from XPC\n"
            try? log2.appendToFile(at: "/tmp/auxin-app-debug.log")
            print("DEBUG ProjectDetailContentView: Got \(commitData.count) commits from XPC for \(project.path)")
            if !commitData.isEmpty {
                print("DEBUG ProjectDetailContentView: First commit data: \(commitData[0])")
            }
            DispatchQueue.main.async {
                // Parse commit history from XPC response
                var loadedCommits: [CommitInfo] = []

                for commit in commitData {
                    // Try both "id" and "hash" keys for backwards compatibility
                    guard let id = (commit["id"] as? String) ?? (commit["hash"] as? String),
                          let message = commit["message"] as? String else {
                        print("ProjectDetailContentView: Skipping commit - missing id or message: \(commit)")
                        continue
                    }

                    // Timestamp and author are optional - use defaults if not present
                    let timestamp = commit["timestamp"] as? Date ?? Date()
                    let author = commit["author"] as? String ?? "Unknown"

                    // Parse metadata if available
                    var metadata: CommitMetadata? = nil
                    if let metadataDict = commit["metadata"] as? [String: Any] {
                        metadata = CommitMetadata(
                            bpm: metadataDict["bpm"] as? Double,
                            sampleRate: metadataDict["sample_rate"] as? Int,
                            keySignature: metadataDict["key_signature"] as? String,
                            timeSignature: metadataDict["time_signature"] as? String,
                            units: metadataDict["units"] as? String,
                            layerCount: metadataDict["layer_count"] as? Int,
                            componentCount: metadataDict["component_count"] as? Int,
                            groupCount: metadataDict["group_count"] as? Int,
                            sceneCount: metadataDict["scene_count"] as? Int,
                            objectCount: metadataDict["object_count"] as? Int,
                            materialCount: metadataDict["material_count"] as? Int,
                            renderEngine: metadataDict["render_engine"] as? String,
                            resolution: metadataDict["resolution"] as? String,
                            fps: metadataDict["fps"] as? Int,
                            frameRange: metadataDict["frame_range"] as? String,
                            tags: metadataDict["tags"] as? [String],
                            fileSizeBytes: metadataDict["file_size_bytes"] as? Int
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

                let log3 = "[ProjectDetailContentView] Parsed \(loadedCommits.count) commits\n"
                try? log3.appendToFile(at: "/tmp/auxin-app-debug.log")
                print("DEBUG ProjectDetailContentView: Parsed \(loadedCommits.count) commits successfully")
                print("DEBUG ProjectDetailContentView: Setting commits array and isLoading=false")
                commits = loadedCommits
                isLoading = false
            }
        }
    }

    private func openInApplication() {
        let projectURL = URL(fileURLWithPath: project.path)
        NSWorkspace.shared.open(projectURL)
    }
}

struct ProjectHeaderView: View {
    let project: Project

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack {
                VStack(alignment: .leading, spacing: 2) {
                    HStack(spacing: 6) {
                        Image(systemName: project.iconName)
                            .foregroundColor(.accentColor)
                        Text(project.displayName)
                            .font(.title2)
                            .fontWeight(.semibold)
                    }

                    Text(project.path)
                        .font(.caption)
                        .foregroundColor(.secondary)
                }

                Spacer()

                if project.isLocked {
                    VStack(alignment: .trailing, spacing: 1) {
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
                    .padding(6)
                    .background(Color.orange.opacity(0.1))
                    .cornerRadius(8)
                }
            }

            HStack(spacing: 12) {
                StatView(label: "Type", value: project.projectType.displayName)
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
        VStack(alignment: .leading, spacing: 2) {
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
                MetadataView(metadata: metadata, projectType: project.projectType)
            }
        }
        .padding(.vertical, 2)
    }
}

/// Display metadata based on project type
struct MetadataView: View {
    let metadata: CommitMetadata
    let projectType: ProjectType

    var body: some View {
        HStack(spacing: 8) {
            switch projectType {
            case .logicPro:
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

            case .sketchup:
                if let units = metadata.units {
                    Label(units, systemImage: "ruler")
                        .font(.caption)
                }
                if let layers = metadata.layerCount {
                    Label("\(layers) layers", systemImage: "square.3.layers.3d")
                        .font(.caption)
                }
                if let components = metadata.componentCount {
                    Label("\(components) components", systemImage: "cube")
                        .font(.caption)
                }

            case .blender:
                if let scenes = metadata.sceneCount {
                    Label("\(scenes) scenes", systemImage: "film")
                        .font(.caption)
                }
                if let objects = metadata.objectCount {
                    Label("\(objects) objects", systemImage: "cube.transparent")
                        .font(.caption)
                }
                if let materials = metadata.materialCount {
                    Label("\(materials) materials", systemImage: "paintpalette")
                        .font(.caption)
                }
                if let engine = metadata.renderEngine {
                    Label(engine, systemImage: "camera")
                        .font(.caption)
                }
            }
        }
        .foregroundColor(.secondary)
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
