import SwiftUI

struct ProjectListContentView: View {
    let projects: [Project]
    @Binding var selectedProject: Project?
    @State private var projectToDelete: Project?
    @State private var showingDeleteAlert = false

    var body: some View {
        List(projects, id: \.path, selection: $selectedProject) { project in
            ProjectRowView(project: project)
                .tag(project)
                .swipeActions(edge: .trailing, allowsFullSwipe: false) {
                    Button(role: .destructive) {
                        projectToDelete = project
                        showingDeleteAlert = true
                    } label: {
                        Label("Remove", systemImage: "trash")
                    }
                }
                .contextMenu {
                    Button {
                        openInLogic(project)
                    } label: {
                        Label("Open in Logic Pro", systemImage: "music.note")
                    }

                    Divider()

                    Button(role: .destructive) {
                        projectToDelete = project
                        showingDeleteAlert = true
                    } label: {
                        Label("Remove from Monitoring", systemImage: "trash")
                    }
                }
        }
        .navigationTitle("Projects")
        .listStyle(.sidebar)
        .alert("Remove Project", isPresented: $showingDeleteAlert) {
            Button("Cancel", role: .cancel) {
                projectToDelete = nil
            }
            Button("Remove", role: .destructive) {
                if let project = projectToDelete {
                    removeProject(project)
                }
                projectToDelete = nil
            }
        } message: {
            if let project = projectToDelete {
                Text("Are you sure you want to stop monitoring \(project.displayName)? This will not delete the project files or repository.")
            }
        }
    }

    private func removeProject(_ project: Project) {
        OxenDaemonXPCClient.shared.unregisterProject(path: project.path) { success in
            if success {
                // Refresh the project list
                NotificationCenter.default.post(name: .refreshProjects, object: nil)

                // Clear selection if deleted project was selected
                DispatchQueue.main.async {
                    if selectedProject?.path == project.path {
                        selectedProject = nil
                    }
                }
            }
        }
    }

    private func openInLogic(_ project: Project) {
        let projectURL = URL(fileURLWithPath: project.path)
        NSWorkspace.shared.open(projectURL)
    }
}

struct ProjectRowView: View {
    let project: Project

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Text(project.displayName)
                    .font(.system(size: 14, weight: .medium))

                if project.isLocked {
                    Image(systemName: "lock.fill")
                        .foregroundColor(.orange)
                        .font(.system(size: 12))
                }

                Spacer()
            }

            Text(project.path)
                .font(.system(size: 11))
                .foregroundColor(.secondary)
                .lineLimit(1)
                .truncationMode(.middle)

            Text(projectStatus(project))
                .font(.system(size: 10))
                .foregroundColor(.secondary)
        }
        .padding(.vertical, 4)
    }

    private func projectStatus(_ project: Project) -> String {
        if project.isLocked, let lockedBy = project.lockedBy {
            return "Locked by \(lockedBy)"
        } else if let lastCommit = project.lastCommit {
            let formatter = RelativeDateTimeFormatter()
            formatter.unitsStyle = .abbreviated
            let relativeDate = formatter.localizedString(for: lastCommit, relativeTo: Date())
            return "\(project.commitCount) commits • Last: \(relativeDate)"
        } else {
            return "No commits yet"
        }
    }
}
