import SwiftUI

struct ProjectListContentView: View {
    let projects: [Project]
    @Binding var selectedProject: Project?

    var body: some View {
        List(projects, id: \.path, selection: $selectedProject) { project in
            ProjectRowView(project: project)
                .tag(project)
        }
        .navigationTitle("Projects")
        .listStyle(.sidebar)
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
