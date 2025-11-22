import SwiftUI
import Foundation

extension String {
    func appendToFile(at path: String) throws {
        let data = self.data(using: .utf8)!
        if FileManager.default.fileExists(atPath: path) {
            let fileHandle = try FileHandle(forWritingTo: URL(fileURLWithPath: path))
            fileHandle.seekToEndOfFile()
            fileHandle.write(data)
            try fileHandle.close()
        } else {
            try data.write(to: URL(fileURLWithPath: path))
        }
    }
}

struct ContentView: View {
    @StateObject private var viewModel = ProjectListViewModel()
    @State private var selectedProject: Project?
    @State private var showingMergeHelper = false
    @State private var showingProjectWizard = false

    var body: some View {
        NavigationSplitView {
            // Left sidebar - Project List
            ProjectListContentView(
                projects: viewModel.projects,
                selectedProject: $selectedProject
            )
            .navigationSplitViewColumnWidth(min: 250, ideal: 300, max: 400)
        } detail: {
            // Right panel - Project Details
            let log = "[ContentView.detail] Evaluating detail pane, selectedProject=\(selectedProject?.path ?? "nil")\n"
            let _ = try? log.appendToFile(at: "/tmp/auxin-app-debug.log")
            let _ = print("DEBUG ContentView.detail: selectedProject=\(selectedProject?.path ?? "nil")")

            if let project = selectedProject {
                let log2 = "[ContentView.detail] Creating ProjectDetailContentView for: \(project.path)\n"
                let _ = try? log2.appendToFile(at: "/tmp/auxin-app-debug.log")
                let _ = print("DEBUG ContentView.detail: Creating ProjectDetailContentView for: \(project.path)")

                ProjectDetailContentView(project: project)
                    .id(project.path)  // Force view recreation when project changes
            } else {
                let log3 = "[ContentView.detail] Showing placeholder (no project selected)\n"
                let _ = try? log3.appendToFile(at: "/tmp/auxin-app-debug.log")
                let _ = print("DEBUG ContentView.detail: Showing placeholder (no project selected)")

                VStack {
                    Spacer()
                    Text("Select a project to view details")
                        .foregroundColor(.secondary)
                        .font(.title3)
                    Spacer()
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            }
        }
        .toolbar {
            ToolbarItemGroup(placement: .automatic) {
                Button(action: {
                    showingProjectWizard = true
                }) {
                    Label("Add Project", systemImage: "plus.circle")
                }

                Button(action: {
                    viewModel.loadProjects()
                }) {
                    Label("Refresh", systemImage: "arrow.clockwise")
                }
            }
        }
        .sheet(isPresented: $showingMergeHelper) {
            if let project = selectedProject {
                MergeHelperView(project: project)
            }
        }
        .sheet(isPresented: $showingProjectWizard) {
            ProjectWizardView()
        }
        .onAppear {
            viewModel.loadProjects()
        }
        .onChange(of: selectedProject) { oldValue, newValue in
            let log = "[ContentView] selectedProject changed from '\(oldValue?.path ?? "nil")' to '\(newValue?.path ?? "nil")'\n"
            try? log.appendToFile(at: "/tmp/auxin-app-debug.log")
            print("DEBUG ContentView: selectedProject changed from '\(oldValue?.path ?? "nil")' to '\(newValue?.path ?? "nil")'")
        }
        .onChange(of: viewModel.projects) { oldValue, newValue in
            // Auto-select first project for debugging
            let log = "[ContentView] onChange: projects.count=\(newValue.count), selectedProject=\(selectedProject?.path ?? "nil")\n"
            try? log.appendToFile(at: "/tmp/auxin-app-debug.log")

            if selectedProject == nil, let firstProject = newValue.first {
                let log2 = "[ContentView] Auto-selecting: \(firstProject.path)\n"
                try? log2.appendToFile(at: "/tmp/auxin-app-debug.log")
                selectedProject = firstProject

                let log3 = "[ContentView] After selection: selectedProject=\(selectedProject?.path ?? "nil")\n"
                try? log3.appendToFile(at: "/tmp/auxin-app-debug.log")
            }
        }
        .onReceive(NotificationCenter.default.publisher(for: .refreshProjects)) { _ in
            viewModel.loadProjects()
        }
        .onReceive(NotificationCenter.default.publisher(for: .showMergeHelper)) { _ in
            if selectedProject != nil {
                showingMergeHelper = true
            } else {
                showNoProjectAlert()
            }
        }
        .onReceive(NotificationCenter.default.publisher(for: .showProjectWizard)) { _ in
            showingProjectWizard = true
        }
        .overlay(alignment: .bottom) {
            SwiftUIStatusBar()
        }
    }

    private func showNoProjectAlert() {
        let alert = NSAlert()
        alert.messageText = "No Project Selected"
        alert.informativeText = "Please select a project first"
        alert.alertStyle = .informational
        alert.addButton(withTitle: "OK")
        alert.runModal()
    }
}
