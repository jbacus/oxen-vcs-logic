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
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                VStack(alignment: .leading, spacing: 4) {
                    HStack(spacing: 8) {
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
    @State private var isPlayingBounce = false

    var body: some View {
        HStack(alignment: .top, spacing: 12) {
            // Thumbnail
            if let metadata = commit.metadata,
               let thumbnailPath = metadata.thumbnailPath {
                let fullPath = project.path + "/.auxin/thumbnails/" + thumbnailPath
                if let nsImage = NSImage(contentsOfFile: fullPath) {
                    Image(nsImage: nsImage)
                        .resizable()
                        .aspectRatio(contentMode: .fill)
                        .frame(width: 80, height: 60)
                        .clipShape(RoundedRectangle(cornerRadius: 6))
                        .overlay(
                            RoundedRectangle(cornerRadius: 6)
                                .stroke(Color.secondary.opacity(0.3), lineWidth: 1)
                        )
                } else {
                    placeholderThumbnail
                }
            } else {
                placeholderThumbnail
            }

            // Commit details
            VStack(alignment: .leading, spacing: 4) {
                HStack {
                    Text(commit.message)
                        .font(.body)
                        .fontWeight(.medium)

                    Spacer()

                    // Bounce playback button
                    if let metadata = commit.metadata,
                       let bouncePath = metadata.bouncePath {
                        Button(action: {
                            playBounce(bouncePath: bouncePath)
                        }) {
                            Image(systemName: isPlayingBounce ? "waveform.circle.fill" : "waveform.circle")
                                .foregroundColor(.blue)
                        }
                        .buttonStyle(.plain)
                        .help("Play audio bounce")
                    }

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
        }
        .padding(.vertical, 4)
    }

    private var placeholderThumbnail: some View {
        RoundedRectangle(cornerRadius: 6)
            .fill(Color.secondary.opacity(0.2))
            .frame(width: 80, height: 60)
            .overlay(
                Image(systemName: project.iconName)
                    .foregroundColor(.secondary)
            )
    }

    private func playBounce(bouncePath: String) {
        let fullPath = project.path + "/.auxin/bounces/" + bouncePath
        let url = URL(fileURLWithPath: fullPath)

        guard FileManager.default.fileExists(atPath: fullPath) else {
            print("Bounce file not found: \(fullPath)")
            return
        }

        // Use afplay to play the audio file in the background
        let task = Process()
        task.executableURL = URL(fileURLWithPath: "/usr/bin/afplay")
        task.arguments = [fullPath]

        isPlayingBounce = true

        task.terminationHandler = { _ in
            DispatchQueue.main.async {
                self.isPlayingBounce = false
            }
        }

        do {
            try task.run()
        } catch {
            print("Failed to play bounce: \(error)")
            isPlayingBounce = false
        }
    }
}

/// Display metadata based on project type
struct MetadataView: View {
    let metadata: CommitMetadata
    let projectType: ProjectType

    var body: some View {
        HStack(spacing: 12) {
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
